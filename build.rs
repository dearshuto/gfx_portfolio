use std::io::Write;
use std::path::Path;

fn main() {
    convert_triangle_resources();
    convert_mandelbrot_resoruces();
}

fn convert_triangle_resources() {
    let triangle_vs = convert(
        include_str!("portfolio/resources/shaders/triangle.vs"),
        naga::ShaderStage::Vertex,
    );
    let triangle_fs = convert(
        include_str!("portfolio/resources/shaders/triangle.fs"),
        naga::ShaderStage::Fragment,
    );
    write_to_file(&triangle_vs, "triangle.vs.spv");
    write_to_file(&triangle_fs, "triangle.fs.spv");
}

fn convert_mandelbrot_resoruces() {
    /*
    let mandelbrot_vs = convert(
        include_str!("portfolio/resources/shaders/triangle.vs"),
        naga::ShaderStage::Vertex,
    );
    let mandelbrot_fs = convert(
        include_str!("portfolio/resources/shaders/triangle.fs"),
        naga::ShaderStage::Fragment,
    );
    */
}

fn write_to_file<TPath>(data: &[u8], path: TPath)
where
    TPath: AsRef<Path>,
{
    let root = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let mut output_file = std::path::PathBuf::from(root);
    output_file.push("portfolio");
    output_file.push("outputs");
    output_file.push(path);
    std::fs::File::create(output_file)
        .unwrap()
        .write(data)
        .unwrap();
}

fn convert(source: &str, shader_stage: naga::ShaderStage) -> Vec<u8> {
    let glsl_options = naga::front::glsl::Options::from(shader_stage);
    let module = naga::front::glsl::Frontend::default()
        .parse(&glsl_options, source)
        .unwrap();
    let info = naga::valid::Validator::new(
        naga::valid::ValidationFlags::all(),
        naga::valid::Capabilities::all(),
    )
    .validate(&module)
    .unwrap();
    naga::back::spv::Options::default();

    let u8_data = unsafe {
        let options = naga::back::spv::Options::default();
        let mut data = naga::back::spv::write_vec(&module, &info, &options, None).unwrap();

        let ratio = std::mem::size_of::<u32>() / std::mem::size_of::<u8>();
        let length = data.len() * ratio;
        let capacity = data.capacity() * ratio;
        let ptr = data.as_mut_ptr() as *mut u8;
        let u8_data: Vec<u8> = Vec::from_raw_parts(ptr, length, capacity).clone();

        // 元データが 2 重に破棄されないように、元データを破棄しないようにする
        std::mem::forget(data);

        u8_data
    };

    return u8_data;
}
