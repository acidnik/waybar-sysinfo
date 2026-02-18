{ pkgs ? import <nixpkgs> { } }:
pkgs.mkShell {
  nativeBuildInputs = with pkgs; [
    cargo
    pkg-config
    rustfmt
  ];
  buildInputs = with pkgs; [
    glib
    gtk3
  ];
}
