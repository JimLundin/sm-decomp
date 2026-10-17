# Rust Ceres doors

The third Rust gameplay consumer owns the complete E23F Ceres door actor:
initialization, its six private main states, eight door instructions, the two
shared steam instructions when called for E23F, and BA48's four-block solidity
setup. The generic enemy/PLM scheduler, collision, rendering, room loading and
hardware queue consumption remain in recovered C.

`ceres-door` operates on shared WRAM and immutable ROM. Its one foreign gameplay
service spawns an E509 escape puff with an explicit parameter and position.
The native adapter copies WRAM synchronously around that service and observes
the external spawn position. Actual dispatcher entries are counted; calls
between private main states do not become additional scheduled entries.

## Comparison

The `ceres-doors` scenario uses the same 9,380 inputs as the scrolling milestone,
including the shot at step 8,380. It retains projectile and scrolling checks and
adds actual closed → proximity → animation → open observations,
parameter-2 decoration updates, and the DF45 → DF8D → DFD7 crossings
with resumed input. Per-slot native branch counts distinguish the shaft door
in slot 1 from its decoration in slot 0.

The initial frozen implementation is `ca4f20959a1b9ced89d8e731deff6b494e195473`;
`bd4e28c` corrects the live requirement without changing gameplay or native code.
Checked executables, native source/library, integration source and focused
test/control logs are retained under `out/ceres-door/`.
The first C invocation failed an incorrect requirement for BA48 solid setup:
DF45's default DF57 state selects C976 (haze); only boss-event state DF71 selects
C96E (solid setup). The complete authenticated C trace remains reusable, while
its failed invocation/result remain unchanged. The corrected scenario reports
BA48 as unexercised; direct native controls cover its dispatcher boundary.

The corrected candidate passes all 9,380 observations: complete WRAM, passive
external state and ordered effects match the C trace. All 2,735 reached door
entries complete in Rust, with zero C entries or refusals. The normal CLI reaches
the same final WRAM and PNG. The forced warm rebuild plus comparison costs
21.419 s, including 0.908 s for the build. A deliberately wrong proximity return
is rejected at its first changed step, 8,692, in 19.122 s.
Documentation-only reuse costs 0.303 s with zero game processes and authenticates
the retained native package, trace and result after worker retirement.

| Parameter-1 exit actor | Closed | Proximity / animation | Open |
| --- | ---: | ---: | ---: |
| DF45, slot 1 | 8,261 | 8,692 | 8,712 |
| DF8D, slot 1 | 8,831 | 8,991 | 9,011 |
| DFD7, slot 1 | 9,140 | 9,358 | 9,378 |

Room exits occur at steps 8,726 and 9,030; movement resumes in the destination
rooms at 8,832 and 9,141. Parameter-0 entry actors close on this route but do not
reopen. Eight of the thirteen scheduled entry kinds are exercised. Evidence is
under `runs/dev-01`, `runs/cli-rust-02` and `runs/fault-02`; the independent review
is under `evidence/verifier/`.

From `restart/`, using the retained complete C trace:

```sh
DOOR_LIBRARY="$PWD/out/ceres-door/native/8cf2945825f87c13bb3fc6cb240d1e59b3bb3b76a95c847624cad9978e62c0ad/libsm_native.so"
./out/ceres-door/bin-v2/sm-pilot dev --scenario ceres-doors \
  --library "$DOOR_LIBRARY" --rom /home/jim/sm-decomp/rom/sm.sfc \
  --reference out/ceres-door/runs/reference-01 --out out/ceres-door/runs/my-edit
```

Use a fresh output directory. `dev` builds incrementally and reuses an unchanged
passing result; `check` always runs the candidate. The retained reference stays
fixed across gameplay edits. The normal `sm-restart run --gameplay rust` installs
all three consumers together.

This route exercises pre-escape behavior. The escape countdown/puff branch has
focused Rust tests and native service controls, but no live C/Rust comparison
here. No original-ROM agreement follows from this recovered-C comparison; the
earlier first-Fire result remains limited to its declared 441-byte projection.
