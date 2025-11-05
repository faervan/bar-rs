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
          dbus # bluetooth
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
              dbus
            ];
            inherit nativeBuildInputs;
            LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath buildInputs;
          });
        };
        packages = let
          buildInputs = with pkgs; [ 
              libudev-zero
              libxkbcommon
              openssl
              pkg-config
              wayland
              dbus
          ];
          craneLib = crane.mkLib pkgs;
          doCheck = false;
          src = self;
          version = cargoTomlConfig.package.version;
          warpped-bar-rs = pkgs.writeShellScriptBin "wrapped-bar-rs" ''
            export LD_LIBRARY_PATH="${pkgs.lib.makeLibraryPath buildInputs}"
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
