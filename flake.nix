{
  description = "another fetch tool written in rust";

  inputs = {
    nixpkgs.url = github:NixOS/nixpkgs/nixos-unstable;
    flake-utils.url = github:numtide/flake-utils;
  };
  outputs = {
    self,
    nixpkgs,
    flake-utils,
  }:
    {
      overlay = self.overlays.default;
      overlays = {
        fetch-rs = _: final: {
          fetch-rs = self.packages.${final.system}.fetch-rs;
        };
        default = self.overlays.fetch-rs;
      };
    }
    // flake-utils.lib.eachDefaultSystem (
      system: let
        pkgs = import nixpkgs {inherit system;};
      in rec
      {
        formatter = pkgs.alejandra;
        packages = rec {
          fetch-rs = pkgs.callPackage ./default.nix {};
          default = fetch-rs;
        };
        devShells.default = pkgs.mkShell rec {
          buildInputs = with pkgs; [rustfmt cargo rustc rust-analyzer];
        };
      }
    );
}
