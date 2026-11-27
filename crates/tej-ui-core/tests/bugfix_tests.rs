use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use tej_ui_core::{
    DependencyGraph, Memo, NodeId, ReactiveRuntime, Signal, SignalId, Store, WorkerBridge,
    WorkerStatus, batch, create_effect,
};

#[test]
fn test_graph_topological_depth_and_transitive_bfs() {
    let mut graph = DependencyGraph::new();

    let sig_a = SignalId::new(1, 1);
    let sig_b = SignalId::new(2, 1);
    let sig_c = SignalId::new(3, 1);
    let sig_d = SignalId::new(4, 1);

    let node_b = NodeId::new(10, 1);
    let node_c = NodeId::new(11, 1);
    let node_d = NodeId::new(12, 1);
    let node_e = NodeId::new(13, 1);

    graph.register_node_output(node_b, sig_b);
    graph.register_node_output(node_c, sig_c);
    graph.register_node_output(node_d, sig_d);

    // Diamond: A -> B, A -> C, (B, C) -> D -> E
    graph.add_dependency(node_b, sig_a);
    graph.add_dependency(node_c, sig_a);
    graph.add_dependency(node_d, sig_b);
    graph.add_dependency(node_d, sig_c);
    graph.add_dependency(node_e, sig_d);

    assert_eq!(graph.node_depth(node_b), 1);
    assert_eq!(graph.node_depth(node_c), 1);
    assert_eq!(graph.node_depth(node_d), 2);
    assert_eq!(graph.node_depth(node_e), 3);

    // Transitive BFS drains queue and returns all 4 nodes sorted by ascending depth
    let order = graph.collect_subscribers_topological(sig_a);
    assert_eq!(order.len(), 4);
    assert!(order[0] == node_b || order[0] == node_c);
    assert!(order[1] == node_b || order[1] == node_c);
    assert_eq!(order[2], node_d);
    assert_eq!(order[3], node_e);

    // Cycle guard: adding a back-edge does not infinite-loop
    graph.add_dependency(node_b, sig_d);
    let order_with_cycle = graph.collect_subscribers_topological(sig_a);
    assert_eq!(order_with_cycle.len(), 4);
}

#[test]
fn test_diamond_graph_glitch_free_single_evaluation_and_no_double_init() {
    let a = Signal::new(1i32);

    let b_evals = Arc::new(AtomicUsize::new(0));
    let b_evals_c = Arc::clone(&b_evals);
    let b = Memo::new(move || {
        b_evals_c.fetch_add(1, Ordering::SeqCst);
        a.get() * 2
    });

    let c_evals = Arc::new(AtomicUsize::new(0));
    let c_evals_c = Arc::clone(&c_evals);
    let c = Memo::new(move || {
        c_evals_c.fetch_add(1, Ordering::SeqCst);
        a.get() * 10
    });

    let d_evals = Arc::new(AtomicUsize::new(0));
    let d_evals_c = Arc::clone(&d_evals);
    let observed_glitch = Arc::new(AtomicBool::new(false));
    let observed_glitch_c = Arc::clone(&observed_glitch);

    let d = Memo::new(move || {
        d_evals_c.fetch_add(1, Ordering::SeqCst);
        let bv = b.get();
        let cv = c.get();
        // Invariant: cv is always 5 * bv
        if cv != bv * 5 {
            observed_glitch_c.store(true, Ordering::SeqCst);
        }
        bv + cv
    });

    // No double-evaluation on Memo::new
    assert_eq!(b_evals.load(Ordering::SeqCst), 1);
    assert_eq!(c_evals.load(Ordering::SeqCst), 1);
    assert_eq!(d_evals.load(Ordering::SeqCst), 1);
    assert_eq!(d.get(), 12);

    // Single update to A must evaluate D exactly once, glitch-free
    a.set(3);
    assert_eq!(d.get(), 36);
    assert_eq!(d_evals.load(Ordering::SeqCst), 2);
    assert!(!observed_glitch.load(Ordering::SeqCst));
}

#[test]
fn test_memo_equality_check_skips_downstream_notification() {
    let n = Signal::new(2i32);
    let is_even = Memo::new(move || n.get() % 2 == 0);

    let effect_runs = Arc::new(AtomicUsize::new(0));
    let effect_runs_c = Arc::clone(&effect_runs);
    let eff = create_effect(move || {
        let _ = is_even.get();
        effect_runs_c.fetch_add(1, Ordering::SeqCst);
    });

    assert_eq!(effect_runs.load(Ordering::SeqCst), 1);

    // 4 is still even -> Memo value unchanged -> effect must NOT re-run
    n.set(4);
    assert_eq!(effect_runs.load(Ordering::SeqCst), 1);

    // 5 is odd -> Memo changes to false -> effect MUST re-run
    n.set(5);
    assert_eq!(effect_runs.load(Ordering::SeqCst), 2);

    eff.dispose();
    is_even.dispose();
    n.dispose();
}

#[test]
fn test_raii_guards_recover_from_panics_in_batch_and_effect() {
    let s = Signal::new(1i32);

    // Caught panic inside batch must restore BATCH.depth to 0
    let batch_res = catch_unwind(AssertUnwindSafe(|| {
        batch(|| {
            s.set(2);
            panic!("simulated batch panic");
        });
    }));
    assert!(batch_res.is_err());

    // Caught panic inside effect must pop OBSERVER_STACK
    let eff_res = catch_unwind(AssertUnwindSafe(|| {
        let _ = create_effect(move || {
            let _ = s.get();
            panic!("simulated effect panic");
        });
    }));
    assert!(eff_res.is_err());

    // Runtime remains healthy: new signals and effects propagate immediately (not stuck in batch)
    let healthy = Signal::new(10i32);
    let seen = Arc::new(AtomicUsize::new(0));
    let seen_c = Arc::clone(&seen);
    let eff = create_effect(move || {
        seen_c.store(healthy.get() as usize, Ordering::SeqCst);
    });

    assert_eq!(seen.load(Ordering::SeqCst), 10);
    healthy.set(42);
    assert_eq!(seen.load(Ordering::SeqCst), 42);

    eff.dispose();
    healthy.dispose();
    s.dispose();
}

#[test]
fn test_signal_and_memo_disposal() {
    let s = Signal::new(5i32);
    let m = Memo::new(move || s.get() * 3);

    assert!(ReactiveRuntime::is_signal_valid(s.id()));
    assert!(ReactiveRuntime::is_signal_valid(m.signal().id()));
    assert_eq!(m.get(), 15);

    m.dispose();
    assert!(!ReactiveRuntime::is_signal_valid(m.signal().id()));

    // Updating s after memo disposal must not panic
    s.set(10);
    assert_eq!(s.get(), 10);

    s.dispose();
    assert!(!ReactiveRuntime::is_signal_valid(s.id()));
}

#[test]
fn test_store_undo_redo_no_deadlock_and_zero_history_limit() {
    // 1. Re-entrant effect reading store.can_undo()/can_redo() during undo/redo must not deadlock
    let store = Store::new(0i32, |s, delta: i32| s + delta);
    let store_clone = store.clone();
    let observed_can_undo = Arc::new(AtomicBool::new(false));
    let obs_c = Arc::clone(&observed_can_undo);

    let eff = create_effect(move || {
        let _ = store_clone.get();
        obs_c.store(store_clone.can_undo(), Ordering::SeqCst);
    });

    store.dispatch(10);
    store.dispatch(20);
    assert_eq!(store.get(), 30);

    assert!(store.undo());
    assert_eq!(store.get(), 10);
    assert!(observed_can_undo.load(Ordering::SeqCst));

    assert!(store.redo());
    assert_eq!(store.get(), 30);

    eff.dispose();

    // 2. max_history == 0 must never panic on remove(0)
    let zero_hist_store = Store::with_history_limit(0i32, |s, delta: i32| s + delta, 0);
    zero_hist_store.dispatch(1);
    zero_hist_store.dispatch(2);
    assert_eq!(zero_hist_store.get(), 3);
    assert!(!zero_hist_store.can_undo());
    assert!(!zero_hist_store.undo());
}

#[test]
fn test_worker_poll_reentrant_effect_no_deadlock() {
    let worker = WorkerBridge::<u32, u32, String>::new();
    let worker_clone = worker.clone();
    let reentered = Arc::new(AtomicBool::new(false));
    let reentered_c = Arc::clone(&reentered);

    let eff = create_effect(move || {
        let status = worker_clone.status_signal().get();
        if status.is_completed() {
            // Re-entrantly polling inside a reactive subscriber must not deadlock on receiver lock
            worker_clone.poll();
            reentered_c.store(true, Ordering::SeqCst);
        }
    });

    let handle = worker.spawn(|_reporter, _cancel| Ok(99));
    handle.join();

    worker.poll();
    assert_eq!(worker.status(), WorkerStatus::Completed(99));
    assert!(reentered.load(Ordering::SeqCst));

    eff.dispose();
}

#[test]
fn test_reactive_runtime_epoch_advances_on_signal_write() {
    let before = ReactiveRuntime::epoch();
    let s = Signal::new(1i32);
    s.set(2);
    let after = ReactiveRuntime::epoch();
    assert!(after > before);
    s.dispose();
}
