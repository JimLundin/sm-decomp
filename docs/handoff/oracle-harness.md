# Handoff: oracle harness (2026-11-05)

Branch `research/oracle-harness` from main `52c21178d`; GATE OK `dc1b687ca`. Plan and findings: docs/oracle/HARNESS.md. Measurements: docs/oracle/MEASUREMENTS.md.

## What is true now

- `crates/oracle` depends only on `sm_data`, for the digest. It runs the unmodified ROM on upstream LakeSnes `9db90b8`, with no ROM patches.
  - One step is one video frame; the cut is the RTI of that frame's NMI.
  - Each record holds the memories, the CPU, `PpuRegisters`, the APU port log and the BGR555 picture.
  - Optional tiers add the SPC state, writes with line, dot and source, watched-PC hits and a profile.
- Fix 0001 (`native/fixes/`, the APU clock against the real master clock):
  - the any% movie matches the lsnes lag log exactly through frame 395;
  - state, room and position match through 8566;
  - the logic is one frame behind from 8314, and Ceres desyncs at about 8567.
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
  - An exit panics with the exit status and the tail of all three, and keeps the directory. `SM_LSNES_KEEP=1` always keeps it.
- Resume: `Start::State` loads the state from the script (`OH_START`, `load-state`). On the command line lsnes rejects it (core type snes_ntsc against ROM type SNES) and hangs.
  - From seed 16 (frame 17829) the lag log matches in every column but `lagged`, which the log samples differently.
- `movie --from STATE FRAME` resumes there. `--native-seeds CSV DIR` can be repeated, with one save serving captures that share a frame. `--no-memories` drops the memories (lsnes only).
- Native seeds: `NNNN.lss` (the state before step N) and `NNNN.record` (the raw record of step N-1).
  - 16 are in `$TEMP/ohx/data/lsnes-native/any-boundary/` (private).
  - 17 to 19 (frames 18109, 19514, 19809) are in `$TEMP/ohx/nm/`, cut in a run that later hung; check them before use.
- `corpus --oracle lsnes --native-seeds DIR` maps `CAPTURE:INDEX` to `DIR/CAPTURE/INDEX.lss` (`Start::State`); `run_row_with` goes through `Backend` (`Observe::backend`, `Observe::native_seeds`).
  - Checked on chain-any-0001-DF45, -0006-E0B5 and -0012-91F8: they run to their tick count, at 39 frames/s.
  - wram, vram, cgram, oam and ppu differ, as under LakeSnes; sram matches.
- The 27 stalled rows are not re-run: none of their 13 captures has a native seed yet (table in MEASUREMENTS.md; the cut is prepared, item 2).

## Next, in order

1. Done: the cut callback is on the NMI handler's RTI. Open question: the record's PC is one past the interrupted opcode (lsnes's exec-callback convention, kept for byte identity). Compare it with the LakeSnes record and settle it in the cross-oracle report (item 4).
2. Cut the 13 captures the 27 stalled rows need: prepared, not yet run. `$TEMP/ohx/cut_seeds.sh 100|low|any` (private) runs `movie --oracle lsnes --no-memories` with one `--native-seeds` per capture. The CSVs are `movies/room-seeds-<capture>.csv` filtered to the needed rows (`$TEMP/ohx/cut-<capture>.csv`). The three runs are independent:

   | Run | Movie | Captures (indices) | Start | `--frames` | Time at 81 frames/s |
   | --- | --- | --- | --- | --- | --- |
   | `100` | `100.inputs` | `100-input-phase` (27, 104, 142, 305, 324), `100-mb-phases` (324, 325, 329), `100-boundary` (259), `100-credits` (336) | power-on | 222658 | about 46 min |
   | `low` | `low.inputs` | `low-input-phase` (129) | power-on | 100732 | about 21 min |
   | `any` | `any.inputs` | `any-boundary` (110, 190) | native seed 16, frame 17829 | 116360 | about 20 min |

   - A seed at movie frame F is saved after step F-1 and copied one step later, so each run stops at the last F plus 1.
   - Seeds go to `$TEMP/ohx/data/lsnes-native/<capture>/NNNN.lss`; logs go to `$TEMP/ohx/cut-<run>.log`.
   - Success: one `native seed` line per seed (10, 1, 2; frame 209945 serves two captures), then `frames ... in`, then `cut_seeds <run> exit 0`.
   - Proof: cutting `any-boundary:17` (frame 18109) from seed 16 ran at 81 frames/s. `corpus --oracle lsnes --native-seeds` then ran `chain-any-0017-9E9F` from it to its 592 ticks, with sram equal and the other streams differing, as on the other native rows.
   - `scenario-one-energy-crocomire:142` is `100-input-phase:142` (movie frame 99412) with energy `$09C2` = 1 and reserve energy `$09D6` = 0 (`tools/scenario_seeds.py`). The `100` run cuts the base. The edit needs a WRAM write request in `lsnes_server.lua`, plus a way to apply it before the save or after the load. Neither exists yet. Its rows are `death` and `file-copy-clear`.
   - Then run `corpus --oracle lsnes --native-seeds DIR --only ...` on the 27 rows (88 under the diagnosis).
3. Door lengths and busy spans from lsnes. Fix the DMA-under-blank count to check INIDISP per write.
4. A cross-oracle report, LakeSnes against lsnes, per stream per frame. Then the re-lock (the root's call).
5. Speed: dump only the memories a step needs (the `memories` flag), or hash them in Lua.

Orphaned lsnes processes remain from hung or killed runs: PIDs 6176 and 14972 from the previous lane, 24536, 22000 and 25512 from this one. Stop-Process and taskkill could not end them from this shell.

## Pitfalls

- A Tick is an NMI that interrupted a wait site (`game::WAIT_SITES`, which includes the message-box input loop), not a "polled" NMI.
- Seeds resume at `seed_boundary_pc` (`$80:8348`), not `pc`.
- Old-lock alignment: present pad k+1 until the k-th requested NMI.
- A Perl `s|..|..|` with `\|` in the pattern is alternation; use `$TEMP/ohx/py/rep.py`. Kill processes by PID only.

## Cleanup

- `$TEMP/ohx/data` (1.3 GB, ROM-derived) and `$TEMP/ohx/bundles` (328 MB): keep them for the next lane.
- `$TEMP/oracle-harness-target`: the build target; keep it while lanes build. `$TEMP/ohx/data/lsnes-native` (2 MB) and `$TEMP/ohx/nm`: keep.
- Kept lsnes run directories `%TEMP%/oracle-lsnes-*` (about 5 MB each, a ROM copy inside): delete them when done.
