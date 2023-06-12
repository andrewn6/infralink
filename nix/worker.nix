{ pkgs ? import <nixpkgs> { system = "x86_64-linux"; } }:
pkgs.dockerTools.buildLayeredImage {
  name = "worker";
  tag = "latest";
  contents = [ 
  (pkgs.rustPlatform.buildRustPackage {
      cargoSha256 = "";
      name = "worker";
      src = ../worker;
    })
  ];
}
