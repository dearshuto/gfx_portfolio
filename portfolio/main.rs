use std::sync::{Arc, Mutex};

use demolib::Workspace;
use eframe::CreationContext;

fn main() {
    #[cfg(not(target_arch = "wasm32"))]
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(4)
        .build()
        .unwrap();

    #[cfg(target_arch = "wasm32")]
    let runtime = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();

    let runtime = Arc::new(runtime);
    run(runtime);
}

fn run(runtime: Arc<tokio::runtime::Runtime>) {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let options = eframe::NativeOptions {
            renderer: eframe::Renderer::Wgpu,
            ..Default::default()
        };

        eframe::run_native(
            "My Window",
            options,
            Box::new(|cc| Box::new(App::new(runtime, cc))),
        )
        .unwrap();
    }

    #[cfg(target_arch = "wasm32")]
    {
        let web_options = eframe::WebOptions::default();

        wasm_bindgen_futures::spawn_local(async {
            eframe::WebRunner::new()
                .start(
                    "canvas", // hardcode it
                    web_options,
                    Box::new(|cc| Box::new(App::new(runtime, cc))),
                )
                .await
                .expect("failed to start eframe");
        });
    }
}

struct App {
    #[allow(dead_code)]
    runtime: Arc<tokio::runtime::Runtime>,
    workspace: Arc<Mutex<Workspace>>,
}

impl App {
    pub fn new(runtime: Arc<tokio::runtime::Runtime>, context: &CreationContext) -> Self {
        let workspace = Arc::new(Mutex::new(Workspace::new()));
        if let Some(render_state) = &context.wgpu_render_state {
            let target_format = render_state.target_format;
            let device = render_state.device.clone();
            let demo_manager =
                demolib::DemoManager::new(workspace.clone(), device.clone(), target_format);
            let _ = context
                .wgpu_render_state
                .as_ref()
                .unwrap()
                .renderer
                .write()
                .paint_callback_resources
                .insert(demo_manager);
            Self { workspace, runtime }
        } else {
            Self { runtime, workspace }
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint();
        eframe::egui::SidePanel::left("Demo List")
            .resizable(false)
            .default_width(150.0)
            .show(ctx, |ui| {
                let mut binding = self.workspace.lock();
                let workspace = binding.as_mut().unwrap();
                let mut current_demo_type = workspace.get_current_demo_type();
                for (demo_type, lanel) in Workspace::get_demo_types() {
                    ui.radio_value(&mut current_demo_type, *demo_type, *lanel);
                }
                workspace.set_demo_type(current_demo_type);
            });

        eframe::egui::CentralPanel::default().show(ctx, |ui| {
            eframe::egui::Frame::canvas(ui.style()).show(ui, |ui| {
                let (rect, _response) = ui.allocate_exact_size(
                    eframe::egui::vec2(400.0, 400.0),
                    eframe::egui::Sense::drag(),
                );

                let callback = {
                    let function = eframe::egui_wgpu::CallbackFn::new()
                        .prepare(move |_device, _queue, _command_encoder, render_resources| {
                            let demo_manager: &mut demolib::DemoManager =
                                render_resources.get_mut().unwrap();

                            /*
                            let task = tokio::spawn(async {

                                for _index in 0..3 {
                                     println!("Update");
                                     std::thread::sleep(std::time::Duration::from_millis(10));
                                }
                            });
                            */

                            demo_manager.update();

                            // futures::executor::block_on(task).unwrap();
                            Vec::default()
                        })
                        .paint(move |_info, render_pass, render_resources| {
                            let demo_manager: &demolib::DemoManager =
                                render_resources.get().unwrap();
                            demo_manager.draw(render_pass);
                        });
                    eframe::egui::PaintCallback {
                        rect,
                        callback: Arc::new(function),
                    }
                };

                ui.painter().add(callback);
            });
        });
    }
}
