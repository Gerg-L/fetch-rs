{
  lib,
  rustPlatform,
}:
rustPlatform.buildRustPackage rec {
  pname = "fetch-rs";
  version = "1.0.0";

  src = ./.;

  buildInputs = [
  ];

  cargoSha256 = "sha256-9A1bhXc9gtjpxVfVLyTQ7AEsc3EEBrnBzOuyOBU6T9E=";

  meta = with lib; {
    homepage = "https://github.com/ISnortPennies/fetch-rs";
    description = "";
    license = licenses.unlicense;
    maintainers = with maintainers; [];
    platforms = platforms.linux;
  };
}
