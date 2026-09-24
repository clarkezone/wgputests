use bytemuck::{Pod, Zeroable};
use glam::{Vec2, Vec3};
use wgpu::util::DeviceExt;
use winit::event::{ElementState, WindowEvent};
use winit::keyboard::{KeyCode, PhysicalKey};

use crate::logic_core::Viewport;

const SPECTRAL_SAMPLES: usize = 24;
const BEAM_COUNT: usize = 1 + SPECTRAL_SAMPLES * 2;
const HDR_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Beam {
    start_radius: [f32; 4],
    end_energy: [f32; 4],
    color: [f32; 4],
}

impl Beam {
    fn new(start: Vec2, end: Vec2, radius: f32, energy: f32, color: Vec3) -> Self {
        Self {
            start_radius: [start.x, start.y, 0.0, radius],
            end_energy: [end.x, end.y, 0.0, energy],
            color: [color.x, color.y, color.z, 0.0],
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Uniforms {
    resolution_time: [f32; 4],
    camera_frame: [f32; 4],
    vertices: [[f32; 4]; 3],
    beams: [Beam; BEAM_COUNT],
}

struct Targets {
    size: [u32; 2],
    scene: wgpu::TextureView,
    horizontal: wgpu::TextureView,
    vertical: wgpu::TextureView,
    horizontal_binding: wgpu::BindGroup,
    vertical_binding: wgpu::BindGroup,
    composite_binding: wgpu::BindGroup,
}

pub struct PrismaticScene {
    scene_pipeline: wgpu::RenderPipeline,
    horizontal_pipeline: wgpu::RenderPipeline,
    vertical_pipeline: wgpu::RenderPipeline,
    composite_pipeline: wgpu::RenderPipeline,
    post_layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
    uniforms: wgpu::Buffer,
    scene_binding: wgpu::BindGroup,
    targets: Option<Targets>,
    time: f32,
    last_elapsed: Option<f32>,
    paused: bool,
    angle_offset: f32,
}

impl PrismaticScene {
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let shader = device.create_shader_module(wgpu::include_wgsl!("prismatic.wgsl"));
        let uniforms = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("prismatic optics uniforms"),
            contents: bytemuck::bytes_of(&Uniforms::zeroed()),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let scene_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("prismatic scene layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });
        let scene_binding = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("prismatic scene binding"),
            layout: &scene_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniforms.as_entire_binding(),
            }],
        });
        let texture_entry = |binding| wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Texture {
                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                view_dimension: wgpu::TextureViewDimension::D2,
                multisampled: false,
            },
            count: None,
        };
        let post_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("prismatic bloom layout"),
            entries: &[
                texture_entry(0),
                texture_entry(1),
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("prismatic bloom sampler"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });
        Self {
            scene_pipeline: pipeline(device, &shader, &scene_layout, "scene_fs", HDR_FORMAT),
            horizontal_pipeline: pipeline(
                device,
                &shader,
                &post_layout,
                "horizontal_fs",
                HDR_FORMAT,
            ),
            vertical_pipeline: pipeline(device, &shader, &post_layout, "vertical_fs", HDR_FORMAT),
            composite_pipeline: pipeline(device, &shader, &post_layout, "composite_fs", format),
            post_layout,
            sampler,
            uniforms,
            scene_binding,
            targets: None,
            time: 0.0,
            last_elapsed: None,
            paused: false,
            angle_offset: 0.0,
        }
    }

    pub fn handle_input(&mut self, event: &WindowEvent) -> bool {
        let WindowEvent::KeyboardInput { event, .. } = event else {
            return false;
        };
        if event.state != ElementState::Pressed {
            return false;
        }
        match event.physical_key {
            PhysicalKey::Code(KeyCode::ArrowLeft) => {
                self.angle_offset = (self.angle_offset - 0.015).max(-0.06);
            }
            PhysicalKey::Code(KeyCode::ArrowRight) => {
                self.angle_offset = (self.angle_offset + 0.015).min(0.06);
            }
            PhysicalKey::Code(KeyCode::Space) if !event.repeat => self.paused = !self.paused,
            PhysicalKey::Code(KeyCode::KeyR) => {
                self.angle_offset = 0.0;
                self.time = 0.0;
            }
            _ => return false,
        }
        true
    }

    pub fn update(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        elapsed: f32,
        viewport: Viewport,
    ) {
        if let Some(previous) = self.last_elapsed
            && !self.paused
        {
            self.time += (elapsed - previous).clamp(0.0, 0.1);
        }
        self.last_elapsed = Some(elapsed);
        let size = [viewport.width as u32, viewport.height as u32].map(|value| value.max(1));
        if self
            .targets
            .as_ref()
            .is_none_or(|targets| targets.size != size)
        {
            self.targets = Some(self.create_targets(device, size));
        }
        let (vertices, beams) = trace_spectrum(self.time, self.angle_offset);
        queue.write_buffer(
            &self.uniforms,
            0,
            bytemuck::bytes_of(&Uniforms {
                resolution_time: [size[0] as f32, size[1] as f32, self.time, 0.0],
                camera_frame: frame_spectrum(&vertices, &beams, size[0] as f32 / size[1] as f32),
                vertices: vertices.map(|v| [v.x, v.y, 0.0, 0.0]),
                beams,
            }),
        );
    }

    fn create_targets(&self, device: &wgpu::Device, size: [u32; 2]) -> Targets {
        let texture = |label, dimensions: [u32; 2]| {
            device
                .create_texture(&wgpu::TextureDescriptor {
                    label: Some(label),
                    size: wgpu::Extent3d {
                        width: dimensions[0],
                        height: dimensions[1],
                        depth_or_array_layers: 1,
                    },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: HDR_FORMAT,
                    usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                        | wgpu::TextureUsages::TEXTURE_BINDING,
                    view_formats: &[],
                })
                .create_view(&Default::default())
        };
        let scene = texture("prismatic HDR scene", size);
        let bloom_size = size.map(|value| value.div_ceil(2));
        let horizontal = texture("prismatic bloom horizontal", bloom_size);
        let vertical = texture("prismatic bloom vertical", bloom_size);
        let binding = |first: &wgpu::TextureView, second: &wgpu::TextureView| {
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("prismatic post-process binding"),
                layout: &self.post_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(first),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(second),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::Sampler(&self.sampler),
                    },
                ],
            })
        };
        Targets {
            size,
            horizontal_binding: binding(&scene, &scene),
            vertical_binding: binding(&horizontal, &horizontal),
            composite_binding: binding(&scene, &vertical),
            scene,
            horizontal,
            vertical,
        }
    }

    pub fn render(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        output: &wgpu::TextureView,
        viewport: Viewport,
    ) {
        let targets = self
            .targets
            .as_ref()
            .expect("update initializes prism targets");
        for (pipeline, binding, target, final_pass) in [
            (
                &self.scene_pipeline,
                &self.scene_binding,
                &targets.scene,
                false,
            ),
            (
                &self.horizontal_pipeline,
                &targets.horizontal_binding,
                &targets.horizontal,
                false,
            ),
            (
                &self.vertical_pipeline,
                &targets.vertical_binding,
                &targets.vertical,
                false,
            ),
            (
                &self.composite_pipeline,
                &targets.composite_binding,
                output,
                true,
            ),
        ] {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("prismatic render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: target,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            if final_pass {
                pass.set_viewport(
                    viewport.x,
                    viewport.y,
                    viewport.width,
                    viewport.height,
                    0.0,
                    1.0,
                );
                pass.set_scissor_rect(
                    viewport.x as u32,
                    viewport.y as u32,
                    viewport.width as u32,
                    viewport.height as u32,
                );
            }
            pass.set_pipeline(pipeline);
            pass.set_bind_group(0, binding, &[]);
            pass.draw(0..3, 0..1);
        }
    }
}

fn pipeline(
    device: &wgpu::Device,
    shader: &wgpu::ShaderModule,
    layout: &wgpu::BindGroupLayout,
    entry: &str,
    format: wgpu::TextureFormat,
) -> wgpu::RenderPipeline {
    let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some(entry),
        bind_group_layouts: &[Some(layout)],
        immediate_size: 0,
    });
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some(entry),
        layout: Some(&layout),
        vertex: wgpu::VertexState {
            module: shader,
            entry_point: Some("fullscreen_vs"),
            compilation_options: Default::default(),
            buffers: &[],
        },
        fragment: Some(wgpu::FragmentState {
            module: shader,
            entry_point: Some(entry),
            compilation_options: Default::default(),
            targets: &[Some(wgpu::ColorTargetState {
                format,
                blend: None,
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        primitive: Default::default(),
        depth_stencil: None,
        multisample: Default::default(),
        multiview_mask: None,
        cache: None,
    })
}

fn refract(direction: Vec2, normal: Vec2, eta: f32) -> Option<Vec2> {
    let cosine = -normal.dot(direction);
    let discriminant = 1.0 - eta * eta * (1.0 - cosine * cosine);
    (discriminant >= 0.0)
        .then(|| (eta * direction + (eta * cosine - discriminant.sqrt()) * normal).normalize())
}

fn intersect(origin: Vec2, direction: Vec2, vertices: &[Vec2; 3]) -> Option<(Vec2, Vec2)> {
    let mut closest = f32::INFINITY;
    let mut hit = None;
    for index in 0..3 {
        let a = vertices[index];
        let edge = vertices[(index + 1) % 3] - a;
        let denominator = direction.perp_dot(edge);
        if denominator.abs() < 1e-6 {
            continue;
        }
        let distance = (a - origin).perp_dot(edge) / denominator;
        let fraction = (a - origin).perp_dot(direction) / denominator;
        if distance > 1e-4 && (0.0..=1.0).contains(&fraction) && distance < closest {
            closest = distance;
            hit = Some((
                origin + direction * distance,
                Vec2::new(edge.y, -edge.x).normalize(),
            ));
        }
    }
    hit
}

fn wavelength_color(wavelength: f32) -> Vec3 {
    let gaussian = |center: f32, width: f32| (-0.5 * ((wavelength - center) / width).powi(2)).exp();
    Vec3::new(
        gaussian(615.0, 30.0) + 0.25 * gaussian(425.0, 18.0),
        gaussian(540.0, 24.0),
        gaussian(455.0, 24.0),
    )
}

fn trace_spectrum(time: f32, offset: f32) -> ([Vec2; 3], [Beam; BEAM_COUNT]) {
    let rotation = Vec2::from_angle(0.025 * (time * 0.23).sin());
    let vertices = [
        Vec2::new(-1.0, -0.75),
        Vec2::new(1.0, -0.75),
        Vec2::new(0.0, 0.982),
    ]
    .map(|v| rotation.rotate(v));
    let direction = Vec2::from_angle(0.25 + offset + 0.015 * (time * 0.31).sin());
    let aim = Vec2::new(-0.45, 0.04);
    let source = aim - direction * 3.1;
    let (entry, normal) = intersect(source, direction, &vertices)
        .expect("the bounded source angles always intersect the prism");
    let mut beams = [Beam::zeroed(); BEAM_COUNT];
    beams[0] = Beam::new(source, entry, 0.018, 5.0, Vec3::new(0.88, 0.94, 1.0));
    for index in 0..SPECTRAL_SAMPLES {
        let wavelength = 420.0 + 260.0 * index as f32 / (SPECTRAL_SAMPLES - 1) as f32;
        // Cauchy dispersion for an illustrative high-dispersion optical glass.
        let ior = 1.40 + 0.035 / (wavelength * 0.001).powi(2);
        let inside = refract(direction, normal, 1.0 / ior)
            .expect("air-to-glass refraction cannot internally reflect");
        let (exit, exit_normal) = intersect(entry + inside * 0.001, inside, &vertices)
            .expect("a ray inside the convex prism must hit an exit face");
        let outgoing = refract(inside, -exit_normal, ior)
            .expect("bounded source angles stay below the total internal reflection limit");
        let color = wavelength_color(wavelength);
        beams[1 + index * 2] = Beam::new(entry, exit, 0.013, 0.28, color);
        let length = ((-1.45 - exit.y) / outgoing.y).clamp(1.0, 7.0);
        beams[2 + index * 2] = Beam::new(exit, exit + outgoing * length, 0.012, 1.5, color);
    }
    (vertices, beams)
}

fn frame_spectrum(vertices: &[Vec2; 3], beams: &[Beam; BEAM_COUNT], aspect: f32) -> [f32; 4] {
    let mut min = Vec2::splat(f32::INFINITY);
    let mut max = Vec2::splat(f32::NEG_INFINITY);
    for point in vertices
        .iter()
        .copied()
        .chain(beams.iter().flat_map(|beam| {
            [
                Vec2::new(beam.start_radius[0], beam.start_radius[1]),
                Vec2::new(beam.end_energy[0], beam.end_energy[1]),
            ]
        }))
    {
        let projected = Vec2::new(point.x, point.y / (1.0_f32 + 0.27 * 0.27).sqrt());
        min = min.min(projected);
        max = max.max(projected);
    }
    let center = (min + max) * 0.5;
    let half_extent = (max - min) * 0.5 + Vec2::splat(0.55);
    [
        center.x,
        center.y,
        half_extent.y.max(half_extent.x / aspect),
        0.0,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spectrum_exits_prism_throughout_animation_and_input_range() {
        for step in 0..600 {
            for offset in [-0.06, 0.0, 0.06] {
                let (vertices, beams) = trace_spectrum(step as f32 * 0.5, offset);
                for beam in beams {
                    assert!(
                        beam.start_radius
                            .iter()
                            .chain(beam.end_energy.iter())
                            .all(|v| v.is_finite())
                    );
                    assert!(beam.start_radius[3] > 0.0);
                    assert!(beam.end_energy[3] > 0.0);
                }
                let violet = beams[2];
                let red = beams[BEAM_COUNT - 1];
                assert!((red.end_energy[0] - violet.end_energy[0]).abs() > 0.1);
                for aspect in [0.5, 1.0, 1.6, 3.0] {
                    let frame = frame_spectrum(&vertices, &beams, aspect);
                    for beam in beams {
                        assert!((beam.end_energy[0] - frame[0]).abs() < frame[2] * aspect);
                        let y = beam.end_energy[1] / (1.0_f32 + 0.27 * 0.27).sqrt();
                        assert!((y - frame[1]).abs() < frame[2]);
                    }
                }
            }
        }
    }

    #[test]
    fn snell_refraction_and_total_internal_reflection() {
        let direction = Vec2::new(0.5, -3.0_f32.sqrt() / 2.0);
        let transmitted = refract(direction, Vec2::Y, 1.0 / 1.5).unwrap();
        assert!((transmitted.x - 1.0 / 3.0).abs() < 1e-6);
        assert!(refract(Vec2::new(0.9, -(1.0_f32 - 0.81).sqrt()), Vec2::Y, 1.5).is_none());
    }
}
