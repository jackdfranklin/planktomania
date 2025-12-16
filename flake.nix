{
  description = "Bevy Game";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
		utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, utils }:
		utils.lib.eachDefaultSystem (system:
			let
				pkgs = import nixpkgs { inherit system; };
			in
			{
				devShell = with pkgs; mkShell {
					nativeBuildInputs = [ pkg-config ];
					buildInputs = [ cargo rustc rustfmt ];
				};
			}
		);
}
