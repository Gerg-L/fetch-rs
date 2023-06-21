{
  description = "another fetch tool written in rust";

  inputs.nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";

  outputs = {
    nixpkgs,
    self,
    ...
  }: let
    inherit (nixpkgs) lib;
    withSystem = f:
      lib.fold lib.recursiveUpdate {}
      (map (s: f s) ["x86_64-linux" "x86_64-darwin" "aarch64-linux" "aarch64-darwin"]);
  in
    withSystem (
      system: let
        pkgs = nixpkgs.legacyPackages.${system};
      in {
        overlay = final: _: lib.filterAttrs (n: _: n == "default") self.packages.${final.system};

        formatter.${system} = pkgs.alejandra;

        packages.${system} = {
          fetch-rs = pkgs.callPackage (
            {rustPlatform}:
              rustPlatform.buildRustPackage {
                name = "fetch-rs";
                src = self;
                cargoLock.lockFile = ./Cargo.lock;
              }
          ) {};
          default = self.packages.${system}.fetch-rs;
        };

        devShells.${system}.default = pkgs.mkShell {
          packages = [
            pkgs.rustfmt
            pkgs.rust-analyzer
          ];
          inputsFrom = [
            self.packages.${system}.default
          ];
        };
      }
    );
}
