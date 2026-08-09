use std::{
    env, fs,
    path::{Path, PathBuf},
    process::Command,
};

use sha2::{Digest, Sha256};

const EXPECTED_VENDORED_TREE_SHA256: &str =
    "9892c22a1a05adf0775615f1b845886f8f1be96ad7b6f71093103eaec546a511";

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
    "-DCMAKE_C_COMPILER=/usr/bin/cc",
    "-DCMAKE_CXX_COMPILER=/usr/bin/c++",
    "-DCMAKE_C_COMPILER_AR=/usr/bin/ar",
    "-DCMAKE_CXX_COMPILER_AR=/usr/bin/ar",
    "-DCMAKE_C_COMPILER_RANLIB=/usr/bin/ranlib",
    "-DCMAKE_CXX_COMPILER_RANLIB=/usr/bin/ranlib",
    "-DCMAKE_MAKE_PROGRAM=/usr/bin/make",
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
    "-DGIT_EXECUTABLE=",
    "-DCMAKE_DISABLE_FIND_PACKAGE_Git=ON",
    "-DCMAKE_FIND_USE_PACKAGE_REGISTRY=OFF",
    "-DCMAKE_FIND_USE_SYSTEM_PACKAGE_REGISTRY=OFF",
    "-DCMAKE_FIND_USE_CMAKE_PATH=OFF",
    "-DCMAKE_FIND_USE_CMAKE_ENVIRONMENT_PATH=OFF",
    "-DCMAKE_FIND_USE_SYSTEM_ENVIRONMENT_PATH=OFF",
    "-DCMAKE_FIND_USE_CMAKE_SYSTEM_PATH=OFF",
    "-DCMAKE_FIND_USE_INSTALL_PREFIX=OFF",
    "-DCMAKE_FIND_PACKAGE_NO_PACKAGE_REGISTRY=ON",
    "-DCMAKE_FIND_PACKAGE_NO_SYSTEM_PACKAGE_REGISTRY=ON",
    "-DCMAKE_EXPORT_NO_PACKAGE_REGISTRY=ON",
    "-DCMAKE_DISABLE_SOURCE_CHANGES=ON",
    "-DCMAKE_DISABLE_IN_SOURCE_BUILD=ON",
];

const CLOSED_CMAKE_ENVIRONMENT: &[&str] = &[
    "CC",
    "CXX",
    "AR",
    "RANLIB",
    "NM",
    "STRIP",
    "OBJCOPY",
    "READELF",
    "CPPFLAGS",
    "CFLAGS",
    "CXXFLAGS",
    "LDFLAGS",
    "CPATH",
    "C_INCLUDE_PATH",
    "CPLUS_INCLUDE_PATH",
    "OBJC_INCLUDE_PATH",
    "LIBRARY_PATH",
    "COMPILER_PATH",
    "GCC_EXEC_PREFIX",
    "GCC_SPECS",
    "CCC_OVERRIDE_OPTIONS",
    "CLANG_CONFIG_FILE",
    "LD_PRELOAD",
    "LD_AUDIT",
    "LD_LIBRARY_PATH",
    "CMAKE_GENERATOR",
    "CMAKE_GENERATOR_INSTANCE",
    "CMAKE_GENERATOR_PLATFORM",
    "CMAKE_GENERATOR_TOOLSET",
    "CMAKE_BUILD_PARALLEL_LEVEL",
    "CMAKE_CONFIG_TYPE",
    "CMAKE_BUILD_TYPE",
    "CMAKE_CONFIGURATION_TYPES",
    "CMAKE_C_COMPILER",
    "CMAKE_CXX_COMPILER",
    "CMAKE_C_COMPILER_ARG1",
    "CMAKE_CXX_COMPILER_ARG1",
    "CMAKE_C_COMPILER_TARGET",
    "CMAKE_CXX_COMPILER_TARGET",
    "CMAKE_C_COMPILER_EXTERNAL_TOOLCHAIN",
    "CMAKE_CXX_COMPILER_EXTERNAL_TOOLCHAIN",
    "CMAKE_C_COMPILER_AR",
    "CMAKE_CXX_COMPILER_AR",
    "CMAKE_C_COMPILER_RANLIB",
    "CMAKE_CXX_COMPILER_RANLIB",
    "CMAKE_C_COMPILER_LAUNCHER",
    "CMAKE_CXX_COMPILER_LAUNCHER",
    "CMAKE_MAKE_PROGRAM",
    "CMAKE_AR",
    "CMAKE_RANLIB",
    "CMAKE_LINKER",
    "CMAKE_NM",
    "CMAKE_OBJCOPY",
    "CMAKE_STRIP",
    "CMAKE_READELF",
    "CMAKE_C_FLAGS_INIT",
    "CMAKE_CXX_FLAGS_INIT",
    "CMAKE_EXE_LINKER_FLAGS_INIT",
    "CMAKE_SHARED_LINKER_FLAGS_INIT",
    "CMAKE_MODULE_LINKER_FLAGS_INIT",
    "CMAKE_TOOLCHAIN_FILE",
    "CMAKE_SYSROOT",
    "CMAKE_SYSROOT_COMPILE",
    "CMAKE_SYSROOT_LINK",
    "CMAKE_STAGING_PREFIX",
    "CMAKE_FIND_ROOT_PATH",
    "CMAKE_FIND_ROOT_PATH_MODE_PROGRAM",
    "CMAKE_FIND_ROOT_PATH_MODE_LIBRARY",
    "CMAKE_FIND_ROOT_PATH_MODE_INCLUDE",
    "CMAKE_FIND_ROOT_PATH_MODE_PACKAGE",
    "CMAKE_SYSTEM_NAME",
    "CMAKE_SYSTEM_PROCESSOR",
    "CMAKE_CROSSCOMPILING_EMULATOR",
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

const CLOSED_BUILD_PATH: &str = "/usr/bin:/bin";
const SYSTEM_CMAKE: &str = "/usr/bin/cmake";

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

fn require_vendored_source_root(directory: &Path) {
    let source_metadata = fs::symlink_metadata(directory).unwrap_or_else(|error| {
        panic!("could not inspect verified llama.cpp source root: {error}")
    });
    assert!(
        !source_metadata.file_type().is_symlink() && source_metadata.is_dir(),
        "verified llama.cpp source root must be a real directory"
    );
}

fn vendored_tree_digest(directory: &Path, root: &Path, digest: &mut Sha256) {
    let mut entries = fs::read_dir(directory)
        .unwrap_or_else(|error| panic!("could not read verified llama.cpp source: {error}"))
        .map(|entry| {
            entry.unwrap_or_else(|error| {
                panic!("could not inspect verified llama.cpp source: {error}")
            })
        })
        .collect::<Vec<_>>();
    entries.sort_by_key(|entry| entry.file_name());

    for entry in entries {
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
            vendored_tree_digest(&path, root, digest);
        } else if file_type.is_file() {
            if path == root.join("PROVENANCE.json") {
                continue;
            }
            let relative = path
                .strip_prefix(root)
                .expect("verified llama.cpp source path must remain under its root")
                .to_str()
                .expect("verified llama.cpp source paths must be UTF-8")
                .replace('\\', "/");
            digest.update(relative.as_bytes());
            digest.update([0]);
            digest.update(Sha256::digest(fs::read(&path).unwrap_or_else(|error| {
                panic!("could not read verified llama.cpp source file: {error}")
            })));
            digest.update([0]);
        } else {
            panic!(
                "verified llama.cpp source must contain only directories and regular files: {}",
                path.display()
            );
        }
    }
}

fn verify_vendored_tree_digest(directory: &Path) {
    // Every verification point re-checks the source root too, so a source-root
    // replacement after initial setup cannot redirect a later tree walk.
    require_vendored_source_root(directory);
    let mut digest = Sha256::new();
    vendored_tree_digest(directory, directory, &mut digest);
    let observed = format!("{:x}", digest.finalize());
    assert_eq!(
        observed, EXPECTED_VENDORED_TREE_SHA256,
        "verified llama.cpp source tree digest does not match pinned provenance"
    );
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
    require_vendored_source_root(&source_dir);
    verify_vendored_tree_digest(&source_dir);
    register_vendored_source_tree(&source_dir);

    let mut configure = Command::new(SYSTEM_CMAKE);
    configure
        .arg("-S")
        .arg(&source_dir)
        .arg("-B")
        .arg(&build_dir)
        // Discard any cached CMake configuration in Cargo's generated output
        // directory. The closed options below must be the complete build
        // configuration, rather than inheriting a prior cache entry.
        .arg("--fresh")
        .arg("-DCMAKE_BUILD_TYPE=Release");
    configure.args(LLAMA_CPP_CMAKE_OPTIONS);
    configure.env_clear();
    for variable in CLOSED_CMAKE_ENVIRONMENT {
        configure.env_remove(variable);
    }
    configure.env("PATH", CLOSED_BUILD_PATH);

    // Change-registration walks the full source tree. Re-check after that walk
    // and immediately before CMake evaluates the vendored configuration.
    verify_vendored_tree_digest(&source_dir);
    run(
        &mut configure,
        "closed llama.cpp static-library configuration",
    );

    // Configuration reads the vendored CMake sources. Re-check their pinned
    // content before the subsequent compiler invocation so a mid-build source
    // change fails closed instead of compiling a different tree.
    verify_vendored_tree_digest(&source_dir);

    let mut build = Command::new(SYSTEM_CMAKE);
    build
        .arg("--build")
        .arg(&build_dir)
        .arg("--config")
        .arg("Release")
        .arg("--target")
        .arg("llama");
    build.env_clear();
    for variable in CLOSED_CMAKE_ENVIRONMENT {
        build.env_remove(variable);
    }
    build.env("PATH", CLOSED_BUILD_PATH);
    run(&mut build, "closed llama.cpp static-library build");

    // Compilation reads the vendored sources too. Refuse to hand its output to
    // Cargo if those sources changed at any point during the closed build.
    verify_vendored_tree_digest(&source_dir);

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
