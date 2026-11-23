use tej_ui_layout::Point;
use tej_ui_view::{bayer8, dither_test, erf_approx, gaussian_shadow_alpha, sd_rounded_rect};

#[test]
fn test_bayer8_threshold_bounds_and_symmetry() {
    for y in 0..16 {
        for x in 0..16 {
            let t = bayer8(x, y);
            assert!((0.0..=1.0).contains(&t), "Threshold must be within [0, 1]");
        }
    }

    // Modulo 8 periodicity
    assert_eq!(bayer8(0, 0), bayer8(8, 0));
    assert_eq!(bayer8(3, 5), bayer8(11, 13));

    // Dither test checks
    assert!(!dither_test(0, 0, 0.0));
    assert!(dither_test(0, 0, 1.0));
}

#[test]
fn test_erf_approximation_accuracy() {
    assert_eq!(erf_approx(0.0), 0.0);
    assert!((erf_approx(1.0) - 0.8427).abs() < 1e-3);
    assert!((erf_approx(-1.0) - -0.8427).abs() < 1e-3);
    assert!((erf_approx(3.0) - 0.9999).abs() < 1e-3);
}

#[test]
fn test_sd_rounded_rect_and_gaussian_shadow() {
    let half_size = Point::new(50.0, 50.0);
    let radii = (8.0, 8.0, 8.0, 8.0);

    // Center is inside
    let d_center = sd_rounded_rect(Point::new(0.0, 0.0), half_size, radii);
    assert!(d_center < 0.0);

    // On straight edge
    let d_edge = sd_rounded_rect(Point::new(50.0, 0.0), half_size, radii);
    assert!((d_edge - 0.0).abs() < 1e-3);

    // Outside
    let d_outside = sd_rounded_rect(Point::new(60.0, 0.0), half_size, radii);
    assert!((d_outside - 10.0).abs() < 1e-3);

    // Gaussian shadow alpha decreases with distance
    let a_close = gaussian_shadow_alpha(0.0, 4.0);
    let a_far = gaussian_shadow_alpha(10.0, 4.0);
    assert!(a_close > a_far);
    assert!((a_close - 0.5).abs() < 1e-3);
}
