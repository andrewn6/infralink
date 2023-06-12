{ pkgs ? import <nixpkgs> { system = "x86_64-linux"; } }:
pkgs.dockerTools.buildLayeredImage {
  name = "ping-server";
  tag = "latest";
  contents = [ 
  (pkgs.rustPlatform.buildRustPackage {
      cargoSha256 = "";
      name = "ping-server";
      src = ../services/ping-server;
    })
  ];
}
