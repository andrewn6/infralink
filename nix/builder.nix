{ pkgs ? import <nixpkgs> { system = "x86_64-linux"; } }:
pkgs.dockerTools.buildLayeredImage {
  name = "builder";
  tag = "latest";
  contents = [ 
  (pkgs.rustPlatform.buildRustPackage {
      cargoSha256 = "";
      name = "builder";
      src = ../services/builder;
    })
  ];
}
