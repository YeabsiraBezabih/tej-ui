struct ViewportUniform {
    viewport_size: vec2<f32>,
    scale_factor: f32,
    _pad: f32,
};

@group(0) @binding(0) var<uniform> viewport: ViewportUniform;

struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) color: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
};

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    // Scale logical coordinates by display scale_factor before converting to NDC [-1, 1] with Y down
    let phys_pos = in.position * viewport.scale_factor;
    let ndc_x = (phys_pos.x / viewport.viewport_size.x) * 2.0 - 1.0;
    let ndc_y = 1.0 - (phys_pos.y / viewport.viewport_size.y) * 2.0;
    out.clip_position = vec4<f32>(ndc_x, ndc_y, 0.0, 1.0);
    out.color = in.color;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}
