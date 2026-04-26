{
  description = "intuitive";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    crane.url = "github:ipetkov/crane";

    treefmt-nix = {
      url = "github:numtide/treefmt-nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      nixpkgs,
      flake-utils,
      rust-overlay,
      crane,
      treefmt-nix,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ rust-overlay.overlays.default ];
        };

        rustToolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
        craneLib = (crane.mkLib pkgs).overrideToolchain (_: rustToolchain);

        commonArgs = {
          src = craneLib.cleanCargoSource ./.;
          strictDeps = true;
        }
        // craneLib.crateNameFromCargoToml { cargoToml = ./intuitive/Cargo.toml; };
        cargoArtifacts = craneLib.buildDepsOnly commonArgs;

        treefmt = treefmt-nix.lib.evalModule pkgs {
          projectRootFile = "flake.nix";
          programs.nixfmt.enable = true;
          programs.rustfmt = {
            enable = true;
            package = rustToolchain;
          };
        };
      in
      {
        devShells.default = craneLib.devShell {
          packages = [ pkgs.cargo-expand ];
        };

        formatter = treefmt.config.build.wrapper;

        checks.cargo-test = craneLib.cargoTest (
          commonArgs
          // {
            inherit cargoArtifacts;
            nativeBuildInputs = [ pkgs.cargo-expand ];
          }
        );

        apps.fixtures = {
          type = "app";
          program = toString (
            pkgs.writeShellScript "fixtures" ''
              set -euo pipefail
              cd "$(git rev-parse --show-toplevel)"
              TRYBUILD=overwrite cargo test -p intuitive-macros --test ui
              MACROTEST=overwrite cargo test -p intuitive-macros --test expand
            ''
          );
        };
      }
    );
}
