{ self, rustPlatform }:
rustPlatform.buildRustPackage {
  name = "fetch-rs";
  src = self;
  cargoLock.lockFile = ./Cargo.lock;
  meta.mainProgram = "fetch-rs";
}
