use crate::DemoType;

pub struct Workspace {
    demo_type: DemoType,
}

impl Workspace {
    pub fn new() -> Self {
        Self {
            demo_type: DemoType::Triangle,
        }
    }

    pub fn update(&mut self, _device: &wgpu::Device, _queue: &wgpu::Queue) {}

    pub fn draw<'a>(&self, _render_pass: &mut wgpu::RenderPass<'a>) {}

    pub fn get_demo_types() -> &'static [(DemoType, &'static str)] {
        &[
            (DemoType::Triangle, "Triangle"),
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
}
