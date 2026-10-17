# Normal camera

Ticket12 adds the complete thirteen-routine `MainScrollingRoutine` closure to
the ordinary Rust runtime. It covers normal X/Y following, distance arithmetic,
autoscroll constraints, slow grapple motion and the Spore Spawn finish hook.
The root owns only WRAM: one bounded private calculation validates the actual
accesses, buffers passive observations, then publishes fourteen output words.
Unsupported inputs leave WRAM and observations unchanged; an observer error is
terminal and cannot retry in C.

The native hook intercepts the actual routine body. In the latched state8
caller, collision can publish state9 before camera following runs; checking
the visible state to decide ownership would miss those real calls. Normal
following calculates X distance, handles X, calculates Y distance and handles
Y before publishing all four previous Samus position/fraction words.
Same-call X results feed Y constraints. Integer-stationary axes still calculate
fractional distance and invoke the corresponding autoscroller.

`sm-pilot --scenario camera` uses the same 9,380 inputs and retains the five
preceding consumers' material requirements. New passive effects39–46 expose
balanced routine order and before/after layer, actor and previous positions.
Material checks use motion within camera roots, so directional IRQ movement
cannot satisfy normal-camera following. Rust owns every admitted camera call;
zero C entries/refusals and balanced completions are required.

The native package is prepared once and the C trace is captured once for its
identity. Candidate edits reuse that trace. From `restart/`:

```sh
CAMERA_LIBRARY="$PWD/out/camera/native/377a7a5bffacefbc4c91afa1c8960e19ea6773713469c39b8263f63faf4dc81a/libsm_native.so"
./out/camera/bin/sm-pilot dev --scenario camera \
  --library "$CAMERA_LIBRARY" --rom /home/jim/sm-decomp/rom/sm.sfc \
  --reference out/camera/runs/reference-01 --out out/camera/runs/my-edit
```

Output directories must be fresh. `--fault yes` adds one pixel after the first
real Y-camera movement; comparison still uses the unchanged C reference.
The ordinary `sm-restart run --gameplay rust` installs all six consumers.

The eighteen focused Rust tests cover fractions, wrapping, odd table indices,
integer-stationary axes, selector branches, same-call constraints, slow mode,
frozen autoscrollers, finish-hook clamping and atomic late refusal. Native
controls cover44 valid C/refusal fixtures, all13 routines, state9 body calls,
passive commit timing and21 fatal ownership modes. These complement live
comparison; unreached branches and original-ROM agreement remain separate.

Exact worker/integration source, binaries, controls, comparison results and
cleanup records are retained under `out/camera/`. Ticket11's full Ceres result
belongs to its earlier frozen source; this change does not inherit that result
or widen the projectile Mode7 admission domain.

## Verified integration

Frozen source `482e6b9` passes its first9,380-step C reference and Rust
candidate, comparing complete WRAM, passive external state and ordered effects.
The new C trace also preserves every WRAM/external-state hash from ticket10's
reference. All five prior consumers' material gates continue to pass.

All934 camera calls complete in Rust with zero owned C entries/refusals.
Eleven of13 routines are reached; the finish dispatcher and Spore hook remain
focused coverage. The measured routine vector in ABI order is
`[934,0,0,934,934,934,934,352,433,147,390,325,218]`.
The state9 camera calls occur at8,726 in DF45 and9,030 in DF8D. Every call
preserves current actor position and publishes all four previous-position
words;848 calls change previous state, including527 X-fraction and488
Y-fraction publications. Slow mode is not reached.

Camera-local integer motion occurs on182 Y calls in DF45,103 X calls in
DF8D and104 Y calls in DFD7. These observations distinguish normal camera
movement from doorway setup or IRQ motion.

| Check | Result | Wall time |
| --- | --- | ---: |
| New C reference | All material gates pass |12.600 s |
| First warm rebuild + comparison | All9,380 steps match |12.834 s |
| Ordinary Rust CLI | Final WRAM/PNG match, zero CPU fallback |8.037 s |
| Deliberate +1 Y pixel | First WRAM divergence at8,412 |10.269 s |
| README-only edit | Reused passing result, zero game processes |0.151 s |

The warm run includes a real0.549-second rebuild after touching byte-identical
gameplay source; the rebuilt executable hash is unchanged. The final affected
library/runner suites pass103 tests. Strict workspace/all-target Clippy,
formatting and builds pass; two constant-chunk iterator lint corrections
changed no gameplay. A rejected `--capture-every 0` CLI invocation ran no game;
the successful invocation uses9,380. Original failure logs are retained.

Both worker worktrees are retired with exact source/logs preserved. Finished
batch cleanup, authentication and independent result review are recorded under
`out/camera/evidence/`; no additional gameplay run is needed for cleanup.
