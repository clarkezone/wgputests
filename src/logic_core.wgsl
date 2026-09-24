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
    @location(7) material: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) color_emissive: vec4<f32>,
    @location(3) material: vec4<f32>,
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
    output.material = input.material;
    return output;
}

fn distribution_ggx(normal: vec3<f32>, half_direction: vec3<f32>, roughness: f32) -> f32 {
    let alpha = roughness * roughness;
    let alpha_squared = alpha * alpha;
    let normal_half = max(dot(normal, half_direction), 0.0);
    let denominator = normal_half * normal_half * (alpha_squared - 1.0) + 1.0;
    return alpha_squared / max(3.14159265 * denominator * denominator, 0.0001);
}

fn geometry_schlick_ggx(normal_view: f32, roughness: f32) -> f32 {
    let r = roughness + 1.0;
    let k = (r * r) / 8.0;
    return normal_view / max(normal_view * (1.0 - k) + k, 0.0001);
}

fn geometry_smith(
    normal: vec3<f32>,
    view_direction: vec3<f32>,
    light_direction: vec3<f32>,
    roughness: f32,
) -> f32 {
    return geometry_schlick_ggx(max(dot(normal, view_direction), 0.0), roughness)
        * geometry_schlick_ggx(max(dot(normal, light_direction), 0.0), roughness);
}

fn fresnel_schlick(cosine: f32, f0: vec3<f32>) -> vec3<f32> {
    return f0 + (vec3<f32>(1.0) - f0) * pow(1.0 - cosine, 5.0);
}

fn standard_light(
    normal: vec3<f32>,
    view_direction: vec3<f32>,
    light_direction: vec3<f32>,
    radiance: vec3<f32>,
    base_color: vec3<f32>,
    roughness: f32,
    metalness: f32,
) -> vec3<f32> {
    let half_direction = normalize(view_direction + light_direction);
    let normal_light = max(dot(normal, light_direction), 0.0);
    let f0 = mix(vec3<f32>(0.04), base_color, metalness);
    let fresnel = fresnel_schlick(max(dot(half_direction, view_direction), 0.0), f0);
    let distribution = distribution_ggx(normal, half_direction, roughness);
    let geometry = geometry_smith(normal, view_direction, light_direction, roughness);
    let specular = distribution * geometry * fresnel
        / max(4.0 * max(dot(normal, view_direction), 0.0) * normal_light, 0.0001);
    let diffuse = (vec3<f32>(1.0) - fresnel) * (1.0 - metalness)
        * base_color / 3.14159265;
    return (diffuse + specular) * radiance * normal_light;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let normal = normalize(input.normal);
    let view_direction = normalize(scene.camera_position.xyz - input.world_position);
    let base_color = input.color_emissive.rgb;
    let roughness = input.material.x;
    let metalness = input.material.y;
    let directional_direction = normalize(scene.light_direction.xyz);
    let directional = standard_light(
        normal,
        view_direction,
        directional_direction,
        vec3<f32>(0.6),
        base_color,
        roughness,
        metalness,
    );
    let point_position = vec3<f32>(0.0, 1.0, 0.0);
    let to_point = point_position - input.world_position;
    let point_distance = length(to_point);
    let point_direction = to_point / max(point_distance, 0.001);
    let cutoff = clamp(1.0 - point_distance / 25.0, 0.0, 1.0);
    let cyan = vec3<f32>(0.0, 0.9, 1.0);
    let point = standard_light(
        normal,
        view_direction,
        point_direction,
        cyan * scene.light_direction.w * cutoff,
        base_color,
        roughness,
        metalness,
    );
    let ambient = base_color * 0.3;
    let emissive = base_color * input.color_emissive.a;
    let color = ambient + directional + point + emissive;
    return vec4<f32>(color, 1.0);
}
