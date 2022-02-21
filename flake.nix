{
  description = "Rustly";

  inputs = {
    nixpkgs.url      = github:nixos/nixpkgs/nixos-unstable;
    rust-overlay.url = github:oxalica/rust-overlay?rev=d480bb17451c57cf4ef67a14f6772f752ced382c;
    flake-utils.url  = github:numtide/flake-utils;
    nixgl.url        = github:guibou/nixGL;
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils, nixgl }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        rust = pkgs.rust-bin.selectLatestNightlyWith (toolchain: toolchain.default.override {
          extensions = [
            "rust-src"
            "rustfmt-preview"
            "rust-analyzer-preview"
          ];
        });
      in
      with pkgs;
      {
        devShell = pkgs.mkShell rec {
          buildInputs = [
            rust
            glslang
            vulkan-headers
            vulkan-loader
            vulkan-validation-layers
            vulkan-tools

            xorg.libX11
            xorg.libXau
            xorg.libXdmcp
            xorg.libXcursor
            xorg.libXrandr
            xorg.libXext

            which
            gdb
          ];

          VK_LAYER_PATH = "${pkgs.vulkan-validation-layers}/share/vulkan/explicit_layer.d";
          LD_LIBRARY_PATH = "${lib.makeLibraryPath buildInputs}";
          RUST_BACKTRACE = 1;
          CFG_RELEASE_CHANNEL = "nightly";
          CARGO_HOME = ".cargo";
        };
      }
    );
}
