# The oracle

The oracle runs the original, unmodified ROM on an emulator and records what the engine is
compared against. It lives in `crates/oracle` (the emulator adapters, the recorder, the command
line) and `crates/oracle_record` (the record types, with no emulator, so `verify` can read
records without linking one). Neither depends on the engine. The comparison contract it serves
is [ARCHITECTURE §11](architecture/ARCHITECTURE.md#11-the-engine--oracle-comparison-contract).

Section labels in parentheses, such as (RELOCK §2) or (MEASUREMENTS (o)), are the names code
comments cite.

## 1. Decision and contract

### Real, beam-scheduled emulators, run unpatched

Every backend is a complete SNES: the 65816 on the master-clock beam, DMA and HDMA, NMI and IRQ
timing, auto-joypad, per-line PPU, and the SPC700 and DSP running the game's own sound driver.
With a running beam no ROM patch is needed, so `OracleId::patches` is empty. An oracle that
needed patches, or shared code with the engine, proved little in earlier attempts.

| Backend | What | How it is linked |
| --- | --- | --- |
| **lsnes** rr2-beta23 (bsnes v085 compatibility core) | the emulator the TAS movies were recorded on; any% in sync through every seed checked | a separate GPL process driven by `tools/oracle/lsnes_server.lua` (`src/lsnes.rs`); no emulator code linked or copied |
| **LakeSnes** `9db90b8` (angelo-wf, MIT) | the fast oracle, with named fixes (below) | upstream sources compiled unedited with a C bridge (`native/bridge.c`) that renames `ppu_write`, `ppu_runLine`, `ppu_handleVblank` and the DMA B-bus calls to hooks; one Rust call per video frame |
| **MesenCE** 2.2.1 (GPL-3) | a third vote on disputed frames | its released binary as a separate process, through `tools/oracle/mesen_lag_log.lua` |

### Which oracle records which stream (D25)

The gate never runs an oracle: it compares the engine with the lock's recorded digests. So
accuracy decides which oracle records a stream, and oracle speed matters only for re-locking,
lockstep and cross-oracle reports. Each stream's lock comes from the most accurate oracle that
produces it. Today that is **lsnes for every recorded stream** (memories, `ppu`, `apu`, `video`,
`timing`), since lsnes gives the picture, the CPU port log and the PPU registers too
(MEASUREMENTS (o), (r)). The five PPU fields lsnes doesn't expose are not compared (Q4).
LakeSnes stays the fast oracle for lockstep and for tier b (SPC state), and its records count only
on frames where its memories equal lsnes's. The `ReferenceProfile` names the oracle per stream;
changing one re-locks that stream oracle-side.

### The frame record (contract v1)

One record per video frame, lag or not. Record k covers (cut k-1, cut k] and the state at cut k.

- **The cut:** the RTI that ends the NMI serviced in vblank k (`Cut::NmiReturn`). With NMI off it
  is vblank start (`Cut::VblankWithoutNmi`); an NMI still running when vblank ends is
  `Cut::NmiStillRunning`.
- **NMI k:** its kind (`Requested` when the handler read the joypad, else `Lag`), its interrupted
  PC (the instruction the RTI returns to), and `overrun` when that PC is not one of
  `game::WAIT_SITES`.
- **Facts:** the pad auto-read at vblank k, joypad register reads, the overscan flag, the beam
  position where the main thread entered its wait.
- **Raw state at cut k:** CPU registers, WRAM 128 KiB, VRAM 64 KiB, CGRAM 512 B, OAM 544 B, SRAM,
  and `PpuRegisters` (INIDISP to SETINI, NMITIMEN, HTIME/VTIME, HDMAEN, MEMSEL, the eight DMA
  channels' `$43x0-$43xA`), rebuilt from decoded emulator state so they survive a state load.
- **The picture scanned out in the interval** (the one NMI k-1 committed): 224 or 239 lines of
  256 BGR555 dots after colour math, with each line's brightness and forced blank.
- **APU port log:** CPU and DMA reads and writes of `$2140-$2143`, in order.
- **Tier b:** SPC700 registers, the ports each way, SPC RAM and the 128 DSP registers.
- **Tier a:** every PPU register write with its line, dot and source (main, NMI, IRQ, DMA, HDMA),
  watched-PC hits (up to 32 addresses and 16384 hits a step; each `$420B` write is a pseudo hit
  `0x01000000 | MDMAEN`), SPC-side port writes with cycle stamps. Not digested.
- **Profile:** instructions per program bank.

Given the ROM, the start and the pads, records are bit-identical on every host.

### The picture compared

Two pictures are equal when every dot shows the same 8-bit colour (`Picture::shown`, the
`picture` part of `oracle cross`, the `video` stream):

- a dot's 5-bit components `c` at its line's master brightness `b` (0-15) show as
  `round((b + 1) * c * 255 / 496)` each (`record::shown_channel`, lsnes's palette for bsnes dots);
- a dot on a forced-blank line shows black.

At brightness 15 the map is one-to-one, so equality is BGR555 equality. Below 15 lsnes's
screenshot can't tell some components apart, nor brightness 0 from 1. Each backend keeps its
stored dots (LakeSnes exact per line; lsnes the brightest level that produces the screenshot,
the smallest component per value), so the comparison is exact wherever any screenshot could tell.
The compositor writes the same PPU-domain dots, so no emulator's colour conversion is the authority.

### lsnes streams

`Lsnes::record_streams` (the server's `T <picture> <apu> <ppu>` request, before the first step)
adds three streams; otherwise records are unchanged.

- **Picture:** lsnes's screenshot of the video frame the step ends in, taken in `on_frame` after
  lsnes hands the frame to its screen buffer, sent as its own PNG (base64) and mapped back to BGR555.
  Equal as shown to LakeSnes on 1001 of 1001 any% steps at 0-1000 and 20000-21000. Streams cost lsnes about 26%.
- **APU port log:** the CPU side only (`memory.registerwrite`/`registerread`), hooked in every bank
  that maps the I/O page, because the game reaches the ports through its data bank (`$82`, `$85`).
- **PPU registers:** rebuilt from bsnes's decoded fields in `memory.getregisters()`, without
  `nmitimen`, `htime`, `vtime`, `hdmaen` and `memsel`.

**Watched-PC hits** (`corpus --watch-pcs`, `Lsnes::open_watching`) come from the server's trace
(`OH_TRACE`, `OH_TRACE_LOG`), so the server script and its SHA-256 stay as they are: an exec
callback per address logs `pc ADDRESS LINE DOT`, and each record sent logs `cut FRAME`. The client
watches the NMI handler's RTI too and splits the log there: a hit after the RTI and before the
record is sent ran in the next frame, so it goes to the next record; a resumed run drops what ran
before its first RTI, as the server's counts do. lsnes hits carry no peek (`peek` is 0). Watch
instruction starts only: lsnes's exec callback fires on operand fetches too.

The lsnes cut is the interrupted instruction reached again with the stack restored. The server
finds the NMI handler's one `PLY PLX PLA PLD PLB RTI` from the ROM (`$80:9601`), takes the cut when
S equals the NMI's S, and derives the registers after the RTI. A frame without an NMI is cut at
lsnes's frame boundary (line 241). lsnes fixes a frame's input at its start, so an NMI that runs
past line 241 is cut at its RTI only if the client previewed the next pad (`Backend::preview`;
MEASUREMENTS (i)); without a preview it is cut at the boundary as `NmiStillRunning`. lsnes's
saved state is taken at the boundary that ends the step it was requested before.

## 2. Toolchain and layout

**LakeSnes.** `bash tools/oracle/build_lakesnes.sh [--target x86_64-windows-gnu|x86_64-linux-gnu]`
fetches the pinned upstream files (each checked against its SHA-256), applies
`crates/oracle/native/fixes/*.patch` to a copy, compiles them and the bridge with **Zig 0.15.2**,
writes `liboracle_lakesnes.a` and prints its directory; put that on `LIBRARY_PATH` for Cargo
(`#[link(kind = "static", modifiers = "-bundle")]`). Overrides: `ZIG` (default the
`sm-decomp-tools` Zig, else `zig` on `PATH`) and `SM_LAKESNES_CACHE` (default
`sm-decomp-tools/lakesnes-oracle-9db90b8`). Zig because it is the project's one C toolchain (no
MSVC or MinGW gcc on the Windows hosts); no `build.rs` because Defender blocks freshly built
executables, build scripts included. Linux adds `-fPIC -D_DEFAULT_SOURCE`.

**lsnes.** rr2-beta23, at `%LOCALAPPDATA%/sm-decomp-experiments/20261025/tas-alignment/lsnes-rr2-beta23/`
or wherever `SM_LSNES` points. Requests go to its stdin; each record goes to a file it
overwrites, announced as `R <bytes>` on stdout (text mode on Windows, so no binary there). Each
lsnes runs in its own run directory with `APPDATA=<run dir>/config`, keeps `stderr.txt`, the
server's `lua.log` (`OH_LOG`) and `lsnes.log` there, and reports them if it exits. `SM_LSNES_KEEP=1`
keeps the directory. A record not announced within 60 s (first step) or 30 s (later) kills that
lsnes and fails the row as silent. The script loads a start itself (`OH_START`, `load-state`):
lsnes rejects a state given on its command line with this ROM and hangs.

**MesenCE.** 2.2.1, commit `20ba206cef5ba207c21203176d02cb9f43dda9fb`, from
`https://github.com/nesdev-org/MesenCE/releases/download/2.2.1/Mesen_2.2.1_Windows.zip`
(zip SHA-256 `c19e10b3b3865eb0e86e4d01bf09a5027a4452081c04b5bd5b6cbbf76e346fd6`, `Mesen.exe`
`8ef403d6b9af32075416193914d7993ce8480b347fe1f20a0cfd9815374933e7`), unpacked in
`%LOCALAPPDATA%/sm-decomp-tools/mesence-2.2.1/` and run from a copy beside a `settings.json` of
`{}`, which makes it portable:

```sh
rm -rf mesence-2.2.1-run/Saves   # a kept save changes the boot from frame 268
SM_MESEN_INPUTS=movies/any.inputs SM_MESEN_LOG=out.csv SM_MESEN_FRAMES=9001 \
  mesence-2.2.1-run/Mesen.exe --testRunner --timeout=150 --doNotSaveSettings \
  --debug.scriptWindow.allowIoOsAccess=true --snes.port1.type=SnesController \
  --snes.ramPowerOnState=AllZeros --snes.spcClockSpeedAdjustment=40 \
  tools/oracle/mesen_lag_log.lua rom/sm.smc
```

It writes the column layout of `tools/lsnes_lag_log.lua` (about 120 frames/s).

**Crate layout.** `crates/oracle/src`: `lib.rs` (the `Oracle` trait, `Start`, `Tiers`, `OracleId`),
`lakesnes.rs`, `lsnes.rs`, `streams.rs` (per-stream digests), `lock.rs` (the old lock's pins and
entries), `seed.rs` (lsnes room captures), `corpus.rs` (running rows), `relock.rs` and
`relock_check.rs` (format 2), `ladder.rs`, `measure.rs`, `cross.rs`, `cli.rs`, `bin/oracle.rs`.
`crates/oracle_record`: `FrameRecord`, its codec, the lsnes raw record decoder, `game::WAIT_SITES`
and `inputs.rs`. No crates.io dependency. A backend implements `Oracle` (open a start, step one
video frame with a pad, save its state, set tiers and watched PCs); a second backend is a second
implementation, and `oracle cross` steps two with the same pads.

## 3. Inputs, starts, ladders and movies

**Inputs.** The `.inputs` format: one hexadecimal JOY1 word per line, LF, one line per video
frame (`oracle_record::inputs`). A row's **frame-pad file** (`<row>.pads`, RELOCK b1) has one
line per frame of the row plus one for the frame after, which the last frame's NMI reads when
it outlasts the frame: N + 1 lines.

**Starts.**
- `Start::PowerOn { wram_fill }`: the movies were recorded on bsnes v085, which fills 0x55.
- `Start::State(blob)`: the emulator's own state. **Native seeds** are lsnes states cut from
  the movies, named `CAPTURE:INDEX` and stored as `DIR/CAPTURE/NNNN.lss` (the state before step
  N) and `NNNN.record` (the raw record of step N-1, always with the memories). A **derived seed**
  adds `NNNN.words`: word edits on a base state, no emulation (`oracle derive-seed`).
- `Start::Capture`: an lsnes room capture (WRAM, VRAM, CGRAM, OAM, SRAM, SPC RAM and a register
  CSV), for migration only (`LEGACY(seed)`, [LEGACY](LEGACY.md#the-oracle)). It resumes at
  `seed_boundary_pc` (`$80:8348`), not `pc`, with the SPC700 at the driver's idle loop. Some rows
  then stall on sound; native seeds don't.

**Ladders.** Long emulator runs are the exception. Each movie keeps lsnes states every 10000
frames (`crates/oracle/src/ladder.rs`), so a cut or a measurement resumes from the nearest rung.
- `oracle movie ROM movies/NAME.inputs --oracle lsnes --ladder N DIR` writes `DIR/NAME/FFFFFF.lss`,
  `FFFFFF.record` and `ladder.txt` (each rung's frame and its record's SHA-256).
- `oracle ladder-check ROM INPUTS DIR [--rungs A..B]` resumes each rung, runs to the next and prints
  `equal` or the differing parts; exit 1 on any difference. Reproduces means the record before the
  next rung is byte-identical; `.lss` bytes are not compared (equal machines can save different bytes).
- `oracle ladder-rung INPUTS LADDERS FRAME` prints `RUNG PATH` for `movie --from PATH RUNG`;
  `movie ... --native-seeds CSV DIR --from-ladder LADDERS` starts at the latest rung before the
  earliest seed. Rungs: 100% 22, low% 16, any% 12; every `ladder-check` equal.
- The nightly check runs each movie whole with `--ladder` into a fresh directory, diffs
  `ladder.txt`, and `ladder-check`s the stored ladder.

**Movie sync.** `oracle movie ROM movies/any.inputs --lag-log CSV` compares each frame with the
lsnes lag log (`any-boundary/lag.csv`): `$05B6`, `$05B8`, the lag flag, the game state, the room
and Samus's position. lsnes is in sync by definition (all room seeds checked: same room, 0 WRAM
bytes differing). LakeSnes:

| LakeSnes | Exact through | State, room and position through | First difference |
| --- | --- | --- | --- |
| `9db90b8` | 352 | 352 | boot APU upload ends 12 lines early |
| + 0001 (APU clock) | 395 | 8566 | Ceres desync from 8567 |
| + 0002 (bsnes v085 SPC rate) | 395 | 17984 | one more lag frame on the door out of room 97B5 |
| + 0004 (HDMA overhead) | 395 | **35199** | one frame late out of the door into room 9CB3 (35200); desync from 35202 |

**The any% status: LakeSnes holds through step 35199.** The cause of 35200 is at 35175: LakeSnes
enters its wait at 224/1260, lsnes at 224/1230, so the line-225 NMI lands before
`STA $05B4` (`$80:8340`) and counts a lag frame. The 30 clocks come from HDMA-pause rounding on
HUD lines 0-32 (MEASUREMENTS (m)). No variant tried zeroes them, so there is no fix 0005 ((n), (q)).

The fixes are emulator fixes, not ROM patches, named in `OracleId::fixes`:
- `0001-apu-clock-real-master-rate`: the SPC at 32040 Hz against the real 21 477 272 Hz master clock.
- `0002-apu-clock-bsnes-v085-exact`: bsnes v085's 32040.5 Hz as an exact integer ratio ((h)).
- `0004-hdma-overhead-only-while-a-channel-runs`: no HDMA alignment or overhead on a line where
  every enabled channel has terminated, as anomie's timing doc and bsnes v085 have it ((k), (l)).
- Tried, not applied: `0003-apu-ports-in-time-order` changes no lag-log word in any% 0-18999
  ((j)); the patch is in git history at `8cbdfb119`, `crates/oracle/native/candidates/`.

**Movies.** `movies/manifest.toml` lists the published TAS movies (any% 3653M, low% 3273M,
100% 4010M and others), each with its download link, SHA-256, frame count, emulator (lsnes
rr2-beta23, bsnes v085 compatibility core) and ROM SHA-256. Download the ZIP, extract the
`.lsmv`, check its SHA-256, then import one controller word per video frame:

```sh
python3 tools/movie_inputs.py movies/super-metroid-any-3653.lsmv movies/any.inputs --rom rom/sm.smc
```

`movies/room-seeds-*.csv` list the room captures each movie's seeds come from (read by
`relock_plan.sh`, `relock_cut.sh` and `oracle --seeds`). Movies, inputs, states and logs are
ROM-derived and stay local (`.gitignore`) or in encrypted bundles. The Mother Brain phase rows
start from the `100-mb-phases` captures (her form word `$7E:7800` changing), one per form.

## 4. The format-2 lock (RELOCK §2)

`scenarios/locks/lsnes.toml`, one file per oracle, committed: digests, schedules and fact
results are hashes, frame numbers and booleans. ROM-derived bytes (seeds, records, pads) go in an
encrypted bundle the file pins.

**Key.** (row, oracle, oracle version). The version names everything that decides the record's
bytes: the emulator build, the SHA-256 of `lsnes_server.lua` compiled into the build
(`lsnes::server_sha256`), the record format, and for typed streams the decoder version.

```toml
format = 2
mark_frames = 1000
oracle = { emulator = "lsnes-bsnes", version = "rr2-beta23 (bsnes v085)", server = "<sha256>", fixes = [] }
bundles = [{ name = "relock-v2-lsnes-<id>.tar.gz.enc", sha256 = "<ciphertext sha256>" }]

[[row]]
id = "area-brinstar"
start = "100-input-phase:17"   # a native seed; "power-on" for one row
seed_record = "<sha256 of the seed's .lss>"
pads = "<sha256 of the .pads file>"
frames = 477
ticks = 477                    # requested NMIs of the run
overruns = []                  # frames whose NMI interrupted a non-wait site
nmi_off = []                   # frames cut at vblank without an NMI
facts = {  }                   # "room:9E9F" = true, or "unevaluated"
streams.vram = ["<span digest>"]   # one per 1000-frame span, the last partial
```

Optional keys: `seed_words` (a derived seed), `still_running` (never from pads), `absent = [...]`
(a stream some frame lacked), `rungs`, `decoder`.

**Marks (Q1).** Each mark is the digest of its 1000-frame span, folded from empty; the row's
digest is the fold of its marks (`data::digest::fold_all`). A mismatch names its span, a fix that
heals span 3 shows spans 4 on as green again, and long rows can be recorded as parallel spans.

### Streams (RELOCK §2 "Streams")

| Stream | Oracle side (lsnes record) | Folded |
| --- | --- | --- |
| `vram`, `cgram`, `oam`, `sram` | raw memories at the cut | at quiescent cuts |
| `ppu` | every `PpuRegisters` field in declaration order but the five lsnes lacks (Q4) | at quiescent cuts |
| `apu` | the CPU's `$2140-$2143` reads and writes, `(kind, port, value)`; DMA and SPC entries dropped on both sides | bytes since the last quiescent cut, folded at each quiescent cut; accesses after the row's last are not folded |
| `video` | `Picture::shown()`'s bytes | every frame |
| `timing` | `[cut, nmi, overrun]`: cut 0 RTI, 1 vblank without NMI, 2 still running; NMI 0 none, 1 requested, 2 overrun | every frame |
| `input`, `samus`, `camera`, `room`, `progress`, `actors`, `effects`, `rng` | the projection of `verify::decode`; absent until a decoder records it | at quiescent cuts |
| `wram` | raw, unmasked; determinism and the unowned-RAM report, never the gate | at quiescent cuts |

A cut is **quiescent** when it is `Cut::NmiReturn` with `NmiKind::Requested`, the NMI having
interrupted a wait site (`relock::is_quiescent`). The overrun and NMI-off schedules are the
complement, and the `Recorded` timing source replays them, so both sides fold at the same cuts.
The fold rules are one type, `data::span::SpanFold`, used by `oracle::relock::Recorder` on
`FrameRecord`s and by `verify::fold::EngineFold` on the engine's frames. `audio` is not recorded.

**Facts** are evaluated on the decoded oracle State at record time. Until `tools relock` evaluates
them with `verify::decode`, every fact is `"unevaluated"` and doesn't block green.

## 5. The re-lock (RELOCK §3)

| Step | What | Status |
| --- | --- | --- |
| a1 | the seed-cut plan: `relock_plan.sh NATIVE LADDERS > PLAN`, 504 `cut` lines in 38 windows, 17 `derive` lines, each derive base cut too | done |
| a2 | run the windows (`relock_cut.sh`), 8 at a time | done: all 532 cuts equal their lsnes captures (0 WRAM bytes differing) |
| b1 | `corpus --frame-pads DIR`, then play from pads with preview | done |
| b2 | frame pads for every row | done |
| c | `corpus --record-v2`, the format-2 recorder | done, but facts, typed streams and row rungs (below) |
| d | record every row twice, compare, merge | done: 635 rows in `lsnes.toml`, 642 with the seven movie rows of `scenes-v1` |
| e | bundle, encrypt, upload, pin in `lsnes.toml` | done (`relock-v2-lsnes-6ead7227957a4a10`) |
| f | `verify::lock`: the reader and the per-(row, stream) comparison | done |
| g | the free-run gate: `tools gate`, ratcheting green pairs | done for `area-brinstar`, `item-morph-ball` |

### Step a1: the cut plan

`tools/oracle/relock_cut.sh PLAN OUT LOGS [cut|realign|derive|check|all] [MOVIE-RUNG...]` runs one
`movie --from RUNG --native-seeds ... --no-memories --ppu` job per window, longest first, `JOBS`
(8) at a time, `STAGGER` (4) s apart. `realign`: most captures follow step F-1 of their movie but
some follow step F (the record's `$05B6` one short of the CSV's `nmi`); those are re-cut a step on.
`check`: each seed's room and frame counter equal its CSV row's, and its WRAM equals its capture's
outside the stack page. `derive-seed` checks that each written word stuck: one step on it reads the
written value, or something other than the plain base's step (physics moves Samus's position in that step).

### Step b1: frame pads

`oracle corpus ... --frame-pads DIR` runs each row under the old Tick rule (present pad word k+1
until the k-th requested NMI) and writes `DIR/<row>.pads`; with memories also `<row>.cuts` (cut kind
`R`/`O`/`V`/`S` and a memory hash per frame) and `<row>.end`. `--from-frame-pads DIR` plays the pads,
previewing every next pad, so no cut is `NmiStillRunning`. Its cuts are the Tick run's without the
`S` ones plus the RTIs the Tick run missed; memories equal at every shared cut.

**The 100% chain rows' pads sit one frame early against the movie.** Their `.pads` came from the
Tick run, not from the movie: of the 327 100% chain rows, 0 play the movie's own pads from the
seed (most differ from the first pad), and `chain-0073-A59F` dies to Kraid. A row's pads are the
movie's from the seed record's lsnes frame counter (record bytes 20-23), where the movie's pad for
the seed's first step is: line `first` to `first + frames` of the `.inputs`, one more than the
row's frames. The seven movie rows of `scenes-v1` (`event-zebes-awake`, `item-energy-tank`,
`item-reserve-tank`, `item-charge-beam`, `item-wave-beam`, `item-plasma-beam`,
`boss-kraid-defeated`) are cut that way and play the movie through the progress report each was
chosen for. The chain rows are re-cut and re-locked the same way (§9).

### Step c: the recorder (RELOCK §6)

`oracle corpus ROM LOCK DATA --oracle lsnes --native-seeds DIR --from-frame-pads DIR --record-v2 FILE`
plays each row with the picture and APU tiers and writes the header and one `[[row]]` per row
(`oracle::relock`), printing each row's digest per stream. It refuses any other backend or pad source.
`--sidecar DIR` also writes `DIR/<row>.frames`: one line per frame with the frame, lsnes's frame
counter, the cut kind and each stream's folded digest (`-` for no fold, `absent`). `--dump-frames
ROW:F,...` writes those frames' shown pictures as PNG (`png.rs`).

What remains: (1) evaluate facts on the decoded State; (2) typed streams through
`relock::TypedStream`/`Recorder::with_typed`; (3) row rungs every 5000 frames for long rows; (4)
`--shard I/N` packed by frame count, not round-robin; (5) the pad-mutant proof.

### Recordings check themselves (RELOCK §7)

lsnes's `video` stream rarely differs between two runs (2 of 67 row runs, never on demand; one
screenshot at one frame, every other stream equal). So a lock entry is kept only where two
recordings agree byte for byte:

1. Record every row as wave A with `--sidecar side-a`, then wave B in another shard layout with `--sidecar side-b`.
2. `oracle relock-compare A B --sidecars side-a side-b`: each row `agree` or `DISAGREE`, with the
   first differing span and frame; exit 1 on any disagreement.
3. `oracle relock-merge OUT A B [C]` writes each row's entry that two recordings share and lists rows
   with no agreeing pair (exit 1). Re-record those (`--only ID`) as wave C and merge again.

A recording is one file or its shards' files joined by commas.

### Step f and g: reader, comparison and gate (RELOCK §8)

`verify::lock::Lock::parse_for(text, &oracle)` reads the strict TOML subset (`lock/text.rs`) and
rejects a lock of another oracle version. `Row::compare(&EngineRow)` gives per stream `Equal`,
`Differs { span }`, `Absent(Side)` or `NotLocked`, and per fact `Agrees`, `Disagrees`, `Unevaluated`,
`NotEvaluated`. `RowReport::green()` is ARCHITECTURE D22's green: every compared stream (all but
`wram`) the entry has is `Equal` and no fact blocks. Writer and reader share the fixture
`crates/verify/src/lock/format2.toml`, which `oracle::relock`'s test writes byte for byte.
`oracle relock-refold A SIDECAR_DIR` re-folds a recording's sidecars through `SpanFold` and checks
every mark. The gate is `tools gate` ([STATUS](STATUS.md)).

### Seed records with the PPU registers (RELOCK §9)

`oracle movie --ppu` asks lsnes for the PPU tier alone (`T 0 0 1`); the record is the lean record
followed by `OHS1`, flag 4 and the 146 PPU bytes, and the emulation is unchanged.
`tools/oracle/relock_ppu_recut.sh OLD PLAN WORK [ROW...]` re-cuts the records of a decrypted
bundle, checks each against the old, packs deterministically, encrypts and prints the new
`bundles` line. A re-cut state differs from the old only in `rrdata`, and the `seed_record` pins
are the `.lss` SHA-256, so only the `bundles` line changes.

### Engineering answers

- **Q1.** Span marks, each folded from empty, not one running chain.
- **Q2.** Pads per video frame, converted once from the Tick words (b1).
- **Q3.** Typed streams are absent until the decoder covers a row, keyed by decoder version, and
  re-recorded when a cluster's decode rules land.
- **Q4.** The five `ppu` fields lsnes lacks (`nmitimen`, `htime`, `vtime`, `hdmaen`, `memsel`) are
  not compared, named in ARCHITECTURE §11.6. Taking them from LakeSnes would make one stream depend
  on two oracles. `timing` covers NMI; the IRQ lines are measured (D11).
- **Q5.** `boss-golden-torizo`'s capture comes from a snes9x movie lsnes can't play. The 100%
  movie's visit to room B283 doesn't defeat Golden Torizo, so the row waits for a D26 trick movie.
- **Q6.** Determinism: a second recording of every row in a different shard layout (§7).
- **Q7.** lsnes gives no SPC700 state. The driver model starts from the decoded WRAM sound state;
  a row that needs SPC RAM takes tier b from LakeSnes at a cut where the memories agree.

## 6. Commands

Run from the worktree root; `oracle` is `crates/oracle`'s binary (`cargo build --profile verify -p oracle`
with `LIBRARY_PATH` set). Where Defender blocks fresh executables, run the CLI as a DLL (Pitfalls).

| Command | Does |
| --- | --- |
| `oracle movie ROM INPUTS [--oracle lsnes] [--frames N] [--from STATE FRAME] [--lag-log CSV] [--seeds CSV DIR] [--native-seeds CSV DIR] [--from-ladder L] [--ladder N DIR] [--ppu] [--streams] [--no-memories] [--no-picture] [--trace FROM TO PC,...]` | plays a movie; checks sync, cuts seeds and rungs, traces PCs |
| `oracle corpus ROM scenarios/lock.toml DATA [--oracle lsnes] [--native-seeds DIR] [--frame-pads DIR \| --from-frame-pads DIR] [--record-v2 FILE] [--sidecar DIR] [--dump-frames ROW:F,...] [--cache OUT [--cache-pictures]] [--only ID] [--shard I/N] [--watch-words ADDR,...] [--watch-pcs ADDR,...]` | runs rows: pads, format-2 recordings, the lockstep record cache; prints watched words when they change and watched instruction hits per frame |
| `oracle cross ROM INPUTS [--frames A..B] [--ranges] [--ranges-at A..B] [--streams] [--right-from STATE STEP]` | LakeSnes against lsnes, part by part per step |
| `oracle measure ROM LOCK DATA ID...`, `oracle measure-movie ROM INPUTS FRAMES`, `oracle trace` | the measurements |
| `oracle ladder-check`, `oracle ladder-rung`, `oracle derive-seed` | ladders and derived seeds |
| `oracle relock-compare`, `relock-merge`, `relock-refold` | checking recordings (§5) |
| `tools/oracle/relock_plan.sh` → `relock_cut.sh` → `relock_ppu_recut.sh` | cutting seeds |
| `tools gate`, `tools lockstep`, `tools localise` | the engine against the lock ([STATUS](STATUS.md)) |
| `tools cache-lean LOCK FROM TO ROW...` | writes record cache rows again as the lean tier |
| `tools/ci/pack_lockstep_cache.sh CACHE ROM [OUT]` | packs the record cache for ci-data |

`oracle corpus` reads `scenarios/lock.toml` for the old rows' starts and pads, and every other
`*.toml` beside it (`corpus.toml`, `coverage.toml`, `route-*.toml`) for row facts. Those files are
re-lock inputs only. `DATA/<bundle stem>/movies/<inputs>` holds the inputs. The **record cache**
(`--cache OUT`) holds one directory per row in one of two tiers, with the lock pins and one
`frame offset length cut` line per frame in its index; `verify::cache` refuses a cache whose
pins differ from the lock row. The **full tier** (`--cache-pictures`) is `OUT/<row>/records.bin`,
every `FrameRecord` with its picture in `oracle_record::codec`'s encoding, and `index.txt`. The
**lean tier** (the default) is `lean.bin`, each record without its picture and its five memories
as the bytes that changed since the frame before, whole every 256th frame
(`oracle_record::lean`), and `lean.txt`: about a hundredth of the bytes. Lockstep, the harnesses
and the domain free run read no picture, so they read the lean tier when a row has one and the
full tier otherwise; `tools cache-lean LOCK FROM TO ROW...` writes full rows again as lean ones.
A row recorded with
`--watch-pcs` also has `hits.txt`: `# watched` and the addresses, then `frame pc line dot peek`
per hit. The hits stay out of `records.bin`, so a row recorded again with watches has the same
records byte for byte, and `verify::cache::Cache::record` puts each frame's hits back.

## 7. Data: the ROM, bundles and the cipher

- **ROM:** unheadered, SHA-1 `da957f0d63d14cb441d215462904c4fa8519c613`, SHA-256
  `12b77c4bc9c1832cee8881244659065ee1d84c70c3d29e6eaf92e6798cc2ca72`, at `rom/sm.smc` (ignored).
  CI decrypts it from the private `JimLundin/ci-data` repository with `SM_ROM_KEY`
  (`openssl enc -d -aes-256-cbc -pbkdf2 -iter 600000`) and checks the SHA-1.
- **The cipher of every data bundle:** AES-256-CBC, PBKDF2 with 200000 iterations and SHA-256; the
  passphrase is the hex SHA-256 of `"sm-decomp-seeds-v1"` followed by the ROM's bytes. Bundles are
  assets of the `ci-data` release; a pin is the ciphertext's SHA-256.

```sh
pass=$({ printf 'sm-decomp-seeds-v1'; cat rom/sm.smc; } | sha256sum | cut -d' ' -f1)
curl -fsSL -o b.enc https://github.com/JimLundin/sm-decomp/releases/download/ci-data/NAME
echo "SHA256  b.enc" | sha256sum --check
openssl enc -d -aes-256-cbc -pbkdf2 -iter 200000 -md sha256 -pass "pass:$pass" -in b.enc | tar -xz -C DATA
```

- **The relock bundle** `relock-v2-lsnes-6ead7227957a4a10.tar.gz.enc` (SHA-256
  `41f0b2ed8da8d2fd87c18746f2f06c4f2c746e33ac6de91c72388d7ae5d9009e`), pinned by the `bundles` line
  of `scenarios/locks/lsnes.toml`, unpacks to `relock-v2-lsnes/`: `seeds/<capture>/NNNN.record`,
  `.lss` and `.words`, and the rows' `.pads` (`verify::bundle`).
- **The lockstep record cache** `lockstep-cache-e59c25d39059593d.tar.xz.enc` (SHA-256
  `a77f9800ea881eaa2b3b52477e96132c5e6945e863ad922b1d3fca01b4caacf2`), pinned in
  `.github/workflows/verify.yml` (`CACHE_ASSET`, `CACHE_SHA256`); unpack with `tar -xJ` (about
  7.7 GB). It holds the 53 rows `floors.txt` names, each with its `hits.txt` (the watched
  instructions of every domain, the blocks' scripts included), so CI holds the ratchet's rows and
  every domain's harness floors and coverage. Pack it with
  `ROWS="..." tools/ci/pack_lockstep_cache.sh`; a new cache is a new asset and a commit that
  repins it.
- Only the root uploads (`timeout 600 gh release upload ci-data FILE`). Nothing unencrypted and
  ROM-derived is ever committed or uploaded; logs carry ids, frames, field names and hashes.

## 8. Pitfalls

- **Defender** blocks freshly built executables at random (exit 126), build scripts included.
  Locally, build a cdylib exporting `sm_oracle_run(argc, argv)` around `oracle::cli::main` and call it
  from Python with ctypes, as `tools/lane/test_dll.py` does for tests. Only `$TEMP` allows executables.
  A DLL in use can't be relinked: build under a new name. CI runs ordinary binaries. A registry
  crate with a build script is used through its copy in `vendor/`, the script's output baked in by
  `tools/vendor_nobuild.sh` and patched in by the root `Cargo.toml`; CI's
  `tools/vendor_nobuild_check.sh` holds each copy to the real script.
- **lsnes processes started together** used to race on `%APPDATA%/lsnes`; each now has its own
  config directory. A killed lsnes usually shows as `took no request`.
- **lsnes crashes (fixed):** a cut callback that unregistered itself while lsnes dispatched it made
  lsnes call a freed entry (`attempt to call a nil value`, then death, "the exit at step 18068"). The
  cut callback now stays on the NMI handler's RTI. Unregistering from another callback is clean but
  runs at 7 frames/s instead of 44.
- **Old records:** lsnes records cut before the RTI fix (`9777b468a`) or the PC fix (`2da4d4ae6`,
  record bytes 34-35) differ from new ones; re-cut before mixing them.
- **Each request gets exactly one record.** Reading requests in `on_input`, or emitting at every
  RTI, desyncs the protocol. A previewed pad must come with the next step, or the server quits.
- **Mesen** keeps the cartridge SRAM in `Saves/`; clear it before every run.
- **A Tick** is an NMI that interrupted a wait site (which includes the message-box input loop), not
  a "polled" NMI. Old-lock alignment presents pad k+1 until the k-th requested NMI.
- Orphaned lsnes processes from killed runs may survive; kill only your own PIDs.
- Kept lsnes run directories (`%TEMP%/oracle-lsnes-*`, a ROM copy inside) are yours to delete.

## 9. Open items

- `video` rarely differs between lsnes runs; the cause is open (§5 handles it by agreement).
- Facts, typed streams, row rungs and packed shards in the recorder (§5, step c).
- `oracle corpus` still reads the old `scenarios/lock.toml`; moving it onto `lsnes.toml` and the
  bundle retires that file, `corpus.toml`, `coverage.toml`, `route-*.toml` and `LEGACY(verifier)`.
- LakeSnes past any% 35175: fit the line-0 HDMA pause on both emulators, then fix 0005 ((q)).
- DMA bytes under forced blank are counted only at the cut; a per-write INIDISP check is needed ((a)).
- `oracle` and `oracle_record` tests don't run in CI yet.
- The 100% chain rows' pads, one frame early against the movie (§5, step b1): re-cut them from
  the movie at each seed record's frame counter and re-lock those rows.
- `lsnes.toml`'s `server` pin (`833e4df9...`) is the SHA-256 of an older `lsnes_server.lua`; the
  script's comment changed since (`lsnes::server_sha256` gives `7e2b31f1...`), its behaviour not.
  The next re-lock writes the current one; nothing compares the pin with the build yet.

## Appendix A: measurements (MEASUREMENTS)

Measured on LakeSnes with the fixes of the time and on lsnes, any% unless said. Lines are the V
counter (225 is the first vblank line), dots the master-clock H position (0-1363).

- **(a) Busy spans.** Movie frames 0-8499: 432 busy frames in 31 spans, at boot, the title, the intro
  and the Ceres load, NMIs landing in the APU upload loop (`$80:8092-$8103`) and decompression.
  Seeded door lengths were invalid (stalled migration seeds). Forced-blank DMA bytes: open.
- **(b) Busy frames per row.** 176 of 601 rows have busy frames, 2,231 in all; most on
  `options-menu-tour` (283), chain rows and `area-maridia`.
- **(c) Line timing.** The HDMA object handler returns at line 241-249 on ordinary frames (median
  246-247). IRQ-handler register writes land in the last third of their line (H-blank from dot 1096),
  one write per register per IRQ. These set D7's beam-sync point and D11's dots.
- **(d) Sound.** The driver answers a request in about half a frame (median about 8,500 SPC cycles),
  at worst within one; upload bytes echo in about 21 cycles. Stalls on migration seeds are their
  missing SPC state, not the driver.
- **(e) lsnes runs the any% movie** from power-on and from native seeds; the 27 rows that stalled
  on migration seeds all complete on native seeds. The old exit at 18068 was the callback crash (Pitfalls).
- **(f) Cross-oracle, any% 0-4800.** No difference in the game's own state; every difference is
  power-on SRAM, the cut's definition, sub-instruction NMI timing in the wait loop, or sound timing.
- **(g) MesenCE as a third vote.** At 1342 Mesen agrees with LakeSnes: the "lag" was lsnes's cut at
  line 241, removed by cutting at the RTI ((i)). The lasting lag at 8465 follows the SPC rate:
  +40 (32040 Hz) lags, +41 stays in sync, and bsnes v085's 32040.5 lies between. Fix: 0002.
- **(h) Fix 0002.** At bsnes v085's exact SPC rate LakeSnes follows any% to 17984, 2.1x as far.
- **(i) lsnes cuts at the RTI** also when the NMI outlasts its frame, given the next pad in advance.
  Records of such steps changed; older ladders and seeds were regenerated.
- **(j) Ports in time order** (candidate 0003) don't change any% 0-18999. No fix.
- **(k) The 17985 lag** is HDMA overhead LakeSnes charged on lines after the HUD channel terminated
  (12-18 clocks a line), not the room load. Fix: 0004.
- **(l) Fix 0004** moves LakeSnes's sync to 35199 without moving 0-395, 1342 or 8465.
- **(m) 35175** is a 30-clock miss of the NMI request from HDMA-pause rounding on HUD lines 0-32; lines 33-224 match.
- **(n) HDMA variants** (line overhead 18, request at dot 278, alignment rules, anomie's IRQ point):
  none zeroes the HUD lines; the one that fixes 35175 moves the boot. No fix 0005.
- **(o) lsnes gives the picture, the CPU port log and the PPU registers** from Lua, without linking
  emulator code; it lacks the SPC side and the five PPU fields (Q4).
- **(p) lsnes's HDMA pause** fits on 90 of 90 HUD lines as LakeSnes's lengths starting one CPU cycle
  after the request. Not enough alone to zero 35175.
- **(q) The 3 x 4 grid** of request point (1102, 1104, 1106) and alignment phase (0-6): no cell is 0;
  the residual starts on line 0, the IRQ line. Next: fit the line-0 pause. No fix 0005.
- **(r) lsnes's picture** is the right frame when taken in `on_frame`; PNG transport makes streams
  cost 26%; equal as shown on every step checked.
- **(s) lsnes's APU log after a resume** was empty because the game uses the bank-`$82`/`$85` port
  mirrors; hooking every bank fixed it. It differs from LakeSnes only in handshake poll counts.
- **(t) lsnes's joypad reads** have no bank gap; the one any% difference (step 20213) is a LakeSnes
  read outside the NMI. No change.

## Appendix B: emulator accuracy (ACCURACY)

- **What SM exercises:** APU uploads by handshake (`$80:8059`; a failed handshake hangs at `$8077`
  after `3000h` polls), NMI-gated lag (`$05B4`; the handler has no vblank guard), auto-joypad
  (`$80:9459` spins on `$4212`, so the auto-read start costs cycles only). The
  [bank $80 log](https://patrickjohnston.org/bank/80) is the source.
- **The SPC700 clock is not one number.** A ceramic resonator drives it: 32000 Hz nominal, real
  consoles 32010-32152 Hz and rising as they warm
  ([SNESdev Timing](https://snes.nesdev.org/wiki/Timing#Audio),
  [forum measurements](https://forums.nesdev.org/viewtopic.php?t=24610)). Emulators pick about 32040;
  bsnes v085 32040.5. So there is no single correct lag after an upload on hardware: the movies
  define it as lsnes's.
- **The emulators.** bsnes v085's compatibility core (lsnes) shares the accuracy core's CPU, SMP,
  counters and DMA, and differs only in mid-line PPU writes and DSP sub-sample timing. LakeSnes runs
  the SPC in whole opcodes, catching up at each port access, and starts NMI at dot 0 of line 225.
  Mesen latches port writes per half SPC cycle. Sources:
  [LakeSnes](https://github.com/angelo-wf/LakeSnes),
  [Mesen2 `Spc.cpp`](https://github.com/SourMesen/Mesen2/blob/master/Core/SNES/Spc.cpp),
  [bsnes v086 tree](https://github.com/Dgdiniz/bsnes_v086-source/blob/master/bsnes/snes/Makefile).
- **No SM movie we use is console-verified** ([TASVideos verified list](https://tasvideos.org/Movies-Verified-SNES)).
- **Trust per stream:** game content, either oracle up to a one-frame shift; the frame-exact lag
  pattern and the CPU-SPC handshake, lsnes; picture within a line and sound sample timing, neither
  yet. The engine models upload slowdown with the SPC clock as a parameter, 32040.5 Hz by default.
