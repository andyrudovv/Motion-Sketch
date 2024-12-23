use std::{iter, time::{Duration, Instant}};

use winit::{
    dpi::LogicalSize,
    event::*, 
    event_loop::{ControlFlow, EventLoop}, 
    keyboard::{KeyCode, PhysicalKey}, 
    window::{Window, WindowBuilder}
};

fn window_setup(event_loop: &EventLoop<()>) -> Window {
    let window = WindowBuilder::new()
        .with_title("Motion Sketch")
        .with_decorations(true)
        .with_inner_size(LogicalSize::new(1280, 720))
        .build(&event_loop)
        .unwrap();

    window
}

struct State<'a> {
    surface: wgpu::Surface<'a>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,

    window: &'a Window,
}

impl<'a> State<'a> {
    async fn new(window: &'a Window) -> Self {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor { 
            backends: wgpu::Backends::PRIMARY, 
            ..Default::default()
        });

        let surface = instance.create_surface(window).unwrap();

        let adapter = instance.request_adapter(
    &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::LowPower,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
        }).await.unwrap();

        let (device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor {
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                label: None,
                memory_hints: Default::default()
            },
            None,
        ).await.unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps.formats.iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2
        };
    
        Self {
            window,
            surface,
            device,
            queue,
            config,
            size,
        }
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    fn input(&mut self, event: &WindowEvent) -> bool {
        false
    }

    fn update(&mut self) {
        
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 1.0,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });
        }

        self.queue.submit(iter::once(encoder.finish()));
        output.present();

        Ok(())
    }


    
}


pub async fn run() {
    env_logger::init();
    let event_loop = EventLoop::new().unwrap();
    let window = window_setup(&event_loop);

    let mut state = State::new(&window).await;
    let mut surface_configured = false;
    
    //println!("Inner size: {:?}", window.inner_size());
    //println!("Outer size: {:?}", window.outer_size());
    let target_fps = 10000;
    let frame_duration = Duration::from_secs_f64(1.0 / target_fps as f64);
    let mut last_frame_time = Instant::now();

    let mut frame_count = 0;
    let mut last_fps_time = Instant::now();
    let mut fps = 0;


    let _ = event_loop.run(move |event, control_flow| {
        let now = Instant::now();
        let frame_time = now.duration_since(last_frame_time);

        // If the frame time is shorter than the target, sleep to maintain FPS
        if frame_time < frame_duration {
            let sleep_duration = frame_duration - frame_time;
            std::thread::sleep(sleep_duration);
        }

        last_frame_time = now;
        frame_count += 1;

        // Calculate FPS every second
        if now.duration_since(last_fps_time) >= Duration::from_secs(1) {
            fps = frame_count;
            frame_count = 0;
            last_fps_time = now;
        }

        state.window().set_title(format!("FPS: {}", fps).as_str());
        println!("FPS: {}", fps);

        match event {

            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == state.window().id() => 
                if !state.input(event) {
                    match event {
                        WindowEvent::CloseRequested
                        | WindowEvent::KeyboardInput {
                            event:
                                KeyEvent {
                                    state: ElementState::Pressed,
                                    physical_key: PhysicalKey::Code(KeyCode::Escape),
                                    ..
                                },
                            ..
                        } => control_flow.exit(),
                        WindowEvent::Resized(physical_size) => {
                            log::info!("Physical sized: {physical_size:?}");
                            surface_configured = true;
                            state.resize(*physical_size);
                        },
                        WindowEvent::RedrawRequested => {
                            state.window().request_redraw();

                            if !surface_configured {
                                return;
                            }

                            state.update();
                            match state.render() {
                                Ok(_) => {}
                                // Reconfigure the surface if it's lost or outdated
                                Err(
                                    wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated,
                                ) => state.resize(state.size),
                                // The system is out of memory, we should probably quit
                                Err(wgpu::SurfaceError::OutOfMemory) => {
                                    log::error!("OutOfMemory");
                                    control_flow.exit();
                                }

                                // This happens when the a frame takes too long to present
                                Err(wgpu::SurfaceError::Timeout) => {
                                    log::warn!("Surface timeout")
                                }
                            }
                        },
                        _ => {}
                    }
                },
                _ => {}
        
                }
            }); 
}





 