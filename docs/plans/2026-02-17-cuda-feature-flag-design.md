# CUDA Feature Flag Design

## Goal

Allow users to build xgboost with CUDA/GPU support by enabling a Cargo feature flag.

Usage: `xgboost = { version = "...", features = ["cuda"] }`

## Design

### Feature flag propagation

- `xgboost/Cargo.toml` adds `cuda = ["xgboost-sys/cuda"]`
- `xgboost-sys/Cargo.toml` adds `cuda = []`

### Build changes (xgboost-sys/build.rs)

When `cuda` feature is active:

1. Pass `USE_CUDA=ON` to CMake (instead of `OFF`)
2. Emit link directives for CUDA runtime libs: `cudart_static`, `cuda`
3. Add CUDA lib search path: `/usr/local/cuda/lib64` (standard Linux location)
4. Check `CUDA_PATH` env var as additional search path

### What doesn't change

- No Rust source code changes -- CUDA is a build-time concern handled by xgboost's C++ internals
- Default build (no feature) remains identical to current behavior
- OpenMP, static linking, and all other build settings unchanged
