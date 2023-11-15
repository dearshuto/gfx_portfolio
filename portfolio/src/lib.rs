use std::sync::{Arc, Mutex};

//mod mandelbrot;
mod property_panel;
mod workspace;

use demolib::{Mandelbrot, Triangle};
use eframe::egui_wgpu::{CallbackResources, CallbackTrait};
use futures_intrusive::channel::shared::GenericOneshotReceiver;
pub use property_panel::PropertyPanel;

//pub use mandelbrot::Mandelbrot;
use parking_lot::RawMutex;
use wgpu::BufferAsyncError;
pub use workspace::Workspace;

#[derive(PartialEq, Clone, Copy)]
pub enum DemoType {
    Triangle,
    Mandelbrot,
    Model3d,
    Physics,
    Tetris,
}

pub struct DemoManager<'a> {
    workspace: Arc<Mutex<Workspace>>,
    triangle: Triangle<'a>,
    mandelbrot: Mandelbrot<'a>,
}

impl<'a> DemoManager<'a> {
    pub fn new(
        workspace: Arc<Mutex<Workspace>>,
        device: Arc<wgpu::Device>,
        target: wgpu::TextureFormat,
    ) -> Self {
        Self {
            workspace,
            triangle: Triangle::new(&device, target),
            mandelbrot: Mandelbrot::new(&device, target),
        }
    }

    pub fn update(&mut self) {
        // let workspace = self.workspace.lock().unwrap();
        // match workspace.get_current_demo_type() {
        //     DemoType::Triangle => self.triangle.update(),
        //     _ => {}
        // }
    }

    pub async fn do_something(&mut self) {}

    pub fn draw(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        let workspace = self.workspace.lock().unwrap();
        match workspace.get_current_demo_type() {
            DemoType::Triangle => self.triangle.draw(render_pass),
            DemoType::Mandelbrot => self.mandelbrot.draw(render_pass),
            _ => {}
        }
    }
}

trait IDemoImpl<'a> {
    type TParams;

    fn new(device: &wgpu::Device, target_format: wgpu::TextureFormat) -> Self;

    fn update(
        &mut self,
        param: &Self::TParams,
    ) -> GenericOneshotReceiver<RawMutex, Result<(), BufferAsyncError>>;

    fn draw(&'a self, render_pass: &mut wgpu::RenderPass<'a>);
}

pub struct RenderBridge;

impl RenderBridge {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for RenderBridge {
    fn default() -> Self {
        Self::new()
    }
}

impl CallbackTrait for RenderBridge {
    fn prepare(
        &self,
        _device: &wgpu::Device,
        _queue: &wgpu::Queue,
        _egui_encoder: &mut wgpu::CommandEncoder,
        callback_resources: &mut CallbackResources,
    ) -> Vec<wgpu::CommandBuffer> {
        let Some(demo_manager): Option<&mut DemoManager> = callback_resources.get_mut() else {
            return Vec::new();
        };

        demo_manager.update();

        Vec::new()
    }

    fn paint<'a>(
        &'a self,
        _info: eframe::epaint::PaintCallbackInfo,
        render_pass: &mut wgpu::RenderPass<'a>,
        callback_resources: &'a eframe::egui_wgpu::CallbackResources,
    ) {
        let Some(demo_manager): Option<&DemoManager> = callback_resources.get() else {
            return;
        };

        demo_manager.draw(render_pass);
    }
}
