{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
    devshell.url = "github:numtide/devshell";
    nci.url = "github:yusdacra/nix-cargo-integration";
  };

  outputs = inputs @ {flake-parts, ...}:
    flake-parts.lib.mkFlake {inherit inputs;} {
      imports = [
        inputs.devshell.flakeModule
        inputs.nci.flakeModule
      ];

      flake = {};

      systems = [
        "x86_64-linux"
        "x86_64-darwin"
        "aarch64-linux"
        "aarch64-darwin"
      ];

      perSystem = {
        system,
        pkgs,
        config,
        ...
      }: {
        formatter = pkgs.alejandra;

        devshells.default = {
          packages =
            (with pkgs; [
              nil
              rust-analyzer
              cargo-watch
              cargo-modules
              wasm-pack
            ])
            ++ [config.nci.toolchains.shell];
        };
      };
    };
}
