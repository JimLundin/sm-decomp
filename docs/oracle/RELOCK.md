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

**Step a1: done** (lane `lane/relock-cut`). `tools/oracle/relock_plan.sh NATIVE
LADDERS > PLAN` writes the plan: 504 `cut` lines (325 100%, 176 any%, 2 low%, 1
Spore) in 38 windows (23 + 12 + 2 ladder windows, Spore from power-on), 17
`derive` lines and `skip golden-input-phase 1` (Q5). `tools/oracle/relock_cut.sh
PLAN OUT LOGS [cut|realign|derive|check|all] [MOVIE-RUNG...]` runs it: one
`movie --from RUNG --native-seeds ... --no-memories` job a window, longest
first, `JOBS` (8) at a time `STAGGER` (4) s apart, a log a job. A seed's record
now carries the memories also under `--no-memories`, for the check. `realign`:
a capture usually follows step F-1 of its movie, but 2 of the 39 proved
follow step F (the record's frame counter `$05B6` is one short of the CSV's
`nmi`), so those are re-cut one step on from the seed. The check: each seed's
room and frame counter equal its CSV row's, and its WRAM equals its lsnes
capture's in the old bundles outside the stack page. Proved on 100% windows
90000 and 200000 and any% window 110000 (39 seeds, 47-52 frames/s with 3 at
once, 114-169 s a window): 39 of 39 equal after `realign`; derived
`coverage-metroid` and `-metroid-ice` keep their words, `lower-norfair-hand`
fails `derive-seed`'s one-step check (Samus's X `$0AF6` written `0x68` reads
`0x67`: she moves in that step).

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
  trick movie. Measured (step a1): they don't hold. At 93568 enemy slot 0
  (`$0F78`) holds the same enemy with the same health (`$0F8C` = 13500) as the
  golden capture; the movie takes one hit off it (13350 by 94400) and leaves
  for B1E5 by 95000 with Golden Torizo alive. Its defeat flag (`$D82A` bit 2,
  the `boss:golden-torizo` fact) stays clear through frame 209664.
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

## 5. Step b1 done; how to run b2

Lane `lane/relock-pads`. `oracle corpus ... --frame-pads DIR` runs each row
under the Tick rule and writes `DIR/<row>.pads`; `--from-frame-pads DIR` plays
the rows from those files, previewing every next pad. `--no-memories` drops
the memories from lsnes's records (pads only, no comparison).

**Format.** Text, the `.inputs` format (one hexadecimal JOY1 word per line,
LF), read and written by `oracle::inputs`: one line per video frame, then one
line for the frame after the row, which the last frame's NMI reads when it
outlasts the frame. So a row of N frames has N + 1 lines. Text rather than
binary because it is the format the corpus's inputs already have (one reader,
`diff`-able), and the largest row is about 16,000 lines, 80 KB. A run with
memories also writes `<row>.cuts` (each frame's cut kind, `R`/`O`/`V`/`S`, and a
hash of WRAM, VRAM, CGRAM and OAM) and `<row>.end` (the last frame's WRAM), for
`--from-frame-pads` to compare against. All three are ROM-derived and private.

**Why the two runs emulate the same.** Each step is one lsnes frame's input in
both runs, and frame k + 1 takes pad k in both: without a preview the client
sends it with the next step; with one, the server commits the previewed pad.
They differ only in where they cut. A Tick-run step cut at the boundary with
its NMI running (`S`) is followed by a step cut at that NMI's late RTI, which
then misses the RTI of the NMI that began in its own frame. The run from pads
cuts every frame at its RTI, so its cuts are the Tick run's cuts without the
`S` ones, plus those missed RTIs (all overrun NMIs).

**Proof** (`$TEMP/ohx/relock/pads`, 2026-11-06, 3 rows in parallel, native seeds):

| Row | Ticks | Frames | Tick run: `S` cuts | From pads: `S` cuts | Tick run's RTI cuts found in order, same memories | Tick run's last cut is frame |
| --- | --- | --- | --- | --- | --- | --- |
| `chain-any-0001-DF45` (door) | 412 | 532 | 27 | 0 | 505 of 505 | 530 of 532 |
| `death` | 1800 | 2137 | 27 | 0 | 2110 of 2110 | 2135 of 2137 |
| `chain-any-0006-E0B5` | 1777 | 1778 | 1 | 0 | 1777 of 1777 | 1776 of 1778 |

The end state is the same: the Tick run's last cut is among the cuts from
pads with equal memories (frame N - 2). The run from pads cuts once more
after it, at the RTI of the last frame's NMI (the row's seed resumes inside
an NMI, whose `S` cut shifts the Tick run's cuts by one), so the two runs'
last records differ by 107, 264 and 134 WRAM bytes: that frame's NMI. Pads
from `--no-memories` are byte-identical to those from the run with memories.

**Speed.** With memories, 3 processes at once: 32-36 frames/s each, both
modes. `--no-memories`: 61-65 frames/s for one process. So b2 (about 445,000
frames) is about 2 core-hours: 12 jobs of about 10 minutes, the long rows
(credits-ending 15,967 frames, about 4 minutes) inside them.

**b2** (root job, after a2's native seeds). `$TEMP/ohx/relock/oh` runs the
lane's DLL (`OH_DLL=oracle_pads3`, built by `$TEMP/ohx/relock/build-pads.sh NAME`):

```bash
OHX=$(cygpath -m "$TEMP/ohx"); ROM=C:/Users/Jim.Lundin/Documents/Projects/sm-decomp/rom/sm.smc
for i in $(seq 0 11); do
  OH_DLL=oracle_pads3 "$OHX/relock/oh" corpus "$ROM" scenarios/lock.toml "$OHX/data" \
    --oracle lsnes --native-seeds "$OHX/data/lsnes-native" \
    --frame-pads "$OHX/relock/pads" --no-memories --shard $i/12 \
    > "$OHX/relock/log/b2-$i.log" 2>&1 &
done; wait
```

Each row's line gives its frames (`N frames`); the frame count is also the
`.pads` line count less one. A row without a native seed reports `not run`.
Proves b2 when 638 `.pads` files exist; `--from-frame-pads` with memories on a
sample re-checks rows (without `.cuts` it prints `no <row>.cuts`).

## 6. Step c, first part done: the recorder

Lane `lane/relock-record`. `oracle corpus ... --oracle lsnes --native-seeds DIR
--from-frame-pads DIR --record-v2 FILE` plays each selected row from its frame
pads with the picture and APU port tiers on and writes the file header and one
format-2 `[[row]]` per row (`oracle::relock`). `--record-v2` refuses any other
backend or pad source. It prints each row's digest per stream (the fold of its
marks) on stdout.

**What it writes.** The header: `format = 2`, `mark_frames = 1000` and
`oracle = { emulator, version, server, fixes }`, where `server` is the SHA-256
of the `lsnes_server.lua` compiled into the build (`lsnes::server_sha256`).
No `decoder` or `bundles` yet (no typed stream; step e). Per row:

```toml
[[row]]
id = "chain-any-0001-DF45"
start = "any-boundary:1"
seed_record = "<sha256 of the native seed's .lss>"
pads = "<sha256 of the .pads file>"
frames = 532
ticks = 414
overruns = ["254", "256-257", "...", "408-409"]
nmi_off = []
facts = { "room:DF8D" = "unevaluated" }
streams.vram = ["<span 0>"]
streams.cgram = ["..."]
# oam, sram, ppu, apu, video, timing, wram the same
```

`seed_words` (the SHA-256 of the seed's `.words`) follows `seed_record` for a
derived seed; `still_running` (frames cut with their NMI running) appears only
if nonzero, which a run from pads never gives; `absent = [...]` names a stream
some frame lacked. `ticks` is the run's requested NMIs, not the old lock's
Ticks (414 against 412 on `chain-any-0001-DF45`: the cuts from pads add the
RTIs the Tick run missed, §5).

**Definitions as built.** Quiescent: the cut is `Cut::NmiReturn` with
`NmiKind::Requested`, the NMI having interrupted one of `game::WAIT_SITES`'
loop instructions (`relock::is_quiescent`). `vram`, `cgram`, `oam`, `sram`,
`wram` and `ppu` fold at quiescent cuts, hashed as `streams::v1` does; `ppu`
hashes every `PpuRegisters` field but the five, in declaration order. `apu`
concatenates `(kind, port, value)` of the CPU's reads and writes (DMA and SPC
entries dropped) and folds the bytes since the last quiescent cut at each
quiescent cut; accesses after the row's last quiescent cut are not folded.
`video` folds `Picture::shown()`'s bytes every frame; `timing` folds
`[cut, nmi, overrun]` every frame (cut 0 RTI, 1 vblank without NMI, 2 still
running; NMI 0 none, 1 requested, 2 overrun). Each span starts from the empty
digest; the last, partial span is closed at the row's end.

**The digest moved** to `data::digest` (`Digest`, `fold`, the new `fold_all`,
bit for bit with `sm_data::digest`, which stays for the old verifier).
`oracle` now depends on `data` and `oracle_record` only.

**Proof** (`$TEMP/ohx/relock-record/proof.sh` and `compare.sh`, 2026-11-06,
5 processes at once beside another lane's seed cuts), the three rows of §5
recorded twice (`a`, `b`) and as `--shard 0/3`, `1/3`, `2/3`:

- Every stream but `video` is byte-identical in all five runs, on all three
  rows, every span. The memory streams' row digests also equal those of a run
  without the stream tiers, so asking for the tiers doesn't change the emulation.
- **`video` is not deterministic.** `chain-any-0001-DF45` (1 span) agreed in
  all runs. `chain-any-0006-E0B5`'s span 0 took two values over the five runs
  (span 1 equal); `death`'s spans 0 and 1 took two values (span 2 equal). So
  the whole files `a` and `b` differ, on `chain-any-0006-E0B5`'s `video` only.
  Not localised yet: the next step is to write each frame's video digest
  beside the entry and diff two runs, to see whether it is a few frames (a
  screenshot taken before lsnes has finished the frame, or after a resume)
  or the whole span. Until it is fixed, `video` can't be locked.
- **Not reproduced since** (lane `relock-video`, 2026-11-06, a temporary
  per-frame log of each frame's video digest, cut and video frame counter,
  and a dump of each frame's PNG). In each of the two odd results one run
  disagreed with the other four: `a`'s `chain-any-0006-E0B5` (`f326332c...`
  against `25d56fe9...` in `b` and the shard) and the shard's `death`
  (`5949bdfc...`, `c7cae79e...` against `2b496870...`, `39c63c69...`). Five
  reruns, 2 or 5 processes at once beside the background seed cuts
  (`death` alone twice with the PNG dump; the three rows with the dump; the
  three rows with the log only; the three rows on the proof's own DLL), recorded 22 runs of
  `death` and 15 of each chain row. All gave the majority values, with the
  per-frame logs equal frame for frame: 2,137 frames of `death`, 1,778 of
  `chain-any-0006-E0B5`, 532 of `chain-any-0001-DF45`. So the miss is rare
  (2 of 15 row runs in the proof, 0 of 52 since) and the cause is still
  open. Known about the odd runs: `death`'s odd spans are 0 and 1 (frames
  0-1999) and span 2 is equal, so either one event changed more than a
  single picture or two events happened; none of the three rows has a cut
  with the NMI running, so every record comes from the RTI or the boundary.
- Speed: 11-14 frames/s per process with 5 at once and the tiers on (the
  picture and the APU log; not split between them); 31-35 frames/s without
  the tiers, the same load.

**What remains of step c.**

0. `video` is still rarely nondeterministic and the cause is open. The
   recording now checks itself (§7): every row is recorded at least twice,
   and only entries that two runs agree on are locked.
1. Facts: every fact is written `"unevaluated"`; the room, state, area, item,
   boss and door facts need evaluating on the decoded State (`tools relock`,
   with `verify::decode`).
2. Typed streams: the hook is `relock::TypedStream` and
   `Recorder::with_typed`; nothing implements it, and the header has no
   `decoder` key until something does.
3. Row rungs for the long rows (a native state every 5000 frames) are not
   saved; no `rungs` key.
4. `--shard I/N` is still round-robin by row position, not packed by frame
   count.
5. The step's mutant proof: a mutated pad changes only the spans from its
   frame on. Not run.
6. A reader for format 2 (step f retypes `lock.rs`); entries are only written.

## 7. Recordings check themselves

Lane `lane/relock-verify`. lsnes's `video` stream rarely differs between two
runs of a row (2 of 67 row runs, never on demand; §6), so a lock entry is
kept only where two recordings agree byte for byte.

**The sidecar.** `--record-v2 FILE --sidecar DIR` also writes `DIR/<row>.frames`,
text, one line per video frame after a head line naming the columns:

```text
# frame lsnes_frame cut vram cgram oam sram ppu apu video timing wram
0 8539 R c2285235399f697b f7d7cb48d9441224 ... 2493486742244426
```

The frame index in the row, lsnes's frame counter, the cut kind (`R`, `O`,
`V`, `S`, as `.cuts`) and the digest each stream folded at that frame (`-`
where a quiescent stream didn't fold, `absent` where the record lacked it).
About 140 bytes a frame: 75 KB for `chain-any-0001-DF45`, about 62 MB for the
corpus. ROM-derived; it stays in `$TEMP`. `--dump-frames ROW:F,...` (with
`--sidecar`, repeatable) writes those frames' shown pictures, the dots
`video` digests, as `DIR/<row>.<F>.png`, for a frame a sidecar named.

**Compare and merge** are `oracle` subcommands (`oracle::relock_check`), so
they run through the same DLL runner as the recording: this machine can't run
the `tools` executable. A recording is one file or its shards' files joined by
commas, whose headers must be equal.

- `relock-compare A B [--sidecars DA DB]` prints each row `agree` or
  `DISAGREE`, with the differing non-stream lines and per stream the first
  differing span; with sidecars, the two lines of the first frame whose column
  differs. `only in A`/`only in B` for a row one lacks. Exits 1 on any
  disagreement (the header included), 2 on bad arguments.
- `relock-merge OUT A B [C]` writes the header and each row's entry that at
  least two recordings share byte for byte, in the order rows first appear,
  and prints the rows with no agreeing pair; exits 1 when there are any.

**Step d's procedure.**

1. Record every row (wave A), sharded as planned, with `--sidecar side-a`.
2. Record every row again (wave B) in a different shard layout (another `N`
   in `--shard I/N`, so rows meet different neighbours and load), with
   `--sidecar side-b`.
3. `relock-compare A B --sidecars side-a side-b > compare.txt`. Exit 0: the
   lock is `relock-merge lock.toml A B`.
4. Otherwise re-record the disagreeing rows (`--only ID ...`) as wave C
   with `--sidecar side-c`, and, for the frames compare named, `--dump-frames
   ROW:F,...` to keep their pictures.
5. `relock-merge lock.toml A B C`. Rows it lists have no agreeing pair: keep
   them out of the lock and record them again; keep `compare.txt` and the
   sidecars of the odd runs as the localisation for the cause.

**Proof** (`$TEMP/ohx/relock-verify/proof.sh` and `check.sh`, 2026-11-06, 5
processes beside the background job): the three rows recorded as wave A
(`--shard 0/3`, `1/3`, `2/3`) and wave B (`0/2`, `1/2`, `--dump-frames
death:100,1500`), with sidecars.

- **The miss happened, and the sidecar localised it.** `relock-compare A B`
  (exit 1): `chain-any-0001-DF45` and `death` agree; `chain-any-0006-E0B5`
  disagrees on `video` span 1 only, at exactly one frame: frame 1071 (lsnes
  frame 11234), a cut at the RTI (`R`), video `26ea2588f4177978` in A against
  `1859db1821bb61c8` in B, every other stream's digest equal on that frame.
  So the emulation is the same and one screenshot differs. A third run (wave
  C: `--only chain-any-0006-E0B5 --dump-frames chain-any-0006-E0B5:1070,1071`)
  gave B's value, so A was the odd run; its picture of frame 1071 isn't kept
  (the dump is of the agreeing run, an ordinary frame of Samus jumping in a
  blue shaft). Next time, compare each wave with the third and dump the odd
  run's frames too.
- **An altered entry is caught.** C, a copy of B with `death`'s `video` span 1
  and its frame 1234 in the sidecar changed: `relock-compare A C` reports
  `death: video: span 1 differs` with the two lines of frame 1234, beside the
  natural miss.
- **Merge.** `relock-merge OUT A C B`: 3 entries, 0 rows without a pair
  (`death` from A and B, `chain-any-0006-E0B5` from C and B). `relock-merge
  OUT A C`: 1 entry, and `chain-any-0006-E0B5` and `death` listed (exit 1).
  `relock-merge OUT A B C3` (the third run): 3 entries, `chain-any-0006-E0B5`
  with B's and C3's span 1 (`4e82496a3475bafb`).
- The two `.png` dumps decode as the expected pictures (the Crateria map at
  `death` frame 1500).
