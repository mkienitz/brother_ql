{
  inputs = {
    devshell = {
      url = "github:numtide/devshell";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-parts.url = "github:hercules-ci/flake-parts";
    nci = {
      url = "github:yusdacra/nix-cargo-integration";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    pre-commit-hooks = {
      url = "github:cachix/pre-commit-hooks.nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    treefmt-nix = {
      url = "github:numtide/treefmt-nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    inputs@{ flake-parts, ... }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      imports = [
        inputs.devshell.flakeModule
        inputs.nci.flakeModule
        inputs.pre-commit-hooks.flakeModule
        inputs.treefmt-nix.flakeModule
      ];

      flake = { };

      systems = [
        "x86_64-linux"
        "x86_64-darwin"
        "aarch64-linux"
        "aarch64-darwin"
      ];

      perSystem =
        {
          pkgs,
          config,
          lib,
          ...
        }:
        {
          nci = {
            projects."brother_ql" = {
              numtideDevshell = "default";
              path = ./.;
            };
            crates."brother_ql" = { };
            crates."brother_ql_cli" = { };
          };

          devshells.default = {
            packages = [
              pkgs.nil
              pkgs.rust-analyzer
              pkgs.cargo-watch
              pkgs.cargo-modules
              pkgs.cargo-release
              pkgs.bacon
              pkgs.typst
              pkgs.typstyle
              pkgs.nodejs
            ]
            ++ (lib.optionals pkgs.stdenv.isDarwin [
              pkgs.libiconv
            ]);
            env = [
              {
                # On darwin for example enables finding of libiconv
                name = "LIBRARY_PATH";
                eval = "$DEVSHELL_DIR/lib";
              }
              {
                name = "PATH";
                prefix = "$HOME/.cargo/bin";
              }
            ];
            devshell.startup.pre-commit.text = config.pre-commit.installationScript;
          };

          pre-commit.settings.hooks.treefmt.enable = true;

          treefmt = {
            projectRootFile = "flake.nix";
            programs = {
              deadnix.enable = true;
              statix.enable = true;
              nixfmt.enable = true;
              rustfmt =
                let
                  toml = builtins.fromTOML (builtins.readFile ./Cargo.toml);
                in
                {
                  enable = true;
                  inherit (toml.workspace.package) edition;
                };
              taplo.enable = true;
            };
          };

          packages.default = config.perSystem.packages.lib;
          packages.lib = config.nci.outputs.brother_ql.packages.release;
          packages.cli = config.nci.outputs.brother-label.packages.release;
        };
    };
}
