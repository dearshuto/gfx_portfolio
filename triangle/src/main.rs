use std::time::{Duration, Instant};

use winit::{
    event::{Event, StartCause, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

#[tokio::main]
async fn main() {
    let event_loop = EventLoop::new();
    let Ok(window) = WindowBuilder::new().build(&event_loop) else {
        return;
    };

    let instance = wgpu::Instance::default();

    let surface = unsafe { instance.create_surface(&window) }.unwrap();
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            force_fallback_adapter: false,
            compatible_surface: Some(&surface),
        })
        .await
        .unwrap();
    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                features: wgpu::Features::SPIRV_SHADER_PASSTHROUGH,
                limits: wgpu::Limits::default().using_resolution(adapter.limits()),
            },
            None,
        )
        .await
        .unwrap();

    let timer_length = Duration::from_millis(16);

    let swapchain_capabilities = surface.get_capabilities(&adapter);
    let swapchain_format = swapchain_capabilities.formats[0];
    let mut config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: swapchain_format,
        width: 640,
        height: 480,
        present_mode: wgpu::PresentMode::Fifo,
        #[cfg(not(any(target_os = "macos", windows)))]
        alpha_mode: wgpu::CompositeAlphaMode::Auto,
        #[cfg(target_os = "macos")]
        alpha_mode: swapchain_capabilities.alpha_modes[0],
        #[cfg(target_os = "windows")]
        alpha_mode: swapchain_capabilities.alpha_modes[0],
        view_formats: vec![],
    };
    surface.configure(&device, &config);

    let triangle = demolib::Triangle::new(&device, swapchain_format);

    event_loop.run(move |event, _, control_flow| match event {
        Event::NewEvents(StartCause::Init) => {
            *control_flow = ControlFlow::WaitUntil(Instant::now() + timer_length);
        }
        Event::NewEvents(StartCause::ResumeTimeReached { .. }) => {
            *control_flow = ControlFlow::WaitUntil(Instant::now() + timer_length);
        }
        Event::RedrawRequested(_) => {
            let frame = surface.get_current_texture().unwrap();
            let view = frame
                .texture
                .create_view(&wgpu::TextureViewDescriptor::default());

            let mut command_encoder =
                device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

            {
                let mut render_pass =
                    command_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: None,
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: &view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color {
                                    r: 0.1,
                                    g: 0.2,
                                    b: 0.3,
                                    a: 1.0,
                                }),
                                store: true,
                            },
                        })],
                        depth_stencil_attachment: None,
                    });

                triangle.draw(&mut render_pass);
            }
            queue.submit(Some(command_encoder.finish()));
            frame.present();
        }
        Event::WindowEvent { event, .. } => match event {
            WindowEvent::Resized(size) => {
                config.width = size.width;
                config.height = size.height;
                surface.configure(&device, &config);

                window.request_redraw();
            }
            WindowEvent::CloseRequested => {
                *control_flow = ControlFlow::Exit;
            }
            _ => {}
        },
        _ => {}
    });
}
