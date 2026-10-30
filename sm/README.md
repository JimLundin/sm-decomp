# sm (compat)

The compatibility crate: everything that still needs the SNES machine model.
It holds game code not yet moved to `sm_game`, the RAM-compatible runtime the
game runs on, the emulated hardware and the oracle used for verification.
It only shrinks and gets no new features.

| Path | Contents |
| --- | --- |
| `src/game/` | Game code not yet migrated, including the legacy bank files `sm_*.rs`; `Host` implementations for moved `sm_game` modules; re-export shims at old paths |
| `src/game_state.rs` | `Game`: per-game ownership of the RAM image, kept in its exact SNES layout |
| `src/ram_map.rs` | Names for work RAM addresses |
| `src/rtl.rs`, `src/execution.rs`, `src/host.rs` | Runtime support, backend execution intervals and frontend callbacks |
| `src/snes/` | Emulated hardware: the 65816 CPU (LakeSnes, native C), PPU (`ppu.rs`, with the LakeSnes C core in `backend.c`), DMA, APU, SPC700 and DSP |
| `src/spc_player.rs`, `src/native_audio.rs`, `src/apu_*.rs`, `src/audio_*.rs` | The music and sound-effect sequencer and the APU upload and output paths |
| `src/cpu_infra.rs` | Paired-frame execution of ROM and Rust, and the per-frame comparison |
| `src/verify.rs`, `src/verify_dll.rs`, `src/verify/` | The headless verifier DLL loaded by `tools/verify_dll.py`, and checkpoints |
| `src/verification/` | Verification-only checks registered with the verifier, and the detached `player_detached` example run in CI |
| `src/native_*.c`, `native/stb_image.c` | Native configuration, utility, shader and desktop code from snesrev/zelda3, and the PNG decoder bridge |
| `src/room_pack.rs` | Room-pack bytes and command-line selection |

`tools/cycle_report.py` measures how much of `src/game/` is still tied to
`game_state`, `ram_map`, `rtl`, `cpu_infra` and `snes/`. See the
[verification guide](../movies/README.md) for running the verifier and
[native dependencies](../docs/NATIVE_DEPENDENCIES.md) for the native archives.
