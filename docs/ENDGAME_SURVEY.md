# Endgame survey: running without `sm`

What `sm_game` plus `sm_runtime` (with `sm_app` as the device shell) would
need to run the game with no `sm` crate and no emulated SNES. Surveyed on
`land/batch13` (3ba4533a9) on 2026-10-31. This is a survey, not a plan:
[REVIEW](REVIEW.md) owns the milestone order and
[the accepted destination](target-architecture/README.md) the design.

## Where a frame runs today

`sm_app` calls `sm::rtl::rtl_run_frame`. That calls
`cpu_infra::rtl_run_frame_compare`, which in native mode (`RM_MINE`) runs
`sm_game::gameplay_frame::advance(Entry::Frame, game)` with `Game` as the host,
then `draw_frame_to_ppu`. The gameplay step is already in `sm_game`. Around it,
`sm` supplies:

- the RAM image;
- the emulated PPU, DMA/HDMA and beam timing;
- SPC700 audio;
- ROM access and patching;
- input latching;
- the recorder and snapshots.

`sm/src` outside `verification/` is about 34k lines:

| Area | Lines | Notes |
| --- | --- | --- |
| `sm/src/snes/` (CPU, PPU, DMA, SPC, DSP, cart) | 4.4k | |
| `sm/src/game/` | 12.4k | 51 files, almost all host adapters and Runtime glue |
| `game_state.rs` | 2.1k | |
| `cpu_infra.rs` | 1.7k | |
| `rtl.rs` | 1.5k | |
| `spc_player.rs` | 2.3k | |
| `verify*.rs` | 4.5k | |

## Emulator- or verifier-only (not blockers)

These exist to compare against the original and can stay in a development
oracle (target `sm_tools`). They don't block a native build:

- `verify.rs`, `verify_dll.rs`, `verify/`, `verification/`, `verify_probe.rs`;
- `tracing.rs`, `oracle_mode.rs`, `oracle_patch_policy.rs`;
- the `RM_THEIRS`/`RM_BOTH` paths in `cpu_infra.rs`: `run_one_frame_of_game_emulated`, `_both` and the RunAsmCode hooks;
- `snes/cpu.rs`;
- `recording.rs` and the replay half of `rtl.rs`;
- `rom_patches.rs`, which patches the ROM for the oracle.

## Blockers, biggest first

| # | Blocker | What it means | Suggested owner |
| --- | --- | --- | --- |
| 1 | **The RAM image is the state** | `Game`'s `WorkRam` (`game_state.rs`) is still where much of the game's state lives. `sm_game` reaches it through `CompatibilityRam`/`RawRam` in about 60 files, and `cycle_report` counts 3,534 RAM sites in 101 coupled files. A native build needs those fields in the owned `Session`, with no `Game`. | Domain lanes (one per subsystem cohort), tracked by the RAM ratchet |
| 2 | **Video goes through the emulated PPU** | `draw_frame_to_ppu` settles the frame on the emulated beam. VRAM, CGRAM and OAM transfers and register writes (`display_transfers.rs`, `display_refresh.rs`, `rtl` `WriteReg`, the `Registers` trait in 13 `sm_game` files) land in `snes/ppu.rs`. `sm_runtime`'s scanline renderer reads that PPU state. A native build needs Scene → `sm_runtime` renderer, with the PPU-state inputs owned by `sm_game`/`sm_runtime`. | Rendering lane (the `sm_runtime` owner) |
| 3 | **Beam, NMI and IRQ timing** | The frame settlement, scanline IRQs (`scanline_irq.rs`) and HDMA (`hdma*` in `sm_game`, executed by `snes/dma.rs`) depend on emulated `hPos`/`vPos` and `nmiWanted`. Mid-frame effects (HUD split, Draygon and Phantoon waves, room-loading IRQ commands) need a native per-scanline effect model in the renderer. | Rendering lane, with the HDMA/IRQ owners |
| 4 | **Audio is the SPC700** | `spc_player.rs`, `native_audio.rs`, `audio_*`, `apu_*` and `snes/spc.rs`/`dsp.rs` are about 4k lines. `sm_game` only queues cues (`queue_sound`/`queue_music` in 90 files). A native build needs the `sm_runtime` sequencer, voices and mixer, driven by the cue queue, plus `cpal` output in `sm_app`. | Audio lane (O, if the oracle work frees up) |
| 5 | **ROM reads at run time** | `rtl::rom_record`, `rom_table_word!` and native records (`native_rom.rs`, `rom.rs`, `Records`/`LoadedImage` in `sm_game`; table reads in 64 `sm_game` files) still read the cartridge image while playing. A native build needs prepared `sm_data` records loaded once (an `sm_tools` importer), with no ROM image in `Game`. | Data lane (the `sm_data` owner) |
| 6 | **The frame driver and host shell** | `rtl_run_frame` holds the bug-fix counter, the recorder, Samus mutation adoption, Session native adoption and audio input bracketing. `host.rs` and the `native_*.c` desktop, config and shader code (built with zig) do the rest. The device shell needs a Rust tick loop in `sm_app`/`sm_runtime` that calls `gameplay_frame::advance` on the Session, plus `winit`/`wgpu` replacing SDL2/GL. | Root/app lane |
| 7 | **The compat Runtime lease** | `sm_game` reaches most owned state through `LeaseActors::lease()`, which is `sm`'s `Runtime::new(game)` (import, export and preflight per lease). Botwoon's death signals and the boss `owned_call` routing are Runtime-only. A native build needs `sm_game` to own the Session directly, so leases become plain borrows. | G (actors), with the Session owners |
| 8 | **Host-trait implementations in `sm`** | Dozens of `sm_game` host traits are implemented on `Game` in `sm/src/game/` (`hosts.rs`, `species_host.rs`, `*_runtime.rs`, `menu_host.rs`, `room_state.rs`, `room_entry/compatibility.rs`). Each needs an `sm_game`-side implementation over the Session or `sm_runtime`. Most are thin, but there are many. | Per-domain lanes: E (menus, room state), H (Samus), G (actors) |
| 9 | **Save data and persistence** | SRAM saves (`save_ram.rs`), legacy snapshots (`snapshot_io.rs`, `rtl` save/load) and `persistent_session.rs` live in `sm`. `sm_game` has `save_file.rs` and the session checkpoint format. A native build needs save files written from the Session through `sm_runtime` storage. | E (menus/save) |
| 10 | **Remaining CPU-executed gameplay** | Bang's main AI still calls ROM code via `cpu_infra::call_routine`, which is the only remaining caller. It needs a native port before the CPU can go. | G |

Inputs are close: `sm_game::external_inputs` exists, and `rtl_run_frame`
only latches `input1.currentState` for the frame.

## Suggested order

1. Items 1 and 7 together. The Session owns state, and leases become borrows. Everything else depends on this.
2. Item 5, so no ROM image is needed at run time.
3. Items 2–4: video, timing and audio in `sm_runtime`.
4. Items 6, 8 and 9: the shell, the hosts and saves.
5. Item 10 at any point.

The verifier keeps the emulated path as the comparison until each slice has
replacement evidence, as the migration policy requires.
