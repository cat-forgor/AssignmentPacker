{
  description = "CLI tool that packs C assignment submissions for Canvas upload";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
      in
      {
        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = "ap";
          version = "0.1.0";

          src = self;

          cargoLock = {
            lockFile = ./Cargo.lock;
          };

          meta = with pkgs.lib; {
            description = "Packs C assignment submissions for Canvas upload";
            homepage = "https://github.com/cat-forgor/AssignmentPacker";
            license = licenses.mit;
            mainProgram = "ap";
          };
        };

        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            cargo
            rustc
            rust-analyzer
            clippy
          ];
        };
      });
}
