{
  inputs = {
    utils.url = "github:numtide/flake-utils";
    naersk.url = "github:nix-community/naersk";
    naersk.inputs.nixpkgs.follows = "nixpkgs";
    fenix.url = "github:nix-community/fenix";
    fenix.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs = { self, nixpkgs, utils, naersk, fenix }:
    utils.lib.eachDefaultSystem (system: let
      pkgs = nixpkgs.legacyPackages."${system}";
      rust = fenix.packages.${system}.complete.withComponents [
        "cargo"
        "rustc"
        "rust-src"  # just for rust-analyzer
        "clippy"
      ];

      # Override the version used in naersk
      naersk-lib = naersk.lib."${system}".override {
        cargo = rust;
        rustc = rust;
      };
    in rec {
      # `nix build`
      packages.runalyzer = naersk-lib.buildPackage {
        pname = "runalyzer";
        src = ./runalyzer;
        doCheck = true;
        cargoTestCommands = x:
          x ++ [
            # clippy
            ''cargo clippy --all --all-features --tests -- \
              -D clippy::pedantic \
              -D warnings \
              -A clippy::module-name-repetitions \
              -A clippy::too-many-lines \
              -A clippy::nonminimal_bool''
          ];
      };
      defaultPackage = packages.runalyzer;

      checks = packages;

      # `nix run`
      apps.runalyzer = utils.lib.mkApp {
        drv = packages.runalyzer;
      };
      defaultApp = apps.runalyzer;

      # `nix develop`
      devShell = pkgs.mkShell {
        nativeBuildInputs = [
          fenix.packages.${system}.rust-analyzer
        ] ++
        (with defaultPackage; nativeBuildInputs ++ buildInputs);
      };
    }) // {
      overlay = final: prev: {
        runalyzer = self.packages.${prev.system};
      };
    };
}
