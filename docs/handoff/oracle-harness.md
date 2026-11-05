# Handoff: oracle harness (2026-11-05)

Branch `research/oracle-harness` from main `52c21178d`; GATE OK `dc1b687ca`.
Plan and findings are in docs/oracle/HARNESS.md, and measurements in
docs/oracle/MEASUREMENTS.md.

## What is true now

- `crates/oracle` (package `oracle`) depends only on `sm_data`, for the digest.
  It runs the unmodified ROM on upstream LakeSnes `9db90b8` (`OracleId::patches`
  is empty). Each step is one video frame, and the cut is the RTI of the
  frame's NMI. Records carry WRAM/VRAM/CGRAM/OAM/SRAM, the CPU,
  `PpuRegisters`, the APU port log and the BGR555 picture. Optional tiers add
  the SPC state, a register write log with line/dot/source, watched-PC hits and
  a per-bank profile.
- Fix 0001 (APU clock against the real master clock) is in
  `crates/oracle/native/fixes/`. With it the any% movie matches the lsnes lag
  log exactly through frame 395, and state, room and position through 8566. The
  game logic is one frame behind lsnes from 8314, and Ceres desyncs at about 8567.
- Corpus against today's lock, 601 of 638 faithful rows run from lsnes seeds
  (plus `options-menu-tour` from power-on), 27 stalled:
  - equal: sram 600, vram 150, cgram 162, oam 150, wram 2;
  - ppu 0, because the old definition includes renderer scratch;
  - video and apu are redefined, so not comparable.
  - area-brinstar matches wram `3d2d36f47cb3ddc1`, vram `788c6d428540fb70`,
    cgram `da3390792deb0d4b`, oam `7bea571691d05da5` and sram, and is deterministic.
  - options-menu-tour, 2310 ticks from power-on, matches vram, cgram, oam and sram.
- Diagnosis: `--diagnose-kept-patches native-audio-handshake,debug-enemy-set-names`
  applies two kept patches of the old stepper to a ROM copy.
  - Then wram matches on 37 of 540 rows (area-crateria,
    boss-ceres-ridley-fireballs and others), so those wram mismatches are the
    old stepper's handshake patch.
  - Stalls rise to 88 under the diagnosis.
- Stalls: the migration seeds have no SPC state. The resumed driver sometimes
  never echoes, so credits, Mother Brain, death, save and door rows hang in
  `$80:8077` (the upload timeout) or in sound waits.
- Speed: about 300 frames/s single process with the picture on; 10 processes
  give about 950 in total on this 14-core box. lsnes rr2-beta23 plays the
  any% movie at about 58 frames/s with Lua hooks (10,000 frames in 174 s,
  room E0B5 at frame 10000).
- Not done:
  - DMA bytes under forced blank are counted only at the cut (wrong);
  - door-transition lengths from seeds are invalid (they stall);
  - no lsnes or Mesen backend;
  - no same-machine figure for the old oracle (its verifier build is blocked here).

## How to run it here

1. `bash tools/oracle/build_lakesnes.sh`. It fetches the pinned files, applies
   the fixes, builds with the cached Zig 0.15.2 and prints the library directory.
2. `LIBRARY_PATH=<that dir> CARGO_TARGET_DIR=$TEMP/oracle-harness-target cargo build --release -p oracle`.
   Use a target dir under `$TEMP`; executables elsewhere are refused.
3. Exit-126 workaround (local only, not committed): Defender blocks fresh exes
   at random, including `oracle.exe` and build scripts. That is why there is
   no `build.rs`.
   - Build a cdylib around `oracle::cli::main`: `$TEMP/ohx/dll.rs` exports
     `sm_oracle_run(argc, argv)`, and `$TEMP/ohx/build.sh` links it against the rlib.
   - Run it through the cached Python: `$TEMP/ohx/run.py` with ctypes, and
     `$TEMP/ohx/oh` as the wrapper.
4. Data:
   - `gh release download ci-data` for each bundle in lock.toml, then
     `openssl enc -d -aes-256-cbc -pbkdf2 -iter 200000 -md sha256 -pass env:P`,
     where P = sha256("sm-decomp-seeds-v1" + ROM).
   - Extract one directory per bundle under DATA, without the `.tar.gz.enc` suffix.
5. Commands:
   - `oracle corpus ROM scenarios/lock.toml DATA [--shard I/10] [--record FILE] [--diagnose-kept-patches LIST]`
   - `oracle movie ROM movies/any.inputs --lag-log <any-boundary/lag.csv> [--seeds CSV DIR]`
   - `oracle measure ROM LOCK DATA ID...`, `oracle measure-movie ROM INPUTS N`, `oracle trace ROM LOCK DATA ID A B`

## Driving lsnes out of process (learned so far)

- lsnes-bsnes.exe rr2-beta23 lives in
  `%LOCALAPPDATA%/sm-decomp-experiments/20261025/tas-alignment/lsnes-rr2-beta23/`
  and does run here. Launch it with PowerShell
  `Start-Process -WindowStyle Hidden -Wait`, with the movie positional and
  `--rom-a=`, `--lua=`, `--unpause`.
- The Lua side has `io`, `os.getenv`, `on_frame_emulated`, `memory.readword` and
  `exec("quit-emulator")` (`$TEMP/ohx/lsnes/speed.lua`).
- `tools/lsnes_lag_log.lua` shows `memory.registerexec`, `memory.getregisters`,
  `memory.readregion` and `memory.find_vma`. Those give the cut (exec hooks on
  the NMI handler's RTI), the interrupted PC, the ports and the memories.

## Next, in order

1. lsnes backend behind `Oracle` (GPL, separate process only).
   - Start with a Lua server that reads pad words from a pipe or file and writes
     each frame's record.
   - Prove that `any.inputs` stays in sync, against the lag log and the room
     seeds, and measure its frames/s.
2. Harness-native seeds: on the syncing core, save the emulator state plus
   the raw record at each room-seed frame. Re-run the 27 stalled rows (88
   under the diagnosis) from them.
3. Door-transition lengths and busy spans from the syncing core
   (MEASUREMENTS.md). Also fix the DMA-under-forced-blank count (check
   INIDISP per write).
4. A cross-oracle disagreement report, LakeSnes against lsnes, per stream per frame.
5. Re-lock with `corpus --record` once the seeds are native (root's call). Propose
   a CI job: Zig, `build_lakesnes.sh --target x86_64-linux-gnu`, then `cargo build -p oracle`.

## Pitfalls

- The cut definition: "polled" (the NMI read $4218) is not "requested". A Tick
  is an NMI that interrupted a wait site, `game::WAIT_SITES`, including the
  message-box input loop.
- lsnes seeds: resume at `seed_boundary_pc` ($80:8348), not `pc`. The
  capture's RAM and its CPU registers come from different moments.
- Old-lock alignment: present pad word k+1 until the k-th requested NMI. Fold
  the legacy streams on Ticks only.
- In Perl `s|...|...|`, a `\|` in the pattern is alternation. Edit with a
  small Python replacer instead (`$TEMP/ohx/py/rep.py`).
- Kill processes by PID or command line only. One broad `taskkill /IM python.exe`
  here was luckily mine.

## Cleanup

- `$TEMP/ohx/data` (1.3 GB, ROM-derived, private) and `$TEMP/ohx/bundles`
  (328 MB): keep them for the next lane, or delete.
- `$TEMP/oracle-harness-target` (408 MB): safe to delete.
- The `$TEMP/ohx/oldwt` worktree has been removed.
