let
  # Pinned nixpkgs, deterministic. Last updated: 2/12/21.
  pkgs = import (fetchGit {
    url = "https://github.com/NixOS/nixpkgs";
    ref = "nixos-24.11";
    rev = "1c6e20d41d6a9c1d737945962160e8571df55daa";
  }) { };
in
# Rolling updates, not deterministic.
# pkgs = import (fetchTarball("channel:nixpkgs-unstable")) {};
pkgs.mkShell {
  buildInputs = [
    pkgs.cargo
    pkgs.rustc
    pkgs.rustfmt
  ];
}
