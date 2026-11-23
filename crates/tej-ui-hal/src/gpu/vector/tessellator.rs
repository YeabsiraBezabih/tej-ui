//! High-performance fill and stroke vector path tessellator with buffer reuse.

use crate::gpu::types::PremulColor;
use crate::gpu::vector::path::VectorPath;
use bytemuck::{Pod, Zeroable};
use lyon_tessellation::{
    BuffersBuilder, FillOptions, FillRule, FillTessellator, FillVertex, LineCap, LineJoin,
    StrokeOptions, StrokeTessellator, StrokeVertex, VertexBuffers,
};

/// Hardware vertex representation for 2D tessellated geometry.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
pub struct VectorVertex {
    pub position: [f32; 2],
    pub color: [f32; 4],
}

const _: () = assert!(core::mem::size_of::<VectorVertex>() == 24);

/// Reusable GPU geometry mesh holding vertices and triangle indices.
#[derive(Clone, Debug, Default)]
pub struct TessellatedMesh {
    pub vertices: Vec<VectorVertex>,
    pub indices: Vec<u32>,
}

impl TessellatedMesh {
    #[must_use]
    pub fn new() -> Self {
        Self {
            vertices: Vec::with_capacity(1024),
            indices: Vec::with_capacity(3072),
        }
    }

    /// Clears mesh data without dropping vector capacities.
    pub fn clear(&mut self) {
        self.vertices.clear();
        self.indices.clear();
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.indices.is_empty()
    }
}

/// Dynamic path tessellator converting arbitrary curves and SVG paths into GPU triangle meshes.
#[derive(Default)]
pub struct PathTessellator {
    fill_tess: FillTessellator,
    stroke_tess: StrokeTessellator,
}

impl std::fmt::Debug for PathTessellator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PathTessellator").finish()
    }
}

impl PathTessellator {
    #[must_use]
    pub fn new() -> Self {
        Self {
            fill_tess: FillTessellator::new(),
            stroke_tess: StrokeTessellator::new(),
        }
    }

    /// Tessellates a filled path with specified color and winding rule.
    pub fn tessellate_fill(
        &mut self,
        path: &VectorPath,
        color: PremulColor,
        rule: FillRule,
        out_mesh: &mut TessellatedMesh,
    ) {
        let base_index = out_mesh.vertices.len() as u32;
        let mut buffers: VertexBuffers<VectorVertex, u32> = VertexBuffers::new();

        let options = FillOptions::default()
            .with_fill_rule(rule)
            .with_tolerance(0.1);

        let _ = self.fill_tess.tessellate_path(
            path.inner(),
            &options,
            &mut BuffersBuilder::new(&mut buffers, |v: FillVertex<'_>| VectorVertex {
                position: [v.position().x, v.position().y],
                color: color.channels,
            }),
        );

        out_mesh.vertices.extend(buffers.vertices);
        out_mesh
            .indices
            .extend(buffers.indices.into_iter().map(|idx| base_index + idx));
    }

    /// Tessellates a stroked path with specified line width, cap, and join style.
    pub fn tessellate_stroke(
        &mut self,
        path: &VectorPath,
        color: PremulColor,
        line_width: f32,
        out_mesh: &mut TessellatedMesh,
    ) {
        let base_index = out_mesh.vertices.len() as u32;
        let mut buffers: VertexBuffers<VectorVertex, u32> = VertexBuffers::new();

        let options = StrokeOptions::default()
            .with_line_width(line_width)
            .with_line_cap(LineCap::Round)
            .with_line_join(LineJoin::Round)
            .with_tolerance(0.1);

        let _ = self.stroke_tess.tessellate_path(
            path.inner(),
            &options,
            &mut BuffersBuilder::new(&mut buffers, |v: StrokeVertex<'_, '_>| VectorVertex {
                position: [v.position().x, v.position().y],
                color: color.channels,
            }),
        );

        out_mesh.vertices.extend(buffers.vertices);
        out_mesh
            .indices
            .extend(buffers.indices.into_iter().map(|idx| base_index + idx));
    }
}
