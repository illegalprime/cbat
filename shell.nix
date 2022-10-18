let
  # pin nixpkgs so it doesn't change
  nix_ref = "3d4de2ae3eb4b80f767ef5caca8225f9d86ca03b";
  nixpkgs = fetchTarball {
    url = "https://github.com/NixOS/nixpkgs/archive/${nix_ref}.tar.gz";
    sha256 = "sha256-udXmjI9dVMqTJ+FIf6pAIHyEggbsxl5P2aNy3YASr5Q=";
  };
  # grab mozilla's rust tools as a pinned version as well
  moz_ref = "6070a8ee799f629cb1d0004821f77ceed94d3992";
  mozilla = builtins.fetchTarball {
    url = "https://github.com/mozilla/nixpkgs-mozilla/archive/${moz_ref}.tar.gz";
    sha256 = "1lv3bh83f3f6caq5lk6bgpbq8zdd2xpw54xryccg1mxx8l5dadmq";
  };
  mozilla_rust = import "${mozilla}/rust-overlay.nix";
  # import nixpkgs with mozilla's overlay
  pkgs = import nixpkgs {
    overlays = [ mozilla_rust ];
  };
  crossPkgs = import nixpkgs {
    crossSystem = pkgs.lib.systems.examples.arm-embedded;
  };
  # choose our rust
  rust_channel = pkgs.buildPackages.rustChannelOf {
    channel = "1.64.0";
  };
  # rust platform here is needed to build cargo-hf2
  rust_platform = pkgs.makeRustPlatform {
    rustc = rust_channel.rust.override {extensions = ["rustc-dev"];};
    cargo = rust_channel.cargo;
  };
  # build cargo-hf2, our flashing utility
  cargo-hf2 = rust_platform.buildRustPackage {
    pname = "cargo-hf2";
    version = "0.9.1";
    buildAndTestSubdir = "cargo-hf2";
    src = pkgs.fetchFromGitHub {
      owner = "jacobrosenthal";
      repo = "hf2-rs";
      rev = "v0.3.3";
      sha256 = "sha256-QGE48EgGpsTt2hTw4zmWvW1E2T9p4ed1PTAQIFhtde4=";
    };
    nativeBuildInputs = with pkgs; [
      pkg-config
    ];
    buildInputs = with pkgs; [
      libusb
    ];
    # cargo-hf2 doesn't keep a lock file in git
    # we clone the repo and make our own and keep it here
    cargoLock.lockFile = ./cargo-hf2.lock;
    postPatch = ''
      cp ${./cargo-hf2.lock} Cargo.lock
    '';
  };
in
crossPkgs.stdenv.mkDerivation {
  name = "itsybitsy-m0";
  # since we're in embedded the only dependencies should be rust & hf2
  nativeBuildInputs = [
    cargo-hf2
    (rust_channel.rust.override {
      # add the target architecture
      targets = ["thumbv6m-none-eabi"];
      extensions = ["rust-src"];
    })
    pkgs.rust-analyzer # editor completion
  ];
  depsBuildBuild = [ pkgs.buildPackages.stdenv.cc ];
  # verbose error messages
  RUST_BACKTRACE = 1;
}
