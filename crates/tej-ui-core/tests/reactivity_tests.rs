use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use tej_ui_core::{Memo, Signal, batch, create_effect};

#[test]
fn test_signal_copy_semantics() {
    let count = Signal::new(10);
    // Copy by value
    let c1 = count;
    let c2 = count;
    assert_eq!(c1.get(), 10);
    assert_eq!(c2.get(), 10);

    assert_eq!(std::mem::size_of_val(&count), 8);
}

#[test]
fn test_signal_write_and_update() {
    let s = Signal::new(100);
    assert_eq!(s.get(), 100);

    s.set(200);
    assert_eq!(s.get(), 200);

    s.update(|v| *v += 50);
    assert_eq!(s.get(), 250);
}

#[test]
fn test_effect_reactivity() {
    let count = Signal::new(0);
    let runs = Arc::new(AtomicUsize::new(0));

    let runs_clone = Arc::clone(&runs);
    let _effect = create_effect(move || {
        let val = count.get();
        runs_clone.fetch_add(1 + val, Ordering::SeqCst);
    });

    // Initial run with count=0 added 1
    assert_eq!(runs.load(Ordering::SeqCst), 1);

    // Updating signal triggers effect
    count.set(5);
    // Ran with count=5, adding 6 -> 1 + 6 = 7
    assert_eq!(runs.load(Ordering::SeqCst), 7);
}

#[test]
fn test_memo_derivation() {
    let a = Signal::new(2);
    let b = Signal::new(3);

    let product = Memo::new(move || a.get() * b.get());
    assert_eq!(product.get(), 6);

    a.set(4);
    assert_eq!(product.get(), 12);

    b.set(10);
    assert_eq!(product.get(), 40);
}

#[test]
fn test_diamond_dependency_graph() {
    // A -> B, A -> C, (B, C) -> D
    let a = Signal::new(1);
    let b = Memo::new(move || a.get() * 2);
    let c = Memo::new(move || a.get() * 10);

    let d_eval_count = Arc::new(AtomicUsize::new(0));
    let d_eval_clone = Arc::clone(&d_eval_count);

    let d = Memo::new(move || {
        d_eval_clone.fetch_add(1, Ordering::SeqCst);
        b.get() + c.get()
    });

    assert_eq!(d.get(), 12); // 2 + 10 = 12
    let initial_evals = d_eval_count.load(Ordering::SeqCst);
    assert!(initial_evals > 0);

    // Update root: both branches update, D updates glitch-free
    a.set(2);
    assert_eq!(d.get(), 24); // 4 + 20 = 24
}

#[test]
fn test_batched_mutations() {
    let a = Signal::new(1);
    let b = Signal::new(2);
    let runs = Arc::new(AtomicUsize::new(0));

    let runs_clone = Arc::clone(&runs);
    let _effect = create_effect(move || {
        let _ = a.get() + b.get();
        runs_clone.fetch_add(1, Ordering::SeqCst);
    });

    assert_eq!(runs.load(Ordering::SeqCst), 1);

    // Batched updates: both change, but effect only fires once after batch finishes
    batch(|| {
        a.set(10);
        b.set(20);
    });

    assert_eq!(runs.load(Ordering::SeqCst), 2);
    assert_eq!(a.get(), 10);
    assert_eq!(b.get(), 20);
}

#[test]
fn test_dynamic_branch_subscription() {
    let toggle = Signal::new(true);
    let val_a = Signal::new(10);
    let val_b = Signal::new(20);

    let branch_runs = Arc::new(AtomicUsize::new(0));
    let branch_clone = Arc::clone(&branch_runs);

    let result = Memo::new(move || {
        branch_clone.fetch_add(1, Ordering::SeqCst);
        if toggle.get() {
            val_a.get()
        } else {
            val_b.get()
        }
    });

    assert_eq!(result.get(), 10);

    // When toggle is true, mutating val_b should NOT trigger recalculation
    val_b.set(99);
    // Since val_b wasn't tracked on the true branch, branch_runs count shouldn't change
    assert_eq!(result.get(), 10);

    // Switch toggle to false: now val_b is tracked
    toggle.set(false);
    assert_eq!(result.get(), 99);

    // Now mutating val_b updates result
    val_b.set(100);
    assert_eq!(result.get(), 100);
}
