{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    naersk.url = "github:nix-community/naersk/master";
    nixpkgs-rust.url = "github:NixOS/nixpkgs/nixos-unstable";
    utils.url = "github:numtide/flake-utils";

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs-rust";
    };
  };

  outputs = inputs @ { self, ...}:
    inputs.utils.lib.eachDefaultSystem (system:

      let
        pkgs = import inputs.nixpkgs { inherit system; };
        pythonVersion = builtins.replaceStrings [ "." "\n" ] [ "" "" ] (builtins.readFile ./.python-version);
        python = builtins.getAttr "python${pythonVersion}" pkgs;

        pkgs-rust = import inputs.nixpkgs-rust {
          inherit system;
          overlays = [ inputs.rust-overlay.overlays.default ];
        };

       rust-config = {
          extensions = [ "rust-src" ];
        };

        rust = (pkgs-rust.rust-bin.fromRustupToolchainFile ./rust-toolchain).override rust-config;

        rustfmt-nightly = pkgs-rust.rust-bin.nightly.latest.rustfmt;

      in
      {
        devShells.default = pkgs.mkShellNoCC {
          packages = with pkgs; [
            cargo
            just
            maturin
            openssl
            pkg-config
            poetry
            prek
            present-cli
            python
            ruff
            rustfmt-nightly
            rust
            rustPackages.clippy
          ];
        };
      }
    );
}
