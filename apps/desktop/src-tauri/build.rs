use std::{
    env, fs,
    path::{Path, PathBuf},
    process::Command,
};

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
    "-DLLAMA_CURL=OFF",
    "-DLLAMA_OPENSSL=OFF",
    "-DLLAMA_SUBPROCESS=OFF",
    "-DLLAMA_USE_SYSTEM_GGML=OFF",
    "-DGGML_STATIC=ON",
    "-DGGML_BACKEND_DL=OFF",
    "-DGGML_CPU=ON",
    "-DGGML_LLAMAFILE=OFF",
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
    "-DGIT_EXE=",
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
    "CMAKE_PROJECT_INCLUDE_BEFORE",
    "CMAKE_PROJECT_INCLUDE",
    "CMAKE_PROJECT_TOP_LEVEL_INCLUDES",
    "CMAKE_USER_MAKE_RULES_OVERRIDE",
    "CMAKE_USER_MAKE_RULES_OVERRIDE_C",
    "CMAKE_USER_MAKE_RULES_OVERRIDE_CXX",
    "MAKEFLAGS",
    "MFLAGS",
    "GNUMAKEFLAGS",
];

fn run(command: &mut Command, description: &str) {
    let status = command
        .status()
        .unwrap_or_else(|error| panic!("{description} could not start: {error}"));
    assert!(status.success(), "{description} failed with {status}");
}

fn register_vendored_source_tree(directory: &Path) {
    println!("cargo:rerun-if-changed={}", directory.display());
    let entries = fs::read_dir(directory)
        .unwrap_or_else(|error| panic!("could not read verified llama.cpp source: {error}"));
    let mut paths = entries
        .map(|entry| {
            entry.unwrap_or_else(|error| {
                panic!("could not inspect verified llama.cpp source: {error}")
            })
        })
        .collect::<Vec<_>>();
    paths.sort_by_key(|entry| entry.file_name());

    for entry in paths {
        let path = entry.path();
        let file_type = entry.file_type().unwrap_or_else(|error| {
            panic!("could not classify verified llama.cpp source: {error}")
        });
        assert!(
            !file_type.is_symlink(),
            "verified llama.cpp source must not contain symlinks: {}",
            path.display()
        );
        if file_type.is_dir() {
            register_vendored_source_tree(&path);
        } else if file_type.is_file() {
            println!("cargo:rerun-if-changed={}", path.display());
        } else {
            panic!(
                "verified llama.cpp source must contain only directories and regular files: {}",
                path.display()
            );
        }
    }
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
    let source_metadata = fs::symlink_metadata(&source_dir).unwrap_or_else(|error| {
        panic!("could not inspect verified llama.cpp source root: {error}")
    });
    assert!(
        !source_metadata.file_type().is_symlink() && source_metadata.is_dir(),
        "verified llama.cpp source root must be a real directory"
    );
    register_vendored_source_tree(&source_dir);

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
    for variable in CLOSED_CMAKE_ENVIRONMENT {
        build.env_remove(variable);
    }
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
