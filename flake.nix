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
      }: let
        projectName = crateName;
        crateName = "brother_ql";
        crateOutput = config.nci.outputs.${crateName};
      in {
        formatter = pkgs.alejandra;
        nci = {
          projects.${projectName}.path = ./.;
          crates.${crateName} = {};
        };
        devShells.default = crateOutput.devShell.overrideAttrs (old: {
          nativeBuildInputs =
            (with pkgs; [
              nil
              rust-analyzer
              cargo-watch
              cargo-modules
              cargo-release
            ])
            ++ old.nativeBuildInputs;
        });
        packages.default = crateOutput.packages.release;
      };
    };
}
