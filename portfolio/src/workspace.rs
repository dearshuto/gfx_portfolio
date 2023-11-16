use demolib::TriangleParams;

use crate::DemoType;

pub struct Workspace {
    demo_type: DemoType,
    triangle_params: TriangleParams,
}

impl Workspace {
    pub fn new() -> Self {
        Self {
            demo_type: DemoType::Triangle,
            triangle_params: TriangleParams {
                color: [0.1, 0.2, 0.3],
            },
        }
    }

    pub fn update(&mut self, _device: &wgpu::Device, _queue: &wgpu::Queue) {}

    pub fn draw<'a>(&self, _render_pass: &mut wgpu::RenderPass<'a>) {}

    pub fn get_demo_types() -> &'static [(DemoType, &'static str)] {
        &[
            (DemoType::Triangle, "Triangle"),
            (DemoType::Mandelbrot, "Mandelbrot"),
            (DemoType::Model3d, "Model3d"),
            (DemoType::Tetris, "Tetris"),
            (DemoType::Physics, "Physics"),
        ]
    }

    pub fn get_current_demo_type(&self) -> DemoType {
        self.demo_type
    }

    pub fn set_demo_type(&mut self, demo_type: DemoType) {
        self.demo_type = demo_type;
    }

    pub fn get_triangle_params(&self) -> &TriangleParams {
        &self.triangle_params
    }

    pub fn get_triangle_params_mut(&mut self) -> &mut TriangleParams {
        &mut self.triangle_params
    }
}
