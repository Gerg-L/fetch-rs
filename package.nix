{
  self,
  lib,
  rustPlatform,
}:
let
  _self = /. + (builtins.unsafeDiscardStringContext self);
  toml = ((lib.importTOML (_self + /Cargo.toml)).package);
in
rustPlatform.buildRustPackage {
  pname = toml.name;
  inherit (toml) version;
  src = lib.fileset.toSource {
    root = _self;
    fileset = lib.fileset.unions [
      (_self + /src)
      (_self + /Cargo.toml)
      (_self + /Cargo.lock)
    ];
  };
  cargoLock.lockFile = _self + /Cargo.lock;

  meta.mainProgram = "fetch-rs";
}
