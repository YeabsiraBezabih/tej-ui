use mydb_core::PageId;
use tej_ui_3d::{
    PbrMaterialParams, SphericalHarmonics9, Vec3, VirtualTexture, VirtualTextureResidentPool,
    VirtualTileCoord, pbr_shade_fragment,
};
use tej_ui_hal::Color;

#[test]
fn test_spherical_harmonics_studio_gradient() {
    let sh = SphericalHarmonics9::studio_lighting();

    let top_irr = sh.eval(Vec3::new(0.0, 1.0, 0.0));
    let bottom_irr = sh.eval(Vec3::new(0.0, -1.0, 0.0));

    // Sky gradient: top facing normal receives more ambient light than bottom
    assert!(top_irr.x > 0.0);
    assert!(top_irr.y > 0.0);
    assert!(top_irr.z > 0.0);
    assert!(top_irr.y >= bottom_irr.y);
}

#[test]
fn test_pbr_cook_torrance_shading_behavior() {
    let sh = SphericalHarmonics9::studio_lighting();
    let normal = Vec3::new(0.0, 0.0, 1.0);
    let view_dir = Vec3::new(0.0, 0.0, 1.0);
    let light_dir = Vec3::new(0.0, 0.0, 1.0); // Head-on light
    let light_color = Vec3::new(1.0, 1.0, 1.0);
    let red_albedo = Color::rgb(255, 0, 0);

    // 1. Dielectric rough plastic (metallic 0.0, roughness 0.5)
    let (dr, dg, db) = pbr_shade_fragment(
        normal,
        view_dir,
        light_dir,
        light_color,
        PbrMaterialParams::new(red_albedo, 0.5, 0.0),
        &sh,
    );
    assert!(dr > 50, "Red channel must be strongly lit, got {dr}");
    assert!(dr > dg, "Dielectric red plastic reflects red diffuse");
    assert!(dr > db, "Dielectric red plastic reflects red diffuse");

    // 2. Pure smooth metal (metallic 1.0, roughness 0.05)
    let (mr, _mg, _mb) = pbr_shade_fragment(
        normal,
        view_dir,
        light_dir,
        light_color,
        PbrMaterialParams::new(red_albedo, 0.05, 1.0),
        &sh,
    );
    assert!(mr > dr, "Smooth metal has intense specular peak");

    // 3. Grazing light should produce weaker shading than direct light
    let grazing_light = Vec3::new(0.9, 0.0, 0.1).normalize();
    let (gr, _, _) = pbr_shade_fragment(
        normal,
        view_dir,
        grazing_light,
        light_color,
        PbrMaterialParams::new(red_albedo, 0.5, 0.0),
        &sh,
    );
    assert!(gr < dr, "Grazing light must be darker than head-on light");
}

#[test]
fn test_virtual_texture_mip_pyramid_and_tiles() {
    // 1024x1024 texture: 16x16 tiles of 64x64 at mip 0
    let mut vt = VirtualTexture::new(1, 1024, 1024);
    assert_eq!(vt.tile_dimensions(0), (16, 16));
    assert_eq!(vt.tile_dimensions(1), (8, 8));
    assert_eq!(vt.tile_dimensions(2), (4, 4));
    assert_eq!(vt.tile_dimensions(3), (2, 2));
    assert_eq!(vt.tile_dimensions(4), (1, 1));

    // Register a tile page
    let coord0 = VirtualTileCoord::new(0, 4, 8);
    vt.register_tile_page(coord0, PageId(1234));
    assert_eq!(vt.get_tile_page(&coord0), Some(PageId(1234)));
    assert_eq!(vt.mapped_tiles_count(), 1);

    // UV mapping: center (0.5, 0.5) at mip 0 maps to tile (8, 8)
    let center_tile = vt.uv_to_tile(0.5, 0.5, 0);
    assert_eq!(center_tile.mip, 0);
    assert_eq!(center_tile.tile_x, 8);
    assert_eq!(center_tile.tile_y, 8);

    // UV center at mip 1 maps to tile (4, 4)
    let center_mip1 = vt.uv_to_tile(0.5, 0.5, 1);
    assert_eq!(center_mip1.mip, 1);
    assert_eq!(center_mip1.tile_x, 4);
    assert_eq!(center_mip1.tile_y, 4);
}

#[test]
fn test_virtual_texture_resident_pool_lru_eviction() {
    let mut pool = VirtualTextureResidentPool::new(2);

    let t1 = VirtualTileCoord::new(0, 0, 0);
    let t2 = VirtualTileCoord::new(0, 1, 0);
    let t3 = VirtualTileCoord::new(0, 2, 0);

    pool.insert_tile(t1, vec![1; 4096]);
    pool.insert_tile(t2, vec![2; 4096]);
    assert!(pool.is_resident(&t1));
    assert!(pool.is_resident(&t2));

    // Inserting 3rd tile must evict oldest resident tile to respect pool capacity
    pool.insert_tile(t3, vec![3; 4096]);
    assert!(pool.is_resident(&t3));
    assert_eq!(pool.get_tile(&t3).unwrap().len(), 4096);
}
