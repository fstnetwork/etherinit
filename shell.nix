with import <nixpkgs> {};

stdenv.mkDerivation {
  name = "etherinit-devenv";
  buildInputs = [
    stdenv
    openssl
    pkgconfig
    rustup
  ];
}

