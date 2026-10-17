# Original projectile update comparisons

The retained original shot supplies34 HandleProjectile entry/return pairs over
frames8661..8694. `sm-original compare --scope handle` compares the complete
bounded selection, with each invocation starting from its own captured original
entry state. It does not feed an isolated routine's result into the next call:
other gameplay, interrupts and hardware run between those original invocations.

The fixed original-output projection contains801 bytes: the existing first-Fire
441-byte projectile/word/sound selection plus all ten18-slot trail arrays at
[0xd658,0xd7c0). Complete131072-byte C/Rust equality is a separate requirement.
Raw input, original/C/Rust output, CPU context, call ledger, projection and first
differences are retained. CPU scratch/stack, original hardware timing, native
sound-call order/audio and continuous live replay parity remain outside scope.

The command validates ordinary Handle JSR/RTS linkage and CPU eligibility,
including native mode, DP0, DB90, 16-bit widths and no interrupted call. Fire's
inherited-PHP frame does not apply. A safe Rust invocation precedes each actual-C
call; no PLM service or unsupported inventory is permitted in this captured
ordinary beam/explosion scope. Drawing roots are not selected.

The original capture stays at `out/original/capture-04` because its observer
identity binds its absolute output path. The current command uses the prepared
post-ticket13 native package. Historical first-Fire readers remain retained
under `out/original/bin/` and keep their historical source/library contracts.
Default `compare` still selects first Fire; only `--scope handle` selects34 calls.

From `restart/`, using a fresh output directory after the command is built:

```sh
./target/debug/sm-original compare --scope handle \
  --capture out/original/capture-04 \
  --library out/projectile-escape/native/0eee36456c66e6a447305e1a2097e96e3afd32365a9013c090f8faf5cea6720f/libsm_native.so \
  --rom /home/jim/sm-decomp/rom/sm.sfc \
  --out out/original-updates/my-comparison
```

The existing separate-process30-second deadline applies. The original capture
and current native package are authenticated for the whole batch, rather than
rescanning the full capture for every call. `--fault yes` must produce an explicit
comparison failure; it does not alter the original capture or projection.

Ticket14 passes at integrated source `cbe2b38`: all 34 independent calls agree
on the fixed original projection and complete C/Rust WRAM. This includes first
movement (frame 8661), trail allocation (8664), impact (8676) and removal (8694).
The batch takes 7.134 seconds including batch authentication and retention,
within the 30-second comparison deadline. This is comparison time, not a
rebuild-plus-comparison measurement. No new capture or ticket13 replay ran.

`out/original-updates/handle-01/result.toml` binds the current executable and
compiled source snapshots. `fire-compat-01` passes the unchanged default Fire
comparison. `fault-01` rejects a post-execution one-pixel Rust X mutation at
call 2/frame 8661: C and original retain 145, Rust becomes 144 at WRAM 0xb64.
It stops at that mismatch and authenticates final dependencies in 3.118 seconds.
This control tests the comparator, not a gameplay implementation mutation.

Focused tests: 12 pass, two asset-dependent tests ignored; Clippy and build pass.
The selector's separate retained-capture test also passes. Cleanup retires both
worker worktrees, preserving source and evidence. Independent review and final
hash authentication live under `out/original-updates/evidence/`. The shared
inventory records a bounded verified observation only; full original live
replay and drawing equivalence remain unproved.
