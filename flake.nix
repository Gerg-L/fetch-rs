{

  inputs.nixpkgs = {
    type = "github";
    owner = "NixOS";
    repo = "nixpkgs";
    ref = "nixos-unstable";
  };

  outputs =
    { nixpkgs, self }:
    {
      packages.x86_64-linux =
        let
          pkgs = nixpkgs.legacyPackages.x86_64-linux;
        in
        {
          fetch-rs = pkgs.callPackage ./package.nix { inherit self; };
          default = self.packages.x86_64-linux.fetch-rs;
        };
    };
}
