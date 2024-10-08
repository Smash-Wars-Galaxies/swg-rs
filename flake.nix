{
  description = "Description for the project";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

    flake-parts.url = "github:hercules-ci/flake-parts";
    treefmt-nix.url = "github:numtide/treefmt-nix";

    crane.url = "github:ipetkov/crane";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.rust-analyzer-src.follows = "";
    };
    advisory-db = {
      url = "github:rustsec/advisory-db";
      flake = false;
    };
  };

  outputs = inputs @ {
    flake-parts,
    treefmt-nix,
    crane,
    fenix,
    advisory-db,
    ...
  }:
    flake-parts.lib.mkFlake {inherit inputs;} {
      debug = true;
      imports = [
        inputs.treefmt-nix.flakeModule
        ./.nix/modules/flake-parts/formatter.nix
      ];
      systems = ["x86_64-linux" "aarch64-linux" "aarch64-darwin" "x86_64-darwin"];
      perSystem = {
        config,
        self',
        inputs',
        pkgs,
        system,
        ...
      }: let
        inherit (pkgs) lib;

        testDataFilter = name: type: ((builtins.match ".*resources/.*" name) != null);

        dataOrCargo = path: type:
          (testDataFilter path type) || (craneLib.filterCargoSources path type);

        craneLib = inputs.crane.mkLib pkgs;
        src = lib.cleanSourceWith {
          src = ./.;
          filter = dataOrCargo;
          name = "source";
        };

        commonArgs = {
          inherit src;
          strictDeps = true;

          buildInputs = lib.optionals pkgs.stdenv.isDarwin [
            pkgs.libiconv
          ];
        };

        craneLibLLvmTools =
          craneLib.overrideToolchain
          (inputs'.fenix.packages.complete.withComponents [
            "cargo"
            "llvm-tools"
            "rustc"
          ]);

        cargoArtifacts = craneLib.buildDepsOnly commonArgs;

        individualCrateArgs =
          commonArgs
          // {
            inherit cargoArtifacts;
            inherit (craneLib.crateNameFromCargoToml {inherit src;}) version;
            # NB: we disable tests since we'll run them all via cargo-nextest
            doCheck = false;
          };

        fileSetForCrate = crate:
          lib.fileset.toSource {
            root = ./.;
            fileset = lib.fileset.unions [
              ./Cargo.toml
              ./Cargo.lock
              ./crates/swg_tre
              ./crates/swg_workspace
              crate
            ];
          };

        swg = craneLib.buildPackage (individualCrateArgs
          // {
            pname = "swg";
            cargoExtraArgs = "-p swg";
            src = fileSetForCrate ./crates/swg;
          });
      in {
        checks = {
          inherit swg;

          swg-clippy = craneLib.cargoClippy (commonArgs
            // {
              inherit cargoArtifacts;
              cargoClippyExtraArgs = "--all-targets -- --deny warnings";
            });

          swg-doc = craneLib.cargoDoc (commonArgs
            // {
              inherit cargoArtifacts;
            });

          # Audit dependencies
          swg-audit = craneLib.cargoAudit {
            inherit src advisory-db;
          };

          # Audit licenses
          swg-deny = craneLib.cargoDeny {
            inherit src;
          };

          # Run tests with cargo-nextest
          # Consider setting `doCheck = false` on other crate derivations
          # if you do not want the tests to run twice
          swg-nextest = craneLib.cargoNextest (commonArgs
            // {
              inherit cargoArtifacts;
              partitions = 1;
              partitionType = "count";
            });

          # Ensure that cargo-hakari is up to date
          swg-hakari = craneLib.mkCargoDerivation {
            inherit src;
            pname = "swg-hakari";
            cargoArtifacts = null;
            doInstallCargoArtifacts = false;

            buildPhaseCargoCommand = ''
              cargo hakari generate --diff  # workspace-hack Cargo.toml is up-to-date
              cargo hakari manage-deps --dry-run  # all workspace crates depend on workspace-hack
              cargo hakari verify
            '';

            nativeBuildInputs = [
              pkgs.cargo-hakari
            ];
          };
        };

        packages =
          {
            inherit swg;
          }
          // lib.optionalAttrs (!pkgs.stdenv.isDarwin) {
            swg-llvm-coverage = craneLibLLvmTools.cargoLlvmCov (commonArgs
              // {
                inherit cargoArtifacts;
              });
          };

        devShells.default = craneLib.devShell {
          # Inherit inputs from checks.
          checks = self'.checks;

          packages = [
            pkgs.cargo-bloat
            pkgs.cargo-semver-checks
            pkgs.cargo-msrv
          ];
        };
      };
      flake = {
      };
    };
}
