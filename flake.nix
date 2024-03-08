{
  description = "Build yurtur.top website";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-23.11";

    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    # rust version
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs = {
        nixpkgs.follows = "nixpkgs";
        flake-utils.follows = "flake-utils";
      };
    };
  };

  outputs = { self, nixpkgs, crane, flake-utils, rust-overlay, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };

        inherit (pkgs) lib;

        rust-toolchain =
          pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain;

        craneLib = (crane.mkLib pkgs).overrideToolchain rust-toolchain;
        src = lib.cleanSourceWith {
          src = (craneLib.path ./.);
          filter = path: type:
            builtins.any (filter: filter path type) [
              craneLib.filterCargoSources
            ];
        };

        # but many build.rs do - so we add little bit slowness for simplificaiton and reproduceability
        rustNativeBuildInputs = with pkgs; [ clang pkg-config gnumake ];

        # reusable env for shell and builds
        rustEnv = with pkgs; {
          LD_LIBRARY_PATH = pkgs.lib.strings.makeLibraryPath [
            pkgs.stdenv.cc.cc.lib
            pkgs.llvmPackages.libclang.lib
          ];
          CARGO_NET_GIT_FETCH_WITH_CLI=true;
          LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";
          RUSTUP_TOOLCHAIN = (builtins.fromTOML (builtins.readFile ./rust-toolchain.toml)).toolchain.channel; # for dylint
        };

        gdk = pkgs.google-cloud-sdk.withExtraComponents( with pkgs.google-cloud-sdk.components; [
          gke-gcloud-auth-plugin
        ]);
      in
      {
        checks = {};

        devShells.default = pkgs.mkShell {

          inputsFrom = builtins.attrValues self.checks.${system};

          # Extra inputs can be added here
          nativeBuildInputs = with pkgs; [
            rustNativeBuildInputs
            (lib.optionals pkgs.stdenv.isLinux pkgs.mold)
            pkgs.pkg-config
            gdk
          ];

          buildInputs = [
            # We want the unwrapped version, wrapped comes with nixpkgs' toolchain
            pkgs.rust-analyzer-unwrapped
            pkgs.openssl
            pkgs.protobuf
            pkgs.systemd
            (pkgs.python3.withPackages (python-pkgs: [
              python-pkgs.psycopg2
            ]))
            # Finally the toolchain
            rust-toolchain
          ];
        } // rustEnv;
      });
}
