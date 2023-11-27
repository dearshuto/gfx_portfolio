use std::{collections::HashMap, fs::File, io::Write};

use naga::back::wgsl::WriterFlags;

fn main() {
    let targets = [
        (
            "resources/shaders/triangle.vs",
            "src/triangle.vs.wgsl",
            naga::ShaderStage::Vertex,
        ),
        (
            "resources/shaders/triangle.fs",
            "src/triangle.fs.wgsl",
            naga::ShaderStage::Fragment,
        ),
        (
            "resources/shaders/mandelbrot.vs",
            "src/mandelbrot.vs.wgsl",
            naga::ShaderStage::Vertex,
        ),
        (
            "resources/shaders/mandelbrot.fs",
            "src/mandelbrot.fs.wgsl",
            naga::ShaderStage::Fragment,
        ),
        (
            "resources/shaders/mandelbrot.vs",
            "src/mandelbrot.vs.wgsl",
            naga::ShaderStage::Vertex,
        ),
        (
            "resources/shaders/mandelbrot.fs",
            "src/mandelbrot.fs.wgsl",
            naga::ShaderStage::Fragment,
        ),
        (
            "resources/shaders/model_3d.vs",
            "src/model_3d.vs.wgsl",
            naga::ShaderStage::Vertex,
        ),
        (
            "resources/shaders/model_3d.fs",
            "src/model_3d.fs.wgsl",
            naga::ShaderStage::Fragment,
        ),
    ];

    for (src, dst, stage) in targets {
        let source = std::fs::read_to_string(src).unwrap();
        let vertex_shader_binary = convert_to_spv(&source, stage);
        let mut vertex_shader_binary_file = File::create(dst).unwrap();
        vertex_shader_binary_file
            .write_all(&vertex_shader_binary)
            .unwrap();
    }
}

fn convert_to_spv(source: &str, stage: naga::ShaderStage) -> Vec<u8> {
    convert_to_spv_with_defines(source, stage, &HashMap::default())
}

fn convert_to_spv_with_defines(
    source: &str,
    stage: naga::ShaderStage,
    define_map: &HashMap<String, String>,
) -> Vec<u8> {
    let mut options = naga::front::glsl::Options::from(stage);
    for (key, value) in define_map {
        options.defines.insert(key.clone(), value.clone());
    }
    let vertex_module = naga::front::glsl::Frontend::default()
        .parse(&options, source)
        .unwrap();

    let info = naga::valid::Validator::new(
        naga::valid::ValidationFlags::all(),
        naga::valid::Capabilities::all(),
    )
    .validate(&vertex_module)
    .unwrap();

    let Ok(string) = naga::back::wgsl::write_string(&vertex_module, &info, WriterFlags::all())
    else {
        return Vec::default();
    };

    return string.as_bytes().to_vec();
}
