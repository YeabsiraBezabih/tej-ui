struct ViewportUniform {
    viewport_size: vec2<f32>,
    scale_factor: f32,
    _pad: f32,
};

struct GlyphInstance {
    bounds: vec4<f32>,
    uv_bounds: vec4<f32>,
    color: vec4<f32>,
    clip_bounds: vec4<f32>,
    prim_meta: vec4<u32>,
};

@group(0) @binding(0) var<uniform> viewport: ViewportUniform;
@group(0) @binding(1) var<storage, read> glyphs: array<GlyphInstance>;

@group(1) @binding(0) var atlas_texture: texture_2d<f32>;
@group(1) @binding(1) var atlas_sampler: sampler;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) color: vec4<f32>,
    @location(2) world_pos: vec2<f32>,
    @location(3) clip_bounds: vec4<f32>,
    @location(4) @interpolate(flat) is_color: u32,
};

@vertex
fn vs_main(
    @builtin(vertex_index) v_idx: u32,
    @builtin(instance_index) i_idx: u32
) -> VertexOutput {
    let glyph = glyphs[i_idx];

    var uv_unit = vec2<f32>(0.0, 0.0);
    if (v_idx == 1u) {
        uv_unit = vec2<f32>(1.0, 0.0);
    } else if (v_idx == 2u) {
        uv_unit = vec2<f32>(0.0, 1.0);
    } else if (v_idx == 3u) {
        uv_unit = vec2<f32>(1.0, 1.0);
    }

    let logical_pos = glyph.bounds.xy + uv_unit * glyph.bounds.zw;
    let phys_pos = logical_pos * viewport.scale_factor;

    let ndc_x = (phys_pos.x / viewport.viewport_size.x) * 2.0 - 1.0;
    let ndc_y = 1.0 - (phys_pos.y / viewport.viewport_size.y) * 2.0;

    let uv = mix(glyph.uv_bounds.xy, glyph.uv_bounds.zw, uv_unit);

    var out: VertexOutput;
    out.clip_position = vec4<f32>(ndc_x, ndc_y, 0.0, 1.0);
    out.uv = uv;
    out.color = glyph.color;
    out.world_pos = logical_pos;
    out.clip_bounds = glyph.clip_bounds;
    out.is_color = glyph.prim_meta.x;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var out_color = vec4<f32>(0.0);

    if (in.is_color == 1u) {
        out_color = textureSample(atlas_texture, atlas_sampler, in.uv);
    } else {
        let alpha = textureSample(atlas_texture, atlas_sampler, in.uv).r;
        out_color = in.color * alpha;
    }

    // Clip rect test
    let cb = in.clip_bounds;
    if (in.world_pos.x < cb.x || in.world_pos.x > (cb.x + cb.z) ||
        in.world_pos.y < cb.y || in.world_pos.y > (cb.y + cb.w)) {
        discard;
    }

    if (out_color.a <= 0.0001) {
        discard;
    }

    return out_color;
}
