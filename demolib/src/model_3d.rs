use std::borrow::Cow;

use usd_rs::serializer::PropertyType;
use wgpu::util::DeviceExt;

#[derive(bytemuck::NoUninit, Clone, Copy)]
#[repr(C)]
pub struct Model3dParams {
    mvp: [f32; 16],
}

pub struct Model3d<'a> {
    render_pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    #[allow(dead_code)]
    constant_buffer: wgpu::Buffer,
    _marker: std::marker::PhantomData<&'a ()>,
}

impl<'a> Model3d<'a> {
    pub fn new(device: &wgpu::Device, target_format: wgpu::TextureFormat) -> Self {
        let torus_usd =
            usd_rs::serializer::from_str(include_str!("../resources/models/torus.usda")).unwrap();

        let vertex_shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("model_3d.vs.wgsl"))),
        });
        let pixel_shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("model_3d.fs.wgsl"))),
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(
                &device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: None,
                    bind_group_layouts: &[&device.create_bind_group_layout(
                        &wgpu::BindGroupLayoutDescriptor {
                            label: None,
                            entries: &[wgpu::BindGroupLayoutEntry {
                                binding: 0,
                                visibility: wgpu::ShaderStages::VERTEX,
                                ty: wgpu::BindingType::Buffer {
                                    ty: wgpu::BufferBindingType::Uniform,
                                    has_dynamic_offset: false,
                                    min_binding_size: None,
                                },
                                count: None,
                            }],
                        },
                    )],
                    push_constant_ranges: &[],
                }),
            ),
            vertex: wgpu::VertexState {
                module: &vertex_shader_module,
                entry_point: "main",
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: (std::mem::size_of::<f32>() * 6) as wgpu::BufferAddress,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &[
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x3,
                            offset: 0,
                            shader_location: 0,
                        },
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x3,
                            offset: (std::mem::size_of::<f32>() * 3) as wgpu::BufferAddress,
                            shader_location: 1,
                        },
                    ],
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
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::LessEqual,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: Default::default(),
            multiview: Default::default(),
        });

        let point_and_normal = torus_usd.definitions()[0]
            .properties
            .iter()
            .filter_map(|p| match p.property {
                PropertyType::Points(_) => Some(&p.property),
                PropertyType::Normals(_) => Some(&p.property),
                _ => None,
            })
            .collect::<Vec<&PropertyType>>();
        let (points, normals) = match &point_and_normal[0] {
            PropertyType::Points(points) => match &point_and_normal[1] {
                PropertyType::Normals(normals) => (points, normals),
                _ => panic!(),
            },
            PropertyType::Normals(normals) => match &point_and_normal[1] {
                PropertyType::Points(points) => (points, normals),
                _ => panic!(),
            },
            _ => panic!(),
        };
        let mut vertex_data = Vec::default();
        for index in 0..points.len() {
            let point = &points[index];
            // なぜか法線が 3 つ同じのが出力されてる
            let normal = &normals[3 * index];
            vertex_data.push(point[0]);
            vertex_data.push(point[1]);
            vertex_data.push(point[2]);
            vertex_data.push(normal[0]);
            vertex_data.push(normal[1]);
            vertex_data.push(normal[2]);
        }
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&vertex_data),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_data = torus_usd.definitions()[0]
            .properties
            .iter()
            .find_map(|property| match &property.property {
                PropertyType::FaceVertexIndicies(data) => Some(data),
                _ => None,
            })
            .unwrap()
            .iter()
            .map(|d| *d as u32)
            .collect::<Vec<u32>>();
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&index_data),
            usage: wgpu::BufferUsages::INDEX,
        });

        let projection_matrix =
            nalgebra_glm::perspective_lh_zo(1.0f32, 60f32.to_radians(), 0.1, 100.0);
        let view_matrix = nalgebra_glm::look_at_lh(
            &nalgebra_glm::Vec3::new(2.0, 2.0, 1.5),
            &nalgebra_glm::Vec3::new(0.0, 0.0, 0.0),
            &nalgebra_glm::Vec3::new(0.0, 0.0, 1.0),
        );

        let pv = projection_matrix * view_matrix;
        // Column-Major を Row-Major にするための転置
        let pv = pv.transpose();
        let constant_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            // contents: bytemuck::bytes_of(&Model3dParams {
            //     mvp: Default::default(),
            // }),
            contents: bytemuck::cast_slice(pv.as_slice()),
            usage: wgpu::BufferUsages::UNIFORM,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: constant_buffer.as_entire_binding(),
            }],
        });

        Self {
            render_pipeline,
            bind_group,
            vertex_buffer,
            index_buffer,
            constant_buffer,
            _marker: std::marker::PhantomData,
        }
    }

    pub fn draw(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, &self.bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        render_pass.draw_indexed(0..3456, 0, 0..1);
    }
}
