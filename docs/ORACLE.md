# The oracle

The oracle runs the original, unmodified ROM on lsnes and records what the engine is
compared against. It lives in `crates/oracle` (the lsnes adapter, the recorder, the command
line) and `crates/oracle_record` (the record types, with no emulator, so `verify` can read
records without linking one). Neither depends on the engine. The comparison contract it serves
is [ARCHITECTURE §11](architecture/ARCHITECTURE.md#11-the-engine--oracle-comparison-contract).

Section labels in parentheses, such as (RELOCK §2) or (MEASUREMENTS (o)), are the names code
comments cite.

## 1. Decision and contract

### A real, beam-scheduled emulator, run unpatched

The oracle is **lsnes** rr2-beta23 with its bsnes v085 compatibility core, the emulator the TAS
movies were recorded on; any% stays in sync through every seed checked. It is a complete SNES: the
65816 on the master-clock beam, DMA and HDMA, NMI and IRQ timing, auto-joypad, per-line PPU, and
the SPC700 and DSP running the game's own sound driver. With a running beam no ROM patch is
needed, so `OracleId::patches` is empty: the oracle runs the unpatched ROM and shares no code
with the engine (PRINCIPLES P12). lsnes is a separate GPL process driven by
`tools/oracle/lsnes_server.lua` (`src/lsnes.rs`); no emulator code is linked or copied.

### Which oracle records which stream (D25)

Acceptance replay never runs an oracle: it compares actual production with pinned independent
observations. Each observation comes from the most accurate oracle that produces it. lsnes
records the memories, the CPU port log and the PPU registers (MEASUREMENTS (o)). The five PPU
fields lsnes doesn't expose are not compared (Q4), and no record holds the SPC700 side (Q7).

### The frame record (contract v1)

One record per video frame, lag or not. Record k covers (cut k-1, cut k] and the state at cut k.

- **The cut:** the RTI that ends the NMI serviced in vblank k (`Cut::NmiReturn`). With NMI off it
  is vblank start (`Cut::VblankWithoutNmi`); an NMI still running when vblank ends is
  `Cut::NmiStillRunning`.
- **NMI k:** its kind (`Requested` when the handler read the joypad, else `Lag`), its interrupted
  PC (the instruction the RTI returns to), and `overrun` when that PC is not one of
  `oracle_record::game::WAIT_SITES`.
- **Facts:** the pad auto-read at vblank k, joypad register reads, the overscan flag, the beam
  position where the main thread entered its wait.
- **Raw state at cut k:** CPU registers, WRAM 128 KiB, VRAM 64 KiB, CGRAM 512 B, OAM 544 B, SRAM,
  and `PpuRegisters` (INIDISP to SETINI, NMITIMEN, HTIME/VTIME, HDMAEN, MEMSEL, the eight DMA
  channels' `$43x0-$43xA`), rebuilt from decoded emulator state so they survive a state load.
- **APU port log:** the CPU's reads and writes of `$2140-$2143`, in order.
- **Watched-PC hits:** each with its beam position and four peeked WRAM bytes. Not digested.

Given the ROM, the start and the pads, records are bit-identical on every host.

### lsnes streams

`Lsnes::record_streams` adds the PPU registers, and the APU port log when asked; otherwise
records are unchanged. It sends the server's `T <apu> <ppu>` request before the first step.

- **APU port log:** the CPU side only (`memory.registerwrite`/`registerread`), hooked in every bank
  that maps the I/O page, because the game reaches the ports through its data bank (`$82`, `$85`).
- **PPU registers:** rebuilt from bsnes's decoded fields in `memory.getregisters()`, without
  `nmitimen`, `htime`, `vtime`, `hdmaen` and `memsel`.

**Watched-PC hits** (`corpus --watch-pcs`, `Lsnes::open_watching`) come from the server's trace
(`OH_TRACE`, `OH_TRACE_LOG`), so the server script and its SHA-256 stay as they are: an exec
callback per address logs `pc ADDRESS LINE DOT`, and each record sent logs `cut FRAME`. The client
watches the NMI handler's RTI too and splits the log there: a hit after the RTI and before the
record is sent ran in the next frame, so it goes to the next record; a resumed run drops what ran
before its first RTI, as the server's counts do. A hit carries a peek only when the run asks for
one (`OH_TRACE_PEEK`, `measure` and `movie --trace`); otherwise `peek` is 0. Watch
instruction starts only: lsnes's exec callback fires on operand fetches too.

The lsnes cut is the interrupted instruction reached again with the stack restored. The server
finds the NMI handler's one `PLY PLX PLA PLD PLB RTI` from the ROM (`$80:9601`), takes the cut when
S equals the NMI's S, and derives the registers after the RTI. A frame without an NMI is cut at
lsnes's frame boundary (line 241). lsnes fixes a frame's input at its start, so an NMI that runs
past line 241 is cut at its RTI only if the client previewed the next pad (`Lsnes::preview`;
MEASUREMENTS (i)); without a preview it is cut at the boundary as `NmiStillRunning`. lsnes's
saved state is taken at the boundary that ends the step it was requested before.

## 2. Toolchain and layout

**lsnes.** Set `SM_LSNES` to the rr2-beta23 executable, or install it as `lsnes` on `PATH`.
Requests go to stdin. Each response overwrites a file and announces its size as `R <bytes>` on stdout.
Each process has its own home and config under `<run dir>/config`. The run directory keeps `stderr.txt`, the
server's `lua.log` (`OH_LOG`) and `lsnes.log` there, and reports them if it exits. `SM_LSNES_KEEP=1`
keeps the directory. A record not announced within 60 s (first step) or 30 s (later) kills that
lsnes and fails the row as silent. The script loads a start itself (`OH_START`, `load-state`):
lsnes rejects a state given on its command line with this ROM and hangs.

**Crate layout.** `crates/oracle/src`: `lib.rs` (`Start`, `Streams`, `OracleId`, `OracleError`),
`lsnes.rs` (open a start, step one video frame with a pad, request a save, record streams),
`seed.rs` (lsnes room captures' work RAM), `corpus.rs` (row inputs and playback),
`cache.rs` (the record cache's writer), `ladder.rs`, `measure.rs`, `cli.rs` and `bin/oracle.rs`.
The lock's reader is `data::oracle_lock`, shared by capture and replay.
`crates/oracle_record`: `FrameRecord`, its codec, the lsnes raw record decoder, `oracle_record::game::WAIT_SITES`
and `inputs.rs`. No crates.io dependency, and no test needs lsnes.

## 3. Inputs, starts, ladders and movies

**Inputs.** The `.inputs` format: one hexadecimal JOY1 word per line, LF, one line per video
frame (`oracle_record::inputs`). A row's **frame-pad file** (`<row>.pads`, RELOCK b1) has one
line per frame of the row plus one for the frame after, which the last frame's NMI reads when
it outlasts the frame: N + 1 lines.

**Starts.**
- `Start::PowerOn`: work RAM filled with 0x55, as bsnes v085, which recorded the movies, fills it.
- `Start::State(blob)`: the emulator's own state. **Native seeds** are lsnes states cut from
  the movies, named `CAPTURE:INDEX` and stored as `DIR/CAPTURE/NNNN.lss` (the state before step
  N) and `NNNN.record` (the raw record of step N-1, always with the memories). A **derived seed**
  adds `NNNN.words`: word edits on a base state, no emulation (`oracle derive-seed`).
- An lsnes **room capture** (`target/seeds/<capture>/<NNNN>-<ROOM>-<memory>.bin`, one file for each
  of WRAM, SRAM, VRAM, CGRAM, OAM and APU RAM) is no start: it holds no CPU registers and no
  SPC700, DSP or timer state. `movie --seeds` compares a movie's work RAM with its `-wram.bin`.

**Ladders.** Long emulator runs are the exception. Each movie keeps lsnes states every 10000
frames (`crates/oracle/src/ladder.rs`), so a cut or a measurement resumes from the nearest rung.
- `oracle movie ROM movies/NAME.inputs --ladder N DIR` writes `DIR/NAME/FFFFFF.lss`,
  `FFFFFF.record` and `ladder.txt` (each rung's frame and its record's SHA-256).
- `oracle ladder-check ROM INPUTS DIR [--rungs A..B]` resumes each rung, runs to the next and prints
  `equal` or the differing parts; exit 1 on any difference. Reproduces means the record before the
  next rung is byte-identical; `.lss` bytes are not compared (equal machines can save different bytes).
- `oracle ladder-rung INPUTS LADDERS FRAME` prints `RUNG PATH` for `movie --from PATH RUNG`;
  `movie ... --native-seeds TABLE DIR --from-ladder LADDERS` starts at the latest rung before the
  earliest seed. Rungs: 100% 22, low% 16, any% 12; every `ladder-check` equal.
- No workflow builds or checks the ladders. `ladder-check` runs only by hand.

**Movie sync.** `oracle movie ROM movies/any.inputs --lag-log LOG` compares each frame with the
lsnes lag log `tools/lsnes_lag_log.lua` writes: `$05B6`, `$05B8`, the lag flag, the game state, the
room and Samus's position. The log is binary with a version, an 18-byte record a frame, because a
movie's hundreds of thousands of frames are too many for TOML (`crates/oracle/src/cli/lag_log.rs`).
lsnes is in sync by definition (all room seeds checked: same room, 0 WRAM bytes differing).

**Movies.** `movies/manifest.toml` lists the published TAS movies (any% 3653M, low% 3273M,
100% 4010M and others), each with its download link, SHA-256, frame count, emulator (lsnes
rr2-beta23, bsnes v085 compatibility core) and ROM SHA-256. Download the ZIP, extract the
`.lsmv`, check its SHA-256, then import one controller word per video frame:

```sh
python3 tools/movie_inputs.py movies/super-metroid-any-3653.lsmv movies/any.inputs --rom rom/sm.smc
```

`movies/room-seeds-*.toml` list the room captures each movie's seeds come from
(`oracle_record::room_seeds`, read by `tools/oracle/relock.py`, `oracle --seeds` and the README
report). An lsnes capture writes its own as `rooms.toml`, with the frame counter at each capture as
`nmi`. Movies, inputs, states and logs are ROM-derived and stay local (`.gitignore`) or in encrypted
bundles. The Mother Brain phase rows start from the `100-mb-phases` captures (her form word
`$7E:7800` changing), one per form.

## 4. The lock

`scenarios/locks/lsnes.toml` is committed. It names each row the oracle plays and pins the row's
inputs, which stay in an encrypted bundle.

```toml
format = 2
oracle = { emulator = "lsnes-bsnes", version = "rr2-beta23 (bsnes v085)", server = "<sha256>", fixes = [] }
bundles = [{ name = "relock-v2-lsnes-<id>.tar.gz.enc", sha256 = "<ciphertext sha256>" }]

[[row]]
id = "area-brinstar"
start = "100-input-phase:17"   # a native seed; "power-on" for one row
seed_record = "<sha256 of the seed's .lss>"
pads = "<sha256 of the .pads file>"
frames = 477
```

A row from a derived seed also pins its edits with `seed_words`. `server` is the SHA-256 of the
`lsnes_server.lua` the oracle is built with, and capture refuses a lock that pins another. It
checks the emulator, version and fixes too before it plays a row. `data::oracle_lock::Lock` reads
the file for capture and replay, and refuses an unknown key, a key given twice or a row entered
twice. The lock holds nothing a run produces. The record cache in §6 holds the runs.

## 5. The re-lock (RELOCK §3)

Seed cuts produce the native seeds, and pad files determine each row's inputs. The encrypted
bundle contains the row inputs and seed records. The lock pins that bundle and each row's pad and
native state files.

### Step a1: the cut plan

`python3 tools/oracle/relock.py plan NATIVE LADDERS` writes the plan. Then
`python3 tools/oracle/relock.py cut PLAN OUT LOGS [cut|realign|derive|check|all] [MOVIE-RUNG...]`
runs one `movie --from RUNG --native-seeds ... --no-memories --ppu` job per window, longest first,
`JOBS` (8) at a time, `STAGGER` (4) s apart. `realign`: most captures follow step F-1 of their
movie but some follow step F (the record's `$05B6` one short of the table's `nmi`); those are re-cut
a step on.
`check`: each seed's room and frame counter equal its table's, and its WRAM equals its capture's
outside the stack page. A table without `nmi` takes the counter from the capture's WRAM.
`derive-seed` checks that each written word stuck: one step on it reads the written value, or
something other than the plain base's step (physics moves Samus's position in that step).

### Step b1: frame pads

The bundle's `pads/<row>.pads` contains one pad word per video frame plus the next frame's
word. The runner previews the next word before each step so that lsnes can cut at the NMI's
RTI when the NMI outlasts the frame.

**The 100% chain rows' pads sit one frame early against the movie.** Their `.pads` came from the
Tick run, not from the movie: of the 327 100% chain rows, 0 play the movie's own pads from the
seed (most differ from the first pad), and `chain-0073-A59F` dies to Kraid. A row's pads are the
movie's from the seed record's lsnes frame counter (record bytes 20-23), where the movie's pad for
the seed's first step is: line `first` to `first + frames` of the `.inputs`, one more than the
row's frames. The seven movie rows of `scenes-v1` (`event-zebes-awake`, `item-energy-tank`,
`item-reserve-tank`, `item-charge-beam`, `item-wave-beam`, `item-plasma-beam`,
`boss-kraid-defeated`) are cut that way and play the movie through the progress report each was
chosen for. The any% Ceres chain rows `chain-any-0001` to `chain-any-0011` are cut that way
too: each ends at the next seed's frame counter and crosses its room's door. The 100% chain rows
are re-cut and re-locked the same way (§9).

### Seed records with the PPU registers (RELOCK §9)

`oracle movie --ppu` asks lsnes for the PPU tier alone (`T 0 0 1`); the record is the lean record
followed by `OHS1`, flag 4 and the 146 PPU bytes, and the emulation is unchanged.
`python3 tools/oracle/relock.py ppu-recut OLD PLAN WORK [ROW...]` re-cuts the records of a
decrypted bundle, checks each against the old, packs deterministically, encrypts and prints the
new `bundles` line. The native states must have the same members and identical contents except for
`rrdata` and its derived `rerecords` history count. The lsnes manual §§10.22–10.23 defines these as
load-history metadata; the bundled lsnes source, `src/core/moviedata.cpp`, adds a load ID even on
a read-only load. The
[native-state check](../tools/oracle/relock_seed_state.py) keeps `projectid`, inputs and every
machine-state member exact. The new bundle keeps the old `.lss` files, whose SHA-256 values are
the `seed_record` pins, so only the `bundles` line changes.

### Engineering answers

- **Q2.** Pads per video frame, converted once from the Tick words (b1).
- **Q4.** The five `ppu` fields lsnes lacks (`nmitimen`, `htime`, `vtime`, `hdmaen`, `memsel`) are
  not compared, named in ARCHITECTURE §11.6. The IRQ lines are measured (D11).
- **Q5.** `boss-golden-torizo`'s capture comes from a snes9x movie lsnes can't play. The 100%
  movie's visit to room B283 doesn't defeat Golden Torizo, so the row waits for a D26 trick movie.
- **Q7.** lsnes gives no SPC700 state. The driver model starts from the decoded WRAM sound state;
  no oracle record holds SPC RAM.

## 6. Commands

Run from the worktree root; `oracle` is `crates/oracle`'s binary (`cargo build --profile verify -p oracle`).

| Command | Does |
| --- | --- |
| `oracle movie ROM INPUTS [--frames N] [--from STATE FRAME] [--lag-log LOG [--positions-only]] [--seeds TABLE DIR [--ranges]] [--native-seeds TABLE DIR] [--from-ladder L] [--ladder N DIR] [--ppu] [--streams] [--no-memories] [--watch FROM EVERY] [--trace FROM TO PC,...]` | plays a movie; checks sync, cuts seeds and rungs, traces PCs |
| `oracle corpus ROM LOCK BUNDLE [--cache OUT] [--only ID] [--shard I/N] [--limit N] [--watch-words ADDR,...] [--watch-pcs ADDR,...] [--no-memories]` | plays the selected rows and writes their record caches |
| `oracle measure ROM LOCK BUNDLE ID...`, `oracle trace ROM LOCK BUNDLE ID FROM TO` | measurements and frame traces |
| `oracle ladder-check`, `oracle ladder-rung`, `oracle derive-seed` | ladders and derived seeds |
| `tools/oracle/relock.py plan`, `cut`, `ppu-recut` | cutting seeds |
| `tools replay list`, `tools replay run`, `tools replay check` | indexed production replay against independent observations ([STATUS](STATUS.md)) |
| `tools cache-check LOCK [BUNDLE CACHE ROW [[--carry-watches] WATCH_PCS]]` | checks that a cached row has the lock's pins, was recorded by the lsnes the oracle would run and the lock's server, and watched the addresses |
| `uv run --locked --project tools replay-data pack SOURCE ROM OUT` | packs `SOURCE/cache` and `SOURCE/native` as the replay data for ci-data |

For these commands, `LOCK` is `scenarios/locks/lsnes.toml` and `BUNDLE` is the unpacked
`relock-v2-lsnes/` directory. Capture checks each selected row's pad SHA-256 and length, then
checks the native state and edit file hashes for a seeded start. Use `--no-memories` for cut
counts alone. `oracle trace` shows CPU, room and APU port observations
for a row's frame range. `oracle measure` reads instruction hits, decompression pointers and
the CPU's APU port accesses; lsnes has no PPU write log and no SPC700 side to time.

The **record cache** (`--cache OUT`) holds one directory per row. `OUT/<row>/lean.bin` holds
each record with its five memories as the bytes that changed since the frame before, whole every
256th frame, about 2 KB a frame. `oracle_record::lean` holds that codec. `lean.txt` holds the
lock pins, the SHA-256 of the lsnes executable and of the server script that recorded the row,
and one `frame offset length cut` line per frame. `verify::cache` refuses a cache whose pins
differ from the lock row, and one whose index is in an older format, which has to be recorded
again. `tools cache-check` also refuses one recorded by an lsnes other than the one the oracle
would run, or by a server other than the one the lock pins, so a drifted oracle records the row
again. A row
recorded with `--watch-pcs` also has `hits.txt`: `# watched` and the
addresses, then `frame pc line dot peek` per hit. The hits stay out of `lean.bin`, so a row
recorded again with watches has the same records byte for byte, and
`verify::cache::Cache::record` puts each frame's hits back.

**Making the record cache.** `oracle corpus ROM LOCK BUNDLE --cache CACHE --only ROW` plays
the row's frame pads from its native seed and writes `lean.txt` last.
`python3 tools/oracle/record_caches.py ROWS BUNDLE ROM CACHE [JOBS]` runs the capture for a list
of rows with eight lsnes processes by default. It builds the default `tools` cache reader and
skips a row when `tools cache-check` accepts the existing cache for the lsnes the oracle would
run, found as section 2 says. A row another batch is recording waits for it. Set `WATCH_PCS`
to record the `hits.txt` read by independent causal input providers. `ORACLE` names the oracle
command, `SM_TOOLS` overrides the cache reader and `SM_LOCK` overrides the lock.

The three retired scenario rows used the `golden-input-phase:1` start from a capture without
a native seed in the format-2 bundle. They have no indexed replay cases.

- `boss-golden-torizo` covered the fight in room B283.
- `golden-orb-spawn-parameter` covered the orb spawn in that fight.
- `golden-original-wait` covered a single wait cut in that room.

## 7. Data: the ROM, bundles and the cipher

- **ROM:** unheadered, at `rom/sm.smc` (ignored), pinned by `ROM_SHA256` in the
  [replay data pins](../tools/src/sm_tools/ci/core/replay_data.py), which the `replay-data` tool
  checks it against. CI decrypts it from the private `JimLundin/ci-data` repository with
  `SM_ROM_KEY` (`openssl enc -d -aes-256-cbc -pbkdf2 -iter 600000`).
- **The cipher of every data bundle:** AES-256-CBC, PBKDF2 with 200000 iterations and SHA-256; the
  passphrase is the hex SHA-256 of `"sm-decomp-seeds-v1"` followed by the ROM's bytes. Bundles are
  assets of the `ci-data` release; a pin is the ciphertext's SHA-256.

```sh
pass=$({ printf 'sm-decomp-seeds-v1'; cat rom/sm.smc; } | sha256sum | cut -d' ' -f1)
curl -fsSL -o b.enc https://github.com/JimLundin/sm-decomp/releases/download/ci-data/NAME
echo "SHA256  b.enc" | sha256sum --check
openssl enc -d -aes-256-cbc -pbkdf2 -iter 200000 -md sha256 -pass "pass:$pass" -in b.enc | tar -xz -C DATA
```

- **The relock bundle**, pinned by the `bundles` line of `scenarios/locks/lsnes.toml`, unpacks to
  `relock-v2-lsnes/`: `seeds/<capture>/NNNN.record`, `.lss` and `.words`, and all 651 rows'
  `pads/<row>.pads`. These remain independent source assets;
  indexed replay declares which seed and input formats its admission supports.
- **The replay data** holds the lockstep record cache under `cache/` and the native fixtures
  under `native/`, one asset pinned by `ASSET` and `ASSET_SHA256` in
  the [replay data pins](../tools/src/sm_tools/ci/core/replay_data.py).
  `uv run --locked --project tools replay-data fetch ROM DEST` installs it. The cache retains
  independent recorded rows and watched causal traces.
  Replay case metadata selects required rows; admission checks source pins and record coverage.
  Required acceptance runs the complete index rather than former harness floor-file unions.
  Local selection can use the cases whose required sources are available; missing data fails
  the selected case, never silently skips it.
  `replay-data pack` makes a new asset from a new cache or fixture; a new asset is a commit
  that repins it.
- Only the root uploads (`timeout 600 gh release upload ci-data FILE`). Nothing unencrypted and
  ROM-derived is ever committed or uploaded; logs carry ids, frames, field names and hashes.

## 8. Pitfalls

- **A killed lsnes** usually shows as `took no request`.
- **A callback that unregisters itself** while lsnes dispatches it makes lsnes call a freed entry
  (`attempt to call a nil value`, then death), so the cut callback stays on the NMI handler's RTI.
  Unregistering from another callback is clean but runs at 7 frames/s instead of 44.
- **Records cut before** the RTI fix (`9777b468a`) or the PC fix (`2da4d4ae6`, record bytes
  34-35) differ from current ones; re-cut them before mixing.
- **Each request gets exactly one record.** Reading requests in `on_input`, or emitting at every
  RTI, desyncs the protocol. A previewed pad must come with the next step, or the server quits.
- **A Tick** is an NMI that interrupted a wait site, including the message-box input loop.
- Orphaned lsnes processes from killed runs may survive; kill only your own PIDs.
- Kept lsnes run directories (`%TEMP%/oracle-lsnes-*`, a ROM copy inside) are yours to delete.

## 9. Open items

- The 100% chain rows' pads, one frame early against the movie (§5, step b1): re-cut them from
  the movie at each seed record's frame counter and re-lock those rows.

## Appendix A: measurements (MEASUREMENTS)

Any% unless said. Lines are the V counter (225 is the first vblank line), dots the master-clock
H position (0-1363). Findings (a) to (d) were measured in part on an emulator the oracle no
longer runs, whose code git history keeps; the rest are lsnes's. Letters that left with that emulator are not reused.

- **(a) Busy spans.** Movie frames 0-8499: 432 busy frames in 31 spans, at boot, the title, the intro
  and the Ceres load, NMIs landing in the APU upload loop (`$80:8092-$8103`) and decompression.
- **(b) Busy frames per row.** 176 of 601 rows have busy frames, 2,231 in all; most on
  `options-menu-tour` (283), chain rows and `area-maridia`.
- **(c) Line timing.** The HDMA object handler returns at line 241-249 on ordinary frames (median
  246-247). IRQ-handler register writes land in the last third of their line (H-blank from dot 1096),
  one write per register per IRQ. These set D7's beam-sync point and D11's dots.
- **(d) Sound.** The driver answers a request in about half a frame (median about 8,500 SPC cycles),
  at worst within one; upload bytes echo in about 21 cycles.
- **(e) lsnes runs the any% movie** from power-on and from native seeds; the 27 rows that stalled
  on migration seeds all complete on native seeds. The old exit at 18068 was the callback crash (Pitfalls).
- **(i) lsnes cuts at the RTI** also when the NMI outlasts its frame, given the next pad in advance.
  Before, it cut such steps at line 241, which showed as a lag frame at 1342. Records of such
  steps changed; older ladders and seeds were regenerated.
- **(o) lsnes gives the picture, the CPU port log and the PPU registers** from Lua, without linking
  emulator code; it lacks the SPC side and the five PPU fields (Q4).
- **(r) lsnes's picture** is the right frame when taken in `on_frame`; PNG transport makes streams
  cost 26%. That transport has since been removed.
- **(s) lsnes's APU log after a resume** was empty because the game uses the bank-`$82`/`$85` port
  mirrors; hooking every bank fixed it.
- **(t) lsnes's joypad reads** have no bank gap.

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
- **The emulator.** bsnes v085's compatibility core (lsnes) shares the accuracy core's CPU, SMP,
  counters and DMA, and differs only in mid-line PPU writes and DSP sub-sample timing
  ([bsnes v086 tree](https://github.com/Dgdiniz/bsnes_v086-source/blob/master/bsnes/snes/Makefile)).
- **No SM movie we use is console-verified** ([TASVideos verified list](https://tasvideos.org/Movies-Verified-SNES)).
- **Trust per stream:** game content and the frame-exact lag pattern and CPU-SPC handshake are
  lsnes's; picture within a line and sound sample timing aren't trusted yet. The engine models
  upload slowdown with the SPC clock as a parameter, 32040.5 Hz by default.
