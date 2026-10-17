# Projectile updates during Ceres escape

Ordinary `HandleProjectile` updates now run in Rust when the Ceres-status high
bit is set. The complete scheduler still owns flight, collision, instructions
and deletion. Trail allocation uses the recovered-C signed Mode7 transform,
including separate product truncation, wrapping and M7A in both equations.
The existing synchronous SpawnPLM service and inventory guards are unchanged.

Fire and both projectile drawing roots retain their high-bit exclusion. The
recovered C uses undefined local coordinates in the affected draw paths; this
change claims no active escape-shot rendering or original-ROM parity.

The full scenario uses the unchanged 16,991 saved inputs. It reaches ordered
escape states 20, 21 and 22, then normal Landing Site gameplay. There is no
Fire call. All 706 high-bit update entries now complete in Rust; each renderer
has 707 explicitly attributed high-bit refusals. Other refusals are zero.
All 5,597 postactivation updates complete in Rust; 1,576 earlier C calls remain
an explicit prelude. The other five Rust consumers retain zero owned fallback.

Both integrated runs match all 16,991 full WRAM, external-state and ordered-effect
observations against C. The final warm rebuild and comparison costs 28.665 seconds
(0.548 seconds build); README-only reuse costs 0.962 seconds with no gameplay
process. The earlier run cost 24.403 seconds and its docs reuse 0.166 seconds.
The one new full C reference costs 22.971 seconds and is reused across candidate
edits. Final source is `5850265`; the first comparison remains bound to `3795b3e`.

Active transformed trails have separate actual-C evidence: four bounded matrix
fixtures, each 27 update returns, compare all 128 KiB WRAM and ordered effects.
They cover trail allocation, flight, solid collision, explosion instructions
and deletion. No drawing or host step runs in those 108 pairs; synthetic state
injection does not supply live-from-reset active-shot coverage. The short
9,380-step shot scenario remains a separate regression through the same runner.

From `restart/`, with a fresh output directory:

```sh
./out/projectile-escape/bin/sm-pilot dev --scenario projectile-escape \
  --library out/projectile-escape/native/0eee36456c66e6a447305e1a2097e96e3afd32365a9013c090f8faf5cea6720f/libsm_native.so \
  --rom /home/jim/sm-decomp/rom/sm.sfc \
  --reference out/projectile-escape/runs/reference-01 \
  --out out/projectile-escape/runs/my-edit
```

`--fault yes` deliberately advances slot zero's X after one real high-bit Rust
update, including an inactive slot that should remain unchanged. This fails
full-WRAM comparison at step 14,373. The first scratch-index fault was overwritten
by downstream drawing and is retained as an ineffective probe. Reference data
and comparison rules remain unchanged. Normal CLI play uses the same projectile
implementation and admission.

Retain `out/projectile-escape/` for exact source, prepared native package,
binaries, worker controls, replay results, original failure logs and cleanup.
The material records expose reached gameplay and ownership separately from
active transformed-trail and original-ROM coverage. Ticket 13 records final
validation and retention details.

The separate 9,380-step C/Rust comparison passes its active shot lifecycle and
all six consumers' material checks. Normal CLI full-Ceres replay passes with
final WRAM and PNG identical to the full reference. There are 125 passing
library/runner tests (three asset-dependent tests remain ignored), plus the
108 actual-C update-return pairs; strict Clippy and builds pass.

Both integrated worker worktrees and their disposable targets are retired.
Small source snapshots remain at their original paths without Git/build output.
Both utility cleanup passes report zero additional recognized intermediates;
gross allocated retirement reclaimed 817,868,800 bytes. All 689 pre-cleanup
hashes pass, and native/reference/result authentication after cleanup reuses the
final result in 0.259 seconds with zero game processes. The primary warm target,
exact source snapshots, prepared C library, inputs, logs and binaries are retained.
