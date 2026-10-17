# Bounded original-ROM shot capture

`sm-original` authors and captures one fixed original-input scenario. It does
not compare recovered C or Rust and exposes no state-injection interface.

```sh
cargo run --manifest-path restart/original/Cargo.toml -- capture \
  --lsnes /path/to/authenticated/lsnes-dumpavi.util \
  --rom /path/to/rom/sm.sfc \
  --movie /path/to/movies/super-metroid-any-3653.lsmv \
  --libraries /path/to/lsnes-build-deps/usr/lib/x86_64-linux-gnu \
  --out /path/to/new-capture
```

Add `--prepare-only yes` to authenticate and author inputs without launching the
emulator. Output must be a fresh directory; existing evidence is never reused or
overwritten. Preparation, execution and validation failures return nonzero and
retain a failure result where possible. Request and result manifests are
create-once, read-only TOML, binding input and artifact SHA-256 identities.

The command accepts the pinned pristine ROM, original movie and retained lsnes
binary only. It verifies all 15 movie members, their CRCs, reset-only member
inventory and core/RTC/ROM metadata. All non-input member payloads remain
identical, verified by reopening the new ZIP. The scenario preserves original
input frames 1–8,639, presses **Right** on 8,650, releases for nine frames,
presses physical **X** on 8,660 and releases through 8,850. Other new rows are
neutral. X is hardware word `$0040`; it is not the
native host's `$0200` mask. A missed input or absent shot fails capture.

Scenario v2 tests the source explanation for v1's missing shot: the initial
front-facing pose skips the firing dispatcher. A single Right press transitions
through pose `$26` to right-standing pose `$01`. The declared shot context is
X 130, Y 75, radius 21, direction 8, standing movement and zero vertical
velocity. X=130 is calibrated from original capture-02 before any C/Rust
comparison; the v2 movie is unchanged. Separate `diagnostics/` snapshots retain full WRAM, CPU and selected
player state before Right at frame 8,648 and when actual X is first consumed.
The pre-turn snapshot is saved before asserting pose 0, radius 24, X 128/Y 72
and the Ceres player handlers. These files remain available on failed captures
and never count as routine-call proof. Failure results include measured elapsed
seconds and state whether preparation is included.

The original runs from its unchanged movie reset contract with private config,
explicit shared-library lookup, null video dumper and a 120-second process-group
deadline. Shared-library directory file hashes are recorded and checked again
after execution; system loader libraries are not a separately pinned runtime.
No emulator build or legacy pipeline crate is used.

`events.csv` retains a source-ordered reset-prefix ledger: actual controller
register read callback bytes, controller entry/return, dispatch entry/completion,
wait, NMI/IRQ entries and observed frame completions. Requested frame count alone
never establishes completion. Controller callbacks cover all mirrored hardware
banks. Debugger reads never substitute for actual `$4218/$4219` read values.

Full 128-KiB WRAM and CPU context snapshots begin at the actual first fire call
and continue through the same slot's ordinary beam `$8000`, impact `$8700` and
removal. The four admitted routine entries are `$90:B887`, `$90:AECE`,
`$93:8254` and `$93:834D`. Explicit RTS/RTL witnesses and stack-matched dynamic
continuation fetches pair entry and return. Fire inherits a saved-status byte
from its caller; its return frame is not an ordinary fresh JSR frame. Each CPU
sidecar records that context, actual interrupt count and raw CPU registers.
The pinned core increments PC before its program-read callback, so `boundary_pc`
identifies the observed fetch and raw `pc` is one byte ahead. These callbacks
also observe operand fetches; the call/exit/stack checks qualify this fixed set
of boundaries rather than claiming a general instruction tracer.

During the shot, global BUS callbacks retain actual mirrored PPU/APU, CPU I/O
and DMA register reads/writes. They are removed after observed shot removal.
Three source-validated semantic entry markers retain raw A and enclosing call
ID: `plm-entry` at `$84:84E7`, `sfx1-max15-entry` at `$80:9021` and
`sfx2-max6-entry` at `$80:90CB`. Each is immediately followed by
`semantic-status` at the same address/call with raw P in its value. PLM header
is 16-bit A; sound ID is low A before the queue prologue swaps its bytes. These
markers support routine eligibility and sound-request comparison without
claiming equality of original and native MMIO behavior.
WRAM snapshots capture OAM-building buffers; the observer does not synthesize
VRAM, renderer, APU or every internal bus-cycle state. Limits are 512 snapshots,
500,000 ledger events and 120 original frames for the active shot. Unexpected
context, an interrupted first fire, unmatched calls, missing hooks, missing
impact/removal or missing release produces a failed capture.

`result.toml` reports only an original capture result and always states
`original_equivalence = "not-tested"`. Independent routine fixture comparison
and a causally mapped mixed live replay remain separate work.
