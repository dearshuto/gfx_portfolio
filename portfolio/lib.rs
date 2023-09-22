use std::sync::{Arc, Mutex};

//mod mandelbrot;
mod triangle;
mod workspace;

use futures_intrusive::channel::shared::GenericOneshotReceiver;
//pub use mandelbrot::Mandelbrot;
use parking_lot::RawMutex;
pub use triangle::Triangle;
use wgpu::BufferAsyncError;
pub use workspace::Workspace;

#[derive(PartialEq, Clone, Copy)]
pub enum DemoType {
    Triangle,
    Model3d,
    Physics,
    Tetris,
}

pub struct DemoManager<'a> {
    workspace: Arc<Mutex<Workspace>>,
    triangle: Demo<'a, Triangle<'a>>,
}

impl<'a> DemoManager<'a> {
    pub fn new(
        workspace: Arc<Mutex<Workspace>>,
        device: Arc<wgpu::Device>,
        target: wgpu::TextureFormat,
    ) -> Self {
        Self {
            workspace,
            triangle: Demo::<Triangle>::new(device, target),
        }
    }

    pub fn update(&mut self) {
        let workspace = self.workspace.lock().unwrap();
        match workspace.get_current_demo_type() {
            DemoType::Triangle => self.triangle.update(),
            _ => {}
        }
    }

    pub async fn do_something(&mut self) {}

    pub fn draw(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        let workspace = self.workspace.lock().unwrap();
        match workspace.get_current_demo_type() {
            DemoType::Triangle => self.triangle.draw(render_pass),
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

struct Demo<'a, TResource: IDemoImpl<'a>> {
    device: Arc<wgpu::Device>,
    resource: Option<TResource>,
    format: wgpu::TextureFormat,
    _marker: std::marker::PhantomData<&'a ()>,
}

impl<'a, TDemoImpl: IDemoImpl<'a>> Demo<'a, TDemoImpl> {
    pub fn new(device: Arc<wgpu::Device>, target_format: wgpu::TextureFormat) -> Self {
        Self {
            device,
            resource: None,
            format: target_format,
            _marker: std::marker::PhantomData,
        }
    }

    pub fn update(&mut self) {
        if self.resource.is_none() {
            let demo = TDemoImpl::new(&self.device, self.format);

            self.resource = Some(demo);
        }

        //self.resource.as_mut().unwrap().update();
    }

    pub fn draw(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        self.resource.as_ref().unwrap().draw(render_pass);
    }
}
