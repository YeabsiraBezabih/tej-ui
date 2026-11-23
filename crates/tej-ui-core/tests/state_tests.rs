use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use tej_ui_core::{
    Resource, ResourceState, Signal, Store, clear_context_stack, provide_context, use_context,
    with_context,
};

#[derive(Clone, Debug, PartialEq, Eq)]
struct ThemeConfig {
    name: String,
    dark_mode: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct UserSession {
    user_id: u64,
    username: String,
}

#[test]
fn test_ambient_context_provision_and_retrieval() {
    clear_context_stack();

    assert_eq!(use_context::<ThemeConfig>(), None);

    let default_theme = ThemeConfig {
        name: "Enterprise".into(),
        dark_mode: false,
    };
    provide_context(default_theme.clone());

    assert_eq!(use_context::<ThemeConfig>(), Some(default_theme));

    let session = UserSession {
        user_id: 42,
        username: "alice".into(),
    };
    provide_context(session.clone());

    assert_eq!(use_context::<UserSession>(), Some(session));
    // Verify parallel types do not interfere
    assert_eq!(
        use_context::<ThemeConfig>(),
        Some(ThemeConfig {
            name: "Enterprise".into(),
            dark_mode: false
        })
    );
}

#[test]
fn test_scoped_context_stack_override() {
    clear_context_stack();

    let root_theme = ThemeConfig {
        name: "Light".into(),
        dark_mode: false,
    };
    provide_context(root_theme.clone());

    assert_eq!(
        use_context::<ThemeConfig>().map(|t| t.name),
        Some("Light".into())
    );

    let dark_theme = ThemeConfig {
        name: "Dark".into(),
        dark_mode: true,
    };

    with_context(dark_theme, || {
        assert_eq!(
            use_context::<ThemeConfig>().map(|t| t.name),
            Some("Dark".into())
        );
        assert_eq!(
            use_context::<ThemeConfig>().map(|t| t.dark_mode),
            Some(true)
        );
    });

    // Guard popped; restored to root theme
    assert_eq!(
        use_context::<ThemeConfig>().map(|t| t.name),
        Some("Light".into())
    );
    assert_eq!(
        use_context::<ThemeConfig>().map(|t| t.dark_mode),
        Some(false)
    );
}

#[test]
fn test_resource_data_fetching_lifecycle() {
    let call_count = Arc::new(AtomicUsize::new(0));
    let cc = Arc::clone(&call_count);

    let res = Resource::<String, String>::new(move || {
        cc.fetch_add(1, Ordering::SeqCst);
        Ok("Fetched payload from database".into())
    });

    assert_eq!(
        res.get(),
        ResourceState::Ready("Fetched payload from database".into())
    );
    assert_eq!(res.data(), Some("Fetched payload from database".into()));
    assert_eq!(call_count.load(Ordering::SeqCst), 1);

    // Manual refetch
    res.refetch();
    assert_eq!(call_count.load(Ordering::SeqCst), 2);
    assert!(res.has_data().get());
}

#[test]
fn test_resource_with_reactive_source() {
    let query_id = Signal::new(101u64);

    let res = Resource::<String, String>::new_with_source(query_id, move |id| {
        if id == 404 {
            Err("Not Found".into())
        } else {
            Ok(format!("Record #{id}"))
        }
    });

    assert_eq!(res.get(), ResourceState::Ready("Record #101".into()));

    // Mutate source signal
    query_id.set(202);
    assert_eq!(res.get(), ResourceState::Ready("Record #202".into()));

    // Error branch
    query_id.set(404);
    assert_eq!(res.get(), ResourceState::Error("Not Found".into()));
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct CounterState {
    value: i32,
}

enum CounterAction {
    Increment,
    Decrement,
    Add(i32),
}

#[test]
fn test_store_reducer_and_undo_redo() {
    let store = Store::new(CounterState { value: 0 }, |state, action| match action {
        CounterAction::Increment => CounterState {
            value: state.value + 1,
        },
        CounterAction::Decrement => CounterState {
            value: state.value - 1,
        },
        CounterAction::Add(n) => CounterState {
            value: state.value + n,
        },
    });

    assert_eq!(store.get().value, 0);
    assert!(!store.can_undo());

    store.dispatch(CounterAction::Increment);
    assert_eq!(store.get().value, 1);
    assert!(store.can_undo());

    store.dispatch(CounterAction::Decrement);
    assert_eq!(store.get().value, 0);

    store.dispatch(CounterAction::Add(10));
    assert_eq!(store.get().value, 10);

    // Derived selector memo
    let is_positive = store.select(|s| s.value > 0);
    assert!(is_positive.get());

    // Undo action (was 10, previous was 0)
    assert!(store.undo());
    assert_eq!(store.get().value, 0);

    // Undo again (was 0, previous was 1)
    assert!(store.undo());
    assert_eq!(store.get().value, 1);

    // Undo again (was 1, previous was initial 0)
    assert!(store.undo());
    assert_eq!(store.get().value, 0);

    // Cannot undo past initial state
    assert!(!store.undo());
    assert_eq!(store.get().value, 0);

    // Redo action (restores 1)
    assert!(store.redo());
    assert_eq!(store.get().value, 1);

    // Redo action (restores 0)
    assert!(store.redo());
    assert_eq!(store.get().value, 0);

    // Redo action (restores 10)
    assert!(store.redo());
    assert_eq!(store.get().value, 10);
}

#[test]
fn test_worker_bridge_progress_and_cancel_on_unmount() {
    use std::thread;
    use std::time::Duration;
    use tej_ui_core::{WorkerBridge, WorkerStatus};

    // 1. Successful execution with progress reporting
    let worker = WorkerBridge::<String, u32, String>::new();
    assert_eq!(worker.status(), WorkerStatus::Idle);
    assert_eq!(worker.progress(), None);

    let handle = worker.spawn(|reporter, cancel| {
        for pct in [25, 50, 75, 100] {
            if cancel.is_cancelled() {
                return Err("Aborted".to_string());
            }
            reporter.report(pct);
            thread::sleep(Duration::from_millis(5));
        }
        Ok("Computed Hash: ABCD1234".to_string())
    });

    handle.join();

    assert_eq!(
        worker.status(),
        WorkerStatus::Completed("Computed Hash: ABCD1234".to_string())
    );
    assert_eq!(worker.progress(), Some(100));

    // 2. Cancellation on unmount (RAII drop of WorkerHandle)
    let cancel_worker = WorkerBridge::<String, u32, String>::new();
    {
        let _unmounted_handle = cancel_worker.spawn(|_reporter, cancel| {
            for _ in 0..100 {
                if cancel.is_cancelled() {
                    return Err("Cancelled".to_string());
                }
                thread::sleep(Duration::from_millis(5));
            }
            Ok("Finished".to_string())
        });
        // _unmounted_handle drops here -> triggers cancel()!
    }

    thread::sleep(Duration::from_millis(30));
    assert!(
        cancel_worker.status().is_cancelled()
            || matches!(cancel_worker.status(), WorkerStatus::Failed(_))
    );
}
