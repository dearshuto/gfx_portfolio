use std::borrow::Cow;

use futures_intrusive::channel::shared::GenericOneshotReceiver;
use parking_lot::RawMutex;
use wgpu::{util::DeviceExt, BufferAsyncError};

pub struct TriangleParams {
    pub color: [f32; 3],
}

pub struct Triangle<'a> {
    render_pipeline: wgpu::RenderPipeline,
    //shader_module: wgpu::ShaderModule,
    vertex_buffer: wgpu::Buffer,
    constant_buffer: wgpu::Buffer,
    _merker: std::marker::PhantomData<&'a ()>,
}

impl<'a> Triangle<'a> {
    pub fn new(device: &wgpu::Device, target_format: wgpu::TextureFormat) -> Self {
        let vertex_shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("triangle.vs.wgsl"))),
        });
        let pixel_shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("triangle.fs.wgsl"))),
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(
                &device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: None,
                    bind_group_layouts: &[],
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
                    format: target_format,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: Default::default(),
            depth_stencil: Default::default(),
            multisample: Default::default(),
            multiview: Default::default(),
        });

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&[-0.5f32, -0.5, 0.5, -0.5, 0.0, 0.5]),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let constant_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&[1.0f32, 1.0, 1.0, 1.0]),
            usage: wgpu::BufferUsages::UNIFORM,
        });
        Self {
            render_pipeline,
            vertex_buffer,
            constant_buffer,
            _merker: std::marker::PhantomData,
        }
    }

    pub fn update(
        &mut self,
        _params: &TriangleParams,
    ) -> GenericOneshotReceiver<RawMutex, Result<(), BufferAsyncError>> {
        let (sender, receiver) = futures_intrusive::channel::shared::oneshot_channel();
        self.constant_buffer
            .slice(..)
            .map_async(wgpu::MapMode::Write, move |v| sender.send(v).unwrap());
        receiver
    }

    pub fn draw(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.draw(0..3, 0..1);
        // render_pass.draw_indexed(0..6, 0 /*base*/, 0..1 /*instance*/);
    }
}
