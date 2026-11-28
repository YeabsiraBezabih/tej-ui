//! Wavefront OBJ parser for loading Blender exported 3D assets.

use super::suzanne::BlenderMesh;

/// Parses a Wavefront OBJ string (exported by Blender) into a structured `BlenderMesh`.
pub fn parse_obj(content: &str, mesh_name: &str) -> Result<BlenderMesh, String> {
    let mut vertices = Vec::new();
    let mut normals = Vec::new();
    let mut colors = Vec::new();
    let mut triangles = Vec::new();

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let mut parts = line.split_whitespace();
        let Some(tag) = parts.next() else {
            continue;
        };

        match tag {
            "v" => {
                let x: f32 = parts.next().and_then(|s| s.parse().ok()).unwrap_or(0.0);
                let y: f32 = parts.next().and_then(|s| s.parse().ok()).unwrap_or(0.0);
                let z: f32 = parts.next().and_then(|s| s.parse().ok()).unwrap_or(0.0);
                vertices.push([x, y, z]);

                // Parse optional vertex colors (r, g, b in range 0.0 .. 1.0)
                if let (Some(r), Some(g), Some(b)) = (
                    parts.next().and_then(|s| s.parse::<f32>().ok()),
                    parts.next().and_then(|s| s.parse::<f32>().ok()),
                    parts.next().and_then(|s| s.parse::<f32>().ok()),
                ) {
                    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                    let cr = (r.clamp(0.0, 1.0) * 255.0) as u8;
                    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                    let cg = (g.clamp(0.0, 1.0) * 255.0) as u8;
                    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                    let cb = (b.clamp(0.0, 1.0) * 255.0) as u8;
                    colors.push(tej_ui_hal::Color::rgb(cr, cg, cb));
                }
            }
            "vn" => {
                let x: f32 = parts.next().and_then(|s| s.parse().ok()).unwrap_or(0.0);
                let y: f32 = parts.next().and_then(|s| s.parse().ok()).unwrap_or(0.0);
                let z: f32 = parts.next().and_then(|s| s.parse().ok()).unwrap_or(0.0);
                normals.push([x, y, z]);
            }
            "f" => {
                let mut face_indices = Vec::new();
                for part in parts {
                    let vertex_idx_str = part.split('/').next().unwrap_or("");
                    if let Ok(idx) = vertex_idx_str.parse::<usize>()
                        && idx > 0
                    {
                        face_indices.push((idx - 1) as u32);
                    }
                }

                // Triangulate polygons / quads via triangle fan
                if face_indices.len() >= 3 {
                    for i in 1..(face_indices.len() - 1) {
                        triangles.push([face_indices[0], face_indices[i], face_indices[i + 1]]);
                    }
                }
            }
            _ => {}
        }
    }

    if vertices.is_empty() {
        return Err("No vertex positions found in OBJ file".to_string());
    }

    if let Err(e) = crate::nanite::sanitize_mesh_geometry(&vertices, &normals, &triangles) {
        return Err(format!("Mesh sanitization failed for '{mesh_name}': {e}"));
    }

    Ok(BlenderMesh {
        name: mesh_name.to_string(),
        vertices,
        normals,
        colors,
        triangles,
    })
}

/// Loads a Blender OBJ model from a file path.
pub fn load_obj_from_file(path: impl AsRef<std::path::Path>) -> Result<BlenderMesh, String> {
    let path_ref = path.as_ref();
    let content = std::fs::read_to_string(path_ref)
        .map_err(|e| format!("Failed to read OBJ file '{}': {e}", path_ref.display()))?;
    let name = path_ref
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Blender_Mesh");
    parse_obj(&content, name)
}
