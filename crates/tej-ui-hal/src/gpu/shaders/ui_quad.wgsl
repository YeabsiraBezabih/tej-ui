struct ViewportUniform {
    viewport_size: vec2<f32>,
    scale_factor: f32,
    _pad: f32,
};

struct QuadInstance {
    bounds: vec4<f32>,
    clip_bounds: vec4<f32>,
    fill_color: vec4<f32>,
    border_color: vec4<f32>,
    corner_radii: vec4<f32>,
    border_widths: vec4<f32>,
    clip_radii: vec4<f32>,
    prim_meta: vec4<u32>,
};

@group(0) @binding(0) var<uniform> viewport: ViewportUniform;
@group(0) @binding(1) var<storage, read> instances: array<QuadInstance>;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) local_pos: vec2<f32>,
    @location(1) rect_size: vec2<f32>,
    @location(2) world_pos: vec2<f32>,
    @location(3) @interpolate(flat) instance_idx: u32,
};

@vertex
fn vs_main(
    @builtin(vertex_index) v_idx: u32,
    @builtin(instance_index) i_idx: u32
) -> VertexOutput {
    let inst = instances[i_idx];
    let prim_kind = inst.prim_meta.x;

    var expansion: f32 = 1.0;
    if (prim_kind == 2u) {
        expansion = max(1.0, ceil(inst.border_widths.x * 3.0));
    }

    var uv = vec2<f32>(0.0, 0.0);
    if (v_idx == 1u) {
        uv = vec2<f32>(1.0, 0.0);
    } else if (v_idx == 2u) {
        uv = vec2<f32>(0.0, 1.0);
    } else if (v_idx == 3u) {
        uv = vec2<f32>(1.0, 1.0);
    }

    let origin = inst.bounds.xy - vec2<f32>(expansion);
    let size = inst.bounds.zw + vec2<f32>(expansion * 2.0);
    let logical_pos = origin + uv * size;
    let phys_pos = logical_pos * viewport.scale_factor;

    let ndc_x = (phys_pos.x / viewport.viewport_size.x) * 2.0 - 1.0;
    let ndc_y = 1.0 - (phys_pos.y / viewport.viewport_size.y) * 2.0;

    var out: VertexOutput;
    out.clip_position = vec4<f32>(ndc_x, ndc_y, 0.0, 1.0);
    out.local_pos = logical_pos - inst.bounds.xy;
    out.rect_size = inst.bounds.zw;
    out.world_pos = logical_pos;
    out.instance_idx = i_idx;
    return out;
}

fn sd_rounded_box(p: vec2<f32>, half_size: vec2<f32>, radii: vec4<f32>) -> f32 {
    var r: f32 = radii.x;
    if (p.x > 0.0 && p.y < 0.0) {
        r = radii.y;
    } else if (p.x > 0.0 && p.y > 0.0) {
        r = radii.z;
    } else if (p.x < 0.0 && p.y > 0.0) {
        r = radii.w;
    }
    r = min(r, min(half_size.x, half_size.y));
    let q = abs(p) - half_size + vec2<f32>(r);
    return min(max(q.x, q.y), 0.0) + length(max(q, vec2<f32>(0.0))) - r;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let inst = instances[in.instance_idx];
    let prim_kind = inst.prim_meta.x;
    let sf = viewport.scale_factor;

    let half_size = in.rect_size * 0.5;
    let p = in.local_pos - half_size;

    let dist_logical = sd_rounded_box(p, half_size, inst.corner_radii);
    let dist_phys = dist_logical * sf;

    var color = vec4<f32>(0.0);

    if (prim_kind == 0u) {
        let alpha_factor = clamp(0.5 - dist_phys, 0.0, 1.0);
        color = inst.fill_color * alpha_factor;
    } else if (prim_kind == 1u) {
        let alpha_outer = clamp(0.5 - dist_phys, 0.0, 1.0);
        let border_w = inst.border_widths.x;
        let inner_half = max(half_size - vec2<f32>(border_w), vec2<f32>(0.0));
        let inner_radii = max(inst.corner_radii - vec4<f32>(border_w), vec4<f32>(0.0));
        let dist_inner_phys = sd_rounded_box(p, inner_half, inner_radii) * sf;
        let alpha_inner = clamp(0.5 - dist_inner_phys, 0.0, 1.0);

        let border_factor = clamp(alpha_outer - alpha_inner, 0.0, 1.0);
        color = inst.border_color * border_factor + inst.fill_color * alpha_inner;
    } else if (prim_kind == 2u) {
        let blur = max(inst.border_widths.x, 0.5);
        let shadow_alpha = exp(-max(dist_logical, 0.0) / blur);
        color = inst.fill_color * shadow_alpha;
    }

    let clip_half = inst.clip_bounds.zw * 0.5;
    let clip_p = (in.world_pos - inst.clip_bounds.xy) - clip_half;
    let clip_dist_phys = sd_rounded_box(clip_p, clip_half, inst.clip_radii) * sf;
    let clip_alpha = clamp(0.5 - clip_dist_phys, 0.0, 1.0);
    color = color * clip_alpha;

    if (color.a <= 0.0001) {
        discard;
    }

    return color;
}
