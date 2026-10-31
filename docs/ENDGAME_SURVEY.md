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

Inputs are close: `sm_game::external_inputs` exists, and the frame takes
controller 1 as `gameplay_frame::TickInput` (see item 6 below).

### Item 5: what still reads the ROM (2026-10-31)

Run-time reads fall in three families. Counts are call sites in `sm_game`.

1. **Fixed tables.** These are `rom_table_word!`, `Content::table` and table
   pointers indexed by suit, frame, area or angle (about 60 sites). Each one
   becomes an `sm_data` constant pair (address, length), imported once in
   `register_rom` through `ImportedWords`/`ImportedGrid` and held in
   `RomRecords`. An index past the table, or an image other than the imported
   allocation (a patched oracle ROM), keeps the original load, so behaviour
   stays exact. About 40 tables are converted this way already (the
   `*_tables.rs`, `menu_lists`, `boss_lists` and `gameplay_tables` modules;
   their remaining "reads" are those fallbacks). Samus's palette pointer
   tables are the latest. Still raw: Samus's pose table ($91:B629, bytes per
   pose), the arm cannon offsets ($90:C7DF, $90:C7A5), the beam trail
   ($90:C3B1), the atmosphere table ($90:8BFF), demo inputs ($91:8000), sprite
   object tables ($B4:BDA8) and the HDMA wave ($88:EA8B).
2. **Pointer-chased content.** These are `native_record`, `Content::record`
   and `RomRecord` at an address taken from RAM or another table (about 220
   sites). They read palettes ($9B), spritemaps, tilemaps, room headers, map
   data, and the instruction lists of PLMs, enemies, sprite objects, palette
   effects, animated tiles and HDMA objects. Fixed tables cannot cover these.
   They need typed content keyed by the original address, such as a palette
   bank (`address → [u16; 16]`) or a decoded instruction-list arena per
   interpreter, imported once. The biggest consumers are `actors` (138 sites),
   `menus` (69), `player_projectiles` (33), `animated_tiles` and
   `palette_effects` (14 and 13). Each interpreter's lists are one
   conversion, and that conversion should replace the per-step decoder rather
   than wrap it.
3. **Image-identity hosts.** `native_image()` (42 sites) and `loaded_image()`
   (76 sites) mostly pass the image to families 1 and 2, or check which
   allocation is loaded. They go away with the two families above.

A native build therefore needs family 2 first. Family 1 is cheap and nearly
done, and family 3 follows.

### Item 6: the frame driver (2026-10-31)

`sm_app` calls `rtl_run_frame(inputs, game)` once per frame. Around the
gameplay step, it still does these things in `sm`:

| Step | Where | What a native driver does instead |
| --- | --- | --- |
| Refuse CPU-mode frames for a prepared world; check Samus's binding (`preflight_samus_runtime`) | `rtl.rs` | The Session opens only on a validated binding (done: `scene::samus_binding`) |
| Open a Samus mutation interval, then adopt its result (`samus_mutation`, `finish_current`) | `rtl.rs`, `cpu_infra.rs` | Not needed once nothing outside the Session mutates the ROM or RAM |
| Cancel opposite d-pad directions (`0x30`, `0xc0`) unless verifying | `rtl.rs` | Host input policy: `sm_app` maps devices to `TickInput` |
| Replay or record the input word, snapshot flags | `rtl.rs` (`StateRecorder`) | `sm_app`'s input journal (CONTRACTS: Tick admission) |
| The bug-fix counter at `$C2` and its ROM patches | `rtl.rs` | Removed: snesrev patch selection, not gameplay |
| Bracket the frame's audio input (`begin_input`/`complete_input`), push the APU queue | `rtl.rs` | `TickOutput` cues to the `sm_runtime` sequencer |
| Run the frame: latch controller 1, `gameplay_frame::advance(Entry::Frame)` on `Game`, then `draw_frame_to_ppu` | `cpu_infra.rs` (`RM_MINE`) | `gameplay_frame::tick(input, session)`, then Scene to the `sm_runtime` renderer |
| Reconcile Session native adoption (`reconcile_current`) | `session_native_adoption.rs` | Not needed once Game holds no Session copy |
| Present the PPU frame (`sm_desktop_DrawPpuFrameWithPerf`) and read SDL input | `sm_app` with the `native_*.c` shell | `winit`/`wgpu` presenting the Scene |

The input side is small: the frame reads controller 1 only through
`menus::host::Host::nmi_controller_word`, the joypad register in NMI, and
the driver sets nothing else. The output side is the hard part. Display is
items 2–3 (the emulated PPU and beam), and audio is item 4 (the SPC700).

**Plan.**

1. Done in this change: `gameplay_frame::TickInput { buttons }` and
   `gameplay_frame::tick`. The tick latches controller 1 through
   `Host::latch_controller`, then runs the frame. Both of `sm`'s native
   frame paths (`RM_MINE` and the verifier's both-mode) call it.
2. G: one frame-level Runtime lease spans the whole tick, so the per-family
   lease borrows become borrows of that one lease (item 7). This starts
   after step 1 and G's family conversions.
3. Move the recorder, the opposite-direction policy and the input journal
   into `sm_app`, which hands `rtl_run_frame` a `TickInput`. The bug-fix
   counter stays oracle-only.
4. Return a `TickOutput` from `tick`: the frame's queued sound cues (already
   Session state) and the owned Display's Scene. `sm_app` then drives
   `sm_runtime` with them. This depends on items 2–4.
5. With items 1, 5 and 7 done, `tick` runs on the Session with no `Game`,
   and the Samus mutation and native adoption steps go away.

## Suggested order

1. Items 1 and 7 together. The Session owns state, and leases become borrows. Everything else depends on this.
2. Item 5, so no ROM image is needed at run time.
3. Items 2–4: video, timing and audio in `sm_runtime`.
4. Items 6, 8 and 9: the shell, the hosts and saves.
5. Item 10 at any point.

The verifier keeps the emulated path as the comparison until each slice has
replacement evidence, as the migration policy requires.
