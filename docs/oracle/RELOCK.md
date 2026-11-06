# Re-locking the corpus on lsnes

The plan for recording the new engine's locks on the unmodified ROM, oracle-side,
before the engine exists (ARCHITECTURE §11.4 D22, D25; §15 step 1). The root runs it
as short lanes and background jobs. Decisions it rests on: the gate runs no oracle;
lsnes (rr2-beta23, bsnes v085, its own process) records every stream of every row;
seeds are lsnes native states cut from the movie ladders; long runs are the exception.

Sources: [ARCHITECTURE](../architecture/ARCHITECTURE.md) §11, §15;
[HARNESS](HARNESS.md); [MEASUREMENTS](MEASUREMENTS.md) (e), (i), (o), (r);
[the harness handoff](../handoff/oracle-harness.md); [today's lock](../../scenarios/lock.toml).

## 1. Inventory

Counted from `scenarios/lock.toml`, `movies/room-seeds-*.csv` and
`$TEMP/ohx/data/lsnes-native/` on 2026-11-06.

| | Count |
| --- | --- |
| Faithful rows (`oracle = "faithful"`) | 638 |
| Rows from power-on (`options-menu-tour`, 2310 Ticks) | 1 |
| Rows from a seed | 637 |
| Distinct seed captures they name | 551 |
| Already cut as lsnes native seeds (28 movie captures, 1 derived) | 29 |
| Movie captures still to cut | 505 |
| Derived seeds still to derive (`scenario-*`, `tools/scenario_seeds.py`) | 17 |

The 505 captures to cut, by movie and ladder rung (rungs every 10000 frames,
`$TEMP/ohx/data/lsnes-ladder/`):

| Movie | Captures | Windows (rung → last capture) | Notes |
| --- | --- | --- | --- |
| 100% (`100-input-phase`, `100-boundary`, `100-mb-phases`) | 325 | 23: rung 0 (power-on) to 220000 | 2-23 captures a window |
| any% (`any-boundary`) | 176 | 12: rungs 10000 to 120000 | |
| low% (`low-input-phase`) | 2 | 2: rungs 50000, 90000 | |
| Spore Spawn RTA (`spore-input-phase:38`) | 1 | no ladder: one run from power-on to about frame 30232 | the one long cut |
| `golden-input-phase:1` (`boss-golden-torizo`) | 1 | none | captured from the snes9x JXQ 100% movie (docs/ITERATION.md), which lsnes can't play; see Q5 |

The 17 derived seeds (coverage-metroid, -ice, -botwoon, crystal-flash, dachora,
draygon-repel, etecoon-tunnel-jump, kihunter-wings, lower-norfair-hand, oum-roll,
ridley-swoop, the 4 special-beam seeds, tatori-touch, wrecked-ship-hand) need
`oracle derive-seed` from a base capture. Every base is itself used by a row, so
it is cut anyway. Deriving is word edits on the base state, with no emulation
(the crocomire seed is the model: `0142.lss` + `0142.words`).

**Recording time.** The rows hold 418,259 Ticks. Overrun and NMI-off frames add
about 5% (MEASUREMENTS (e): chain rows +5%, death rows +19%), so about 445,000
frames. At lsnes's 35 frames/s with streams (MEASUREMENTS (r)) that is 3.5
core-hours. Under load lsnes runs about 20 frames/s per process with 12-13 at once
((e)), about 250 frames/s in total, so **about 30 minutes wall** on this machine.
A job of ≤ 5 min at 20 frames/s is ≤ 6000 frames: bin-packed, about 80 jobs.
Five rows are longer than that (`chain-0324-DD58` 10134 Ticks,
`chain-any-0190-DD58` 10175, `credits-postroll-wave` 11790, `credits` 11900,
`credits-ending` 15900; 59,899 Ticks). They run whole, one job each, started
first at low concurrency (5-8 min at 35 frames/s), and the recorder saves a
row rung every 5000 frames so that later re-locks split them (§2).

**Seed-cut time.** 38 windows of ≤ 10000 frames without memories or streams
(100 frames/s alone, as the ladder regeneration ran) plus the Spore run: about
400,000 frames. At 8 at a time, about 15 min wall; each window job 2-4 min,
the Spore job about 8 min.

## 2. The new lock

**Where.** The digests, overrun schedules and facts' results are hashes, frame
numbers and booleans, so the lock is committed: one file per oracle,
`scenarios/locks/lsnes.toml`. Its size: under 500 marks per stream over the
whole corpus, a few hundred KB of text with the schedules. ROM-derived bytes
(native seeds, their records, per-frame pads, the new `file-copy-clear` inputs)
go in one new encrypted ci-data bundle that the file pins by SHA-256.
`scenarios/lock.toml` stays as it is for the old verifier until `sm/` is deleted.

**Key.** (row, oracle, oracle version). The version names everything that
decides the record's bytes: the emulator build, the SHA-256 of
`tools/oracle/lsnes_server.lua`, and the record format. Typed streams also
carry the decoder version (`verify::decode`'s rules), because they are the
oracle's State as decoded (D21).

**Format** (TOML, no JSON):

```toml
format = 2
mark_frames = 1000
oracle = { emulator = "lsnes", version = "rr2-beta23 bsnes-v085", server = "<sha256>", fixes = [] }
decoder = "<verify decoder version>"
bundles = [{ name = "relock-v2-lsnes-<sha>.tar.gz.enc", sha256 = "<sha256>" }]

[[row]]
id = "area-brinstar"
start = "100-input-phase:17"          # a native seed by capture name; "power-on" for one row
seed_record = "<sha256 of the seed's .record>"
pads = "<sha256 of the per-frame pad file>"
frames = 263
ticks = 250
overruns = ["118-121", "200"]         # row frames whose NMI interrupted a non-wait site
nmi_off = ["5-9"]                     # frames cut at vblank without an NMI
rungs = [5000, 10000]                 # native states in the bundle, long rows only
facts = { "room:9E9F" = true, "area:brinstar" = true, "probe:transitions.phase.2" = "unevaluated" }
streams.vram = ["<span digest>", "..."]   # one digest per 1000-frame span, the last one partial
streams.video = ["..."]
```

**Marks.** Each mark is the digest of its 1000-frame span, folded from empty;
the row's digest is the fold of its marks. The gate computes the same on the
engine. A mismatch names its span, a fix that heals span 3 shows spans 4 and
later as green again (progress inside a row, D22), and a long row can be
recorded as parallel spans from its row rungs (Q1).

**Streams** (definitions D20, D21, D23; folded per D21):

| Stream | Oracle side (lsnes record) | Folded |
| --- | --- | --- |
| `vram`, `cgram`, `oam`, `sram` | raw memories at the cut, as `streams::v1` hashes them today | at quiescent cuts |
| `ppu` | `PpuRegisters` without `nmitimen`, `htime`, `vtime`, `hdmaen`, `memsel`, which lsnes doesn't expose ((o)); the five are named as not compared (§11.6, `LEGACY(oracle)`) | at quiescent cuts |
| `apu` | the CPU's `$2140-$2143` reads and writes in order, `(kind, port, value)`; lsnes's bus hook sees no DMA side, so the engine's projection drops DMA entries too | concatenated over each busy span, folded at its closing quiescent cut |
| `video` | the picture as shown (HARNESS "The picture compared", `Picture::shown`): each dot's 8-bit colour at its line's brightness, black on forced blank, 224 or 239 lines | every frame |
| `timing` | per frame: the cut kind, the NMI kind and the overrun flag | every frame |
| `input`, `samus`, `camera`, `room`, `progress`, `actors`, `effects`, `rng` | the projection of `verify::decode` of the record; absent while the decoder can't decode the row | at quiescent cuts |
| `wram` | raw, unmasked; for oracle determinism and the nightly unowned-RAM report, not the gate | at quiescent cuts |

A cut is quiescent when it is an NMI return whose NMI interrupted a wait site
(`game::WAIT_SITES`, kind `Requested`). The overrun and NMI-off schedules are the
complement; the `Recorded` timing source replays them, so both sides fold at
the same cuts by construction. Partial comparison at busy cuts (D21) waits for
the cost model's routine domains: a later format bump. `audio` (tier c) is not
recorded: lsnes doesn't give it.

**Facts.** Evaluated on the decoded oracle State at record time. The 156
`probe:` facts are old-engine instrumentation and can't be evaluated on an
oracle; they are written `"unevaluated"` until each is rewritten as a typed
predicate (§11.3). Rows keep their room, state, area, item, boss and door facts
(784 of the 940 facts in the scenario files).

**What changes from today's lock.**

- Recorded by lsnes on the unmodified ROM, not the patched snesrev stepper.
- One record per video frame, not per Tick; pads per video frame, not a word per
  Tick (the per-frame pads come from the Tick words once, step b).
- A mark per stream, not one `combined` mark; spans, not one running chain.
- `video` as shown, not snesrev RGB; `apu` the CPU port log, not the native
  player's queue; `ppu` architectural, not renderer scratch; `wram` unmasked
  and out of the gate (no `LOCK_WRAM_MASK`).
- New: `timing`, the overrun and NMI-off schedules, facts' results, typed
  streams, the oracle version and the decoder version.
- Seeds named by capture and pinned by their record's SHA-256, not by a
  migration capture's observation hash.

**What carries over** (ARCHITECTURE §4.1 names the lock reader in the closed
list): the small TOML line reader of `crates/oracle/src/lock.rs`, the bundle and
pin model (ciphertext SHA-256 per bundle, file SHA-256 per pin), the corpus
runner's row loop and stall limit (`corpus.rs`), and the fold of
`sm_data::digest`, moved into `data` so `verify` uses it without an `sm_*`
crate. Retyped as the AGENTS.md style asks: a `Stream` enum, `RowId` and
`Frame` newtypes, a `Digest` newtype, `TryFrom<&str>` at the edge.

## 3. Steps

Each is one lane of ≤ 30 min or one root background job. Order and dependencies:
0 → (a, b) in parallel → c → d → e → f; g after f.

| Step | Who | What | Depends | Proves |
| --- | --- | --- | --- | --- |
| 0 | lane | The APU port callbacks register before the server's `load-state`, so a resumed lsnes logs the port accesses ((r): empty after resume; 637 of 638 rows resume). | — | `cross --frames 20000..21000 --streams --right-from 020000.lss 20000`: lsnes logs a nonzero access count close to LakeSnes's 404, as from power-on (845,053 against 845,048) |
| a1 | lane | A cut plan: per movie window, the CSV of captures (as `cut_seeds.sh`'s `filter`), the derive list, and `tools/oracle/cut_windows.sh` (one window per call: `movie --from RUNG.lss F --native-seeds ...`). | — | the plan lists 505 captures in 38 windows + 17 derivations |
| a2 | root job | Run the 38 windows, 8 at a time, and the 17 `derive-seed`s. | a1 | each seed's record equals its lsnes room capture: same room, 0 WRAM bytes differing (as `movie --seeds` checked for seeds 1-16); two windows re-cut give byte-equal records |
| b1 | lane | `corpus --frame-pads DIR`: run each row under today's Tick rule (pad word k+1 until the k-th requested NMI) and write the pad of every video frame and the frame count; then corpus reads per-frame pads and previews the next pad (`Backend::preview`), so every cut is the RTI ((i)). | 0 | on 3 rows, a per-frame-pad rerun with preview reproduces the Tick run's memories at every cut, and reports 0 `NmiStillRunning` cuts |
| b2 | root job | Run `--frame-pads` over all rows, no memories, about 12 jobs. | a2, b1 | 638 pad files; frame counts per row for sharding |
| c | lane (maybe two) | `corpus --record` writes format 2: the stream encoders over `FrameRecord` (shown picture, CPU port log, the trimmed `ppu`, quiescent folding, span marks, `timing`, the schedules, row rungs); typed streams and facts through a `tools relock` binary that drives `oracle::corpus::run_row_with` and folds `verify` projections in the same pass. `--shard I/N` packs by frame count. | b1 | a 2-row lock, recorded twice, byte-identical; a mutated pad changes the expected spans only |
| d | root job | Record every row: 5 long rows first, then about 80 jobs of ≤ 6000 frames, 12 at a time (about 35 min wall). Then a second wave re-records 64 rows (all long ones included) in a different shard layout. | a2, b2, c | 638 entries; the 64 re-recorded entries byte-identical; every non-probe fact holds on the oracle, else the row is listed and left unlocked |
| e | root | Bundle the seeds, records, rungs, pad files and `file-copy-clear`'s new inputs (`c9084e8f...`, 2230 Ticks), encrypt as today (`openssl enc -aes-256-cbc -pbkdf2 -iter 200000 -md sha256`, passphrase sha256("sm-decomp-seeds-v1" + ROM)), upload to `ci-data`, commit `scenarios/locks/lsnes.toml` with the bundle's SHA-256. | d | the download decrypts and every pin's SHA-256 matches |
| f | lane | `verify::lock`: the carried-over reader, retyped; the comparison of an engine run's spans with the lock per (row, stream); green (row, stream) pairs reported. `verify` reads the lock, the pads and the seed record from the bundle; the engine starts from the seed's record through the decoder. | e | a lock mutant per stream (one flipped span) fails exactly that pair; the reader rejects a lock of another oracle version |
| g | lane | Wire it into the gate: `tools verify` free-runs slice 0's rows (`area-brinstar`, `item-morph-ball`) with `Recorded` timing against the lock and ratchets the green pairs. | f | the ratchet file exists; a decoder mutant breaks a row |

Background afterwards: the nightly full-movie runs (D26), and re-recording the
typed streams when the decoder version changes (§4 Q3).

## 4. Open questions (engineering; answered)

- **Q1. Running chain or span marks?** Span marks (each span folded from
  empty). Same power to detect, plus localisation, per-span progress and
  parallel recording of long rows. D22's "running digest with a mark every
  1,000 frames" reads onto it: the row digest is the fold of the marks.
- **Q2. Per-Tick words or per-frame pads?** Per-frame, converted once (b).
  D20 makes pads per video frame; and only with the next pad known can lsnes
  cut at the RTI when an NMI outlasts its frame ((i)), which corpus can't do
  today.
- **Q3. Typed streams before the decoder covers a row?** Recorded as absent and
  keyed by decoder version; re-recorded (one 35-minute background job) when
  a cluster's decode rules land. Storing every quiescent WRAM image to
  re-derive without the oracle would be about 1 GB of bundle; not worth it yet.
- **Q4. The five `ppu` fields lsnes lacks?** Not compared, named in §11.6 and
  LEGACY. Taking them from LakeSnes on frames where memories agree would make
  one stream depend on two oracles, and LakeSnes desyncs after any% 35199.
  The `timing` stream already covers NMI; the IRQ lines are measured (D11).
- **Q5. `boss-golden-torizo`'s seed?** Its capture comes from a snes9x movie we
  can't replay on lsnes. Rebase the row on the 100% movie's own visit to room
  B283 (`100-input-phase:132`, frame 93568) with that movie's pads; it is
  accepted only if its boss facts hold on the oracle, else it waits for a D26
  trick movie.
- **Q6. Determinism proof cost?** A 10% re-record (64 rows, every long row) in
  a different shard layout, rather than doubling the 3.5 core-hours.
- **Q7. Seeds without tier b?** lsnes gives no SPC700 state. The engine's
  driver model works at the port level (D18) and starts from the decoded WRAM
  sound state; a row that turns out to need SPC RAM takes tier b from LakeSnes
  at the same cut, only where its memories equal lsnes's.
- **Q8. Jobs over 5 minutes?** Three: the five long rows (5-8 min) and the
  Spore cut (about 8 min), once. The row rungs make later re-locks of the long
  rows parallel; a Spore ladder is cut by the same run.

**Questions for Jim:** none. Every choice above keeps the goal and decisions as
given; Q5 changes one row's start, which its facts must still prove.
