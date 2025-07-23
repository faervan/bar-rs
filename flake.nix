{ 
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.05";

    crane.url = "github:ipetkov/crane";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs = {
        nixpkgs.follows = "nixpkgs";
        flake-utils.follows = "flake-utils";
      };
    };
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils, crane, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        cargoToml = "${self}/Cargo.toml";
        cargoTomlConfig = builtins.fromTOML (builtins.readFile cargoToml);
        nativeBuildInputs = with pkgs; [ 
          # free, grep, printf, awk are assuemed installed
          pkg-config
          playerctl
          pulseaudio # pactl
          wayland
          wireplumber # wpctl
        ];
      in
      {
        devShells = {
          default = pkgs.mkShell (rec {
            buildInputs = with pkgs; [ 
              libudev-zero
              libxkbcommon
              openssl
              pkg-config
              pkgs.rust-bin.stable.${cargoTomlConfig.package.rust-version}.minimal
              wayland
            ];
            inherit nativeBuildInputs;

            # ICED needed additional LD_LIBRARY_PATH
            shellHook = ''
              export LD_LIBRARY_PATH="$LD_LIBRARY_PATH:${builtins.toString (pkgs.lib.makeLibraryPath [pkgs.wayland])}"
            '';
          });
        };
        packages = let
          buildInputs = with pkgs; [ 
              libudev-zero
              libxkbcommon
              openssl
              pkg-config
              wayland
          ];
          craneLib = crane.mkLib pkgs;
          doCheck = false;
          src = self;
          version = cargoTomlConfig.package.version;
          warpped-bar-rs = pkgs.writeShellScriptBin "wrapped-bar-rs" ''
            export LD_LIBRARY_PATH="$LD_LIBRARY_PATH:${builtins.toString (pkgs.lib.makeLibraryPath [pkgs.wayland])}"
            exec ${bar-rs}/bin/bar-rs "$@"
          '';
          bar-rs = craneLib.buildPackage {
            inherit buildInputs cargoToml doCheck nativeBuildInputs src version;
            cargoArtifacts = craneLib.buildDepsOnly {
              inherit buildInputs src;
            };
            pname = "${cargoTomlConfig.package.name}";
          };
        in
        {
          default = warpped-bar-rs;
        };
      }
    );
}
