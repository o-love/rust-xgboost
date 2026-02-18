# CUDA Feature Flag Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Allow users to compile xgboost with CUDA/GPU support via `features = ["cuda"]`.

**Architecture:** Add a `cuda` feature to the `xgboost` crate that propagates to `xgboost-sys`, where `build.rs` conditionally sets `USE_CUDA=ON` and links CUDA runtime libraries.

**Tech Stack:** Rust feature flags, CMake, CUDA toolkit

---

### Task 1: Add `cuda` feature to xgboost-sys

**Files:**
- Modify: `xgboost-sys/Cargo.toml:11` (after `[lib]` section, before `[dependencies]`)

**Step 1: Add the feature section**

Add a `[features]` section to `xgboost-sys/Cargo.toml` after line 14 (`doctest = false`):

```toml
[features]
cuda = []
```

**Step 2: Verify it parses**

Run: `cargo metadata --manifest-path xgboost-sys/Cargo.toml --no-deps --format-version 1 | grep -o '"cuda"'`
Expected: `"cuda"`

**Step 3: Commit**

```bash
git add xgboost-sys/Cargo.toml
git commit -m "feat: add cuda feature flag to xgboost-sys"
```

---

### Task 2: Add `cuda` feature to root xgboost crate

**Files:**
- Modify: `Cargo.toml:19` (after `[dependencies]` section)

**Step 1: Add the feature section**

Add a `[features]` section to `Cargo.toml` after line 19 (`indexmap = "2"`):

```toml
[features]
cuda = ["xgboost-sys/cuda"]
```

**Step 2: Verify it parses and propagates**

Run: `cargo metadata --no-deps --format-version 1 | grep -o '"cuda"'`
Expected: `"cuda"` appears (feature is recognized)

**Step 3: Commit**

```bash
git add Cargo.toml
git commit -m "feat: add cuda feature flag to xgboost crate, forwarding to xgboost-sys"
```

---

### Task 3: Update build.rs to conditionally enable CUDA

**Files:**
- Modify: `xgboost-sys/build.rs:12` (the `USE_CUDA` line)

**Step 1: Replace the hardcoded USE_CUDA line**

In `xgboost-sys/build.rs`, replace line 12:

```rust
        .define("USE_CUDA", "OFF")
```

with:

```rust
        .define("USE_CUDA", if cfg!(feature = "cuda") { "ON" } else { "OFF" })
```

**Step 2: Verify default build still works (no CUDA)**

Run: `cargo build -p xgboost-sys 2>&1 | tail -5`
Expected: Builds successfully with no CUDA-related errors (same as before).

**Step 3: Commit**

```bash
git add xgboost-sys/build.rs
git commit -m "feat: conditionally enable USE_CUDA based on cuda feature flag"
```

---

### Task 4: Add CUDA library linking when feature is enabled

**Files:**
- Modify: `xgboost-sys/build.rs:46` (after the pthread link line)

**Step 1: Add conditional CUDA linking block**

After line 46 (`println!("cargo:rustc-link-lib=pthread");`), add:

```rust
    // Link CUDA runtime libraries when cuda feature is enabled
    if cfg!(feature = "cuda") {
        // Standard CUDA toolkit location
        println!("cargo:rustc-link-search=native=/usr/local/cuda/lib64");

        // Also check CUDA_PATH env var if set
        if let Ok(cuda_path) = env::var("CUDA_PATH") {
            println!("cargo:rustc-link-search=native={}/lib64", cuda_path);
        }

        println!("cargo:rustc-link-lib=cudart_static");
        println!("cargo:rustc-link-lib=cuda");
    }
```

**Step 2: Verify default build still works (no CUDA)**

Run: `cargo build -p xgboost-sys 2>&1 | tail -5`
Expected: Builds successfully, no CUDA link directives emitted.

**Step 3: Commit**

```bash
git add xgboost-sys/build.rs
git commit -m "feat: link CUDA runtime libs when cuda feature is enabled"
```

---

### Task 5: Verify default build is unchanged

**Step 1: Clean build without cuda feature**

Run: `cargo clean && cargo build 2>&1 | tail -5`
Expected: Builds successfully, identical behavior to before these changes.

**Step 2: Commit design doc and plan**

```bash
git add docs/
git commit -m "docs: add CUDA feature flag design and implementation plan"
```
