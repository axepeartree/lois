fn main() {
    println!("cargo:rerun-if-changed={}", "./shaders");
    let mut compiler = shaderc::Compiler::new().expect("Unable to create shaderc compiler");
    let vs_spirv = compiler
        .compile_into_spirv(
            include_str!("./shaders/shader.vert"),
            shaderc::ShaderKind::Vertex,
            "shader.vert",
            "main",
            None,
        )
        .unwrap();
    let fs_spirv = compiler
        .compile_into_spirv(
            include_str!("./shaders/shader.frag"),
            shaderc::ShaderKind::Fragment,
            "shader.frag",
            "main",
            None,
        )
        .unwrap();
    std::fs::write("shaders/out/shader.vert.spv", vs_spirv.as_binary_u8()).unwrap();
    std::fs::write("shaders/out/shader.frag.spv", fs_spirv.as_binary_u8()).unwrap();
}