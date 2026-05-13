struct Uniforms {
    time:   f32,
    width:  f32,
    height: f32,
    _pad:   f32,
}

@group(0) @binding(0)
var<uniform> u: Uniforms;

@vertex
fn vs_main(@builtin(vertex_index) vi: u32) -> @builtin(position) vec4<f32> {
    let x = f32((vi & 1u) * 2u) - 1.0;
    let y = f32((vi >> 1u) * 2u) - 1.0;
    return vec4<f32>(x, y, 0.0, 1.0);
}

fn hash(p: vec2<f32>) -> f32 {
    return fract(sin(dot(p, vec2<f32>(127.1, 311.7))) * 43758.5453);
}

fn noise(p: vec2<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);
    let u = f * f * (3.0 - 2.0 * f);
    return mix(
        mix(hash(i + vec2<f32>(0.0, 0.0)), hash(i + vec2<f32>(1.0, 0.0)), u.x),
        mix(hash(i + vec2<f32>(0.0, 1.0)), hash(i + vec2<f32>(1.0, 1.0)), u.x),
        u.y,
    );
}

@fragment
fn fs_main(@builtin(position) pos: vec4<f32>) -> @location(0) vec4<f32> {
    let uv = pos.xy / vec2<f32>(u.width, u.height);
    let t  = u.time * 0.4;

    // Animated noise layers
    var n = noise(uv * 4.0 + t * 0.3);
    n    += noise(uv * 8.0 - t * 0.5) * 0.5;
    n    += noise(uv * 16.0 + t * 0.7) * 0.25;
    n     = n / 1.75;

    // Radial vignette
    let center = uv - 0.5;
    let vignette = 1.0 - dot(center, center) * 2.0;

    // Red pulse on center
    let dist = length(center);
    let pulse = sin(t * 2.0 - dist * 12.0) * 0.5 + 0.5;
    let glow  = pulse * max(0.0, 1.0 - dist * 3.5) * 0.6;

    let red   = vec3<f32>(0.8, 0.0, 0.0);
    let dark  = vec3<f32>(0.03, 0.0, 0.0);
    var col   = mix(dark, red, n * vignette);
    col      += red * glow;
    col       = clamp(col, vec3<f32>(0.0), vec3<f32>(1.0));

    return vec4<f32>(col, 1.0);
}
