{ pkgs ? import <nixpkgs> {} }:
pkgs.mkShell {
  name = "dev-environment";
  buildInputs = [
    pkgs.clickhouse
    pkgs.redis
    pkgs.rustc
    pkgs.cargo
    pkgs.rustfmt
    pkgs.rust-analyzer
    pkgs.clippy
  ];
  shellHook = ''
    echo "Infralink"
  '';
}

