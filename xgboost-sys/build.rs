use std::env;
use std::path::PathBuf;

fn main() {
    let target = env::var("TARGET").unwrap();

    // Build XGBoost from source using cmake
    let xgboost_src = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()).join("xgboost");

    let dst = cmake::Config::new(&xgboost_src)
        .define("BUILD_STATIC_LIB", "ON")
        .define("USE_CUDA", "OFF")
        .define("USE_OPENMP", "ON")
        .define("BUILD_TESTING", "OFF")
        .define("GOOGLE_TEST", "OFF")
        .build();

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
