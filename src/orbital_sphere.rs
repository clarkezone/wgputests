use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Quat, Vec3};
use wgpu::util::DeviceExt;

use crate::logic_core::Viewport;

const SPHERE_RADIUS: f32 = 2.2;
const SOURCE_PARTICLE_COUNT: usize = 15_000;
const ORBIT_COUNT: usize = 6;
const ORBIT_SEGMENTS: usize = 90;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct SceneUniforms {
    view_projection: [[f32; 4]; 4],
    group_model: [[f32; 4]; 4],
    viewport_brightness: [f32; 4],
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Particle {
    position_size: [f32; 4],
    color_softness: [f32; 4],
}

impl Particle {
    const ATTRIBUTES: [wgpu::VertexAttribute; 2] =
        wgpu::vertex_attr_array![1 => Float32x4, 2 => Float32x4];

    fn layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &Self::ATTRIBUTES,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct QuadVertex {
    corner: [f32; 2],
}

impl QuadVertex {
    const ATTRIBUTES: [wgpu::VertexAttribute; 1] = wgpu::vertex_attr_array![0 => Float32x2];

    fn layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBUTES,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct LineVertex {
    position: [f32; 3],
}

impl LineVertex {
    const ATTRIBUTES: [wgpu::VertexAttribute; 1] = wgpu::vertex_attr_array![0 => Float32x3];

    fn layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBUTES,
        }
    }
}

#[derive(Clone, Copy)]
struct Orbit {
    radius: f32,
    rotation_x: f32,
    rotation_y: f32,
    node_angle: Option<f32>,
}

pub struct OrbitalSphereScene {
    particle_pipeline: wgpu::RenderPipeline,
    line_pipeline: wgpu::RenderPipeline,
    quad_buffer: wgpu::Buffer,
    quad_index_buffer: wgpu::Buffer,
    bloom_buffer: wgpu::Buffer,
    particle_buffer: wgpu::Buffer,
    particle_count: u32,
    node_buffer: wgpu::Buffer,
    orbit_buffer: wgpu::Buffer,
    uniform_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    orbits: [Orbit; ORBIT_COUNT],
}

impl OrbitalSphereScene {
    pub fn new(
        device: &wgpu::Device,
        color_format: wgpu::TextureFormat,
        depth_format: wgpu::TextureFormat,
    ) -> Self {
        let shader = device.create_shader_module(wgpu::include_wgsl!("orbital_sphere.wgsl"));
        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("orbital sphere uniforms"),
            contents: bytemuck::bytes_of(&SceneUniforms {
                view_projection: Mat4::IDENTITY.to_cols_array_2d(),
                group_model: Mat4::IDENTITY.to_cols_array_2d(),
                viewport_brightness: [1.0, 1.0, 1.45, 0.0],
            }),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("orbital sphere bind group layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("orbital sphere bind group"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("orbital sphere pipeline layout"),
            bind_group_layouts: &[Some(&bind_group_layout)],
            immediate_size: 0,
        });
        let additive_blend = wgpu::BlendState {
            color: wgpu::BlendComponent {
                src_factor: wgpu::BlendFactor::One,
                dst_factor: wgpu::BlendFactor::One,
                operation: wgpu::BlendOperation::Add,
            },
            alpha: wgpu::BlendComponent {
                src_factor: wgpu::BlendFactor::One,
                dst_factor: wgpu::BlendFactor::One,
                operation: wgpu::BlendOperation::Add,
            },
        };
        let particle_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("orbital sphere particle pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("particle_vs"),
                compilation_options: Default::default(),
                buffers: &[Some(QuadVertex::layout()), Some(Particle::layout())],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("particle_fs"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: color_format,
                    blend: Some(additive_blend),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                cull_mode: None,
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: depth_format,
                depth_write_enabled: Some(false),
                depth_compare: Some(wgpu::CompareFunction::LessEqual),
                stencil: Default::default(),
                bias: Default::default(),
            }),
            multisample: Default::default(),
            multiview_mask: None,
            cache: None,
        });
        let line_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("orbital sphere line pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("line_vs"),
                compilation_options: Default::default(),
                buffers: &[Some(LineVertex::layout())],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("line_fs"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: color_format,
                    blend: Some(additive_blend),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::LineList,
                cull_mode: None,
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: depth_format,
                depth_write_enabled: Some(false),
                depth_compare: Some(wgpu::CompareFunction::LessEqual),
                stencil: Default::default(),
                bias: Default::default(),
            }),
            multisample: Default::default(),
            multiview_mask: None,
            cache: None,
        });

        let quad_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("orbital sphere particle quad"),
            contents: bytemuck::cast_slice(&[
                QuadVertex {
                    corner: [-1.0, -1.0],
                },
                QuadVertex {
                    corner: [1.0, -1.0],
                },
                QuadVertex { corner: [1.0, 1.0] },
                QuadVertex {
                    corner: [-1.0, 1.0],
                },
            ]),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let quad_index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("orbital sphere particle indices"),
            contents: bytemuck::cast_slice(&[0_u16, 1, 2, 2, 3, 0]),
            usage: wgpu::BufferUsages::INDEX,
        });
        let (particles, bloom_particles) = create_sphere_particles();
        let bloom_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("orbital sphere particle bloom"),
            contents: bytemuck::cast_slice(&bloom_particles),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let particle_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("orbital sphere particles"),
            contents: bytemuck::cast_slice(&particles),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let node_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("orbital sphere nodes"),
            size: (std::mem::size_of::<Particle>() * 6) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let orbit_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("orbital sphere orbit lines"),
            size: (std::mem::size_of::<LineVertex>() * ORBIT_COUNT * ORBIT_SEGMENTS * 2) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let orbits = std::array::from_fn(|index| Orbit {
            radius: SPHERE_RADIUS * (1.08 + hash(index as f32 * 2.17) * 0.2),
            rotation_x: hash(index as f32 * 3.11) * std::f32::consts::TAU,
            rotation_y: hash(index as f32 * 5.23) * std::f32::consts::TAU,
            node_angle: (index % 2 != 0).then(|| hash(index as f32 * 7.91) * std::f32::consts::TAU),
        });

        Self {
            particle_pipeline,
            line_pipeline,
            quad_buffer,
            quad_index_buffer,
            bloom_buffer,
            particle_buffer,
            particle_count: particles.len() as u32,
            node_buffer,
            orbit_buffer,
            uniform_buffer,
            bind_group,
            orbits,
        }
    }

    pub fn update(
        &self,
        queue: &wgpu::Queue,
        elapsed: f32,
        viewport: Option<Viewport>,
        config: &wgpu::SurfaceConfiguration,
    ) {
        let viewport = viewport.unwrap_or(Viewport {
            x: 0.0,
            y: 0.0,
            width: config.width as f32,
            height: config.height as f32,
        });
        let aspect = viewport.width / viewport.height.max(1.0);
        let projection = glam::camera::rh::proj::directx::perspective(
            45.0_f32.to_radians(),
            aspect,
            0.1,
            1000.0,
        );
        let scale = 1.1;
        let half_fov_tangent = (45.0_f32.to_radians() * 0.5).tan();
        let content_radius = 3.0 * scale;
        let camera_z = content_radius / (half_fov_tangent * aspect.clamp(0.01, 1.0)) + 0.5;
        let view = glam::camera::rh::view::look_at_mat4(
            Vec3::new(0.0, 0.0, camera_z),
            Vec3::ZERO,
            Vec3::Y,
        );
        let group_model = Mat4::from_scale_rotation_translation(
            Vec3::splat(scale),
            Quat::from_euler(glam::EulerRot::XYZ, elapsed * 0.018, elapsed * 0.048, 0.0),
            Vec3::ZERO,
        );
        queue.write_buffer(
            &self.uniform_buffer,
            0,
            bytemuck::bytes_of(&SceneUniforms {
                view_projection: (projection * view).to_cols_array_2d(),
                group_model: group_model.to_cols_array_2d(),
                viewport_brightness: [viewport.width, viewport.height, 1.45, elapsed],
            }),
        );

        let mut line_vertices = Vec::with_capacity(ORBIT_COUNT * ORBIT_SEGMENTS * 2);
        let mut nodes = Vec::with_capacity(6);
        for (index, orbit) in self.orbits.iter().enumerate() {
            let direction = if index % 2 == 0 { 1.0 } else { -1.0 };
            let rotation = Mat4::from_quat(Quat::from_euler(
                glam::EulerRot::XYZ,
                orbit.rotation_x,
                orbit.rotation_y,
                elapsed * 0.024 * direction,
            ));
            for segment in 0..ORBIT_SEGMENTS {
                let start = orbit_point(orbit.radius, segment as f32 / ORBIT_SEGMENTS as f32);
                let end = orbit_point(orbit.radius, (segment + 1) as f32 / ORBIT_SEGMENTS as f32);
                line_vertices.push(LineVertex {
                    position: rotation.transform_point3(start).to_array(),
                });
                line_vertices.push(LineVertex {
                    position: rotation.transform_point3(end).to_array(),
                });
            }
            if let Some(angle) = orbit.node_angle {
                let local = Vec3::new(angle.cos() * orbit.radius, angle.sin() * orbit.radius, 0.0);
                let position = rotation.transform_point3(local);
                nodes.push(Particle {
                    position_size: [position.x, position.y, position.z, 16.0],
                    color_softness: [0.85, 0.275, 0.94, 0.12],
                });
                nodes.push(Particle {
                    position_size: [position.x, position.y, position.z, 5.0],
                    color_softness: [0.98, 0.45, 1.0, 0.9],
                });
            }
        }
        queue.write_buffer(&self.orbit_buffer, 0, bytemuck::cast_slice(&line_vertices));
        queue.write_buffer(&self.node_buffer, 0, bytemuck::cast_slice(&nodes));
    }

    pub fn render(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        color_view: &wgpu::TextureView,
        depth_view: &wgpu::TextureView,
        viewport: Option<Viewport>,
    ) {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("orbital sphere pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: color_view,
                resolve_target: None,
                depth_slice: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.001,
                        g: 0.0,
                        b: 0.004,
                        a: 1.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: depth_view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        if let Some(viewport) = viewport {
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
                viewport.width.max(1.0) as u32,
                viewport.height.max(1.0) as u32,
            );
        }
        pass.set_bind_group(0, &self.bind_group, &[]);
        pass.set_pipeline(&self.line_pipeline);
        pass.set_vertex_buffer(0, self.orbit_buffer.slice(..));
        pass.draw(0..(ORBIT_COUNT * ORBIT_SEGMENTS * 2) as u32, 0..1);

        pass.set_pipeline(&self.particle_pipeline);
        pass.set_vertex_buffer(0, self.quad_buffer.slice(..));
        pass.set_index_buffer(self.quad_index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        pass.set_vertex_buffer(1, self.bloom_buffer.slice(..));
        pass.draw_indexed(0..6, 0, 0..self.particle_count);
        pass.set_vertex_buffer(1, self.particle_buffer.slice(..));
        pass.draw_indexed(0..6, 0, 0..self.particle_count);
        pass.set_vertex_buffer(1, self.node_buffer.slice(..));
        pass.draw_indexed(0..6, 0, 0..6);
    }
}

fn create_sphere_particles() -> (Vec<Particle>, Vec<Particle>) {
    let bright = srgb_to_linear(Vec3::new(0xa7 as f32, 0x8b as f32, 0xfa as f32) / 255.0);
    let dim = srgb_to_linear(Vec3::new(0x70 as f32, 0x1a as f32, 0x75 as f32) / 255.0);
    let mut particles = Vec::with_capacity(SOURCE_PARTICLE_COUNT);
    let mut bloom_particles = Vec::with_capacity(SOURCE_PARTICLE_COUNT);
    for index in 0..SOURCE_PARTICLE_COUNT {
        let phi = (-1.0 + (2.0 * index as f32) / SOURCE_PARTICLE_COUNT as f32).acos();
        let theta = (SOURCE_PARTICLE_COUNT as f32 * std::f32::consts::PI).sqrt() * phi;
        let x = SPHERE_RADIUS * theta.cos() * phi.sin();
        let y = SPHERE_RADIUS * theta.sin() * phi.sin();
        let z = SPHERE_RADIUS * phi.cos();
        let noise = (x * 3.5).sin() * (y * 3.5).cos() * (z * 3.5).sin() + (x * 6.0).cos() * 0.4;
        if noise <= -0.1 {
            continue;
        }
        let distortion = 1.0 + noise * 0.1;
        let color = dim.lerp(bright, if noise > 0.5 { 1.0 } else { 0.3 });
        let position = [x * distortion, y * distortion, z * distortion];
        particles.push(Particle {
            position_size: [position[0], position[1], position[2], 2.5],
            color_softness: [color.x, color.y, color.z, 0.95],
        });
        bloom_particles.push(Particle {
            position_size: [position[0], position[1], position[2], 5.5],
            color_softness: [color.x * 0.16, color.y * 0.16, color.z * 0.16, 0.0],
        });
    }
    (particles, bloom_particles)
}

fn orbit_point(radius: f32, fraction: f32) -> Vec3 {
    let angle = fraction * std::f32::consts::TAU;
    Vec3::new(
        angle.cos() * radius,
        angle.sin() * radius,
        (angle * 4.0).sin() * 0.1,
    )
}

fn srgb_to_linear(color: Vec3) -> Vec3 {
    color.map(|channel| {
        if channel <= 0.04045 {
            channel / 12.92
        } else {
            ((channel + 0.055) / 1.055).powf(2.4)
        }
    })
}

fn hash(value: f32) -> f32 {
    (value.sin() * 43_758.547).fract().abs()
}
