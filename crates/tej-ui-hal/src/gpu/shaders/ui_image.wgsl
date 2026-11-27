struct ViewportUniform {
    viewport_size: vec2<f32>,
    scale_factor: f32,
    _pad: f32,
};

struct ImageInstance {
    bounds: vec4<f32>,
    uv_bounds: vec4<f32>,
    clip_bounds: vec4<f32>,
    params: vec4<f32>,
};

@group(0) @binding(0) var<uniform> viewport: ViewportUniform;
@group(0) @binding(1) var<storage, read> images: array<ImageInstance>;

@group(1) @binding(0) var image_texture: texture_2d<f32>;
@group(1) @binding(1) var image_sampler: sampler;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) local_pos: vec2<f32>,
    @location(2) rect_size: vec2<f32>,
    @location(3) world_pos: vec2<f32>,
    @location(4) clip_bounds: vec4<f32>,
    @location(5) params: vec4<f32>,
};

@vertex
fn vs_main(
    @builtin(vertex_index) v_idx: u32,
    @builtin(instance_index) i_idx: u32
) -> VertexOutput {
    let img = images[i_idx];

    var uv_unit = vec2<f32>(0.0, 0.0);
    if (v_idx == 1u) {
        uv_unit = vec2<f32>(1.0, 0.0);
    } else if (v_idx == 2u) {
        uv_unit = vec2<f32>(0.0, 1.0);
    } else if (v_idx == 3u) {
        uv_unit = vec2<f32>(1.0, 1.0);
    }

    let logical_pos = img.bounds.xy + uv_unit * img.bounds.zw;
    let phys_pos = logical_pos * viewport.scale_factor;

    let ndc_x = (phys_pos.x / viewport.viewport_size.x) * 2.0 - 1.0;
    let ndc_y = 1.0 - (phys_pos.y / viewport.viewport_size.y) * 2.0;

    let uv = mix(img.uv_bounds.xy, img.uv_bounds.zw, uv_unit);

    var out: VertexOutput;
    out.clip_position = vec4<f32>(ndc_x, ndc_y, 0.0, 1.0);
    out.uv = uv;
    out.local_pos = uv_unit * img.bounds.zw;
    out.rect_size = img.bounds.zw;
    out.world_pos = logical_pos;
    out.clip_bounds = img.clip_bounds;
    out.params = img.params;
    return out;
}

fn sd_rounded_box_uniform(p: vec2<f32>, half_size: vec2<f32>, radius: f32) -> f32 {
    let r = min(radius, min(half_size.x, half_size.y));
    let q = abs(p) - half_size + vec2<f32>(r);
    return min(max(q.x, q.y), 0.0) + length(max(q, vec2<f32>(0.0))) - r;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let texel = textureSample(image_texture, image_sampler, in.uv);

    let cb = in.clip_bounds;
    if (in.world_pos.x < cb.x || in.world_pos.x > (cb.x + cb.z) ||
        in.world_pos.y < cb.y || in.world_pos.y > (cb.y + cb.w)) {
        discard;
    }

    let opacity = clamp(in.params.x, 0.0, 1.0);
    let corner_radius = max(in.params.y, 0.0);

    var mask: f32 = 1.0;
    if (corner_radius > 0.0) {
        let half_size = in.rect_size * 0.5;
        let p = in.local_pos - half_size;
        let dist_phys = sd_rounded_box_uniform(p, half_size, corner_radius) * viewport.scale_factor;
        mask = clamp(0.5 - dist_phys, 0.0, 1.0);
    }

    let alpha = texel.a * opacity * mask;
    if (alpha <= 0.0001) {
        discard;
    }

    return vec4<f32>(texel.rgb * alpha, alpha);
}
