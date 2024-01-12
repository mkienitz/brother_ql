{
  description = "A devShell example";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = {
    self,
    nixpkgs,
    rust-overlay,
    flake-utils,
    ...
  }:
    flake-utils.lib.eachDefaultSystem (
      system: let
        overlays = [(import rust-overlay)];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        rust-bin = pkgs.rust-bin.stable.latest.default.override {
          extensions = ["rust-src"];
        };
      in {
        formatter = pkgs.alejandra;
        devShells.default = pkgs.mkShell {
          buildInputs =
            [rust-bin]
            ++ (with pkgs; [
              nil
              rust-analyzer
              clippy
            ]);
          RUST_SRC_PATH = "${rust-bin}/lib/rustlib/src/rust/library";
        };
      }
    );
}
