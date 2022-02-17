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
      in
      with pkgs;
      {
        devShell = mkShell {
          buildInputs = [
            rust-bin.nightly.latest.default
            glslang
            vulkan-headers
            vulkan-loader
            vulkan-validation-layers
            vulkan-tools

            xorg.libX11
            xorg.libXau
            xorg.libXdmcp

            which
            gdb
          ];

          VK_LAYER_PATH = "${pkgs.vulkan-validation-layers}/share/vulkan/explicit_layer.d";
          LD_LIBRARY_PATH = "${pkgs.vulkan-loader}/lib";
          RUST_BACKTRACE = 1;
          CFG_RELEASE_CHANNEL = "nightly";
          CARGO_HOME = ".cargo";
        };
      }
    );
}
