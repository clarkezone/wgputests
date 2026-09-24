struct Beam {
    start_radius: vec4<f32>,
    end_energy: vec4<f32>,
    color: vec4<f32>,
};

struct Scene {
    resolution_time: vec4<f32>,
    camera_frame: vec4<f32>,
    vertices: array<vec4<f32>, 3>,
    beams: array<Beam, 49>,
};

@group(0) @binding(0) var<uniform> scene: Scene;
@group(0) @binding(0) var source: texture_2d<f32>;
@group(0) @binding(1) var bloom: texture_2d<f32>;
@group(0) @binding(2) var linear_sampler: sampler;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn fullscreen_vs(@builtin(vertex_index) index: u32) -> VertexOutput {
    let uv = vec2<f32>(f32((index << 1u) & 2u), f32(index & 2u));
    var output: VertexOutput;
    output.position = vec4<f32>(uv * 2.0 - 1.0, 0.0, 1.0);
    output.uv = vec2<f32>(uv.x, 1.0 - uv.y);
    return output;
}

fn hash(p: vec3<f32>) -> f32 {
    return fract(sin(dot(p, vec3<f32>(127.1, 311.7, 74.7))) * 43758.5453);
}

// Closest approach of the camera ray to a finite light segment.
fn ray_segment(origin: vec3<f32>, ray: vec3<f32>, a: vec3<f32>, b: vec3<f32>) -> vec2<f32> {
    let segment = b - a;
    let offset = a - origin;
    let projected = segment - ray * dot(segment, ray);
    let distance = offset - ray * dot(offset, ray);
    let fraction = clamp(-dot(distance, projected) / max(dot(projected, projected), 0.000001), 0.0, 1.0);
    return vec2<f32>(length(distance + fraction * projected), fraction);
}

fn prism_hit(origin: vec3<f32>, ray: vec3<f32>) -> vec4<f32> {
    var near = -1000.0;
    var far = 1000.0;
    var surface_normal = vec3<f32>(0.0);
    for (var i = 0u; i < 5u; i++) {
        var normal = vec3<f32>(0.0, 0.0, 1.0);
        var plane = 0.48;
        if i < 3u {
            let a = scene.vertices[i].xy;
            let edge = scene.vertices[(i + 1u) % 3u].xy - a;
            normal = vec3<f32>(normalize(vec2<f32>(edge.y, -edge.x)), 0.0);
            plane = dot(normal.xy, a);
        } else if i == 4u {
            normal.z = -1.0;
        }
        let denominator = dot(normal, ray);
        let numerator = plane - dot(normal, origin);
        if abs(denominator) < 0.00001 {
            if numerator < 0.0 {
                return vec4<f32>(-1.0);
            }
        } else {
            let t = numerator / denominator;
            if denominator < 0.0 {
                if t > near {
                    near = t;
                    surface_normal = normal;
                }
            } else {
                far = min(far, t);
            }
        }
    }
    if far < max(near, 0.0) {
        return vec4<f32>(-1.0);
    }
    return vec4<f32>(surface_normal, near);
}

@fragment
fn scene_fs(input: VertexOutput) -> @location(0) vec4<f32> {
    let aspect = scene.resolution_time.x / scene.resolution_time.y;
    let span = scene.camera_frame.z;
    let uv = (input.uv * 2.0 - 1.0) * vec2<f32>(aspect, -1.0) * span;
    let right = vec3<f32>(1.0, 0.0, 0.0);
    let up = normalize(vec3<f32>(0.0, 1.0, -0.27));
    let ray = normalize(vec3<f32>(0.0, -0.27, -1.0));
    let origin = -ray * 8.0
        + right * (uv.x + scene.camera_frame.x) + up * (uv.y + scene.camera_frame.y);
    let hit = prism_hit(origin, ray);
    let world = origin + ray * (-origin.z / ray.z);
    let pixel_width = 2.0 * span / scene.resolution_time.y;
    let time = scene.resolution_time.z;
    var color = vec3<f32>(0.0006, 0.001, 0.0025);
    color += vec3<f32>(0.002, 0.006, 0.014) * exp(-0.28 * dot(uv, uv));

    let floor_t = (-1.47 - origin.y) / ray.y;
    let floor_point = origin + ray * floor_t;
    let floor_visible = floor_t > 0.0;
    if floor_visible {
        let pool = exp(-0.25 * dot(floor_point.xz, floor_point.xz));
        color += vec3<f32>(0.005, 0.008, 0.015) * pool;
        let shadow = exp(-2.2 * dot(floor_point.xz, floor_point.xz));
        color *= 1.0 - 0.65 * shadow;
    }

    var illumination = vec3<f32>(0.0);
    for (var i = 0u; i < 49u; i++) {
        let beam = scene.beams[i];
        let nearest = ray_segment(origin, ray, beam.start_radius.xyz, beam.end_energy.xyz);
        let width = max(beam.start_radius.w, pixel_width * 0.55);
        let core = exp(-2.0 * pow(nearest.x / width, 2.0));
        let scatter = exp(-2.0 * pow(nearest.x / (width * 9.0), 2.0)) * 0.085;
        let fog = 0.8 + 0.2 * sin(nearest.y * 31.0 - time * 0.6 + world.y * 9.0);
        let attenuation = 1.0 - 0.3 * nearest.y;
        let light = beam.color.rgb * beam.end_energy.w;
        illumination += light * (core + scatter * fog) * attenuation;
        if i > 0u && i % 2u == 0u && floor_visible {
            let delta = floor_point.xz - beam.end_energy.xz;
            let caustic = exp(-dot(delta / vec2<f32>(0.075, 0.36), delta / vec2<f32>(0.075, 0.36)));
            color += light * caustic * 0.13;
        }
    }
    if hit.w > 0.0 {
        let p = origin + ray * hit.w;
        let fresnel = pow(1.0 - abs(dot(-ray, hit.xyz)), 4.0);
        let reflection = pow(max(0.0, dot(reflect(ray, hit.xyz), normalize(vec3<f32>(-0.4, 0.8, 1.0)))), 24.0);
        let iridescence = 0.5 + 0.5 * cos(vec3<f32>(0.0, 2.1, 4.2) + p.y * 2.4 + p.x * 1.5 + time * 0.15);
        color += vec3<f32>(0.006, 0.015, 0.027) + iridescence * (0.013 + fresnel * 0.12);
        color += vec3<f32>(0.6, 0.8, 1.0) * reflection * 0.45;
        illumination *= 0.78;
    }
    color += illumination;

    // Beveled edge highlights on the front, back and connecting prism edges.
    for (var i = 0u; i < 3u; i++) {
        let a = scene.vertices[i].xy;
        let b = scene.vertices[(i + 1u) % 3u].xy;
        let front = ray_segment(origin, ray, vec3<f32>(a, 0.48), vec3<f32>(b, 0.48)).x;
        let back = ray_segment(origin, ray, vec3<f32>(a, -0.48), vec3<f32>(b, -0.48)).x;
        let side = ray_segment(origin, ray, vec3<f32>(a, -0.48), vec3<f32>(a, 0.48)).x;
        let width = max(0.007, pixel_width * 0.65);
        let edge = exp(-pow(front / width, 2.0)) + 0.35 * exp(-pow(back / width, 2.0))
            + 0.6 * exp(-pow(side / width, 2.0));
        let tint = 0.6 + 0.4 * cos(vec3<f32>(0.0, 2.0, 4.0) + world.y * 3.0 + f32(i) * 2.0);
        color += tint * edge * 0.65;
    }

    let cell = floor(world * 38.0 + vec3<f32>(time * 0.13, time * 0.2, 0.0));
    let speckle = pow(hash(cell), 100.0);
    color += illumination * speckle * 0.18;
    return vec4<f32>(color, 1.0);
}

fn blur(uv: vec2<f32>, direction: vec2<f32>, threshold: bool) -> vec3<f32> {
    var sum = vec3<f32>(0.0);
    var weights = 0.0;
    for (var i = -8; i <= 8; i++) {
        let weight = exp(-f32(i * i) / 24.0);
        var value = textureSample(source, linear_sampler, uv + direction * f32(i)).rgb;
        if threshold {
            value *= smoothstep(0.25, 1.2, max(value.r, max(value.g, value.b)));
        }
        sum += value * weight;
        weights += weight;
    }
    return sum / weights;
}

@fragment
fn horizontal_fs(input: VertexOutput) -> @location(0) vec4<f32> {
    let size = vec2<f32>(textureDimensions(source));
    let radius = max(0.5, min(size.x, size.y) / 380.0);
    return vec4<f32>(blur(input.uv, vec2<f32>(radius / size.x, 0.0), true), 1.0);
}

@fragment
fn vertical_fs(input: VertexOutput) -> @location(0) vec4<f32> {
    let size = vec2<f32>(textureDimensions(source));
    let radius = max(0.25, min(size.x, size.y) / 380.0);
    return vec4<f32>(blur(input.uv, vec2<f32>(0.0, radius / size.y), false), 1.0);
}

@fragment
fn composite_fs(input: VertexOutput) -> @location(0) vec4<f32> {
    let radiance = textureSample(source, linear_sampler, input.uv).rgb
        + textureSample(bloom, linear_sampler, input.uv).rgb * 0.9;
    let mapped = vec3<f32>(1.0) - exp(-radiance * 1.15);
    return vec4<f32>(mapped, 1.0);
}
