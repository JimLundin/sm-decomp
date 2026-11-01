# Lane O handoff (2026-10-31)

## Audio architecture now (land/batch13 through 74657fef8)
- `sm_runtime::audio`: `ApuRam` (ram.rs) is the 64 KiB APU RAM, `Rc<[Cell<u8>]>`. `dsp` is the S-DSP: BRR voices,
  envelopes, Gaussian interpolation, mixer, noise and echo FIR, with snake_case fields.
  `driver` is the native SPC driver: NativePlayerState/Channel/SfxChan/SfxRegs/DspRegWriteHistory,
  the SPC RAM mirror (save_variables/load_variables), install_bank/transfer, and NativePlayer and its sequencer.
- NativePlayer owns { state, ram, dsp, history: Option<_>, probe: fn(&str) }. All of its methods are safe:
  initialize, render_samples, apply_ports, save_state/load_state, upload(read, source), process_sound_effects.
- `sm`'s Game holds `native_player: Option<NativePlayer>`; native_player_mut() panics if it isn't created.
  create_native_player sets probe = verify_probe::hit. There is no g_spc_player or raw pointer any more.
- sm re-exports the moved items: sm::spc_player::*, sm::types::{NativePlayerState,..}, sm::audio_upload::{install_bank, transfer},
  sm::apu_ram::ApuRam, sm::snes::dsp::{Dsp, DspState, ..}, plus pointer wrappers dsp_reset/dsp_cycle/dsp_write/dsp_saveload.
- Snapshot/checkpoint formats store the legacy field names. sm_runtime's `state_record!` and the checkpoint `fields!` both have a
  `named { field = "legacyName" }` form. The DspState/DspChannel records live in sm_runtime::audio::dsp,
  and ApuRam's in ram.rs (orphan rule). Keep the names, or old checkpoints and snapshots break.

## Still in sm
- The emulated SPC700/APU, used by the oracle only: snes/spc.rs and snes/apu.rs, through the sm dsp wrappers.
- apu_queue.rs (the frame port queue; `queued()` iterates entries oldest first), recording and replay, audio_output.rs (cue/port trace).
- audio_snapshot.rs (PlayerSnapshot/AudioSnapshot; capture_from/restore_into), the checkpoint records in verify/checkpoint.rs.
- audio_upload.rs physical upload control (prepare_physical/physical), native_audio.rs (cue resolution, upload_bank).
- Next steps: a host trait for the cue queue → driver, the sm_app cpal output, then retiring g_use_my_apu_code's
  emulated-SPC branches from the native path. Ask root before touching the oracle's physical path.

## Digest check (native PCM and APU RAM, per frame)
- The verifier doesn't compare native audio. Use the local-only probe: `$TEMP/collab/laneO/pcm_probe2.py` patches sm/src/verify.rs
  so that with SM_LANE_O_PCM it prints `PCMHASH frame pcm apu` (FNV-1a) every 60 frames. Don't commit it.
- `bash $TEMP/collab/laneO/digest.sh BASE_REV ROWS...` checks out BASE_REV in worktree
  `%LOCALAPPDATA%/sm-decomp-experiments/20261030/laneO-audio-base`, probes base and laneO-audio, builds both
  (v100/build_here.sh), runs `pcm_rows.sh` on the chain rows, prints `same`/`DIFF` per row and reverts the probe.
  Commit the tip first. Rows 1, 6, 90, 245 take about 10 min; 324 is a 10k-frame escape and takes about 10 min more.
- Unit tests: `verify_dll.py --unit-tests --direct` with LIBRARY_PATH = the lib dir build_here.sh prints. 147 pass.
- Chain sweep: `bash $TEMP/collab/laneO/chain_all.sh WORKTREE LIBDIR OUTDIR [faithful]`, 331 rows, about 1.5 h at -P 2.
- Seeded corpus rows locally: `box/prep_seeded.sh WORKTREE` imports their movie inputs and seeds (28 rows).
  The authored/checkpoint rows need CI.

## Oracle-fidelity findings (details in the commits and tools/snesrev_patches.py)
- Faithful mode drops snesrev patches per site, by policy, with flip groups enforced by oracle_patch_policy.py --check.
- Row 6: Ceres Ridley's carry chain. Row 324: the sound-clear delay is 2, not snesrev's 6 ($82:8A67).
- Puyo $A2:9BC1 stores the word at $01, which the HUD leaves as $009D (Session PointerScratch $00-$03).
- Message box: hardware reads the pad in the reading delay's last frame. $85:9650 now reads first, and the native dialogue
  runner does too (30b222c4d). This fixed the one-frame-late box close against the lsnes capture.
- $1790 is the RTL written by the enemy load (77418c1d4). normalize_snapshot_ram's remaining 31 ranges are listed
  in docs/LEGACY.md (LEGACY(verifier)).
- The Cue/port trace runs today: segments.yml has an audio_trace dispatch input (f2e951ff0, on main). Checkpoint rows rebuild
  from boot under physical audio and may hit their 60-min limit.

## Traps
- Heredocs collapse backslashes. Write regex or escape-heavy scripts with the Write tool. Python print on Windows writes CRLF,
  and Git Bash grep hides the \r (it bit a `read` loop).
- An empty `SM_ORACLE=` is not the default mode. Unset it.
- build_here.sh caches native libs by a hash that includes sm/src/types.rs, so editing types.rs rebuilds the native lib.
- After a source edit, `--no-build` refuses a stale DLL and prints nothing. Rebuild.
- The gate (`$TEMP/collab/gate.sh`) can fail on other lanes' stale mutation anchors; rerun the anchor script on its own to tell.
- sm_runtime denies warnings, including clippy. Port code with `cargo clippy --fix -p sm_runtime`, then fix the rest by hand.
  No allows.
- sm_runtime can't depend on sm_game, so probe hits go through `probe: fn(&str)`. The orphan rule means sm can't impl
  sm_runtime traits for sm_runtime types.
- Worktrees (each with worktree core.autocrlf false): laneO-audio, laneO-audio-base (digests), laneO-1790, laneO-ci, laneO-puyo, laneO-fix, laneO-spc, laneO (stale).
