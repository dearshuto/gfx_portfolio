use std::{collections::HashMap, fs::File, io::Write};

use naga::back::wgsl::WriterFlags;

fn main() {
    {
        let vertex_shader_binary = convert_to_spv(
            include_str!("resources/shaders/triangle.vs"),
            naga::ShaderStage::Vertex,
        );
        let mut vertex_shader_binary_file = File::create("src/triangle.vs.wgsl").unwrap();
        vertex_shader_binary_file
            .write_all(&vertex_shader_binary)
            .unwrap();

        let pixel_shader_binary = convert_to_spv(
            include_str!("resources/shaders/triangle.fs"),
            naga::ShaderStage::Fragment,
        );
        let mut pixel_shader_binary_file = File::create("src/triangle.fs.wgsl").unwrap();
        pixel_shader_binary_file
            .write_all(&pixel_shader_binary)
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
