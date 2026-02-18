use std::env;
use std::path::PathBuf;

fn main() {
    let target = env::var("TARGET").unwrap();

    // Build XGBoost from source using cmake
    let xgboost_src = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()).join("xgboost");

    let mut cmake_cfg = cmake::Config::new(&xgboost_src);
    cmake_cfg
        .define("BUILD_STATIC_LIB", "ON")
        .define("USE_CUDA", if cfg!(feature = "cuda") { "ON" } else { "OFF" })
        .define("USE_OPENMP", "ON")
        .define("BUILD_TESTING", "OFF")
        .define("GOOGLE_TEST", "OFF");

    // Help CMake find the CUDA toolkit when CUDA_PATH is set
    if cfg!(feature = "cuda") {
        if let Ok(cuda_path) = env::var("CUDA_PATH") {
            cmake_cfg.define("CUDAToolkit_ROOT", &cuda_path);
            cmake_cfg.define("CMAKE_CUDA_COMPILER_TOOLKIT_ROOT", &cuda_path);
            // Set as env var too — CMake needs this during compiler identification,
            // before project-level variables are processed
            cmake_cfg.env("CUDA_PATH", &cuda_path);
        }
    }

    let dst = cmake_cfg.build();

    // Link the static libraries produced by cmake
    let lib_dir = dst.join("lib");
    println!("cargo:rustc-link-search=native={}", lib_dir.display());

    // On some systems cmake puts libs in lib64
    let lib64_dir = dst.join("lib64");
    if lib64_dir.exists() {
        println!("cargo:rustc-link-search=native={}", lib64_dir.display());
    }

    println!("cargo:rustc-link-lib=static=xgboost");
    println!("cargo:rustc-link-lib=static=dmlc");

    // Link C++ standard library
    if target.contains("apple") {
        println!("cargo:rustc-link-lib=c++");
    } else {
        println!("cargo:rustc-link-lib=stdc++");
    }

    // Link OpenMP runtime
    if target.contains("apple") {
        println!("cargo:rustc-link-lib=omp");
    } else {
        println!("cargo:rustc-link-lib=gomp");
    }

    // Link pthreads
    println!("cargo:rustc-link-lib=pthread");

    // Link CUDA runtime libraries when cuda feature is enabled
    if cfg!(feature = "cuda") {
        println!("cargo:rerun-if-env-changed=CUDA_PATH");

        // Standard CUDA toolkit location
        println!("cargo:rustc-link-search=native=/usr/local/cuda/lib64");
        println!("cargo:rustc-link-search=native=/usr/local/cuda/lib");

        // Also check CUDA_PATH env var if set
        if let Ok(cuda_path) = env::var("CUDA_PATH") {
            println!("cargo:rustc-link-search=native={}/lib64", cuda_path);
            println!("cargo:rustc-link-search=native={}/lib", cuda_path);
        }

        println!("cargo:rustc-link-lib=cudart_static");
        println!("cargo:rustc-link-lib=cuda");

        // cudart_static depends on dl and rt on Linux
        if !target.contains("apple") {
            println!("cargo:rustc-link-lib=dl");
            println!("cargo:rustc-link-lib=rt");
        }
    }

    // Generate bindings from the submodule headers
    let include_dir = xgboost_src.join("include");

    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .clang_args(&["-x", "c++", "-std=c++17"])
        .clang_arg(format!("-I{}", include_dir.display()))
        .allowlist_function("XG.*")
        .allowlist_type("bst_ulong")
        .allowlist_type("DMatrixHandle")
        .allowlist_type("BoosterHandle")
        .allowlist_type("XGBoostBatchCSR")
        .generate()
        .expect("Unable to generate bindings.");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings.");
}
