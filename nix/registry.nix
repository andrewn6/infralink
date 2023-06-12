i{ pkgs ? import <nixpkgs> { system = "x86_64-linux"; } }:
pkgs.dockerTools.buildLayeredImage {
  name = "registry";
  tag = "latest";
  contents = [ 
  (pkgs.rustPlatform.buildRustPackage {
      cargoSha256 = "";
      name = "registry";
      src = ../services/registry;
    })
  ];
}
