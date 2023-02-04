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
    flake-utils.lib.eachDefaultSystem (
      system: let
        pkgs = nixpkgs.legacyPackages.${system};
      in rec
      {
        formatter = pkgs.alejandra;
        packages = {
          fetch-rs = pkgs.callPackage ./default.nix {};
          default = self.packages.${system}.fetch-rs;
        };
        overlays = {
          fetch-rs = _: prev: {
            fetch-rs = self.packages.${system}.fetch-rs;
          };
          default = self.overlays.${system}.fetch-rs;
        };
        devShells.default = pkgs.mkShell rec {
          buildInputs = with pkgs; [rustfmt cargo rustc rust-analyzer];
        };
      }
    );
}
