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
  - lsnes exited at step 18068, cause unknown (its stderr is not captured). Speed: 45 frames/s with all memories dumped every frame.
- Native seeds: `--native-seeds CSV DIR` writes `NNNN.lss` (the state before step N) and `NNNN.record` (the raw record of step N-1, the capture's cut). 16 are in `$TEMP/ohx/data/lsnes-native/any-boundary/` (private).
- Not done: `corpus` takes neither `--oracle lsnes` nor native seeds, so no stalled row has been re-run; door lengths from this core are not measured.

## Next, in order

1. Fix lsnes's exit at step 18068: keep its stderr and log Lua errors to a file.
2. `corpus --oracle lsnes --native-seeds DIR`: map `CAPTURE:INDEX` to `DIR/CAPTURE/INDEX.lss` and start with `Start::State`.
   - `run_row_with` must use `Backend`. Then re-run the 27 stalled rows (88 under the diagnosis).
   - Cut seeds for `100-input-phase`, `100-mb-phases` and credits from `movies/100.inputs`, with their CSVs. Cut any-boundary seeds past 17829.
3. Door lengths and busy spans from lsnes. Fix the DMA-under-blank count to check INIDISP per write.
4. A cross-oracle report, LakeSnes against lsnes, per stream per frame. Then the re-lock (the root's call).
5. Speed: dump only the memories a step needs (the `memories` flag), or hash them in Lua.

## Pitfalls

- A Tick is an NMI that interrupted a wait site (`game::WAIT_SITES`, which includes the message-box input loop), not a "polled" NMI.
- Seeds resume at `seed_boundary_pc` (`$80:8348`), not `pc`.
- Old-lock alignment: present pad k+1 until the k-th requested NMI.
- A Perl `s|..|..|` with `\|` in the pattern is alternation; use `$TEMP/ohx/py/rep.py`. Kill processes by PID only.

## Cleanup

- `$TEMP/ohx/data` (1.3 GB, ROM-derived) and `$TEMP/ohx/bundles` (328 MB): keep them for the next lane.
- `$TEMP/oracle-harness-target`: deleted at the end of this lane. `$TEMP/ohx/data/lsnes-native` (2 MB): keep.
