use std::{
    borrow::Cow,
    sync::{Arc, Mutex},
};

//mod mandelbrot;
mod property_panel;
mod workspace;

use demolib::{Mandelbrot, Model3d, Triangle};
use eframe::egui_wgpu::{CallbackResources, CallbackTrait};
use futures_intrusive::channel::shared::GenericOneshotReceiver;
pub use property_panel::PropertyPanel;

//pub use mandelbrot::Mandelbrot;
use parking_lot::RawMutex;
use wgpu::{util::DeviceExt, BufferAsyncError};
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
    model_3d: Model3d<'a>,

    // 四角形描画
    render_pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    color_buffer: wgpu::Texture,
    depth_buffer: wgpu::Texture,
}

impl<'a> DemoManager<'a> {
    pub fn new(
        workspace: Arc<Mutex<Workspace>>,
        device: Arc<wgpu::Device>,
        target: wgpu::TextureFormat,
    ) -> Self {
        let vertex_shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("draw_texture.vs.wgsl"))),
        });
        let pixel_shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("draw_texture.fs.wgsl"))),
        });

        let color_buffer = device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d {
                width: 700,
                height: 700,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[wgpu::TextureFormat::Rgba8Unorm],
        });
        let depth_buffer = device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d {
                width: 700,
                height: 700,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor::default());

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&[
                -1.0f32, 1.0, //
                -1.0, -1.0, //
                1.0, -1.0, //
                1.0, 1.0, //
            ]),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&[0u16, 1, 2, 0, 2, 3]),
            usage: wgpu::BufferUsages::INDEX,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(
                &device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: None,
                    bind_group_layouts: &[&bind_group_layout],
                    push_constant_ranges: &[],
                }),
            ),
            vertex: wgpu::VertexState {
                module: &vertex_shader_module,
                entry_point: "main",
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: (std::mem::size_of::<f32>() * 2) as wgpu::BufferAddress,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &[wgpu::VertexAttribute {
                        format: wgpu::VertexFormat::Float32x2,
                        offset: 0,
                        shader_location: 0,
                    }],
                }],
            },
            fragment: Some(wgpu::FragmentState {
                module: &pixel_shader_module,
                entry_point: "main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: target,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: Default::default(),
            depth_stencil: None,
            multisample: Default::default(),
            multiview: Default::default(),
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&color_buffer.create_view(
                        &wgpu::TextureViewDescriptor {
                            label: None,
                            format: Some(wgpu::TextureFormat::Rgba8Unorm),
                            dimension: Some(wgpu::TextureViewDimension::D2),
                            aspect: wgpu::TextureAspect::All,
                            base_mip_level: 0,
                            mip_level_count: None,
                            base_array_layer: 0,
                            array_layer_count: None,
                        },
                    )),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });

        Self {
            workspace,
            triangle: Triangle::new(&device, wgpu::TextureFormat::Rgba8Unorm),
            mandelbrot: Mandelbrot::new(&device, wgpu::TextureFormat::Rgba8Unorm),
            model_3d: Model3d::new(&device, wgpu::TextureFormat::Rgba8Unorm),
            // 四角形描画
            render_pipeline,
            bind_group,
            vertex_buffer,
            index_buffer,
            color_buffer,
            depth_buffer,
        }
    }

    pub fn update(&mut self, queue: &wgpu::Queue) {
        let workspace = self.workspace.lock().unwrap();
        let triangle_params = workspace.get_triangle_params();
        self.triangle.update(queue, triangle_params);
    }

    pub async fn do_something(&mut self) {}

    pub fn draw_pre(
        &'a self,
        mut command_encoder: wgpu::CommandEncoder,
    ) -> Option<wgpu::CommandEncoder> {
        let workspace = self.workspace.lock().unwrap();

        let texture_view = self.color_buffer.create_view(&wgpu::TextureViewDescriptor {
            label: None,
            format: Some(wgpu::TextureFormat::Rgba8Unorm),
            dimension: Some(wgpu::TextureViewDimension::D2),
            aspect: wgpu::TextureAspect::All,
            base_mip_level: 0,
            mip_level_count: None,
            base_array_layer: 0,
            array_layer_count: None,
        });
        let depth_buffer_view = self.depth_buffer.create_view(&wgpu::TextureViewDescriptor {
            label: None,
            format: Some(wgpu::TextureFormat::Depth32Float),
            dimension: Some(wgpu::TextureViewDimension::D2),
            aspect: wgpu::TextureAspect::All,
            base_mip_level: 0,
            mip_level_count: None,
            base_array_layer: 0,
            array_layer_count: None,
        });

        {
            let is_depth_required = match workspace.get_current_demo_type() {
                DemoType::Triangle => false,
                DemoType::Mandelbrot => false,
                DemoType::Model3d => true,
                _ => false,
            };

            let mut render_pass = command_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &texture_view,
                    resolve_target: None,
                    ops: wgpu::Operations::default(),
                })],
                depth_stencil_attachment: if is_depth_required {
                    Some(wgpu::RenderPassDepthStencilAttachment {
                        view: &depth_buffer_view,
                        depth_ops: Some(wgpu::Operations {
                            load: wgpu::LoadOp::Clear(1.0),
                            store: true,
                        }),
                        stencil_ops: None,
                    })
                } else {
                    None
                },
            });

            match workspace.get_current_demo_type() {
                DemoType::Triangle => self.triangle.draw(&mut render_pass),
                DemoType::Mandelbrot => self.mandelbrot.draw(&mut render_pass),
                DemoType::Model3d => self.model_3d.draw(&mut render_pass),
                _ => {}
            }
        }

        Some(command_encoder)
    }

    pub fn draw(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        // カラーバッファーのコンテンツをスキャンバッファーにコピー
        // ただし eframe にカラーバッファーをコピーする API がないので、カラーバッファーを画面いっぱいに描画することで代用している
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, &self.bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.draw_indexed(0..6, 0, 0..1);
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
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        _egui_encoder: &mut wgpu::CommandEncoder,
        callback_resources: &mut CallbackResources,
    ) -> Vec<wgpu::CommandBuffer> {
        let Some(demo_manager): Option<&mut DemoManager> = callback_resources.get_mut() else {
            return Vec::new();
        };

        demo_manager.update(queue);

        let encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
        if let Some(encoder) = demo_manager.draw_pre(encoder) {
            vec![encoder.finish()]
        } else {
            Vec::default()
        }
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
