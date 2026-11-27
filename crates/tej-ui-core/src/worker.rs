//! Structured concurrency & background worker bridge with progress reporting and cancel-on-unmount RAII handles.

use crate::signal::Signal;
use parking_lot::Mutex;
use std::fmt::Debug;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::thread;

/// Hierarchical cancellation token for cooperative background task cancellation.
#[derive(Clone, Debug, Default)]
pub struct CancellationToken {
    cancelled: Arc<AtomicBool>,
}

impl CancellationToken {
    /// Create a new active (uncancelled) token.
    #[must_use]
    pub fn new() -> Self {
        Self {
            cancelled: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Signal cancellation to the background worker.
    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::SeqCst);
    }

    /// Check if cancellation has been requested.
    #[must_use]
    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::SeqCst)
    }
}

/// Operational status of a background worker.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WorkerStatus<T, E> {
    /// Worker is registered but not currently executing.
    Idle,
    /// Worker is executing in the background.
    Running,
    /// Worker finished successfully with output data.
    Completed(T),
    /// Worker encountered an error.
    Failed(E),
    /// Worker was cancelled before completion.
    Cancelled,
}

impl<T, E> WorkerStatus<T, E> {
    /// Check if the worker is actively running.
    #[must_use]
    pub const fn is_running(&self) -> bool {
        matches!(self, Self::Running)
    }

    /// Check if the worker completed successfully.
    #[must_use]
    pub const fn is_completed(&self) -> bool {
        matches!(self, Self::Completed(_))
    }

    /// Check if the worker was cancelled.
    #[must_use]
    pub const fn is_cancelled(&self) -> bool {
        matches!(self, Self::Cancelled)
    }
}

enum WorkerMessage<T, P, E> {
    Progress(P),
    Completed(T),
    Failed(E),
    Cancelled,
}

type WorkerReceiver<T, P, E> = Arc<Mutex<Option<Receiver<WorkerMessage<T, P, E>>>>>;

/// Progress reporting sink passed to worker threads.

#[derive(Clone, Debug)]
pub struct ProgressReporter<P: Clone + Send + Sync + 'static> {
    sender: Sender<P>,
    cancel_token: CancellationToken,
}

impl<P: Clone + Send + Sync + 'static> ProgressReporter<P> {
    /// Report an incremental progress update to UI signals.
    ///
    /// Returns `false` if the task has been cancelled and should abort early.
    pub fn report(&self, progress: P) -> bool {
        if self.cancel_token.is_cancelled() {
            return false;
        }
        let _ = self.sender.send(progress);
        true
    }

    /// Check if the parent worker task has been cancelled.
    #[must_use]
    pub fn is_cancelled(&self) -> bool {
        self.cancel_token.is_cancelled()
    }
}

/// Structured background worker bridge tracking reactive status, progress, and result.
#[derive(Clone, Debug)]
pub struct WorkerBridge<
    T: Clone + Send + Sync + 'static,
    P: Clone + Send + Sync + 'static,
    E: Clone + Send + Sync + 'static,
> {
    status: Signal<WorkerStatus<T, E>>,
    progress: Signal<Option<P>>,
    result: Signal<Option<std::result::Result<T, E>>>,
    receiver: WorkerReceiver<T, P, E>,
    cancel_token: Arc<parking_lot::RwLock<CancellationToken>>,
}

impl<
    T: Clone + Send + Sync + 'static,
    P: Clone + Send + Sync + 'static,
    E: Clone + Send + Sync + 'static,
> Default for WorkerBridge<T, P, E>
{
    fn default() -> Self {
        Self::new()
    }
}

impl<
    T: Clone + Send + Sync + 'static,
    P: Clone + Send + Sync + 'static,
    E: Clone + Send + Sync + 'static,
> WorkerBridge<T, P, E>
{
    /// Create a new worker bridge with initial idle signals.
    #[must_use]
    pub fn new() -> Self {
        Self {
            status: Signal::new(WorkerStatus::Idle),
            progress: Signal::new(None),
            result: Signal::new(None),
            receiver: Arc::new(Mutex::new(None)),
            cancel_token: Arc::new(parking_lot::RwLock::new(CancellationToken::new())),
        }
    }

    /// Read-only reactive signal for execution status.
    #[must_use]
    pub const fn status_signal(&self) -> Signal<WorkerStatus<T, E>> {
        self.status
    }

    /// Read-only reactive signal for incremental progress.
    #[must_use]
    pub const fn progress_signal(&self) -> Signal<Option<P>> {
        self.progress
    }

    /// Read-only reactive signal for final result.
    #[must_use]
    pub const fn result_signal(&self) -> Signal<Option<std::result::Result<T, E>>> {
        self.result
    }

    /// Current execution status.
    #[must_use]
    pub fn status(&self) -> WorkerStatus<T, E> {
        self.poll();
        self.status.get()
    }

    /// Current progress snapshot.
    #[must_use]
    pub fn progress(&self) -> Option<P> {
        self.poll();
        self.progress.get()
    }

    /// Poll pending messages from the background worker and update reactive signals on the caller's thread.
    pub fn poll(&self) {
        let messages: Vec<WorkerMessage<T, P, E>> = {
            let guard = self.receiver.lock();
            if let Some(rx) = guard.as_ref() {
                let mut drained = Vec::new();
                while let Ok(msg) = rx.try_recv() {
                    drained.push(msg);
                }
                drained
            } else {
                return;
            }
        };

        for msg in messages {
            match msg {
                WorkerMessage::Progress(p) => {
                    self.progress.set(Some(p));
                }
                WorkerMessage::Completed(val) => {
                    self.result.set(Some(Ok(val.clone())));
                    self.status.set(WorkerStatus::Completed(val));
                }
                WorkerMessage::Failed(err) => {
                    self.result.set(Some(Err(err.clone())));
                    self.status.set(WorkerStatus::Failed(err));
                }
                WorkerMessage::Cancelled => {
                    self.status.set(WorkerStatus::Cancelled);
                }
            }
        }
    }

    /// Cancel the active background task.
    pub fn cancel(&self) {
        self.cancel_token.read().cancel();
        self.status.set(WorkerStatus::Cancelled);
    }

    /// Spawn a worker task onto a background thread.
    ///
    /// Returns an RAII `WorkerHandle` that cancels the background task on drop if not detached.
    pub fn spawn<F>(&self, task: F) -> WorkerHandle
    where
        F: FnOnce(ProgressReporter<P>, CancellationToken) -> std::result::Result<T, E>
            + Send
            + 'static,
    {
        self.cancel_token.read().cancel(); // Cancel any prior task
        let new_token = CancellationToken::new();
        *self.cancel_token.write() = new_token.clone();

        self.status.set(WorkerStatus::Running);
        self.progress.set(None);
        self.result.set(None);

        let (msg_tx, msg_rx) = channel::<WorkerMessage<T, P, E>>();
        let (prog_tx, prog_rx) = channel::<P>();

        *self.receiver.lock() = Some(msg_rx);

        let reporter = ProgressReporter {
            sender: prog_tx,
            cancel_token: new_token.clone(),
        };

        let thread_token = new_token.clone();
        let bg_msg_tx = msg_tx.clone();

        // Forwarder thread to bridge progress channel to main worker messages
        thread::spawn(move || {
            while let Ok(prog) = prog_rx.recv() {
                if bg_msg_tx.send(WorkerMessage::Progress(prog)).is_err() {
                    break;
                }
            }
        });

        let join_handle = thread::spawn(move || {
            if thread_token.is_cancelled() {
                let _ = msg_tx.send(WorkerMessage::Cancelled);
                return;
            }

            let outcome = task(reporter, thread_token.clone());

            if thread_token.is_cancelled() {
                let _ = msg_tx.send(WorkerMessage::Cancelled);
            } else {
                match outcome {
                    Ok(val) => {
                        let _ = msg_tx.send(WorkerMessage::Completed(val));
                    }
                    Err(err) => {
                        let _ = msg_tx.send(WorkerMessage::Failed(err));
                    }
                }
            }
        });

        WorkerHandle {
            token: new_token,
            join_handle: Some(join_handle),
            cancel_on_drop: true,
        }
    }
}

/// RAII handle managing task cancellation on scope drop (cancel-on-unmount).
#[derive(Debug)]
pub struct WorkerHandle {
    token: CancellationToken,
    join_handle: Option<thread::JoinHandle<()>>,
    cancel_on_drop: bool,
}

impl WorkerHandle {
    /// Detach the handle so the task continues running in background even after handle drops.
    pub fn detach(mut self) {
        self.cancel_on_drop = false;
    }

    /// Manually request task cancellation.
    pub fn cancel(&self) {
        self.token.cancel();
    }

    /// Wait for task to finish.
    pub fn join(mut self) {
        if let Some(h) = self.join_handle.take() {
            let _ = h.join();
        }
    }
}

impl Drop for WorkerHandle {
    fn drop(&mut self) {
        if self.cancel_on_drop {
            self.token.cancel();
        }
    }
}
