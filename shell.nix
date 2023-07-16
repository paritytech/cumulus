let
  moz_overlay = import (builtins.fetchTarball https://github.com/mozilla/nixpkgs-mozilla/archive/master.tar.gz);
  nixpkgs = import <nixpkgs> {}; # { overlays = [ moz_overlay ]; };
  #rustNightlyChannel = (nixpkgs.rustChannelOf { date = "2019-01-26"; channel = "nightly"; }).rust;
  # rustStableChannel = nixpkgs.latest.rustChannels.stable.rust.override {
  #   targets = [ "wasm32-unknown-unknown" ];
  #   extensions = [
  #     "rust-src"
  #     "rls-preview"
  #     "clippy-preview"
  #     "rustfmt-preview"
  #   ];
  # };
in
  with nixpkgs;
  stdenv.mkDerivation {
    name = "moz_overlay_shell";
    buildInputs = [
      # rustStableChannel
      # rls
      rustup
      openssl.bin openssl.dev
      pkgconfig
      # protobuf compiler, needed for libp2p:
      protobuf
      clang
      # llvmPackages.llvm
    ];
  # For libp2p:
  PROTOC="${protobuf}/bin/protoc";
  PROTOC_INCLUDE="${protobuf}/include";
  LLVM_CONFIG_PATH="${llvm}/bin/llvm-config";
  LIBCLANG_PATH="${llvmPackages.libclang.lib}/lib/";
}
