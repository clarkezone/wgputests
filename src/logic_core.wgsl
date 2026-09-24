struct SceneUniforms {
    view_projection: mat4x4<f32>,
    light_direction: vec4<f32>,
    camera_position: vec4<f32>,
};

@group(0) @binding(0)
var<uniform> scene: SceneUniforms;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) model_0: vec4<f32>,
    @location(2) model_1: vec4<f32>,
    @location(3) model_2: vec4<f32>,
    @location(4) model_3: vec4<f32>,
    @location(5) color_emissive: vec4<f32>,
    @location(6) normal: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) color_emissive: vec4<f32>,
};

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    let model = mat4x4<f32>(input.model_0, input.model_1, input.model_2, input.model_3);
    let world = model * vec4<f32>(input.position, 1.0);
    var output: VertexOutput;
    output.clip_position = scene.view_projection * world;
    output.world_position = world.xyz;
    output.normal = normalize((model * vec4<f32>(input.normal, 0.0)).xyz);
    output.color_emissive = input.color_emissive;
    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let normal = normalize(input.normal);
    let light = normalize(scene.light_direction.xyz);
    let diffuse = max(dot(normal, light), 0.0);
    let view_direction = normalize(scene.camera_position.xyz - input.world_position);
    let rim = pow(1.0 - max(dot(normal, view_direction), 0.0), 2.4);
    let distance_to_core = length(input.world_position);
    let core_light = 1.0 / (1.0 + distance_to_core * distance_to_core * 0.035);
    let lit = 0.3 + diffuse * 0.6 + core_light * 0.22;
    let emissive = input.color_emissive.a;
    let cyan = vec3<f32>(0.0, 0.9, 1.0);
    let color = input.color_emissive.rgb * lit
        + input.color_emissive.rgb * emissive
        + cyan * rim * (0.05 + emissive * 0.35);
    return vec4<f32>(color, 1.0);
}
