use std::env;
use std::path::PathBuf;

fn main() {
    let target = env::var("TARGET").unwrap();

    // Find XGBoost library and include paths.
    // Check environment variables first, then fall back to pkg-config.
    let (lib_dir, include_dir) = match (env::var("XGBOOST_LIB_DIR"), env::var("XGBOOST_INCLUDE_DIR")) {
        (Ok(lib), Ok(inc)) => (Some(lib), Some(inc)),
        _ => {
            // Try pkg-config
            if let Ok(lib) = pkg_config_probe() {
                (None, Some(lib))
            } else {
                // Fall back to common system paths
                (None, find_system_include_dir())
            }
        }
    };

    // Set library search path
    let lib_dir = lib_dir.or_else(find_system_lib_dir);
    if let Some(ref lib_dir) = lib_dir {
        println!("cargo:rustc-link-search=native={}", lib_dir);
    }

    // Link against system XGBoost (dynamic)
    println!("cargo:rustc-link-lib=dylib=xgboost");

    // Link C++ standard library
    if target.contains("apple") {
        println!("cargo:rustc-link-lib=c++");
    } else {
        println!("cargo:rustc-link-lib=stdc++");
    }

    // Generate bindings
    let mut builder = bindgen::Builder::default()
        .header("wrapper.h")
        .clang_args(&["-x", "c++", "-std=c++14"])
        .allowlist_function("XG.*")
        .allowlist_type("bst_ulong")
        .allowlist_type("DMatrixHandle")
        .allowlist_type("BoosterHandle")
        .allowlist_type("XGBoostBatchCSR");

    if let Some(ref inc) = include_dir {
        builder = builder.clang_arg(format!("-I{}", inc));
    }

    let bindings = builder
        .generate()
        .expect("Unable to generate bindings.");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings.");
}

/// Try to find XGBoost via pkg-config, return include dir on success.
fn pkg_config_probe() -> Result<String, ()> {
    let output = std::process::Command::new("pkg-config")
        .args(["--cflags", "xgboost"])
        .output();

    match output {
        Ok(out) if out.status.success() => {
            let flags = String::from_utf8_lossy(&out.stdout);
            for flag in flags.split_whitespace() {
                if let Some(dir) = flag.strip_prefix("-I") {
                    return Ok(dir.to_string());
                }
            }
            Err(())
        }
        _ => Err(()),
    }
}

/// Search common system paths for XGBoost headers.
fn find_system_include_dir() -> Option<String> {
    let candidates = [
        "/usr/include",
        "/usr/local/include",
    ];

    // Also check nix store paths via the XGBOOST_INCLUDE_DIR or by searching
    for candidate in &candidates {
        let header = format!("{}/xgboost/c_api.h", candidate);
        if std::path::Path::new(&header).exists() {
            return Some(candidate.to_string());
        }
    }

    // On NixOS, find the header through the C_INCLUDE_PATH or NIX_CFLAGS_COMPILE
    if let Ok(cflags) = env::var("NIX_CFLAGS_COMPILE") {
        for part in cflags.split_whitespace() {
            if let Some(dir) = part.strip_prefix("-isystem") {
                let header = format!("{}/xgboost/c_api.h", dir);
                if std::path::Path::new(&header).exists() {
                    return Some(dir.to_string());
                }
            }
            // Sometimes -isystem and path are separate tokens
        }
    }

    // Search nix store for xgboost includes
    if let Ok(output) = std::process::Command::new("sh")
        .args(["-c", "find /nix/store -maxdepth 3 -name 'xgboost' -path '*/include/xgboost' 2>/dev/null | head -1"])
        .output()
    {
        if output.status.success() {
            let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !path.is_empty() {
                // Return the parent 'include' directory
                if let Some(include_dir) = std::path::Path::new(&path).parent() {
                    return Some(include_dir.to_string_lossy().to_string());
                }
            }
        }
    }

    None
}

/// Search common system paths and nix store for XGBoost library.
fn find_system_lib_dir() -> Option<String> {
    let candidates = [
        "/usr/lib",
        "/usr/local/lib",
        "/usr/lib64",
        "/usr/local/lib64",
    ];

    for candidate in &candidates {
        let lib = format!("{}/libxgboost.so", candidate);
        if std::path::Path::new(&lib).exists() {
            return Some(candidate.to_string());
        }
    }

    // Search nix store for xgboost lib
    if let Ok(output) = std::process::Command::new("sh")
        .args(["-c", "find /nix/store -maxdepth 3 -name 'libxgboost.so' -path '*/lib/libxgboost.so' 2>/dev/null | head -1"])
        .output()
    {
        if output.status.success() {
            let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !path.is_empty() {
                if let Some(lib_dir) = std::path::Path::new(&path).parent() {
                    return Some(lib_dir.to_string_lossy().to_string());
                }
            }
        }
    }

    None
}
