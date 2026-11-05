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
- Speed: about 300 frames/s for one process with the picture on, about 950 in total for 10 processes on 14 cores. lsnes rr2-beta23 with Lua plays any% at about 58 frames/s (10,000 frames in 174 s).
- Not done:
  - DMA bytes under forced blank are only counted at the cut, which is wrong;
  - door lengths from seeds are invalid, because those rows stall;
  - no lsnes or Mesen backend;
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

## lsnes out of process (learned so far)

- `lsnes-bsnes.exe` (rr2-beta23) is in `%LOCALAPPDATA%/sm-decomp-experiments/20261025/tas-alignment/` and runs here.
  - Launch it with PowerShell `Start-Process -WindowStyle Hidden -Wait`.
  - Pass the movie positionally, plus `--rom-a=`, `--lua=` and `--unpause`.
- Its Lua has `io`, `os.getenv`, `on_frame_emulated`, `memory.*` and `exec("quit-emulator")` (`$TEMP/ohx/lsnes/speed.lua`).
- `tools/lsnes_lag_log.lua` shows `registerexec`, `getregisters` and `readregion`. Those give the cut (exec hooks on the NMI handler's RTI), the interrupted PC and the memories.

## Next, in order

1. An lsnes backend behind `Oracle`, as a GPL separate process: a Lua server that reads pads from a pipe or file and writes each frame's record. Prove that any.inputs syncs, and measure its frames/s.
2. Harness-native seeds: the emulator's state plus the raw record, cut on the syncing core at the room-seed frames. Re-run the 27 stalled rows (88 under the diagnosis).
3. Door lengths and busy spans from the syncing core. Fix the DMA-under-blank count to check INIDISP per write.
4. A cross-oracle report, LakeSnes against lsnes, per stream per frame.
5. Re-lock with `corpus --record` (the root's call). Proposed CI job: Zig, `build_lakesnes.sh --target x86_64-linux-gnu`, `cargo build -p oracle`.

## Pitfalls

- A Tick is an NMI that interrupted a wait site (`game::WAIT_SITES`, which includes the message-box input loop), not a "polled" NMI.
- Seeds resume at `seed_boundary_pc` (`$80:8348`), not `pc`.
- Old-lock alignment: present pad k+1 until the k-th requested NMI.
- A Perl `s|..|..|` with `\|` in the pattern is alternation; use `$TEMP/ohx/py/rep.py`. Kill processes by PID only.

## Cleanup

- `$TEMP/ohx/data` (1.3 GB, ROM-derived) and `$TEMP/ohx/bundles` (328 MB): keep them for the next lane.
- `$TEMP/oracle-harness-target` (408 MB): safe to delete. The oldwt worktree is removed.
