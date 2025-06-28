{
  description = "A flake with devshell using flake-parts";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
    flake-parts.inputs.nixpkgs.follows = "nixpkgs";
    devshell.url = "github:numtide/devshell";
  };

  outputs = inputs@{ self, nixpkgs, flake-parts, ... }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      systems = [ "x86_64-linux" ];
      imports = [ inputs.devshell.flakeModule ];

      perSystem = { config, pkgs, system, ... }: {
        devshells.default = {
          env = [
            {
              name = "OPENSSL_DIR";
              value = "${pkgs.lib.getDev pkgs.openssl}";
            }
            {
              name = "OPENSSL_LIB_DIR";
              value = "${pkgs.lib.getLib pkgs.openssl}/lib";
            }
            {
              name = "LIBCLANG_PATH";
              value = pkgs.lib.makeLibraryPath [ pkgs.libclang ];
            }
          ];
          packages = with pkgs; [
            rustc
            rust-analyzer
            cargo
            clang
            taplo
            toml-cli
            lld
          ];
        };
      };
    };
}
