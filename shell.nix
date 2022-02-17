let
	nixgl   = import (fetchTarball "https://github.com/guibou/nixGL/archive/804f1989b3f0bb3347c02ce136060e29f9fc3340.tar.gz") {};
	rustOverlay = import (fetchTarball "https://github.com/oxalica/rust-overlay/archive/d480bb17451c57cf4ef67a14f6772f752ced382c.tar.gz");
	pkgs    = import (fetchTarball "https://github.com/NixOS/nixpkgs/archive/93ca5ab64f78ce778c0bcecf9458263f0f6289b6.tar.gz") {
		overlays = [ rustOverlay ];
	};
	rust = pkgs.rust-bin.selectLatestNightlyWith (toolchain: toolchain.default.override {
		extensions = [
			"rust-src"
			"rustfmt-preview"
			"rust-analyzer-preview"
		];
	});
in
pkgs.mkShell {
	buildInputs = [
		rust
		pkgs.glslang
		pkgs.vulkan-headers
		pkgs.vulkan-loader
		pkgs.vulkan-validation-layers
		pkgs.vulkan-tools

		pkgs.xorg.libX11
		pkgs.xorg.libXau
		pkgs.xorg.libXdmcp

		pkgs.which
		pkgs.gdb
		#nixgl.nixVulkanNvidia
	];

	VK_LAYER_PATH = "${pkgs.vulkan-validation-layers}/share/vulkan/explicit_layer.d";
	LD_LIBRARY_PATH = "${pkgs.vulkan-loader}/lib";
	RUST_BACKTRACE = 1;
	CFG_RELEASE_CHANNEL = "nightly";
}
