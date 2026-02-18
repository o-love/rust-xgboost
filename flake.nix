{
  description = "rust-xgboost - Rust bindings for XGBoost with CUDA support";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, rust-overlay }:
    let
      system = "x86_64-linux";
      pkgs = import nixpkgs {
        inherit system;
        config.allowUnfree = true;
        overlays = [ rust-overlay.overlays.default ];
      };
      cudaPkgs = pkgs.cudaPackages;

      # CUDA 12.x requires GCC <= 14 as host compiler
      cudaGcc = pkgs.gcc14;
      cudaStdenv = pkgs.gcc14Stdenv;
    in
    {
      devShells.${system}.default = (pkgs.mkShell.override { stdenv = cudaStdenv; }) {
        nativeBuildInputs = with pkgs; [
          # Rust toolchain
          (rust-bin.stable.latest.default.override {
            extensions = [ "rust-src" "rust-analyzer" ];
          })

          # Build tools
          cmake
          pkg-config

          # CUDA toolkit (nvcc, headers, runtime libraries)
          cudaPkgs.cudatoolkit
        ];

        buildInputs = with pkgs; [
          # bindgen needs libclang
          llvmPackages.libclang

          # libstdc++ for C++ linking
          cudaGcc.cc.lib
        ];

        env = let
          gccVersion = cudaGcc.cc.version;
          gccInclude = "${cudaGcc.cc}/include/c++/${gccVersion}";
        in {
          CUDA_PATH = "${cudaPkgs.cudatoolkit}";
          LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";
          CC = "${cudaGcc}/bin/gcc";
          CXX = "${cudaGcc}/bin/g++";
          CUDAHOSTCXX = "${cudaGcc}/bin/g++";
          BINDGEN_EXTRA_CLANG_ARGS =
            "-isystem ${gccInclude} "
            + "-isystem ${gccInclude}/x86_64-unknown-linux-gnu "
            + "-isystem ${pkgs.glibc.dev}/include";
        };

        shellHook = ''
          export LD_LIBRARY_PATH="${pkgs.lib.makeLibraryPath [
            cudaPkgs.cudatoolkit
            cudaGcc.cc.lib
          ]}''${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"

          # NixOS: GPU driver (libcuda.so) lives here at runtime
          if [ -d /run/opengl-driver/lib ]; then
            export LD_LIBRARY_PATH="/run/opengl-driver/lib:$LD_LIBRARY_PATH"
          fi
        '';
      };
    };
}
