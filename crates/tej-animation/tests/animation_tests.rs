use tej_animation::{
    AnimatedSpring, AnimatedValue, Lerp, SpringConfig, SpringSimulation, VsyncTicker,
};
use tej_hal::Color;
use tej_layout::Point;

#[test]
fn test_spring_config_characteristics() {
    let snappy = SpringConfig::snappy();
    assert!(snappy.damping_ratio() > 0.5);
    assert!(snappy.natural_frequency() > 10.0);

    let bouncy = SpringConfig::bouncy();
    assert!(bouncy.damping_ratio() < snappy.damping_ratio());

    let gentle = SpringConfig::gentle();
    assert_eq!(gentle.mass, 1.0);
}

#[test]
fn test_spring_simulation_convergence() {
    let config = SpringConfig::snappy();
    let mut sim = SpringSimulation::new(0.0, config);
    sim.set_target(100.0);

    assert!(!sim.is_at_rest());

    // Step through 2 seconds of simulation (120 frames at 16.6ms)
    for _ in 0..120 {
        sim.step(1.0 / 60.0);
    }

    assert!(sim.is_at_rest());
    assert!((sim.current - 100.0).abs() < 0.01);
    assert_eq!(sim.velocity, 0.0);
}

#[test]
fn test_lerp_interpolations() {
    // 1. f32 Lerp
    assert_eq!(0.0f32.lerp(&10.0, 0.5), 5.0);

    // 2. Point Lerp
    let p1 = Point::new(0.0, 10.0);
    let p2 = Point::new(100.0, 50.0);
    let mid_p = p1.lerp(&p2, 0.5);
    assert_eq!(mid_p.x, 50.0);
    assert_eq!(mid_p.y, 30.0);

    // 3. Color Lerp
    let c1 = Color::rgb(0, 100, 200);
    let c2 = Color::rgb(100, 200, 100);
    let mid_c = c1.lerp(&c2, 0.5);
    assert_eq!(mid_c.r, 50);
    assert_eq!(mid_c.g, 150);
    assert_eq!(mid_c.b, 150);
}

#[test]
fn test_animated_spring_reactive_signal() {
    let mut anim = AnimatedSpring::new(0.0, SpringConfig::bouncy());
    assert_eq!(anim.get(), 0.0);

    anim.set_target(50.0);
    anim.step(1.0 / 60.0);

    // Signal updated reactively
    assert!(anim.get() > 0.0);

    // Step until rest
    for _ in 0..120 {
        anim.step(1.0 / 60.0);
    }

    assert!(anim.is_at_rest());
    assert!((anim.get() - 50.0).abs() < 0.01);
}

#[test]
fn test_animated_value_color_transition() {
    let red = Color::rgb(255, 0, 0);
    let blue = Color::rgb(0, 0, 255);
    let mut anim = AnimatedValue::new(red, SpringConfig::gentle());

    anim.set_target(blue);
    assert_eq!(anim.get().r, 255);

    // Step half way
    for _ in 0..30 {
        anim.step(1.0 / 60.0);
    }

    // Color is blended
    let blended = anim.get();
    assert!(blended.r < 255);
    assert!(blended.b > 0);
}

#[test]
fn test_vsync_ticker_step() {
    let mut ticker = VsyncTicker::new(120.0);
    assert_eq!(ticker.target_fps(), 120.0);

    let dt = ticker.tick();
    assert!(dt > 0.0 && dt <= 0.1);
    assert!(ticker.elapsed_seconds() > 0.0);
}
