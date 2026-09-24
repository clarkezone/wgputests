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
    let directional_light = normalize(scene.light_direction.xyz);
    let directional_diffuse = max(dot(normal, directional_light), 0.0);
    let view_direction = normalize(scene.camera_position.xyz - input.world_position);
    let directional_half = normalize(directional_light + view_direction);
    let directional_specular = pow(max(dot(normal, directional_half), 0.0), 32.0);

    let point_position = vec3<f32>(0.0, 1.0, 0.0);
    let to_point = point_position - input.world_position;
    let point_distance = length(to_point);
    let point_direction = to_point / max(point_distance, 0.001);
    let point_diffuse = max(dot(normal, point_direction), 0.0);
    let point_half = normalize(point_direction + view_direction);
    let point_specular = pow(max(dot(normal, point_half), 0.0), 48.0);
    let attenuation = 1.0 / (1.0 + point_distance * point_distance * 0.14);

    let rim = pow(1.0 - max(dot(normal, view_direction), 0.0), 2.5);
    let emissive = input.color_emissive.a;
    let cyan = vec3<f32>(0.0, 0.9, 1.0);
    let reflection = smoothstep(0.008, 0.02, max(
        input.color_emissive.r,
        max(input.color_emissive.g, input.color_emissive.b),
    ));
    let base_lighting = 0.18 + directional_diffuse * 0.62;
    let white_highlight = vec3<f32>(directional_specular * (0.18 + reflection * 0.12));
    let cyan_light = cyan
        * attenuation
        * (point_diffuse * 0.45 + point_specular * 0.55)
        * (1.0 + reflection * 0.18);
    let color = input.color_emissive.rgb * base_lighting
        + white_highlight
        + cyan_light
        + input.color_emissive.rgb * emissive
        + cyan * rim * (0.025 + emissive * 0.25 + reflection * 0.04);
    return vec4<f32>(color, 1.0);
}
