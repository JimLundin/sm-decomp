# Handoff: oracle harness (2026-11-05)

Branch `research/oracle-harness` from main `52c21178d`; GATE OK `dc1b687ca`. Plan and findings: docs/oracle/HARNESS.md. Measurements: docs/oracle/MEASUREMENTS.md.

## What is true now

- `crates/oracle` depends only on `sm_data`, for the digest. It runs the unmodified ROM on upstream LakeSnes `9db90b8`, with no ROM patches.
  - One step is one video frame; the cut is the RTI of that frame's NMI.
  - Each record holds the memories, the CPU, `PpuRegisters`, the APU port log and the BGR555 picture.
  - Optional tiers add the SPC state, writes with line, dot and source, watched-PC hits and a profile.
- Fixes 0001 and 0002 (`native/fixes/`): the SPC at bsnes v085's 32040.5 Hz against the real master clock, as an exact integer ratio (MEASUREMENTS.md (h), lane `lane/oracle-lakesnes-rate`):
  - the any% movie matches the lsnes lag log exactly through frame 395, as with 0001 alone;
  - state, room and position match through 17984 (0001 alone: 8566), but for single-step samples at transitions; 8314 and 8567 pass;
  - at 17985, the door out of room 97B5, LakeSnes has one more lag frame; one frame behind from 18108, desync from 18587 (stays in room 9E9F).
- Corpus against today's lock: 601 of 638 faithful rows ran (lsnes seeds, plus `options-menu-tour` from power-on); 27 stalled.
  - Equal: sram 600, cgram 162, vram 150, oam 150, wram 2.
  - ppu 0, because the old definition includes renderer scratch. video and apu are redefined.
  - area-brinstar matches wram `3d2d36f47cb3ddc1`, vram `788c6d428540fb70`, cgram `da3390792deb0d4b`, oam `7bea571691d05da5` and sram, deterministically.
  - options-menu-tour (2310 ticks from power-on) matches vram, cgram, oam and sram.
- `--diagnose-kept-patches native-audio-handshake,debug-enemy-set-names` applies two of the old stepper's kept patches to a ROM copy.
  - wram then matches on 37 of 540 rows (area-crateria, boss-ceres-ridley-fireballs and others), so those mismatches are the old handshake patch.
  - 88 rows stall under it.
- Stalls: migration seeds have no SPC state, so the resumed driver sometimes never echoes. Credits, Mother Brain, death, save and door rows then hang in `$80:8077` (the upload timeout) or in sound waits.
- Speed: about 300 frames/s for one process with the picture on, about 950 in total for 10 processes on 14 cores.
- Not done:
  - DMA bytes under forced blank are only counted at the cut, which is wrong;
  - door lengths from seeds are invalid, because those rows stall;
  - no Mesen backend;
  - the old oracle's verifier build is blocked here, so there is no same-machine speed figure for it.

## How to run it on this machine

1. Run `bash tools/oracle/build_lakesnes.sh`. It fetches the pinned files, applies the fixes, builds with the cached Zig 0.15.2 and prints the lib dir.
2. Build: `LIBRARY_PATH=<lib dir> CARGO_TARGET_DIR=$TEMP/oracle-harness-target cargo build --release -p oracle`. Only `$TEMP` allows executables.
3. Exit 126 (local workaround, not committed): Defender blocks fresh exes at random, build scripts included, which is why there is no `build.rs`.
   - Build a cdylib exporting `sm_oracle_run(argc, argv)` around `oracle::cli::main` (`$TEMP/ohx/dll.rs`, `build.sh`).
   - Call it from the cached Python with ctypes (`$TEMP/ohx/run.py`, wrapper `$TEMP/ohx/oh`).
4. Data: `gh release download ci-data` each lock bundle, then `openssl enc -d -aes-256-cbc -pbkdf2 -iter 200000 -md sha256`, with the passphrase sha256("sm-decomp-seeds-v1" + ROM). Untar one directory per bundle (name without `.tar.gz.enc`) under DATA.
5. Commands:
   - `oracle corpus ROM LOCK DATA [--shard I/N] [--record F] [--diagnose-kept-patches L]`
   - `oracle movie ROM INPUTS --lag-log CSV`
   - `oracle measure`, `oracle measure-movie`, `oracle trace`

## lsnes backend (`--oracle lsnes`, commit e8b65b4b2)

- `src/lsnes.rs` drives `lsnes-bsnes.exe` as a separate process with `tools/oracle/lsnes_server.lua`.
  - The exe is rr2-beta23 (bsnes v085), in `%LOCALAPPDATA%/sm-decomp-experiments/20261025/tas-alignment/lsnes-rr2-beta23/`, or wherever `SM_LSNES` points.
  - Requests (`S pad memories`, `V path`, `R`, `Q`) go to its stdin. One record per lsnes frame goes to a file, announced as `R <bytes>` on its stdout. Stdout is text mode on Windows, so no binary goes there.
  - The cut is the interrupted instruction reached again with S restored. With no returned NMI, the cut is the lsnes frame boundary.
  - Reading requests in `on_input`, or emitting at the RTI, desyncs the protocol: records outnumber lsnes frames. Read and emit only at `on_frame_emulated`.
  - Missing fields: picture, APU log, SPC state (SMP_STATE and DSP_STATE are raw core structs), profile, write log, hits, `PpuRegisters`, `overscan`, `legacy_ppu_digest`.
  - `save_state` returns lsnes's savestate from the boundary after the last cut; call `request_save` first. A resumed run reproduces the next cut.
- Sync: `movie ROM any.inputs --oracle lsnes --seeds room-seeds-any-boundary.csv <any-boundary dir>`, from power-on.
  - All 16 room seeds through frame 17829 match: same room, 0 WRAM bytes differing, offset +0. They cover Ceres, the escape and Crateria up to room 97B5. LakeSnes desyncs at about 8567.
  - The NMI counter matches the lsnes lag log. Positions are off by one frame because the log samples at frame end.
  - Speed: 45 frames/s with all memories dumped every frame.
- The exit at 18068 is an lsnes crash, not the game (details: HARNESS.md, "lsnes crashes").
  - lsnes.log fills with `Error in Lua memory callback: attempt to call a nil value`, then lsnes dies (once `0xC0000028`, STATUS_BAD_STACK).
  - It follows the server's cut callback unregistering itself inside lsnes's dispatcher, and is intermittent: from power-on without saves, all 19000 frames ran clean.
  - It happens in runs that save states (`--native-seeds`): 1180 errors and a hang by frame 19829, from seed 16.
  - Fixed: the cut is a callback that stays on the NMI handler's RTI (`$80:9601`, found from the ROM), matched by S equal to the NMI's S. Records are byte-identical over 1000 frames; 17829 to 19829 with `--native-seeds` runs clean at 44 frames/s (HARNESS.md, "lsnes crashes").
- Diagnosis now kept (commit on this branch):
  - lsnes runs in its run directory; `stderr.txt`, the server's `lua.log` (`OH_LOG`) and `lsnes.log` go there.
  - lsnes stopping is an error, not a panic: `OracleError::Lsnes(lsnes::Failure)` with the cause (exited, silent, request or reply refused, bad record), the step, the PID, the exit status and the tail of all three logs; the directory is kept. `SM_LSNES_KEEP=1` always keeps it.
  - `Oracle::step` returns `Result`. In `corpus` the failure ends only its row (`corpus::RowError::Oracle`, printed `ID: failed: ...`); the next row runs. `movie` and the other one-run commands still stop on it.
  - Each lsnes gets its own configuration directory, `APPDATA=<run dir>/config` (lsnes keeps its config in `%APPDATA%/lsnes`), so processes started together no longer race on the shared one.
  - A record not announced within 60 s (the first step) or 30 s (later steps) kills that lsnes (its own child handle) and fails the row as silent. Drop asks lsnes to quit, waits up to 5 s, then kills it.
  - Checked (lane `lane/oracle-lsnes-robust`): 14 corpus processes started within 0.5 s, one chain-any row each, all started and completed (13 rows; chain-any-0011 has no faithful entry). Killing chain-any-0006's lsnes at step 475 failed that row with the log tails, and chain-any-0012 then ran its 1031 ticks in the same process. With `SM_LSNES=cmd.exe` (never answers) both rows failed as silent after 60 s each and the process exited 0.
- Resume: `Start::State` loads the state from the script (`OH_START`, `load-state`). On the command line lsnes rejects it (core type snes_ntsc against ROM type SNES) and hangs.
  - From seed 16 (frame 17829) the lag log matches in every column but `lagged`, which the log samples differently.
- `movie --from STATE FRAME` resumes there. `--native-seeds CSV DIR` can be repeated, with one save serving captures that share a frame. `--no-memories` drops the memories (lsnes only).
- Native seeds: `NNNN.lss` (the state before step N) and `NNNN.record` (the raw record of step N-1).
  - 16 are in `$TEMP/ohx/data/lsnes-native/any-boundary/` (private).
  - 17 to 19 (frames 18109, 19514, 19809) are in `$TEMP/ohx/nm/`, cut in a run that later hung; check them before use.
- `corpus --oracle lsnes --native-seeds DIR` maps `CAPTURE:INDEX` to `DIR/CAPTURE/INDEX.lss` (`Start::State`); `run_row_with` goes through `Backend` (`Observe::backend`, `Observe::native_seeds`).
  - Checked on chain-any-0001-DF45, -0006-E0B5 and -0012-91F8: they run to their tick count, at 39 frames/s.
  - wram, vram, cgram, oam and ppu differ, as under LakeSnes; sram matches.
- The 27 stalled rows all complete on lsnes native seeds (`9db02d2e3`; per-row table in MEASUREMENTS.md (e)): every row reaches the lock's tick count. sram equals the lock on 25 (not `credits-ending`, which saves, nor `file-copy-clear`); wram, vram, cgram, oam and ppu differ on all, as on the other native rows.
  - `death`: Samus dies (`--watch-words 0998,09C2`: energy 0 and game state `0x13` at frame 277, game over `0x1A` at 604), then the movie words take the game over menu, the title and the file select back to gameplay at 1771.
  - `file-copy-clear` runs its ticks but its authored part (after 1,800 death-row words) expects the game over menu, which lsnes left at frame 746; its presses pause the game instead, so no copy or clear happens. The recipe needs re-timing on the unpatched game.
    - Re-timed in `c58d7ccc9`: 600 death-row words, then its own presses; 2230 ticks (was 3612). On lsnes (`--watch-words 0998,0727,0952,19B5,19B7,19B9`): game over `0x1A` at 604, quit chosen at 813 (`$0727=7`), title at 847, file select `4` at 1100, copy (`$0727=0x0D`, slot A to B) at 1521, clear (`$0727=0x19`, slot A) at 2063, main file menu again at 2200. SRAM (a scratch per-slot sum, not committed): slot B takes slot A's bytes and checksum at 1522, slot A is zeroed at 2064. The title needs two A presses (the first skips its zoom; Start alone does nothing). The bundle's `authored-file-copy-clear.inputs` (SHA-256 `c9084e8f5a7cd83eb144fe9f17318609d3faf3181aecd17451f96081552f8cb6`) and the lock are the root's to replace.
- `corpus --watch-words ADDR,...` prints work-RAM words whenever they change, per row, to read an outcome off the oracle.

## Ladders (lane `lane/oracle-ladder`)

- `movie --ladder N DIR`, `ladder-check`, `ladder-rung` and `--from-ladder` (HARNESS.md, "Ladders"). A rung's record always carries the memories, also under `--no-memories`.
- Proved from native seed 16 (frame 17829) on any%, N=1000, `--no-memories`, 100 frames/s:
  - rungs 18000, 19000 and 20000 cut; the manifest's SHA-256 values equal `sha256sum` of the records, and a second cut gave the same records;
  - `ladder-check`: 18000 -> 19000 and 19000 -> 20000 equal, about 11 s each;
  - negative control: with pads 18100-18899 replaced by `0200`, 18000 -> 19000 is different (cut 5, cpu 9, wram 4525, vram 3244, cgram 4, oam 80 bytes) and the exit is 1;
  - native seed 18 (frame 19514) cut with `--from-ladder` (from rung 19000, 5 s) and from seed 16 directly (17 s): the records are equal, and equal to `$TEMP/ohx/nm/0018.record`. The `.lss` bytes differ, as expected, yet both states resumed give identical full records at 19515, 19520 and 19525.
- Full ladders: `$TEMP/ohx/build_ladders.sh 100|low|any [check]` (private), into `$TEMP/ohx/data/lsnes-ladder/<movie>/`, with N=10000, `--no-memories`, from power-on, wrapper `$TEMP/ohx/oh-ladder` (`oracle_ladder.dll`, built by `$TEMP/ohx/build-ladder.sh`). Rungs: 100 -> 22, low -> 16, any -> 12. Not run yet in this lane.

## Next, in order

0. Ladders: the root runs `build_ladders.sh 100`, `low` and `any` in parallel, then `build_ladders.sh <movie> check`. Then cut seeds with `--from-ladder` instead of from power-on (`cut_seeds.sh`), and add the nightly job (run whole with `--ladder` into a fresh directory, diff `ladder.txt`, `ladder-check` the stored ladder). The ladders are ROM-derived: a ci-data bundle is the root's call.

1. Done: the cut callback is on the NMI handler's RTI. Settled (`2da4d4ae6`, lane `lane/oracle-xreport`): the lsnes record's PC was one past the interrupted opcode (`$80:8344`/`$80:8347`, inside `LDA $05B4`/`BNE`); it now holds the stacked return address, as LakeSnes does. Ladders and native-seed records cut with older builds differ in record bytes 34-35: re-cut them with a build that has this commit before mixing them with new records (`ladder-check` compares raw records).
2. Done: the 13 captures are cut (`$TEMP/ohx/cut_seeds.sh`, private; seeds in `$TEMP/ohx/data/lsnes-native/<capture>/`, with `scenario-one-energy-crocomire/0142.lss` + `.words` from `oracle derive-seed`), and all 27 rows complete on them (MEASUREMENTS.md (e)).
   - Done: `file-copy-clear` re-timed (`c58d7ccc9`); its new inputs go in the bundle with the re-lock.
   - The 88 rows that stall under `--diagnose-kept-patches` are not re-run here; that is a diagnosis of the old lock, not needed for the lsnes rows.
3. Door lengths and busy spans from lsnes. Fix the DMA-under-blank count to check INIDISP per write.
4. Done, first version: `oracle cross ROM INPUTS --frames A..B [--ranges] [--ranges-at A..B]` (MEASUREMENTS.md (f)). On any% 0-4800 the game state agrees; the differences are power-on SRAM, the cut's definition, wait-loop NMI timing and sound timing, and lsnes runs one frame behind from 1342 (end of an APU upload). Next for the report:
   - normalise the definitional differences so they stop hiding real ones: cut both backends the same way when the NMI outlasts vblank (lsnes waits for the RTI, or LakeSnes cuts at its boundary), compare SRAM only after the game first writes it, and drop the stack page below S and the pad from the memory comparison;
   - align by game frame (`$05B8`) as well as by step, so that after 1342 the report measures content and not the lag;
   - resume both backends at a ladder rung (`--from` per backend; LakeSnes needs its own state at the rung), to cover 8314 (LakeSnes behind) and 8567 (the Ceres desync) within the 3-minute run limit;
   - then the re-lock (the root's call).
5. LakeSnes past 17985: the rate is lsnes's now, so try a sub-opcode SPC catch-up at port accesses (LakeSnes runs whole opcodes; bsnes v085 syncs per access) as fix 0003, measured on the same movie run (`$TEMP/olr/dump.rs` dumps the seven lag-log words per step; `$TEMP/olr/tail.py` classifies steps as equal, one behind, one ahead or other).
6. Speed: dump only the memories a step needs (the `memories` flag), or hash them in Lua.

Orphaned lsnes processes remain from hung or killed runs: PIDs 6176 and 14972 from the previous lane, 24536, 22000 and 25512 from this one. Stop-Process and taskkill could not end them from this shell.

## Pitfalls

- Fixed: lsnes processes started in the same second raced on `%APPDATA%/lsnes` (`FATAL: Configuration directory ... is not writable`, and no exit). Each now has its own config directory, and a hung start is killed after 60 s. A per-run config starts from lsnes's defaults (the shared `lsneswxw.cfg` held only defaults and audio devices).
- Builds older than this fix still share the config directory and still panic on a dead lsnes; don't start those in a burst.
- A killed lsnes usually shows as `took no request` (the next request's write fails before the read sees the closed pipe); both mean it is gone.
- A Tick is an NMI that interrupted a wait site (`game::WAIT_SITES`, which includes the message-box input loop), not a "polled" NMI.
- Seeds resume at `seed_boundary_pc` (`$80:8348`), not `pc`.
- Old-lock alignment: present pad k+1 until the k-th requested NMI.
- A Perl `s|..|..|` with `\|` in the pattern is alternation; use `$TEMP/ohx/py/rep.py`. Kill processes by PID only.

## Cleanup

- `$TEMP/ohx/data` (1.3 GB, ROM-derived) and `$TEMP/ohx/bundles` (328 MB): keep them for the next lane.
- `$TEMP/oracle-harness-target`: the build target; keep it while lanes build. `$TEMP/ohx/data/lsnes-native` (2 MB) and `$TEMP/ohx/nm`: keep.
- Kept lsnes run directories `%TEMP%/oracle-lsnes-*` (about 5 MB each, a ROM copy inside): delete them when done.
