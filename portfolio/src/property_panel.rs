use std::sync::{Arc, Mutex};

use demolib::TriangleParams;
use eframe::egui::Ui;

use crate::Workspace;

pub struct PropertyPanel {
    workspace: Arc<Mutex<Workspace>>,
}

impl PropertyPanel {
    pub fn new(workspace: Arc<Mutex<Workspace>>) -> Self {
        PropertyPanel { workspace }
    }

    pub fn draw(&self, ui: &mut Ui) {
        let mut workspace = self.workspace.lock().unwrap();
        let demo_type = workspace.get_current_demo_type();

        match demo_type {
            crate::DemoType::Triangle => {
                Self::draw_triangle_properties(ui, workspace.get_triangle_params_mut())
            }
            crate::DemoType::Mandelbrot => Self::draw_mandelbrot_properties(ui),
            crate::DemoType::Model3d => {}
            crate::DemoType::Physics => {}
            crate::DemoType::Tetris => {}
        }
    }

    fn draw_triangle_properties(ui: &mut Ui, triangle_params: &mut TriangleParams) {
        ui.horizontal(|ui| {
            ui.label("Color");
            ui.color_edit_button_rgb(&mut triangle_params.color);
        });
    }

    fn draw_mandelbrot_properties(ui: &mut Ui) {
        ui.label("Nothing");
    }
}
