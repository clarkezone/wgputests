use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec3};
use wgpu::util::DeviceExt;
use winit::event::{ElementState, MouseButton, WindowEvent};
use winit::keyboard::{KeyCode, PhysicalKey};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Vertex {
    position: [f32; 3],
    color: [f32; 3],
}

impl Vertex {
    const ATTRIBUTES: [wgpu::VertexAttribute; 2] =
        wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3];

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
        position: [-1.0, -1.0, 1.0],
        color: [1.0, 0.2, 0.2],
    },
    Vertex {
        position: [1.0, -1.0, 1.0],
        color: [0.2, 1.0, 0.2],
    },
    Vertex {
        position: [1.0, 1.0, 1.0],
        color: [0.2, 0.2, 1.0],
    },
    Vertex {
        position: [-1.0, 1.0, 1.0],
        color: [1.0, 1.0, 0.2],
    },
    Vertex {
        position: [-1.0, -1.0, -1.0],
        color: [1.0, 0.2, 1.0],
    },
    Vertex {
        position: [1.0, -1.0, -1.0],
        color: [0.2, 1.0, 1.0],
    },
    Vertex {
        position: [1.0, 1.0, -1.0],
        color: [1.0, 0.6, 0.2],
    },
    Vertex {
        position: [-1.0, 1.0, -1.0],
        color: [0.7, 0.3, 1.0],
    },
];

const INDICES: &[u16] = &[
    0, 1, 2, 2, 3, 0, 1, 5, 6, 6, 2, 1, 5, 4, 7, 7, 6, 5, 4, 0, 3, 3, 7, 4, 3, 2, 6, 6, 7, 3, 4, 5,
    1, 1, 0, 4,
];

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Uniforms {
    transform: [[f32; 4]; 4],
}

pub struct CubeScene {
    pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    uniform_buffer: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,
    manual_rotation: [f32; 2],
    dragging: bool,
    cursor_position: Option<(f64, f64)>,
}

impl CubeScene {
    pub fn new(
        device: &wgpu::Device,
        color_format: wgpu::TextureFormat,
        depth_format: wgpu::TextureFormat,
    ) -> Self {
        let shader = device.create_shader_module(wgpu::include_wgsl!("shader.wgsl"));
        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("cube transform uniforms"),
            contents: bytemuck::bytes_of(&Uniforms {
                transform: Mat4::IDENTITY.to_cols_array_2d(),
            }),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("cube bind group layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });
        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("cube bind group"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("cube pipeline layout"),
            bind_group_layouts: &[Some(&bind_group_layout)],
            immediate_size: 0,
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("cube pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: Default::default(),
                buffers: &[Some(Vertex::layout())],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: color_format,
                    blend: Some(wgpu::BlendState::REPLACE),
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
            label: Some("cube vertices"),
            contents: bytemuck::cast_slice(VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("cube indices"),
            contents: bytemuck::cast_slice(INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });

        Self {
            pipeline,
            vertex_buffer,
            index_buffer,
            uniform_buffer,
            uniform_bind_group,
            manual_rotation: [0.0; 2],
            dragging: false,
            cursor_position: None,
        }
    }

    pub fn handle_input(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::KeyboardInput { event, .. } if event.state == ElementState::Pressed => {
                let step = 0.12;
                match event.physical_key {
                    PhysicalKey::Code(KeyCode::ArrowLeft | KeyCode::KeyA) => {
                        self.manual_rotation[1] -= step
                    }
                    PhysicalKey::Code(KeyCode::ArrowRight | KeyCode::KeyD) => {
                        self.manual_rotation[1] += step
                    }
                    PhysicalKey::Code(KeyCode::ArrowUp | KeyCode::KeyW) => {
                        self.manual_rotation[0] -= step
                    }
                    PhysicalKey::Code(KeyCode::ArrowDown | KeyCode::KeyS) => {
                        self.manual_rotation[0] += step
                    }
                    _ => return false,
                }
                true
            }
            WindowEvent::MouseInput {
                state,
                button: MouseButton::Left,
                ..
            } => {
                self.dragging = *state == ElementState::Pressed;
                if !self.dragging {
                    self.cursor_position = None;
                }
                true
            }
            WindowEvent::CursorMoved { position, .. } if self.dragging => {
                if let Some((last_x, last_y)) = self.cursor_position {
                    self.manual_rotation[1] += (position.x - last_x) as f32 * 0.01;
                    self.manual_rotation[0] += (position.y - last_y) as f32 * 0.01;
                }
                self.cursor_position = Some((position.x, position.y));
                true
            }
            _ => false,
        }
    }

    pub fn update(&self, queue: &wgpu::Queue, elapsed: f32, config: &wgpu::SurfaceConfiguration) {
        let aspect = config.width as f32 / config.height as f32;
        let projection =
            glam::camera::rh::proj::directx::perspective(45.0_f32.to_radians(), aspect, 0.1, 100.0);
        let view =
            glam::camera::rh::view::look_at_mat4(Vec3::new(0.0, 1.5, 5.0), Vec3::ZERO, Vec3::Y);
        let model = Mat4::from_rotation_x(self.manual_rotation[0] + elapsed * 0.35)
            * Mat4::from_rotation_y(self.manual_rotation[1] + elapsed * 0.55);
        queue.write_buffer(
            &self.uniform_buffer,
            0,
            bytemuck::bytes_of(&Uniforms {
                transform: (projection * view * model).to_cols_array_2d(),
            }),
        );
    }

    pub fn render(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        color_view: &wgpu::TextureView,
        depth_view: &wgpu::TextureView,
    ) {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("cube pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: color_view,
                resolve_target: None,
                depth_slice: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.025,
                        g: 0.035,
                        b: 0.06,
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
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.uniform_bind_group, &[]);
        pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        pass.draw_indexed(0..INDICES.len() as u32, 0, 0..1);
    }
}
