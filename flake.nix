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
      rust = fenix.packages.${system}.stable.withComponents [
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

      telegramsDump = pkgs.fetchurl {
        url = "https://files.dvb.solutions/mapping-run-20220507/telegrams-20220507-2205.csv";
        sha256 = "1hj4hx0p5qrh2p8zm6sbxdbyj2y77xl1wmb7pbpf9y206jhb2fqz";
      };
    in rec {
      # `nix build`
      packages.runalyzer = naersk-lib.buildPackage {
        pname = "runalyzer";
        src = ./runalyzer;
        overrideMain = attrs: {
          patchPhase = ''
            substituteInPlace src/main.rs \
              --replace ../stops.json ${./stops.json} \
              --replace ../trams.json ${./trams.json} \
              --replace ../buses.json ${./buses.json} \
              --replace ../formatted.csv ${telegramsDump}
          '';
        };
        doCheck = true;
        cargoTestCommands = x:
          x ++ [
            # clippy
            ''cargo clippy --all --all-features --tests -- \
              -D clippy::pedantic \
              -D warnings \
              -A clippy::type-complexity \
              -A clippy::too-many-lines''
          ];
      };
      packages.line-info = pkgs.runCommandNoCC "line-info" {
        buildInputs = [ packages.runalyzer ];
      } ''
        mkdir $out
        cd $out
        runalyzer
      '';

      packages.stops = pkgs.stdenv.mkDerivation {
        name = "stops-json";
        src = ./.;
        installPhase = ''
          mkdir -p $out/json
          cp stops.json $out/json/
        '';
      };

      defaultPackage = packages.line-info;

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
        (with packages.runalyzer; nativeBuildInputs ++ buildInputs);
      };
    }) // {
      overlays.default = final: prev: {
        inherit (self.packages.${prev.system})
          runalyzer;
      };
    };
}
