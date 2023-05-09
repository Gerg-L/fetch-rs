{rustPlatform}:
rustPlatform.buildRustPackage {
  name = "fetch-rs";
  src = ./.;
  cargoLock.lockFile = ./Cargo.lock;
}
