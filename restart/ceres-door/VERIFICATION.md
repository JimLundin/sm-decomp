# Verification scope

The dependency-free unit checks exercise the public scheduled-entry interface
with authored WRAM and ROM data. Thirteen tests pass:

- All initialization parameters and both initial palette branches, including
  the precise parameter-2 VRAM descriptor and preservation of queued data.
- The ten instruction callbacks, slot-local property changes, independent boss
  and Ceres-status conditions, strict 48-pixel proximity and signed wrapping.
- Both quake states without unintended random advancement; visibility changes
  selected by the low Ceres-status bit.
- Seven-byte and nine-byte Mode7 descriptors, trailing termination, queue
  preservation, odd palette addresses and the palette-change guard.
- Escape arm/countdown/finish: 49 calls, ten puff requests, shared slot-0
  counters, exact initial point/random/parameters, copied-back foreign writes
  and sound following successful return.
- Owned foreign failure, random end-around carry vectors, sound ring wrap,
  queue fullness and disable guards.
- Four vertical solid-block writes with preserved tile/BTS/neighbor bytes;
  unchanged refusal and validation failures before foreign work or mutation.

The isolated checks use a temporary standalone workspace declaration in the
crate manifest and a private target directory. Integration removes that
declaration and registers the crate in the existing restart workspace.

Commands from the worker worktree:

```sh
cargo test --manifest-path restart/ceres-door/Cargo.toml --offline --target-dir restart/ceres-door/target
cargo clippy --manifest-path restart/ceres-door/Cargo.toml --offline --target-dir restart/ceres-door/target --all-targets -- -D warnings
```

These checks establish the authored test cases and clean compilation. They do
not establish actual-C parity, live exercise of the escape branch, or original
ROM equivalence. The integrator owns the existing combined `sm-pilot` C/Rust
reference comparison and retained door-opening/crossing observations. The
9,380-step pre-escape route does not exercise every translated main state.
