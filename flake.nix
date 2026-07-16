{
  description = "GERC development and verification shell";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
        };
      in
      {
        devShells.default = pkgs.mkShell {
          strictDeps = true;

          packages = with pkgs; [
            rustc
            cargo
            rustfmt
            clippy
            rust-analyzer
            llvmPackages.lldb
            gcc
            clang
            binutils
            mdbook
            zlib
            libpng
            openssl
            curl
            libpcap
            libxml2
            alsa-lib
            ncurses
            libx11
            xorgproto
            git
            pkg-config
          ];

          RUST_SRC_PATH = "${pkgs.rustPlatform.rustLibSrc}";
          CPATH = pkgs.lib.concatStringsSep ":" [
            "${pkgs.stdenv.cc.libc.dev}/include"
            "${pkgs.linuxHeaders}/include"
            "${pkgs.zlib.dev}/include"
            "${pkgs.libpng.dev}/include/libpng16"
            "${pkgs.openssl.dev}/include"
            "${pkgs.curl.dev}/include"
            "${pkgs.libpcap}/include"
            "${pkgs.libxml2.dev}/include/libxml2"
            "${pkgs.alsa-lib.dev}/include"
            "${pkgs.ncurses.dev}/include"
            "${pkgs.libx11.dev}/include"
            "${pkgs.xorgproto}/include"
          ];
          LIBRARY_PATH = pkgs.lib.makeLibraryPath [
            pkgs.zlib
            pkgs.libpng
            pkgs.openssl
            pkgs.curl
            pkgs.libpcap
            pkgs.libxml2
            pkgs.alsa-lib
            pkgs.ncurses
            pkgs.libx11
          ];

          shellHook = ''
            export CC=gcc
            export PATH="$PATH:$PWD:$PWD/target/debug:$PWD/target/release"
          '';
        };
      });
}
