use eframe::{egui_wgpu::Callback, CreationContext};
use std::sync::{Arc, Mutex};

use portfolio::{DemoManager, PropertyPanel, RenderBridge, Workspace};

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
        let options = eframe::NativeOptions::default();

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
    property_panel: PropertyPanel,
}

impl App {
    pub fn new(runtime: Arc<tokio::runtime::Runtime>, context: &CreationContext) -> Self {
        let workspace = Arc::new(Mutex::new(Workspace::new()));
        if let Some(render_state) = &context.wgpu_render_state {
            let target_format = render_state.target_format;
            let device = render_state.device.clone();
            let demo_manager = DemoManager::new(workspace.clone(), device.clone(), target_format);
            context
                .wgpu_render_state
                .as_ref()
                .unwrap()
                .renderer
                .write()
                .callback_resources
                .insert(demo_manager);
            Self {
                workspace: workspace.clone(),
                runtime,
                property_panel: PropertyPanel::new(workspace.clone()),
            }
        } else {
            Self {
                runtime,
                workspace: workspace.clone(),
                property_panel: PropertyPanel::new(workspace.clone()),
            }
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
        eframe::egui::SidePanel::right("Property")
            .resizable(true)
            .default_width(150.0)
            .show(ctx, |ui| {
                ui.heading("Properties");
                self.property_panel.draw(ui);
            });

        eframe::egui::CentralPanel::default().show(ctx, |ui| {
            eframe::egui::Frame::canvas(ui.style()).show(ui, |ui| {
                let (rect, _response) = ui.allocate_exact_size(
                    eframe::egui::vec2(700.0, 700.0),
                    eframe::egui::Sense::drag(),
                );

                let callback = Callback::new_paint_callback(rect, RenderBridge::new());
                ui.painter().add(callback);
            });
        });
    }
}
