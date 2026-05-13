struct QuadVert {
    @location(0) local_pos: vec2<f32>,
}

struct Instance {
    @location(1) pos:        vec2<f32>,
    @location(2) life_ratio: f32,
    @location(3) size:       f32,
    @location(4) heat:       f32,
}

struct Uniforms {
    aspect: f32,
    _pad0:  f32,
    _pad1:  f32,
    _pad2:  f32,
}

@group(0) @binding(0) var<uniform> u: Uniforms;

struct VOut {
    @builtin(position) pos:  vec4<f32>,
    @location(0)       uv:   vec2<f32>,
    @location(1)       life: f32,
    @location(2)       heat: f32,
}

@vertex
fn vs_main(vert: QuadVert, inst: Instance) -> VOut {
    let pixel_size = inst.size * 0.003;
    let offset = vert.local_pos * vec2<f32>(pixel_size / u.aspect, pixel_size);

    var out: VOut;
    out.pos  = vec4<f32>(inst.pos + offset, 0.0, 1.0);
    out.uv   = vert.local_pos;
    out.life = inst.life_ratio;
    out.heat = inst.heat;
    return out;
}

@fragment
fn fs_main(in: VOut) -> @location(0) vec4<f32> {
    let d = length(in.uv);
    if d > 1.0 { discard; }

    let core  = max(0.0, 1.0 - d * 1.8);
    let halo  = max(0.0, 1.0 - d) * 0.5;
    let alpha = (core * core + halo) * in.life;

    let cool = vec3<f32>(0.25, 0.0, 0.0);
    let warm = vec3<f32>(0.85, 0.05, 0.0);
    let hot  = vec3<f32>(1.0,  0.7,  0.3);

    var col = mix(cool, warm, in.life);
    col     = mix(col,  hot,  in.heat * in.life);
    col    *= (core + halo);

    return vec4<f32>(col, alpha);
}
