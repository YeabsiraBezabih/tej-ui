//! Enterprise asset security, integrity checks, and zero-panic geometry sanitization.
//!
//! Validates 3D models at ingestion boundaries to prevent out-of-bounds reads, NaN traps,
//! and malicious memory exploitation before geometry reaches hot rasterization loops.

use thiserror::Error;

/// Errors arising from malformed or corrupted 3D geometry payloads.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum MeshSanitizerError {
    #[error("Non-finite vertex coordinate detected at index {0}: [{1}, {2}, {3}]")]
    NonFiniteCoordinate(usize, String, String, String),

    #[error("Triangle index {0} exceeds vertex buffer length {1}")]
    OutOfBoundsIndex(u32, usize),

    #[error("Degenerate triangle with zero or negative area at triangle index {0}")]
    DegenerateTriangle(usize),

    #[error("Zero-length or non-finite normal vector at index {0}")]
    InvalidNormal(usize),

    #[error("Empty mesh payload containing zero vertices or zero triangles")]
    EmptyMesh,
}

/// Sanitizes an incoming raw mesh payload, verifying bounds and numeric validity.
///
/// Ensures hot loops in the rasterizer and Nanite streamer can execute safely
/// without runtime panics or floating-point traps.
pub fn sanitize_mesh_geometry(
    vertices: &[[f32; 3]],
    normals: &[[f32; 3]],
    triangles: &[[u32; 3]],
) -> Result<(), MeshSanitizerError> {
    if vertices.is_empty() || triangles.is_empty() {
        return Err(MeshSanitizerError::EmptyMesh);
    }

    let vertex_count = vertices.len();

    // 1. Verify vertex coordinates are finite numbers
    for (i, v) in vertices.iter().enumerate() {
        if !v[0].is_finite() || !v[1].is_finite() || !v[2].is_finite() {
            return Err(MeshSanitizerError::NonFiniteCoordinate(
                i,
                v[0].to_string(),
                v[1].to_string(),
                v[2].to_string(),
            ));
        }
    }

    // 2. Verify normals are finite if present
    for (i, n) in normals.iter().enumerate() {
        if !n[0].is_finite() || !n[1].is_finite() || !n[2].is_finite() {
            return Err(MeshSanitizerError::InvalidNormal(i));
        }
    }

    // 3. Verify triangle indices are strictly within bounds
    for (tri_idx, tri) in triangles.iter().enumerate() {
        for &idx in tri {
            if (idx as usize) >= vertex_count {
                return Err(MeshSanitizerError::OutOfBoundsIndex(idx, vertex_count));
            }
        }

        // Check for degenerate single-point or folded edge triangles
        if tri[0] == tri[1] || tri[1] == tri[2] || tri[2] == tri[0] {
            return Err(MeshSanitizerError::DegenerateTriangle(tri_idx));
        }
    }

    Ok(())
}
