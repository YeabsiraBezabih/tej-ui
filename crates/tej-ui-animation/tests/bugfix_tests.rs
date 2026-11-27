use tej_ui_animation::{AnimatedValue, SpringConfig, SpringSimulation};

#[test]
fn test_spring_step_guards_invalid_dt_and_non_positive_mass() {
    let mut sim = SpringSimulation::new(0.0, SpringConfig::snappy());
    sim.set_target(100.0);

    // Non-positive and non-finite dt must return immediately without infinite loop or NaN
    assert_eq!(sim.step(0.0), 0.0);
    assert_eq!(sim.step(-0.016), 0.0);
    assert_eq!(sim.step(f32::NAN), 0.0);
    assert_eq!(sim.step(f32::INFINITY), 0.0);
    assert_eq!(sim.step(f32::NEG_INFINITY), 0.0);
    assert!(sim.current.is_finite());
    assert!(sim.velocity.is_finite());

    // Zero, negative, and NaN mass must not divide by zero or diverge to NaN
    for bad_mass in [0.0, -1.0, f32::NAN, f32::INFINITY] {
        let cfg = SpringConfig::new(bad_mass, 180.0, 24.0);
        let mut bad_sim = SpringSimulation::new(0.0, cfg);
        bad_sim.set_target(50.0);
        let val = bad_sim.step(1.0 / 60.0);
        assert!(val.is_finite(), "mass={bad_mass} produced non-finite value");
        assert!(bad_sim.velocity.is_finite());
        assert_eq!(val, 50.0);
    }
}

#[test]
fn test_animated_value_preserves_momentum_on_retarget() {
    let mut anim = AnimatedValue::new(0.0f32, SpringConfig::snappy());
    anim.set_target(100.0);

    // Step mid-flight to build positive velocity momentum
    for _ in 0..5 {
        anim.step(1.0 / 60.0);
    }
    let vel_before = anim.velocity();
    assert!(vel_before > 0.0, "expected positive velocity mid-flight");

    // Retarget while in motion: velocity momentum must be preserved
    anim.set_target(200.0);
    assert_eq!(anim.velocity(), vel_before);
}

#[test]
fn test_animated_value_allows_bouncy_spring_overshoot() {
    let mut anim = AnimatedValue::new(0.0f32, SpringConfig::bouncy());
    anim.set_target(100.0);

    let mut max_seen = 0.0f32;
    for _ in 0..120 {
        let v = anim.step(1.0 / 60.0);
        max_seen = max_seen.max(v);
    }

    assert!(
        max_seen > 100.0,
        "bouncy spring should overshoot target 100.0 (max_seen={max_seen})"
    );
    assert!(anim.is_at_rest());
}
