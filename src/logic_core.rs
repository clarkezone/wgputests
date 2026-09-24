use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec3, Vec4};
use wgpu::util::DeviceExt;

const INSTANCE_COUNT: usize = 14;

#[derive(Clone, Copy)]
pub struct Viewport {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct SceneUniforms {
    view_projection: [[f32; 4]; 4],
    light_direction: [f32; 4],
    camera_position: [f32; 4],
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Instance {
    model_0: [f32; 4],
    model_1: [f32; 4],
    model_2: [f32; 4],
    model_3: [f32; 4],
    color_emissive: [f32; 4],
    material: [f32; 4],
}

impl Instance {
    const ATTRIBUTES: [wgpu::VertexAttribute; 6] = wgpu::vertex_attr_array![
        1 => Float32x4,
        2 => Float32x4,
        3 => Float32x4,
        4 => Float32x4,
        5 => Float32x4,
        7 => Float32x4
    ];

    fn new(model: Mat4, color: Vec3, emissive: f32, roughness: f32, metalness: f32) -> Self {
        let columns = model.to_cols_array_2d();
        Self {
            model_0: columns[0],
            model_1: columns[1],
            model_2: columns[2],
            model_3: columns[3],
            color_emissive: [color.x, color.y, color.z, emissive],
            material: [roughness, metalness, 0.0, 0.0],
        }
    }

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
struct Vertex {
    position: [f32; 3],
    normal: [f32; 3],
}

impl Vertex {
    const ATTRIBUTES: [wgpu::VertexAttribute; 2] =
        wgpu::vertex_attr_array![0 => Float32x3, 6 => Float32x3];

    fn layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBUTES,
        }
    }
}

const VERTICES: &[Vertex] = &[
    Vertex {
        position: [-0.5, -0.5, 0.5],
        normal: [0.0, 0.0, 1.0],
    },
    Vertex {
        position: [0.5, -0.5, 0.5],
        normal: [0.0, 0.0, 1.0],
    },
    Vertex {
        position: [0.5, 0.5, 0.5],
        normal: [0.0, 0.0, 1.0],
    },
    Vertex {
        position: [-0.5, 0.5, 0.5],
        normal: [0.0, 0.0, 1.0],
    },
    Vertex {
        position: [0.5, -0.5, -0.5],
        normal: [0.0, 0.0, -1.0],
    },
    Vertex {
        position: [-0.5, -0.5, -0.5],
        normal: [0.0, 0.0, -1.0],
    },
    Vertex {
        position: [-0.5, 0.5, -0.5],
        normal: [0.0, 0.0, -1.0],
    },
    Vertex {
        position: [0.5, 0.5, -0.5],
        normal: [0.0, 0.0, -1.0],
    },
    Vertex {
        position: [0.5, -0.5, 0.5],
        normal: [1.0, 0.0, 0.0],
    },
    Vertex {
        position: [0.5, -0.5, -0.5],
        normal: [1.0, 0.0, 0.0],
    },
    Vertex {
        position: [0.5, 0.5, -0.5],
        normal: [1.0, 0.0, 0.0],
    },
    Vertex {
        position: [0.5, 0.5, 0.5],
        normal: [1.0, 0.0, 0.0],
    },
    Vertex {
        position: [-0.5, -0.5, -0.5],
        normal: [-1.0, 0.0, 0.0],
    },
    Vertex {
        position: [-0.5, -0.5, 0.5],
        normal: [-1.0, 0.0, 0.0],
    },
    Vertex {
        position: [-0.5, 0.5, 0.5],
        normal: [-1.0, 0.0, 0.0],
    },
    Vertex {
        position: [-0.5, 0.5, -0.5],
        normal: [-1.0, 0.0, 0.0],
    },
    Vertex {
        position: [-0.5, 0.5, 0.5],
        normal: [0.0, 1.0, 0.0],
    },
    Vertex {
        position: [0.5, 0.5, 0.5],
        normal: [0.0, 1.0, 0.0],
    },
    Vertex {
        position: [0.5, 0.5, -0.5],
        normal: [0.0, 1.0, 0.0],
    },
    Vertex {
        position: [-0.5, 0.5, -0.5],
        normal: [0.0, 1.0, 0.0],
    },
    Vertex {
        position: [-0.5, -0.5, -0.5],
        normal: [0.0, -1.0, 0.0],
    },
    Vertex {
        position: [0.5, -0.5, -0.5],
        normal: [0.0, -1.0, 0.0],
    },
    Vertex {
        position: [0.5, -0.5, 0.5],
        normal: [0.0, -1.0, 0.0],
    },
    Vertex {
        position: [-0.5, -0.5, 0.5],
        normal: [0.0, -1.0, 0.0],
    },
];

const INDICES: &[u16] = &[
    0, 1, 2, 2, 3, 0, 4, 5, 6, 6, 7, 4, 8, 9, 10, 10, 11, 8, 12, 13, 14, 14, 15, 12, 16, 17, 18,
    18, 19, 16, 20, 21, 22, 22, 23, 20,
];

pub struct LogicCoreScene {
    pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    instance_buffer: wgpu::Buffer,
    uniform_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
}

impl LogicCoreScene {
    pub fn new(
        device: &wgpu::Device,
        color_format: wgpu::TextureFormat,
        depth_format: wgpu::TextureFormat,
    ) -> Self {
        let shader = device.create_shader_module(wgpu::include_wgsl!("logic_core.wgsl"));
        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("logic core scene uniforms"),
            contents: bytemuck::bytes_of(&SceneUniforms {
                view_projection: Mat4::IDENTITY.to_cols_array_2d(),
                light_direction: [0.4, 0.8, 0.2, 0.0],
                camera_position: [20.0, 20.0, 20.0, 1.0],
            }),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("logic core bind group layout"),
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
            label: Some("logic core bind group"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("logic core pipeline layout"),
            bind_group_layouts: &[Some(&bind_group_layout)],
            immediate_size: 0,
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("logic core pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: Default::default(),
                buffers: &[Some(Vertex::layout()), Some(Instance::layout())],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: color_format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                cull_mode: Some(wgpu::Face::Back),
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: depth_format,
                depth_write_enabled: Some(true),
                depth_compare: Some(wgpu::CompareFunction::Less),
                stencil: Default::default(),
                bias: Default::default(),
            }),
            multisample: Default::default(),
            multiview_mask: None,
            cache: None,
        });
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("logic core cube vertices"),
            contents: bytemuck::cast_slice(VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("logic core cube indices"),
            contents: bytemuck::cast_slice(INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });
        let instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("logic core instances"),
            size: (std::mem::size_of::<Instance>() * INSTANCE_COUNT) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            pipeline,
            vertex_buffer,
            index_buffer,
            instance_buffer,
            uniform_buffer,
            bind_group,
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
        let projection = glam::camera::rh::proj::directx::orthographic(
            -12.0 * aspect,
            12.0 * aspect,
            -12.0,
            12.0,
            1.0,
            1000.0,
        );
        let view = glam::camera::rh::view::look_at_mat4(Vec3::splat(20.0), Vec3::ZERO, Vec3::Y);
        let pulse = ((elapsed * 2.5).sin() + 1.0) * 0.5;
        queue.write_buffer(
            &self.uniform_buffer,
            0,
            bytemuck::bytes_of(&SceneUniforms {
                view_projection: (projection * view).to_cols_array_2d(),
                light_direction: Vec4::new(10.0, 20.0, 5.0, 1.0 + pulse * 1.5).to_array(),
                camera_position: Vec4::new(20.0, 20.0, 20.0, 1.0).to_array(),
            }),
        );

        let drift = Mat4::from_translation(Vec3::new(0.0, (elapsed * 0.5).sin() * 0.2, 0.0))
            * Mat4::from_rotation_y((elapsed * 0.1).sin() * 0.15);
        let mut instances = Vec::with_capacity(INSTANCE_COUNT);
        instances.push(Instance::new(
            drift
                * Mat4::from_scale_rotation_translation(
                    Vec3::new(16.0, 0.5, 16.0),
                    Default::default(),
                    Vec3::new(0.0, -2.0, 0.0),
                ),
            Vec3::splat(0.006),
            0.0,
            0.9,
            0.1,
        ));
        instances.push(Instance::new(
            drift
                * Mat4::from_scale_rotation_translation(
                    Vec3::new(2.0, 4.0, 2.0),
                    Default::default(),
                    Vec3::new(0.0, 0.25, 0.0),
                ),
            Vec3::new(0.0, 0.898, 1.0),
            0.4 + pulse * 0.6,
            0.2,
            0.0,
        ));

        for index in 0..12 {
            let seed = index as f32;
            let base_angle = seed / 12.0 * std::f32::consts::TAU;
            let radius = 4.0 + hash(seed * 7.13) * 4.0;
            let speed = 0.3 + hash(seed * 2.71) * 0.9;
            let angle = base_angle + elapsed * speed;
            let y_base = -1.0 + hash(seed * 5.37) * 4.0;
            let y_offset = hash(seed * 9.91) * std::f32::consts::TAU;
            let position = Vec3::new(
                angle.cos() * radius,
                y_base + (elapsed * 1.5 + y_offset).sin() * 0.5,
                angle.sin() * radius,
            );
            let size = 0.4 + hash(seed * 3.47) * 0.4;
            let rotation =
                glam::Quat::from_euler(glam::EulerRot::XYZ, elapsed * 0.6, elapsed * 1.2, 0.0);
            let accent = hash(seed * 11.17) > 0.68;
            let color = if accent {
                Vec3::new(0.0, 0.898, 1.0)
            } else {
                Vec3::splat(0.016)
            };
            instances.push(Instance::new(
                drift
                    * Mat4::from_scale_rotation_translation(Vec3::splat(size), rotation, position),
                color,
                if accent { 0.45 + pulse * 0.35 } else { 0.0 },
                if accent { 0.2 } else { 0.8 },
                0.0,
            ));
        }
        queue.write_buffer(&self.instance_buffer, 0, bytemuck::cast_slice(&instances));
    }

    pub fn render(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        color_view: &wgpu::TextureView,
        depth_view: &wgpu::TextureView,
        viewport: Option<Viewport>,
    ) {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("logic core pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: color_view,
                resolve_target: None,
                depth_slice: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.0015,
                        g: 0.0015,
                        b: 0.0015,
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
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.bind_group, &[]);
        pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
        pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        pass.draw_indexed(0..INDICES.len() as u32, 0, 0..INSTANCE_COUNT as u32);
    }
}

fn hash(value: f32) -> f32 {
    (value.sin() * 43_758.547).fract().abs()
}
