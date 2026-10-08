# Self-contained build recipe for the `slop` binary.
#
# Deliberately builds from `pkgs` ALONE - no crate2nix, no rust-overlay. Those
# are declared in devenv.yaml, and devenv 2.0 does NOT evaluate an imported
# project's devenv.yaml when it is consumed cross-project via
# `inputs.<name>.devenv.config.outputs.slop` (cachix/devenv#2205). Anything this
# output touches must therefore resolve from nixpkgs `pkgs` only.
#
# This also makes the file usable as a direct-import fallback for consumers
# that are not on devenv 2.0:
#   import "${inputs.slop}/packages/slop.nix" { inherit pkgs; }
{
  pkgs,
  lib ? pkgs.lib,
  # Repo root, filtered to the build inputs so target/, .devenv/ and .git do not
  # bloat the source or perturb the derivation hash. Overridable by callers.
  src ? lib.fileset.toSource {
    root = ../.;
    fileset = lib.fileset.unions [
      ../Cargo.toml
      ../Cargo.lock
      ../crates
      ../rust-toolchain.toml
    ];
  },
}:
pkgs.rustPlatform.buildRustPackage {
  pname = "slop";
  version = (builtins.fromTOML (builtins.readFile ../Cargo.toml)).workspace.package.version;
  inherit src;

  # Read the lockfile THROUGH `src`, not as `../Cargo.lock`.
  #
  # A bare `../Cargo.lock` path literal is copied into the store as its own
  # object, giving `cargo-vendor-dir` an input that exists only as a loose
  # source path - one whose hash changes whenever Cargo.lock does, i.e. only on
  # a dependency bump. On CI that path is restored from a Nix store cache keyed
  # on devenv.nix/devenv.yaml/devenv.lock/rust-toolchain.toml, none of which a
  # dependency bump touches, so the run reuses a store snapshot taken when a
  # DIFFERENT Cargo.lock was current and eval dies with
  #
  #   error: path '/nix/store/...-Cargo.lock' is not valid
  #
  # Resolving it through `src` gives the same file with no second store object:
  # the fileset source above already contains Cargo.lock, and it is a real build
  # input, so it is cached and garbage-collected together with the build that
  # needs it instead of drifting from it.
  cargoLock.lockFile = src + "/Cargo.lock";

  # Build only the `slop` binary, not the whole workspace.
  cargoBuildFlags = [
    "--bin"
    "slop"
  ];

  # Tests run in the dev shell / CI; keep the distributable artifact lean.
  doCheck = false;

  meta = {
    description = "MAHDTech GitHub profile automation CLI";
    mainProgram = "slop";
    license = with lib.licenses; [
      mit
      asl20
    ];
  };
}
