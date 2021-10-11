{
    nixgl ? import (fetchTarball "https://github.com/guibou/nixGL/archive/804f1989b3f0bb3347c02ce136060e29f9fc3340.tar.gz") {},
    pkgs ? import (fetchTarball "https://github.com/NixOS/nixpkgs/archive/dd98b100651cfbb8804f32d852f75ef7c97a6b74.tar.gz") {}
}:

pkgs.mkShell {
  buildInputs = [

	pkgs.rustc
	pkgs.cargo
	pkgs.glslang
	pkgs.vulkan-headers
	pkgs.vulkan-loader
	pkgs.vulkan-validation-layers
	pkgs.vulkan-tools

	pkgs.xorg.libX11
	pkgs.xorg.libXau
	pkgs.xorg.libXdmcp

	pkgs.gdb
	pkgs.rust-analyzer
	pkgs.rustfmt
	nixgl.nixVulkanNvidia
  ];

  VK_LAYER_PATH = "${pkgs.vulkan-validation-layers}/share/vulkan/explicit_layer.d";
  LD_LIBRARY_PATH = "${pkgs.vulkan-loader}/lib";
}
