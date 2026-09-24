mod cube;
mod logic_core;
mod orbital_sphere;
mod prismatic;
mod ui;

use std::sync::Arc;
use std::time::Instant;

use cube::CubeScene;
use egui_wgpu::{Renderer as EguiRenderer, RendererOptions, ScreenDescriptor};
use logic_core::LogicCoreScene;
use orbital_sphere::OrbitalSphereScene;
use prismatic::PrismaticScene;
use ui::{Experience, UiLayout};
use winit::application::ApplicationHandler;
use winit::dpi::{PhysicalPosition, PhysicalSize};
use winit::event::{ElementState, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowAttributes, WindowId};

const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth24Plus;

struct DepthTexture {
    view: wgpu::TextureView,
}

impl DepthTexture {
    fn new(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration) -> Self {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("depth texture"),
            size: wgpu::Extent3d {
                width: config.width,
                height: config.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: DEPTH_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });

        Self {
            view: texture.create_view(&wgpu::TextureViewDescriptor::default()),
        }
    }
}

struct Renderer {
    window: Arc<Window>,
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    depth_texture: DepthTexture,
    cube: CubeScene,
    logic_core: LogicCoreScene,
    orbital_sphere: OrbitalSphereScene,
    prismatic: PrismaticScene,
    egui_context: egui::Context,
    egui_state: egui_winit::State,
    egui_renderer: EguiRenderer,
    experience: Experience,
    started_at: Instant,
}

impl Renderer {
    async fn new(window: Arc<Window>) -> Result<Self, String> {
        let size = window.inner_size();
        let instance = wgpu::Instance::default();
        let surface = instance
            .create_surface(window.clone())
            .map_err(|error| format!("failed to create rendering surface: {error}"))?;
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
                apply_limit_buckets: false,
            })
            .await
            .map_err(|error| format!("no compatible GPU adapter found: {error}"))?;
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("native 3D device"),
                ..Default::default()
            })
            .await
            .map_err(|error| format!("failed to create GPU device: {error}"))?;

        let capabilities = surface.get_capabilities(&adapter);
        let surface_format = capabilities
            .formats
            .iter()
            .copied()
            .find(wgpu::TextureFormat::is_srgb)
            .unwrap_or(capabilities.formats[0]);
        let present_mode = capabilities
            .present_modes
            .iter()
            .copied()
            .find(|mode| *mode == wgpu::PresentMode::Fifo)
            .unwrap_or(capabilities.present_modes[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            color_space: Default::default(),
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode,
            alpha_mode: capabilities.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        let cube = CubeScene::new(&device, config.format, DEPTH_FORMAT);
        let logic_core = LogicCoreScene::new(&device, config.format, DEPTH_FORMAT);
        let orbital_sphere = OrbitalSphereScene::new(&device, config.format, DEPTH_FORMAT);
        let prismatic = PrismaticScene::new(&device, config.format);
        let depth_texture = DepthTexture::new(&device, &config);

        let egui_context = egui::Context::default();
        let mut visuals = egui::Visuals::dark();
        visuals.panel_fill = egui::Color32::TRANSPARENT;
        visuals.window_fill = egui::Color32::from_rgb(10, 10, 10);
        visuals.selection.bg_fill = egui::Color32::from_rgb(0, 170, 190);
        egui_context.set_visuals(visuals);
        let egui_state = egui_winit::State::new(
            egui_context.clone(),
            egui::ViewportId::ROOT,
            window.as_ref(),
            Some(window.scale_factor() as f32),
            window.theme(),
            Some(device.limits().max_texture_dimension_2d as usize),
        );
        let egui_renderer = EguiRenderer::new(&device, config.format, RendererOptions::default());

        Ok(Self {
            window,
            surface,
            device,
            queue,
            config,
            depth_texture,
            cube,
            logic_core,
            orbital_sphere,
            prismatic,
            egui_context,
            egui_state,
            egui_renderer,
            experience: Experience::LogicCore,
            started_at: Instant::now(),
        })
    }

    fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width == 0 || new_size.height == 0 {
            return;
        }

        self.config.width = new_size.width;
        self.config.height = new_size.height;
        self.surface.configure(&self.device, &self.config);
        self.depth_texture = DepthTexture::new(&self.device, &self.config);
    }

    fn handle_input(&mut self, event: &WindowEvent) -> bool {
        let egui_response = self.egui_state.on_window_event(&self.window, event);
        if egui_response.consumed {
            return true;
        }

        if let WindowEvent::KeyboardInput { event, .. } = event
            && event.state == ElementState::Pressed
        {
            match event.physical_key {
                PhysicalKey::Code(KeyCode::Digit1) => {
                    self.experience = Experience::Cube;
                    return true;
                }
                PhysicalKey::Code(KeyCode::Digit2) => {
                    self.experience = Experience::LogicCore;
                    return true;
                }
                PhysicalKey::Code(KeyCode::Digit3) => {
                    self.experience = Experience::OrbitalSphere;
                    return true;
                }
                PhysicalKey::Code(KeyCode::Digit4) => {
                    self.experience = Experience::Prismatic;
                    return true;
                }
                _ => {}
            }
        }

        match self.experience {
            Experience::Cube => self.cube.handle_input(event),
            Experience::Prismatic => self.prismatic.handle_input(event),
            _ => false,
        }
    }

    fn render(&mut self) -> RenderOutcome {
        let raw_input = self.egui_state.take_egui_input(&self.window);
        let elapsed = self.started_at.elapsed().as_secs_f32();
        let mut experience = self.experience;
        let mut layout = UiLayout::default();
        let mut full_output = self.egui_context.run_ui(raw_input, |root_ui| {
            layout = ui::draw(root_ui, &mut experience, elapsed);
        });
        self.experience = experience;
        self.egui_state
            .handle_platform_output(&self.window, full_output.platform_output);
        let pixels_per_point = full_output.pixels_per_point;
        let paint_jobs = self
            .egui_context
            .tessellate(full_output.shapes, pixels_per_point);

        for (id, image_deltas) in full_output.textures_delta.set.drain() {
            for image_delta in image_deltas {
                self.egui_renderer
                    .update_texture(&self.device, &self.queue, id, &image_delta);
            }
        }

        let (output, reconfigure_after_present) = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(texture) => (texture, false),
            wgpu::CurrentSurfaceTexture::Suboptimal(texture) => (texture, true),
            wgpu::CurrentSurfaceTexture::Timeout | wgpu::CurrentSurfaceTexture::Occluded => {
                return RenderOutcome::Skipped;
            }
            wgpu::CurrentSurfaceTexture::Outdated | wgpu::CurrentSurfaceTexture::Lost => {
                return RenderOutcome::Reconfigure;
            }
            wgpu::CurrentSurfaceTexture::Validation => return RenderOutcome::Fatal,
        };
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("frame encoder"),
            });

        match self.experience {
            Experience::Cube => {
                self.cube.update(&self.queue, elapsed, &self.config);
                self.cube
                    .render(&mut encoder, &view, &self.depth_texture.view);
            }
            Experience::LogicCore => {
                let viewport = layout
                    .logic_viewport
                    .map(|rect| rect_to_pixels(rect, pixels_per_point, &self.config));
                self.logic_core
                    .update(&self.queue, elapsed, viewport, &self.config);
                self.logic_core
                    .render(&mut encoder, &view, &self.depth_texture.view, viewport);
            }
            Experience::OrbitalSphere => {
                let viewport = layout
                    .logic_viewport
                    .map(|rect| rect_to_pixels(rect, pixels_per_point, &self.config));
                self.orbital_sphere
                    .update(&self.queue, elapsed, viewport, &self.config);
                self.orbital_sphere
                    .render(&mut encoder, &view, &self.depth_texture.view, viewport);
            }
            Experience::Prismatic => {
                let viewport = rect_to_pixels(
                    layout
                        .logic_viewport
                        .expect("prismatic scene has a viewport"),
                    pixels_per_point,
                    &self.config,
                );
                self.prismatic
                    .update(&self.device, &self.queue, elapsed, viewport);
                self.prismatic.render(&mut encoder, &view, viewport);
            }
        }

        let screen_descriptor = ScreenDescriptor {
            size_in_pixels: [self.config.width, self.config.height],
            pixels_per_point,
        };
        let user_command_buffers = self.egui_renderer.update_buffers(
            &self.device,
            &self.queue,
            &mut encoder,
            &paint_jobs,
            &screen_descriptor,
        );
        {
            let render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("egui overlay"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            self.egui_renderer.render(
                &mut render_pass.forget_lifetime(),
                &paint_jobs,
                &screen_descriptor,
            );
        }

        let mut submissions = user_command_buffers;
        submissions.push(encoder.finish());
        self.queue.submit(submissions);
        self.queue.present(output);

        for id in full_output.textures_delta.free.drain() {
            self.egui_renderer.free_texture(&id);
        }

        if reconfigure_after_present {
            RenderOutcome::Reconfigure
        } else {
            RenderOutcome::Rendered
        }
    }
}

fn rect_to_pixels(
    rect: egui::Rect,
    pixels_per_point: f32,
    config: &wgpu::SurfaceConfiguration,
) -> logic_core::Viewport {
    let x = (rect.min.x * pixels_per_point)
        .round()
        .clamp(0.0, config.width as f32);
    let y = (rect.min.y * pixels_per_point)
        .round()
        .clamp(0.0, config.height as f32);
    let max_x = (rect.max.x * pixels_per_point)
        .round()
        .clamp(x, config.width as f32);
    let max_y = (rect.max.y * pixels_per_point)
        .round()
        .clamp(y, config.height as f32);

    logic_core::Viewport {
        x,
        y,
        width: (max_x - x).max(1.0),
        height: (max_y - y).max(1.0),
    }
}

enum RenderOutcome {
    Rendered,
    Skipped,
    Reconfigure,
    Fatal,
}

#[derive(Default)]
struct App {
    renderer: Option<Renderer>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.renderer.is_some() {
            return;
        }

        let attributes = WindowAttributes::default()
            .with_title("Native WebGPU Experiences")
            .with_position(PhysicalPosition::new(180, 100))
            .with_inner_size(PhysicalSize::new(1280, 800));
        let window = match event_loop.create_window(attributes) {
            Ok(window) => Arc::new(window),
            Err(error) => {
                log::error!("failed to create window: {error}");
                event_loop.exit();
                return;
            }
        };

        match pollster::block_on(Renderer::new(window)) {
            Ok(renderer) => self.renderer = Some(renderer),
            Err(error) => {
                log::error!("{error}");
                event_loop.exit();
            }
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        let Some(renderer) = self.renderer.as_mut() else {
            return;
        };
        if renderer.window.id() != window_id || renderer.handle_input(&event) {
            return;
        }

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::KeyboardInput { event, .. }
                if event.state == ElementState::Pressed
                    && event.physical_key == PhysicalKey::Code(KeyCode::Escape) =>
            {
                event_loop.exit();
            }
            WindowEvent::Resized(size) => renderer.resize(size),
            WindowEvent::RedrawRequested => match renderer.render() {
                RenderOutcome::Rendered | RenderOutcome::Skipped => {}
                RenderOutcome::Reconfigure => renderer.resize(renderer.window.inner_size()),
                RenderOutcome::Fatal => {
                    log::error!("surface validation failed; exiting");
                    event_loop.exit();
                }
            },
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(renderer) = &self.renderer {
            renderer.window.request_redraw();
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Poll);
    event_loop.run_app(&mut App::default())?;
    Ok(())
}
