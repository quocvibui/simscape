struct Uniforms {
    time: f32,
    amplitude: f32,
    resolution: vec2<f32>,
    color: vec4<f32>,
};

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

struct VertexOutput {
    @builtin(position) pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) ix: u32) -> VertexOutput {
    var out: VertexOutput;
    let amp = uniforms.amplitude;
    let t = uniforms.time;

    let tri = ix / 3u;          // which triangle (0..59)
    let vert = ix % 3u;         // which vertex in that triangle (0, 1, 2)
    let total = 60.0;
    let angle = (f32(tri) / total) * 6.28318 + t * 0.5;

    // triangle shape: center point + two outer points
    var x: f32;
    var y: f32;
    if (vert == 0u) {
        x = 0.0;
        y = 0.0;
    } else {
        let spread = 0.05;
        let offset = f32(vert) * spread - spread * 0.5;
        let r = 0.3 + amp * 2.0; // audio controls reach
        x = cos(angle + offset) * r;
        y = sin(angle + offset) * r;
    }

    // aspect ratio correction
    x *= uniforms.resolution.y / uniforms.resolution.x;

    out.pos = vec4<f32>(x, y, 0.0, 1.0);
    out.uv = vec2<f32>(x, y) * 0.5 + 0.5;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let t = uniforms.time;
    let uv = in.uv;

    // trippy animated color pattern
    let amp = uniforms.amplitude; // audio in
    let r = sin(uv.x * (10.0 + amp * 30.0) + t) * 0.5 + 0.5;
    let g = sin(uv.y * (10.0 + amp * 30.0) + t * 1.3) * 0.5 + 0.5;
    let b = sin((uv.x + uv.y) * (8.0 + amp * 20.0) + t * 0.7) * 0.5 + 0.5;

    return vec4<f32>(r * uniforms.color.r, g * uniforms.color.g, b * uniforms.color.b, 1.0);
}
