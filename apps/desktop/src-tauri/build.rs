use std::{env, path::PathBuf, process::Command};

const LLAMA_CPP_CMAKE_OPTIONS: &[&str] = &[
    "-DBUILD_SHARED_LIBS=OFF",
    "-DLLAMA_BUILD_NUMBER=10326",
    "-DLLAMA_BUILD_COMMIT=3653e6d6d547ec763317d9ecd0ace334a7e21359",
    "-DGGML_BUILD_NUMBER=10326",
    "-DGGML_BUILD_COMMIT=3653e6d6d547ec763317d9ecd0ace334a7e21359",
    "-DLLAMA_BUILD_COMMON=OFF",
    "-DLLAMA_BUILD_TESTS=OFF",
    "-DLLAMA_BUILD_TOOLS=OFF",
    "-DLLAMA_BUILD_EXAMPLES=OFF",
    "-DLLAMA_BUILD_SERVER=OFF",
    "-DLLAMA_BUILD_APP=OFF",
    "-DLLAMA_BUILD_UI=OFF",
    "-DLLAMA_USE_PREBUILT_UI=OFF",
    "-DLLAMA_BUILD_MTMD=OFF",
    "-DLLAMA_TOOLS_INSTALL=OFF",
    "-DLLAMA_TESTS_INSTALL=OFF",
    "-DLLAMA_OPENSSL=OFF",
    "-DLLAMA_SUBPROCESS=OFF",
    "-DLLAMA_USE_SYSTEM_GGML=OFF",
    "-DGGML_STATIC=ON",
    "-DGGML_BACKEND_DL=OFF",
    "-DGGML_CPU=ON",
    "-DGGML_NATIVE=OFF",
    "-DGGML_BLAS=OFF",
    "-DGGML_ACCELERATE=OFF",
    "-DGGML_OPENMP=OFF",
    "-DGGML_CCACHE=OFF",
    "-DGGML_CUDA=OFF",
    "-DGGML_HIP=OFF",
    "-DGGML_VULKAN=OFF",
    "-DGGML_KOMPUTE=OFF",
    "-DGGML_METAL=OFF",
    "-DGGML_SYCL=OFF",
    "-DGGML_CANN=OFF",
    "-DGGML_MUSA=OFF",
    "-DGGML_OPENCL=OFF",
    "-DGGML_WEBGPU=OFF",
    "-DGGML_OPENVINO=OFF",
    "-DGGML_ET=OFF",
    "-DGGML_HEXAGON=OFF",
    "-DGGML_ZDNN=OFF",
    "-DGGML_VIRTGPU=OFF",
    "-DGGML_RPC=OFF",
];

const CLOSED_CMAKE_ENVIRONMENT: &[&str] = &[
    "CC",
    "CXX",
    "CPPFLAGS",
    "CFLAGS",
    "CXXFLAGS",
    "LDFLAGS",
    "CMAKE_GENERATOR",
    "CMAKE_GENERATOR_INSTANCE",
    "CMAKE_GENERATOR_PLATFORM",
    "CMAKE_GENERATOR_TOOLSET",
    "CMAKE_TOOLCHAIN_FILE",
    "CMAKE_PREFIX_PATH",
    "CMAKE_INCLUDE_PATH",
    "CMAKE_LIBRARY_PATH",
    "CMAKE_PROGRAM_PATH",
    "CMAKE_FRAMEWORK_PATH",
    "CMAKE_APPBUNDLE_PATH",
];

fn run(command: &mut Command, description: &str) {
    let status = command
        .status()
        .unwrap_or_else(|error| panic!("{description} could not start: {error}"));
    assert!(status.success(), "{description} failed with {status}");
}

fn build_llama_cpp() {
    let manifest_dir =
        PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").expect("manifest directory"));
    let source_dir = manifest_dir.join("../../../third_party/llama.cpp");
    let output_dir = PathBuf::from(env::var_os("OUT_DIR").expect("Cargo output directory"));
    let build_dir = output_dir.join("m63-llama.cpp-build");

    assert!(
        source_dir.join("PROVENANCE.json").is_file(),
        "missing verified llama.cpp source"
    );
    println!("cargo:rerun-if-changed={}", source_dir.display());

    let mut configure = Command::new("cmake");
    configure
        .arg("-S")
        .arg(&source_dir)
        .arg("-B")
        .arg(&build_dir)
        .arg("-DCMAKE_BUILD_TYPE=Release");
    configure.args(LLAMA_CPP_CMAKE_OPTIONS);
    for variable in CLOSED_CMAKE_ENVIRONMENT {
        configure.env_remove(variable);
    }
    run(
        &mut configure,
        "closed llama.cpp static-library configuration",
    );

    let mut build = Command::new("cmake");
    build
        .arg("--build")
        .arg(&build_dir)
        .arg("--target")
        .arg("llama");
    run(&mut build, "closed llama.cpp static-library build");

    println!(
        "cargo:rustc-link-search=native={}",
        build_dir.join("src").display()
    );
    println!(
        "cargo:rustc-link-search=native={}",
        build_dir.join("ggml/src").display()
    );
    for library in ["llama", "ggml", "ggml-base", "ggml-cpu"] {
        println!("cargo:rustc-link-lib=static={library}");
    }
    println!("cargo:rustc-link-lib=dylib=stdc++");
}

fn main() {
    build_llama_cpp();
    tauri_build::build();
}
