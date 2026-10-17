# First original-ROM comparison

`sm-original` captures one bounded original shot and compares its first
`FireUnchargedBeam` invocation with actual recovered C and Rust. It is independent
of the warm gameplay development command in [PILOT.md](PILOT.md).

The first fixture passed in **0.88 seconds**: all 131,072 C/Rust WRAM bytes
agree, and both match the original return on all 441 selected bytes. An injected
one-pixel Rust-output mismatch was rejected in **0.82 seconds**, at WRAM `0xB64`
(expected 141, actual 142); C still matched the original. Results are retained in
`out/original/fire-01/` and `out/original/fire-fault-01/`.

After correcting the later drawing witness, `capture-04` observed the complete
original shot lifecycle in **37.918 seconds**. Its first Fire call also passes
the same comparison (`fire-02`, **1.63 seconds**, including authentication of
the larger capture). The compared input and outputs are byte-identical to the
earlier pair. These timings exclude building the command.

The original capture starts from reset using the pristine pinned ROM, retained
lsnes executable and movie. It preserves movie rows 1..8639, presses Right on
row 8650, releases, presses X on row 8660 and releases through row 8850. Original
hardware consumes that X at the controller poll in frame 8661. Movie rows,
original frames and native host steps are distinct counters.

## Run the comparison

Use the retained reader below with the retained capture and native package.
It authenticates their exact historical host recipe while current gameplay
continues to evolve. Its rebuildable workspace is
`out/original/source/lifecycle/restart/`. The command requires a fresh output
directory and enforces a 30-second deadline in a separate process:

```sh
./out/original/bin/sm-original-lifecycle compare \
  --capture out/original/capture-04 \
  --library out/scroll/native/2c9c9a3cf51a059220afac1ee04727c956e45d067e10e17405bf3adbd268fc7f/libsm_native.so \
  --rom /home/jim/sm-decomp/rom/sm.sfc \
  --out out/original/fire-new
```

Use another fresh output directory and `--fault yes` for the control that
changes the Rust projectile X coordinate by one pixel. It must exit nonzero.
This injects a mismatch after Rust execution to check the comparator; it does
not mutate the gameplay implementation. The earlier gameplay pilot has its
own implementation-fault control.

The command authenticates the capture, its inputs and retained files, then
validates the ledger from reset through the first Fire return. This includes
actual paired controller reads, Right/release/X consumption, contiguous completed
frames, entry/exit/return CPU context and the interrupt ledger. A later capture
failure remains recorded as a failure; it does not discard an earlier complete
and independently valid call. No game replay is needed to compare that call again.

Actual C and Rust each start from the same complete original entry WRAM. Both
use the native host's prepared ROM view, whose hash is recorded separately from
the pristine original ROM. The C invocation is a real exported routine call with
ownership counters. Rust runs the existing projectile implementation with no
C fallback; an unexpected PLM request fails eligibility.

The comparison requires complete 131,072-byte C/Rust WRAM agreement and compares
both outputs against the original return on the fixed 441-byte projection:

| Selection | Bytes |
| --- | ---: |
| Eighteen ten-slot projectile arrays, `[0xB64,0xCCC)` | 360 |
| Words at `0xB5E,0xCCC,0xCCE,0xDC0,0xDC4,0xDDE,0x18AC` | 14 |
| Sound-queue storage, `[0x643,0x686)` | 67 |

This projection was declared before reading comparison differences. It excludes
CPU stack, direct-page scratch and removed collision temporaries by source
contract. Full original/C/Rust outputs remain retained for inspection.

The eligible original call is standing in DF45 at X130/Y75, pose 1, with an empty
projectile pool and one ordinary shot. It contains one sound1/max15 request for
sound 11, no PLM call and no interrupt. Its four math-register events are retained;
hardware effects, native sound-call order and audible output are not compared.
Neither full original WRAM nor full projectile lifecycle or live replay parity
is claimed by this fixture.

## Capture evidence

Four captures are retained under `out/original/`:

- `capture-01`: X alone did not shoot because Samus was front-facing. The
  consumed input and failed shot condition remain recorded.
- `capture-02`: Right reached Fire; the initial X128..129 guard failed. Independent
  diagnostics showed that turning moves Samus to X130. Cost 32.623 seconds.
- `capture-03`: the corrected X130 guard produced the shot. Complete Fire
  entry 73917, RTS 73924 and return 73925 occur in original frame 8661. Cost 38.441 seconds.
  Later drawing instrumentation failed, so this capture's lifecycle status
  remains failed and no impact/removal is proved.
- `capture-04`: corrected drawing witness, unchanged inputs. All 8,850 frames
  complete with 148,677 events, 206 matched call snapshots, one Fire call and
  34 entry/return pairs each for Handle, DrawPlayer and DrawBomb. The same slot
  impacts at frame 8676 and is removed at frame 8694. No pending call remains.

The later failure has a concrete source diagnosis: the DrawBomb witness was
placed at `$93:83BC`, a BPL displacement byte. The actual sequence is BPL at
`$93:83BB`, PLP at `$93:83BD`, RTL at `$93:83BE`. The operand-fetch hook runs
before PLP and therefore sees the wrong stack depth. Source `fac67e6` corrects
the observer and validator to `$93:83BE`; `capture-04` proves that repair in the
original execution. Lifecycle C/Rust parity remains outside the Fire fixture.

Keep capture directories at their current absolute paths: generated observer
identity includes the output path. Retain the pinned ROM, source movie, lsnes
executable and shared-library directory named in `request.toml`, plus the complete
native library package. Source snapshots, checked commands, failure diagnostics
and cleanup/checksum receipts live alongside the retained runs.
The older `capture-03` pair remains reusable with `out/original/bin/sm-original-fixture`,
whose observer contract matches it; use `sm-original-lifecycle` for `capture-04`.

See [ticket07](../.scratch/restart-pipeline/issues/07-original-df45-shot.md) and
the [call contract](../.scratch/restart-pipeline/original-calls.md) for exact
source identities and comparison evidence.
