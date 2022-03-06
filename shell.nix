{ pkgs ? import <nixpkgs> {} }:

with pkgs;
mkShell {
  packages = [ go_1_17 goimports ];
}
