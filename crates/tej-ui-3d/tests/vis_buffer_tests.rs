use tej_ui_3d::{
    EdgeEquation, ResolvedTriangle, ShadingContext, TileBounds, TileGrid, TileVisBuffer, Vec3,
    VisBuffer, VisPixel, resolve_tile, shade_fragment,
};
use tej_ui_hal::Color;

#[test]
fn test_vis_pixel_packing_and_unpacking() {
    let depth = 0.75_f32;
    let slot = 123_456_u32;
    let tri_id = 2_047_u16;

    let pixel = VisPixel::pack(depth, slot, tri_id);
    assert!(!pixel.is_empty());
    assert!((pixel.depth() - depth).abs() < 1e-6);
    assert_eq!(pixel.meshlet_slot(), slot);
    assert_eq!(pixel.tri_id(), tri_id);

    // Empty pixel invariant
    let empty = VisPixel::EMPTY;
    assert!(empty.is_empty());
    assert_eq!(empty.depth(), 0.0);
    assert_eq!(empty.meshlet_slot(), 0);
    assert_eq!(empty.tri_id(), 0);
}

#[test]
fn test_vis_pixel_bounds_and_clamping() {
    // 20-bit slot mask boundary
    let max_slot = VisPixel::SLOT_MASK as u32;
    let max_tri = VisPixel::TRI_MASK as u16;
    let pixel = VisPixel::pack(1.0, max_slot, max_tri);
    assert_eq!(pixel.meshlet_slot(), max_slot);
    assert_eq!(pixel.tri_id(), max_tri);
    assert!((pixel.depth() - 1.0).abs() < 1e-6);

    // Overflow bits must be masked cleanly
    let overflow_slot = max_slot + 100;
    let overflow_tri = max_tri + 50;
    let clamped_pixel = VisPixel::pack(0.5, overflow_slot, overflow_tri);
    assert_eq!(clamped_pixel.meshlet_slot(), overflow_slot & 0xF_FFFF);
    assert_eq!(clamped_pixel.tri_id(), overflow_tri & 0x0FFF);

    // Negative and NaN depths clamp to 0.0
    let nan_pixel = VisPixel::pack(f32::NAN, 1, 1);
    assert_eq!(nan_pixel.depth(), 0.0);
    let neg_pixel = VisPixel::pack(-0.5, 1, 1);
    assert_eq!(neg_pixel.depth(), 0.0);
    let over_pixel = VisPixel::pack(2.5, 1, 1);
    assert_eq!(over_pixel.depth(), 1.0);
}

#[test]
fn test_vis_pixel_reversed_z_update() {
    let mut pixel = VisPixel::pack(0.4, 10, 1);

    // Farther fragment in Reversed-Z (smaller depth) must NOT overwrite
    let updated_farther = pixel.update_if_closer(0.2, 20, 2);
    assert!(!updated_farther);
    assert_eq!(pixel.meshlet_slot(), 10);
    assert_eq!(pixel.tri_id(), 1);
    assert!((pixel.depth() - 0.4).abs() < 1e-6);

    // Closer fragment in Reversed-Z (larger depth) MUST overwrite
    let updated_closer = pixel.update_if_closer(0.85, 30, 3);
    assert!(updated_closer);
    assert_eq!(pixel.meshlet_slot(), 30);
    assert_eq!(pixel.tri_id(), 3);
    assert!((pixel.depth() - 0.85).abs() < 1e-6);
}

#[test]
fn test_tile_grid_and_bounds() {
    let grid = TileGrid::new(40, 25);
    // 40 / 16 = 3 tiles x (0..15, 16..31, 32..39)
    // 25 / 16 = 2 tiles y (0..15, 16..24)
    assert_eq!(grid.tiles_x, 3);
    assert_eq!(grid.tiles_y, 2);
    assert_eq!(grid.total_tiles(), 6);

    let t00 = grid.tile_bounds(0, 0).unwrap();
    assert_eq!(
        t00,
        TileBounds {
            min_x: 0,
            max_x: 15,
            min_y: 0,
            max_y: 15
        }
    );
    assert_eq!(t00.width(), 16);
    assert_eq!(t00.height(), 16);

    // Clamped boundary tile
    let t21 = grid.tile_bounds(2, 1).unwrap();
    assert_eq!(
        t21,
        TileBounds {
            min_x: 32,
            max_x: 39,
            min_y: 16,
            max_y: 24
        }
    );
    assert_eq!(t21.width(), 8);
    assert_eq!(t21.height(), 9);

    // Triangle tile bounding box clamp
    let range = grid.clamp_bbox_to_tiles(5.0, 10.0, 35.0, 20.0).unwrap();
    assert_eq!(range, (0, 0, 2, 1));

    // Outside viewport bbox returns None
    assert!(grid.clamp_bbox_to_tiles(-20.0, -20.0, -5.0, -5.0).is_none());
}

#[test]
fn test_tile_vis_buffer_and_global_blit() {
    let mut tile = TileVisBuffer::new();
    assert_eq!(size_of::<TileVisBuffer>(), 2048); // Exactly 2 KB L1 cache size

    tile.update_if_closer(4, 5, 0.9, 42, 7);
    assert_eq!(tile.get_pixel(4, 5).meshlet_slot(), 42);
    assert_eq!(tile.get_pixel(4, 5).tri_id(), 7);

    let mut global = VisBuffer::new(32, 32);
    let bounds = TileBounds {
        min_x: 16,
        max_x: 31,
        min_y: 16,
        max_y: 31,
    };
    tile.blit_to_global(bounds, &mut global);

    let blitted = global.get_pixel(16 + 4, 16 + 5);
    assert_eq!(blitted.meshlet_slot(), 42);
    assert_eq!(blitted.tri_id(), 7);
    assert!((blitted.depth() - 0.9).abs() < 1e-6);
}

#[test]
fn test_edge_equation_incremental_stepping() {
    // Edge from (0, 0) to (10, 0): line y = 0
    let edge = EdgeEquation::from_points(0.0, 0.0, 10.0, 0.0);
    assert_eq!(edge.eval(5.0, 0.0), 0.0);

    // Step in X
    let mut val = edge.eval(0.0, 2.0);
    val += edge.step_x();
    assert_eq!(val, edge.eval(1.0, 2.0));

    // Step in Y
    let mut val_y = edge.eval(3.0, 0.0);
    val_y += edge.step_y();
    assert_eq!(val_y, edge.eval(3.0, 1.0));
}

#[test]
fn test_shade_fragment_shading() {
    let tri = ResolvedTriangle::new(
        [(0.0, 0.0, 1.0), (10.0, 0.0, 1.0), (0.0, 10.0, 1.0)],
        [
            Vec3::new(0.0, 0.0, 1.0),
            Vec3::new(0.0, 0.0, 1.0),
            Vec3::new(0.0, 0.0, 1.0),
        ],
        [
            Color::rgb(200, 100, 50),
            Color::rgb(200, 100, 50),
            Color::rgb(200, 100, 50),
        ],
    );
    let ctx = ShadingContext::default();
    let shaded = shade_fragment(2.0, 2.0, &tri, &ctx);
    assert!(shaded.is_some());
    let color = shaded.unwrap();
    assert_eq!((color >> 24) & 0xFF, 0xFF); // Alpha 255
}

#[test]
fn test_deferred_resolve_tile_zero_overdraw() {
    let width = 16;
    let height = 16;
    let mut vis_buffer = VisBuffer::new(width, height);

    // Pass 1: Write far triangle (slot 0, tri 0) at depth 0.3
    // Then write near triangle (slot 1, tri 0) at depth 0.8 on top
    for y in 2..8 {
        for x in 2..8 {
            vis_buffer.update_pixel(x, y, 0.3, 0, 0);
            vis_buffer.update_pixel(x, y, 0.8, 1, 0);
        }
    }

    let tri_far = ResolvedTriangle::new(
        [(0.0, 0.0, 1.0), (16.0, 0.0, 1.0), (0.0, 16.0, 1.0)],
        [
            Vec3::new(0.0, 0.0, 1.0),
            Vec3::new(0.0, 0.0, 1.0),
            Vec3::new(0.0, 0.0, 1.0),
        ],
        [
            Color::rgb(255, 0, 0),
            Color::rgb(255, 0, 0),
            Color::rgb(255, 0, 0),
        ],
    );
    let tri_near = ResolvedTriangle::new(
        [(0.0, 0.0, 1.0), (16.0, 0.0, 1.0), (0.0, 16.0, 1.0)],
        [
            Vec3::new(0.0, 0.0, 1.0),
            Vec3::new(0.0, 0.0, 1.0),
            Vec3::new(0.0, 0.0, 1.0),
        ],
        [
            Color::rgb(0, 255, 0),
            Color::rgb(0, 255, 0),
            Color::rgb(0, 255, 0),
        ],
    );

    let mut color_buffer = vec![0u32; width * height];
    let bounds = TileBounds {
        min_x: 0,
        max_x: 15,
        min_y: 0,
        max_y: 15,
    };
    let ctx = ShadingContext {
        specular_strength: 0.0, // Disable specular to test pure color channel contribution
        ..Default::default()
    };

    resolve_tile(
        bounds,
        &vis_buffer,
        &|slot, _tri| {
            if slot == 0 {
                Some(tri_far)
            } else {
                Some(tri_near)
            }
        },
        &ctx,
        &mut color_buffer,
    );

    // Verify the winning pixels are green (near triangle), not red (far triangle)
    let p = color_buffer[4 * width + 4];
    let r = (p >> 16) & 0xFF;
    let g = (p >> 8) & 0xFF;
    assert_eq!(r, 0, "Red channel must be zero (far triangle occluded)");
    assert!(g > 100, "Green channel must be lit from near triangle");
}
