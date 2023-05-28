{
  description = "another fetch tool written in rust";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  outputs = {
    nixpkgs,
    self,
    ...
  }: let
    lib = nixpkgs.lib;
    withSystem = f:
      lib.foldAttrs lib.mergeAttrs {}
      (map (s: lib.mapAttrs (_: v: {${s} = v;}) (f s))
        ["x86_64-linux" "x86_64-darwin" "aarch64-linux" "aarch64-darwin"]);
  in
    {
      overlay = _: final: self.packages.${final.system};
    }
    // withSystem (
      system: let
        pkgs = nixpkgs.legacyPackages.${system};
      in {
        formatter = pkgs.alejandra;
        packages.fetch-rs = pkgs.callPackage ./. {};
        devShells.default = pkgs.mkShell {
          packages = [
            pkgs.rustfmt
            pkgs.cargo
            pkgs.rustc
            pkgs.rust-analyzer
          ];
        };
      }
    );
}
