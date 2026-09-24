struct SceneUniforms {
    view_projection: mat4x4<f32>,
    group_model: mat4x4<f32>,
    viewport_brightness: vec4<f32>,
};

@group(0) @binding(0)
var<uniform> scene: SceneUniforms;

struct ParticleInput {
    @location(0) corner: vec2<f32>,
    @location(1) position_size: vec4<f32>,
    @location(2) color_softness: vec4<f32>,
};

struct ParticleOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) local: vec2<f32>,
    @location(1) color_softness: vec4<f32>,
};

@vertex
fn particle_vs(input: ParticleInput) -> ParticleOutput {
    var clip = scene.view_projection
        * scene.group_model
        * vec4<f32>(input.position_size.xyz, 1.0);
    let pixel_offset = input.corner
        * input.position_size.w
        * 2.0
        / scene.viewport_brightness.xy;
    clip.x += pixel_offset.x * clip.w;
    clip.y += pixel_offset.y * clip.w;

    var output: ParticleOutput;
    output.clip_position = clip;
    output.local = input.corner;
    output.color_softness = input.color_softness;
    return output;
}

@fragment
fn particle_fs(input: ParticleOutput) -> @location(0) vec4<f32> {
    let distance = length(input.local);
    if distance > 1.0 {
        discard;
    }
    let hard_circle = 1.0 - smoothstep(0.72, 1.0, distance);
    let soft_halo = pow(max(1.0 - distance, 0.0), 2.2);
    let alpha = mix(soft_halo, hard_circle, input.color_softness.a);
    let color = input.color_softness.rgb * scene.viewport_brightness.z * alpha;
    return vec4<f32>(color, alpha);
}

struct LineInput {
    @location(0) position: vec3<f32>,
};

struct LineOutput {
    @builtin(position) clip_position: vec4<f32>,
};

@vertex
fn line_vs(input: LineInput) -> LineOutput {
    var output: LineOutput;
    output.clip_position = scene.view_projection
        * scene.group_model
        * vec4<f32>(input.position, 1.0);
    return output;
}

@fragment
fn line_fs() -> @location(0) vec4<f32> {
    return vec4<f32>(0.31, 0.12, 0.82, 0.42) * scene.viewport_brightness.z;
}
