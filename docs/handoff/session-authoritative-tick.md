# Session authoritative for the whole tick: a staged plan (2026-11-03)

> Retired since this plan was written: `word_transfers!`, `register_writes!` and
> `RawRam` are gone from the code (checked on `caae95ea9`). Stage notes below that
> name them describe the code at their own date.

Design lane `tick-plan`, on main `ecb4dee74`. This note has no engine changes. It plans
docs/PERFORMANCE.md step 2: game code runs as a client of one Session for the whole tick,
not the Session as a guest of `Game`.

All counts below come from `python tools/ram_ratchet.py` and from greps on `ecb4dee74`.
Line comments are excluded, as the ratchet excludes them. The ratchet stands at
`sm_game_ram_sites` 189, `foreign_calls` 336, `game_host_impls` 97 and `session_entries` 116.

## 1. Today's tick, end to end

### Entry

Two entries run the engine's tick. Both wrap it in `with_frame_session`
(`sm/src/game/session_runtime.rs:1123`):

- `rtl_run_frame_compare`, `RM_MINE` (`sm/src/cpu_infra.rs:1564`). Engine-only runs use it:
  the lock gate, headless runs and free runs.
- The paired lockstep frame (`cpu_infra.rs:1363`). It runs the oracle CPU frame, restores the
  before image into `Game`, then runs the engine's tick.

Each entry is the same sequence:

```
audio_output.enter_execution(position)
with_frame_session(game, |game| gameplay::tick(TickInput { buttons }, game))   // H = Game
draw_frame_to_ppu(game)        // settlement: beam, HDMA engine, scanline IRQs; outside the frame session
capture snapshot / observe / compare           // verifier only
```

`gameplay_frame::tick` (`crates/sm_game/src/gameplay_frame.rs:245`) is generic, but
`gameplay_frame::Host` has only one implementation, `impl ... for Game`
(`sm/src/game/gameplay.rs:13`). So the tick runs on the RAM image. The Session comes and
goes as short leases.

### What a lease costs

Code opens a lease in one of two ways:

- `Runtime::new(game)` (`session_runtime.rs:164`), from sm/.
- `RuntimeAccess::runtime()`, from sm_game. Its implementation for Game,
  `sm/src/game/hosts.rs:21`, is `Runtime::new`.

There are 116 `.runtime()` calls in `crates/sm_game/src` (`session_entries`). There are 154
`Runtime::new(` calls in sm/src outside `verification/`, and 377 counting it.

Opening a lease:
1. Takes the retained persistent Session (`Game::lease_persistent_session`,
   `game_state.rs:2074`). Only Owned verification starts have one. Replays run Legacy
   presentation, so their slot is always `Legacy`.
2. Otherwise takes the frame's kept Session (`reuse_frame_session`, `session_runtime.rs:1147`,
   PR #179). It closes the shared write log, reads back each logged byte the Session projects
   (`take_written_bytes`, :1135), observes the inputs again and takes Game's held owners
   (`take_game_owners`: inventory, FX, menu and actor identity).
3. Otherwise decodes the whole RAM image (`import_session`).

Releasing a lease (`relinquish`, :509) does the following inside a frame tick:
- clones the whole Session (`state.clone()`, :561);
- defers the export (`export_frame_session`, :1430; `WorkRam::defer_export`,
  `game_state.rs:528`, PR #181). The Session's bytes are written at the first RAM access they
  cover (`settle_deferred`, :635);
- opens the write log again (`open_write_log`, :468; this drops any earlier log).

`foreign(f)` (:387) does all of this in the middle of a lease:
- preflight, then export (deferred where the layout is known, `export_session_for_foreign`,
  :1468);
- revokes four Game domains and runs `f(&mut Game)`;
- captures Samus, takes the logged bytes back (`resume_session`, :1192), adopts the
  presentation and FX sources again, and observes all the inputs again.

`owned_call` (:448), from #184/#187, runs typed code on the current lease without any of
this.

### The frame, in order

Phase bodies are on Game unless a lease is named. `gameplay_frame.rs` line numbers.

| # | Step | Host | Leases / foreign |
| --- | --- | --- | --- |
| 1 | `latch_controller`: joypad register | Game | none |
| 2 | `begin_session_interval` (:278 → `game_state.rs:2196`): Game adopts its inventory, menu, FX, scroll, frame-display, sequence and context images | Game | none |
| 3 | Read `$0611`, `resume_boot`, clear `$0612..$0615` | Game (raw RAM) | none |
| 4 | `update_hdma` (:299 → `hdma_objects::update_hdma`, `hdma_objects.rs:362`) | Game | `handle_music_queue` on Game (`music.rs:16`, the music cues); the power bomb explosion through `sequence_state`; `run_objects` opens `hdma_lease()` when admissible, and otherwise runs the `Bridged` RAM interpreter (`hdma_program.rs:135`). Every pre-action (`with_object`, :53) and call is a `foreign` (`hdma_dispatch.rs:232/256/271/307`) |
| 5 | `advance_random` (:300 → `Game::next_random`, `game_state.rs:1258`): RAM `$05E5` | Game | none |
| 6 | Sprite high table and cursor; `with_session(clear_samus_requests)` (:303) | Game, then a lease | 1 lease |
| 7 | State dispatch (`GAME_STATE_ENTRIES`) | mixed | below |
| 7a | Gameplay pass `advance_gameplay` (:802): actor selection, palette effects, projectile preparation, enemy actors, enemy projectiles, animated tiles, contacts, camera, `draw_scene`, actor tiles and HUD, scrolls | each through `game.runtime()` | about 12 leases |
| 7b | ...interleaved with `player_projectiles_hit_samus`, `message_boxes::display`, `advance_player`, `run_room_mechanisms`, `advance_room`, `finish_health_and_time`, `finish_frame` | Game; `advance_player` and the mechanisms open their own `Runtime::new` inside (`player_update_runtime.rs:113..262`, `room_mechanisms/mod.rs` with 12) | several more |
| 7c | Cinematics (`advance_native_state` → `title_sequence`, `intro`, `ending_and_credits`), menus (file, map, options, game over, pause), demo, death | Game; the 11 cohort copies (`session_cohort::with`, `session_cohort.rs:113`) load and store through the RAM projection | leases only where a callee opens one |
| 8 | `advance_sound_effects` (:337 → `music::advance_sound_effects` → `native_audio::emit`) | Game | none |
| 9 | Sprite finish, joypad filter word, coroutine check | Game | none |
| 10 | NMI: `display_refresh::run` (:353, `display_refresh.rs:626` → `scene::commit::refresh`, `commit.rs:43`) through the `Refresh` adapter: palette and OAM DMA, Samus and animated tiles, register mirrors, HDMA binding, Mode 7, tilemap queue, VRAM readbacks into WRAM, HDMA enable and the frame counters | Game: register writes and DMA through `rtl::write_reg` | none |
| 11 | `controller_input::sample_latched`; resume byte `$0611` | Game | none |
| — | `with_frame_session` ends: the kept Session is marked `crossed` | | |
| 12 | `draw_frame_to_ppu` (`cpu_infra.rs:1162`): `scene::settle::settle` (`settle.rs:24`) steps the emulated beam. HDMA reads its tables from WRAM line by line. `scanline_irq::advance` (`scanline_irq.rs:52`) runs `scene::irq::advance` on `ScanlineIrq(game)` | Game | any lease here exports at once: the frame session is no longer active |
| 13 | Verifier: snapshot (RAM, VRAM, APU queue), `observation::observe` (`observation/mod.rs:197`) and `verify_snapshots_eq` with `NORMALIZED_RAM` (`cpu_infra.rs:440`) | Game | reading RAM settles the deferred export |

Audio has two clocks:
- Requests ($0617..$0665, $0686, $07F3/$07F5) are RAM bytes in Legacy. They have a typed
  owner, `AudioRequests` (`session_owners.rs:80`), only under `Control::Owned`, that is, in
  Owned verification starts.
- Cues go to the APU queue through `native_audio::emit` on Game, at steps 4 and 8, and from
  `foreign` calls.

On the Runtime, `SoundEffects::queue_sound` (`session_runtime.rs:2149`) runs through
`foreign` in Legacy, and on the lease (`requests::queue_sfx`) only with an Owned display.

PERFORMANCE.md's baseline (main `07244dfda`, before #179/#181) counted the following per tick:
- 12-22 leases, 6-11 of them fresh imports;
- 8.5-24 `foreign` calls;
- 14-34 decodes and 21-45 encodes.

On Kraid, encode and decode were 55 of the 61 ms engine tick. No count has been taken since
#179/#181/#184/#187.

### Game code that runs outside any lease

These run on `Game` with no lease open:
- steps 1-5, 7b, 7c, 8-12, and the bodies of all 336 `foreign` closures;
- the domains Game holds between leases (inventory, menu, FX, scroll, frame display,
  sequence, context). These are a second state owner, adopted at step 2 and taken into each
  lease.

## 2. The blockers: what each reads and writes, and what has to change

`foreign` calls by class. The classes come from a closure-body scan; sites that mix two
classes are counted once.

| Class | Sites | Where |
| --- | ---: | --- |
| Diagnostics: `unreachable`, `report_unreachable`, `rtl_unreachable`, `invalid_interrupt`, `legacy_assertion` (all diverge) | 62 | 43 files: 29 in sm_game (actors, `hdma_program`, `hdma_dispatch`), 14 in sm/src/game |
| Diagnostics: `invalid_window`, `native_rom_diagnostic` (set `g_fail`) | 8 + 6 mixed | sm/src/game: player collision, movement and update; room entry content |
| Audio requests (music and sounds) | 26 | crocomire 4, ceres_ridley 3, torizo 3, room_mechanisms/runtime 3, kraid 2, phantoon 2, room_entry/compatibility 2, and others |
| Verifier hooks and RNG (`next_random`, `observe_kraid_flags`, `observe_native_draw_read`) | 11 | kraid 4, mother_brain_fight 2, yard, boss_lease, enemy projectile effects, player transitions, mechanism content |
| HDMA spawn and free, effects, enable | 18 | hdma_program 5, hdma_dispatch 3, crocomire 2, mother_brain_fight 2, hosts 2, session_runtime 2, and others |
| Registers (`Registers for Runtime`, multiplicand, room entry) | 10 | session_runtime 4 (`:2129`), room_entry/compatibility 3, and others |
| Other gameplay effects (player commands 17, projectile spawns, ROM records, palettes, events, saves, X-ray window) | about 152 | hosts.rs 17, session_runtime 14, torizo 12, room_entry/compatibility 10, room_state 8, and others |
| Verification cases that test `foreign` itself | 39 | sm/src/verification, 15 files |

### 2.1 Cohort copies (11 `sm_game_ram_sites`)

- **Where.** `cinematic.rs` 5 (`FadePlanes` $2000..$2FFF, `PaletteBuffer` $C000..$C1FF),
  `cinematic_states.rs` 4 (`Cinematic`), `cinematic_objects.rs` 1 (`ShotWords`) and
  `ending_screens.rs` 1. One more caller in sm/ (`gameplay.rs:86`) does not count.
- **Reads and writes.** `load` reads the host's projection. `store`
  (`session_cohort.rs:84`) writes Session fields on one lease only when
  `session_retained()`; replays never have that, so the bytes go to RAM.
- **Why they are stuck.** They run inside step 7c on Game, and no Game owner holds those
  words. A helper that opened its own lease mid-frame failed (see section 4).
- **Change.** Run the cinematic states on the lease. Each `Cohort` already has a
  `session_words` mapping to Session fields, so on a lease the copy becomes direct field
  access (`&mut session....`) and the cohort calls go.

### 2.2 Music queueing

- **Reads and writes.** The music queue ($0619..$063F) and the sound queues
  ($0643..$0665) through `AudioHost`.
- **Game and the Runtime differ.** Game stores with `compatibility_write_owned_byte`: the
  RAM byte, plus the Session projection and its observations when a persistent Session is
  `Ready`. The Runtime stores with `Runtime::set_byte`, and publishes meanings only when
  `self.persistent` is set. In Legacy, `queue_sound` and the `queue_music_*` paths go through
  `foreign` to `crate::game::music::*` on Game.
- **Cues.** The cue sink (`native_audio::emit`) needs `&mut Game`: it uses the APU queue
  (`game.apu_queue`), not RAM.
- **Change.** In two steps:
  1. Queue on the lease over `AudioHost for Runtime` (`session_runtime.rs:844`). In Legacy
     the bytes stay RAM-only, so `set_byte` writes RAM directly, exactly as the Game code
     inside `foreign` does.
  2. Make `AudioRequests` a Session owner in Legacy too, with every writer moved onto it.
     Then `handle_music_queue` and `advance_sound_effects` run on the lease, and emit cues
     through a narrow cue port on Game.

### 2.3 Verifier hooks

- **`next_random`.** `Game::next_random` reads and writes RAM `$05E5`. It calls the
  verify-only `player_transitions_composed::random_after`, which asserts that "foreign effect
  must own RAM" and reads RAM words. The Session already owns the word
  (`session.timing.random_number`, `session_encoding.rs:850`), and `ExternalInputs::random`
  observes it.
- **`observe_kraid_flags`.** Calls `kraid_flags_observer::observe(step, game)`, which reads
  RAM.
- **Change.** The lease computes `sm_game::random::next` on the Session field and calls
  `words_stored()`, as the `foreign` resume re-observed the inputs. The verifier hooks
  take a read view: `Runtime::byte` projects the Session in place of RAM. `random_after`'s
  assertion becomes "the lease's projection of these words", which tests the same thing.

### 2.4 HDMA: spawn, free and the interpreter

- **`place_hdma`** (`hdma_objects.rs:120`) needs `menus::host::Host` (Game). The table
  words it writes are Session-owned (`hdma_word`). It also writes the DMA channel control and
  target registers, $43x0/$43x1. Those are register writes that neither read nor write WRAM.
- **`free_hdma_object`** (`hdma_objects.rs:164`) needs `WorkRamAccess` only for its
  `legacy_overflow` store past the table.
- **`run_objects`** opens a lease only when `runtime_admissible()`. Otherwise it runs
  `Bridged` (`hdma_objects.rs` `Bridged`), a RAM-copy interpreter on Game.
- **Pre-actions.** On the lease, every pre-action that touches its object
  (`Interpreter::with_object`, `hdma_program.rs:53`) and every `Call` is a `foreign`. These are
  the liquid surfaces and backgrounds, X-ray restore, sky scroll, explosion windows and
  Tourian statue. So a room with water or lava pays one full export and import per object per
  frame. This is also why `liquid_effects.rs` keeps 11 ratchet sites: the RNG swap and
  `LiquidWords::read`, with no lease to take the values from.
- **Change.** Run the pre-actions on the lease:
  - implement their host bounds (`DispatchHost` = menus `Host`, `explosion_window::Host`,
    `xray_background::Host`, `room_visuals::StatueHost`) for `Runtime`, or narrow them to
    `SessionAccess`, display `Access` and a register port;
  - spawn and free on the lease;
  - retire `Bridged` once the tick lease makes admissibility hold for the whole interpreter.

### 2.5 `report_unreachable` and the diagnostics

- **What they do.** `rtl::report_unreachable(_game)` and `invalid_interrupt(_game)`
  (`rtl.rs:914-919`) ignore the game. They print, then `__assert_fail`, which is
  `process::exit(2)` in the verifier (`verify_dll.rs:379`) and abort elsewhere.
  `legacy_assertion` (`session_runtime.rs:2093`) also diverges. `invalid_window` sets
  `g_fail` and prints.
- **Change.** None of them needs the RAM image. Give the leases a diverging
  `fn unreachable(&mut self) -> !` that calls the diagnostic without exporting. This is stage 1.

### 2.6 The NMI `Refresh` adapter's register writes

- **What they do.** `display_refresh::Refresh` (`display_refresh.rs:494`) implements
  `scene::commit::RefreshHost` with `register_writes!`/`write_register` on Game.
  `display_refresh.rs` holds 27 of the 77 register-write sites; the full survey is in
  `owned-display-registers.md`.
- **Readers.** DMA from WRAM (palette $C000, OAM $0370, the tilemap queue), PPU ports, and
  readbacks *into* WRAM.
- **On a lease today.** Every register write is a `foreign` (`impl Registers for Runtime`,
  `session_runtime.rs:2129`).
- **Change.** In two steps:
  1. A lease-side hardware port (stage 4): publish the Session's bytes before a DMA reads
     WRAM, and take written bytes back after a DMA or WRAM-port write into WRAM.
  2. Convert the writes to typed `display::video::Command`s with the encoder in sm/ (lane
     `owned-pres`). Their removal from sm_game still needs the Owned presentation or the
     native renderer to be the replay authority, which is retirement step 1 in
     `owned-display-registers.md`.

### 2.7 Settlement, HDMA engine and IRQ

`draw_frame_to_ppu` runs after the frame session closes. HDMA reads WRAM tables on every
line, so the Session's bytes must be in RAM by then; settling the deferred export puts them
there.

The IRQ handler is typed (`scene::irq::advance`, through `ScanlineIrq`), but it writes the
display history words and registers on Game. Its RAM writes reach the next tick's first lease
through the write log. ENGINE_STATE_MIGRATION's table applies here: "Scanline, HDMA and IRQ
activity after gameplay ... do not assume the Rust gameplay return is the final writer of the
frame."

## 3. The staged sequence

The rules for every stage:
- It lands on its own, `[exact]`, behind PR Verify: check-lock of every locked row, plus
  lockstep of the rows listed.
- It runs chain rooms 1, 6, 100, 200 and 320 (sm-check).
- No stage opens a lease the code did not already open (section 4).
- `foreign_calls` may not rise inside a landing. A stage that must add a delegation removes
  more in the same landing.

Deltas are against `ecb4dee74`, and the lane remeasures them.

| Stage | Change | `sm_game_ram_sites` | `foreign_calls` | Other |
| --- | --- | ---: | ---: | --- |
| 0 | Per-tick counters on main | 0 | 0 | reported only |
| 1 | Diagnostics leave `foreign` | 0 | −62 (−70 with `invalid_window`) | |
| 2 | RNG and verifier hooks on the lease | 0 | −11 | |
| 3a | Audio requests on the lease (Legacy bytes stay RAM) | 0 | about −24 | |
| 3b | `AudioRequests` as a Legacy Session owner; music queue and sound advance on the lease | −1 | about −2 | format bump |
| 4 | Hardware port on the lease: `publish` + direct registers | 0 | about −12 | |
| 5 | HDMA interpreter, pre-actions, spawn and free on the lease; `Bridged` retired | about −16 | about −20 | |
| 6 | Gameplay-pass phases take the lease (sm/ entries) | 0 | 0 to −10 | `Runtime::new` 154 → about 100 |
| 7 | Cinematics and menus on the lease; cohort copies gone | about −19 | about −5 | |
| 8 | Pause hooks, save slots and demo on owners | about −16 | small | format bumps |
| 9 | Inversion: one tick lease | 0 | small | `session_entries` 116 → about 30; `game_host_impls` −1 to −6 |
| 10 | NMI and settlement inside the tick; typed display commands | up to −77 (with the display-command lanes) | about −5 | |
| 11 | Remaining family lease merges; `foreign` deleted | about −48 (sm_game machinery and stragglers) | the rest (about 160, plus 39 in verification) | |

### Stage 0: per-tick counters (optional; can run in parallel)

- **Change.** Land the `SM_PERF_COUNTERS` instrumentation from the temporary
  `collab/perf-baseline` (PERFORMANCE.md "Baseline") under `cfg(sm_verify_dll)`. It counts
  leases, fresh imports, kept reuses, encodes, decodes and `foreign` calls per tick.
- **Add** a nested-lease count: a lease opened while a `foreign` interval is active. Make
  `ram_ratchet.py` report `runtime_opens` (the `Runtime::new` calls outside
  `sm/src/verification`; 154 now).
- **Why.** Stages 6 and 9 move few strict sites; these numbers show their effect.
- **Proof.** Counters only; PR Verify unchanged.

### Stage 1: diagnostics leave `foreign` (the first lane; section 6)

- **Delta.** `foreign_calls` 336 → 274. Adding the eight pure `invalid_window` and
  `native_rom_diagnostic` calls takes it to 266.
- **Proof.** These paths exit the process, so no passing row reaches them. The proof is the
  gate, clippy and PR Verify.

### Stage 2: RNG and verifier hooks on the lease

- **Change.**
  - `next_random` (6 sites) stores `session.timing.random_number`, then `words_stored()`.
  - `observe_kraid_flags` (4) and `observe_native_draw_read` (1) observe a read view of the
    lease.
  - `player_transitions_composed::random_after` checks the lease's projection.
- **Delta.** `foreign_calls` −11.
- **Rows.**
  - Kraid: `boss-kraid` and the four `boss-kraid-pause-*` rows.
  - Mother Brain: `boss-mother-brain`, `mother-brain-phase2`, `mother-brain-glass`.
  - Random enemies: `enemy-metroid`, `enemy-covern-roach`, `enemy-kihunter-wings`, `enemy-yapping-maw`.
  - Ridley: `boss-ridley`, `boss-ridley-swoop`.
  - The player transitions verification case.

### Stage 3a: audio requests on the lease

- **Change.**
  - `SoundEffects for Runtime`, the enemy projectile `queue_sound` and `queue_music`, the
    mechanism and room entry sounds, and the boss `queue_music_delayed*` run
    `music_requests::*` over `AudioHost for Runtime`.
  - The Owned branch (`has_owned_display`) is unchanged.
  - In verify builds, assert that no export layout covers $0617..$0665.
- **Delta.** `foreign_calls` about −24.
- **Rows.**
  - The three rows the SFX loss broke: `item-speed-booster`, `enemy-fire-arc`,
    `music-fir-selector10`.
  - Boss music: `boss-crocomire`, `boss-ceres-ridley`, `boss-bomb-torizo`, `boss-phantoon`, `boss-kraid`.
  - Door sound: `door-transition`.
  - Also `death`, `credits`.
- **Comparison.** The APU queue compare (`cpu_infra.rs` "APU queued ports compare") and its
  lock digest stream.

### Stage 3b: `AudioRequests` as a Legacy Session owner

A decision (D3).
- **Change.**
  - The requests become typed for every presentation, with every writer moved: music
    requests, the sm/ queue clears, the boot WRAM clear and the oracle restore.
  - `update_hdma`'s `handle_music_queue` and the frame's `advance_sound_effects` run on the
    lease. The cue sink is a port on Game.
- **Delta.** `controller_input.rs`'s `IS_UPLOADING_APU` (−1), and the remaining audio
  `foreign` calls.
- **Rows.** As 3a, plus `save-death-reload` and `samus-gunship-save` (music after a load).
- **Done (lane `tick-s3b`, 2026-11-03).** D3 taken provisionally: cues stay immediate.
  - `LegacyControl::audio` (`session_owners::LegacyAudio`) owns $0617..$0640,
    $0643..$0687 and $07F3..$07F6. Every lease decodes it, every export encodes it, so the
    lease's request stores reach the owner and RAM takes them at the export. The APU-side
    readers on Game read RAM, which a deferred export serves or settles first. Game-side
    writers (the music queue and sound advance, boot, the queue clears inside `foreign`,
    the oracle restore) still store RAM outside leases, and the next lease takes them from
    the write log or decodes. The Legacy control has no checkpoint, so no format bump.
  - Verify builds assert, on every lease audio request, that the control projects every
    request byte, and, at every APU-side read on Game, that a kept frame Session current
    with RAM holds the byte RAM does.
  - The mixed foreign calls of the Bomb Torizo (fight start, defeat), Botwoon, Crocomire
    and Phantoon deaths, and the room entry's door sound and sound queue reset, run on the
    lease (`foreign_calls` 183 to 178, `game_host_impls` 96 to 95).
  - Left: `handle_music_queue` (step 4) and `advance_sound_effects` (step 8) run on Game
    with no lease open; moving them needs a lease the frame already holds there (stage 6),
    since a new mid-frame lease is ruled out. Ridley's death (`ridley.rs` drop scatter plus
    music) needs the enemy projectile host on Ridley's lease, which makes its
    `queue_sound` calls ambiguous between the two sound traits.

### Stage 4: the lease's hardware ports (lane `tick-s4`, 2026-11-04)

D2 (default, 2026-11-04): a narrow typed device port retires a `foreign` call, because the
engine's own runtime and renderer implement it. A port is a small typed interface the engine
calls for one hardware job. The emulated backend in sm/ implements each port by making the
identical hardware writes and reads, as #214's `Video::apply` encoder does. A port impl that
hands a generic closure to `foreign` does not count.

**The device interval.** `foreign` exports the Session to RAM, runs its action inside a Samus
mutation scope, and re-imports. A register that holds no WRAM, ROM or Samus state needs none
of that. `Game::device_execution` splits the backend and host as `execution_parts` does, without
the unscoped-writer Samus revocation, and `rtl::write_device`/`read_device` make one register
access through it. They take only device registers: the PPU ($2100-$213F), the WRAM port
address ($2181-$2183), the CPU registers ($4200-$421F, with MDMAEN only when it starts no
transfer) and the DMA channel parameters ($4300-$437F). The register observer still sees
each write. `display_commands::device_only` also leaves out `StopDma` and `ClearCpuIo`: the
room background check snapshots RAM at every MDMAEN write. Verify builds assert the write
epoch did not move across each device write, each device read and each device-only command.

On the lease, each port call runs inside `owned_player_call`, so the observations are
refreshed on entry and on return as the former `foreign` call left them. No RAM moves, so
the lease stays current and nothing is exported or imported.

Because nothing is exported, RAM no longer holds what the Session changed since its last
export. The former `foreign` export had published it. The scroll stores' observation refresh
(`Runtime::refresh_scroll_inputs`) reads RAM, so it now takes the time-frozen flag ($0A78)
from the Session, as it already took the drop origin and the grapple start. Before that
change, `boss-draygon` (@1520) and `chain-any-0144-DA60` (@1542) queued the enemy BG2 tilemap
in the first frozen frame. The cause was the X-ray scope's HDMA spawn, which no longer
published the flag, or the reserve tanks' freeze.

| Port | Signature | Runtime impl | Reaches the emulated hardware |
| --- | --- | --- | --- |
| `hardware::Multiplier` | `mode7_product(multiplicand: u16, multiplier: u8) -> u16`, `cpu_multiplicand(u8)` | device interval | M7A low, M7A high, M7B, then MPYM and MPYH reads; WRMPYA |
| `display::video::Video` (device commands) | `apply(producer, Command)` | device interval when `display_commands::device_only` | the encoder's register writes, through `Wire` |
| `hdma_objects::Allocator` | `write_channel_control(register, control)` | device interval | DMAPx then BBADx |
| `hardware::Bus` | `read_bus_byte(address: u32) -> u8` | WRAM and its low mirror from the lease's projection; $2180 reads the lease's byte at the port address and advances it; the rest from the backend | one backend read per non-WRAM byte |
| `decompression::DecompressionHost` (boss lease) | as on `Game` | the lease: bytes through its projection, the cursor in `Game::decompress_src` | none (ROM reads only) |

**Outcome.** `foreign_calls` 72 → 64 (`7deecc6fc`, `d7f1d3f2a` on `collab/tick-s4`): Mother
Brain's neck, Ridley's tail, Yapping Maw's multiplicand, the HDMA program's channel byte, the
lava background's scroll target, the allocator's channel control, the slope BTS read and
Kraid's decompression. `game_host_impls` 85 → 84 (the neck `Host` for Game). The
device-only commands that already ran inside `Video for Runtime`'s one `foreign` (NMI enable,
the scanline IRQ, `initialize_room_effects`' channel bytes and WRAM port) no longer export.

**What stays on `foreign`, and why.**
- The Mode 7 queue (Ceres Ridley, ×2): its bytes ($02D0-$032F) are among those whose
  publication still needs a semantic counterpart, so a lease store there fails a persistent
  owned Display. It moves when the queue has a Display owner field.
- Commands and room entry writes that start a DMA (MDMAEN non-zero) or move WRAM: the DMA
  reads WRAM the lease has not published. They need the `publish` step below.
- The room entry device writes (`device_word`, `device_byte`, `device_vram_address`): the
  room background check (`room_background_cases` id 4) swaps the loaded ROM image inside the
  VMADDH and DMAP1 writes and snapshots RAM there. That needs the Samus mutation scope and
  the exported RAM `foreign` gives it. They move once the observer reads the lease (lane
  `fc10` owns the observer).
- `publish` (encode the Session's spans to RAM without the release, then keep the epoch) is
  the next step for the DMA starts: VRAM copy (`copy_to_vram_now`), the VRAM decompressions,
  the animated tile upload, `flush_tilemaps` and the grapple tiles.

**Rows.** `enemy-yapping-maw`; `boss-mother-brain`, `mother-brain-phase2` and
`mother-brain-glass` (the neck); `boss-ridley` and `boss-ridley-swoop` (the tail);
`boss-kraid` (its decompression); `volcano-room-lavaquake` and `area-norfair` (lava
background); `boss-crocomire` (HDMA spawn); `area-crateria` and `xray-activate-release` (HDMA
program channel bytes); `door-transition` and `door-loading-lag-nmi` (room-entry NMI, IRQ and
HDMA channel commands); slopes on every area route, `terrain-quicksand` and the
`terrain-*-hand` rows.

### Stage 4b: `publish` and the transfer port (lane `tick-s4b`, 2026-11-04)

**`Runtime::publish()`.** The lease writes its Session to RAM as its release would export it
(`copy_spans` over `encode_spans`) and keeps going. It clones nothing, decodes nothing and
returns no domain to Game. The Session stays the authority and RAM projects it.
- An export a frame lease deferred earlier is written first, as any RAM write writes it.
- When a write log is open around the lease (a nested lease inside `foreign`), the log takes
  the published bytes, and the outer call adopts them as it would this lease's release.
- A persistent lease takes the new write epoch, as its release does.
- `publish` is not a lease and never decodes. It is the export half of `foreign` without
  the release.

**The DMA start.** `Runtime::start_dma(mask, source)` starts the channels through
`rtl::start_device_dma`. That is the MDMAEN write `write_reg` makes, with the register
observer, inside the device interval, so Samus authority and the lease stay current.
- Before the start it reads the channel registers (DMAP, BBAD, A1T, A1B, DAS) and the WRAM
  port address, and lists the work RAM bytes the channels read and write.
- Verify builds assert that RAM holds the Session's projection of every byte the channels
  read (`verification/transfer_checks.rs`).
- Bytes a channel writes go back to the Session through `take_written_bytes` over just those
  bytes. That means a B-to-A readback's A-bus bytes, or an A-to-B transfer to $2180 at the
  port address. This is the typed decode of the range, not the write log: the log is shared,
  and it overflows at 4 KiB.

**The ports.** Each runs inside `owned_player_call`, so the observations are refreshed as the
former `foreign` call left them. Writes are identical and in the same order.
- `Video for Runtime` (the DMA and WRAM commands: `TilesNow`, `TilesNowBytePort`,
  `TilemapNow`, `ReadTilemapNow`, `ReadVramNow`, `TilesThroughPort(Bytes)`,
  `ObjectsAndPaletteNow`, `StopDma`, `ClearCpuIo`) publishes, then encodes through
  `display_commands::Lease`. That wire sends device registers through the device interval
  and MDMAEN starts through `start_dma`. The encoder's `Wire` carries the upload bracket
  (`producing`/`produced`, which now reach the PPU through the device split) and the room
  DMA start.
- `EntryEffects::copy_vram` publishes, then runs `CopyToVramNow` on the same wire, with the
  prepared room image as the DMA's ROM source.
- `EntryEffects::decompress_vram` reads ROM and VRAM and writes VRAM through device
  registers only, so it needs no publish. Its decoder output runs on the lease.
- The animated tile upload reads bank $87 only, so it starts on the wire with no publish.
- `EntryEffects::decompress` and both forms of `decode_level_and_copy_planes` call
  `Runtime::decompress_to_ram` and `Runtime::decompress_level`. These publish, run the same
  decoder and the level's plane split on the RAM image, and take back exactly the decoded
  bytes and the two plane destinations. The former calls overflowed the write log and
  decoded the whole image again. `room_level_copy`'s check still pins the split's
  failure-prefix behaviour on the RAM image the port writes.

**Outcome.** `foreign_calls` 62 → 55 (`c3bd4f25a`, `cf22ccb9a`, `2b9013193` on
`collab/tick-s4b`). `game_host_impls` 83 is unchanged and `legacy_engine_ram_sites` is
445 → 444.

**What stays on `foreign`, and why.**
- `flush_tilemaps` (`room_scrolls.rs`). Its RAM stores interleave with its DMA starts: the
  queue sentinel, the tail, and each scrolling enable byte, cleared through the owned-byte
  path. The enable bytes ($0956-$098D) are among those a persistent owned Display needs a
  semantic counterpart for, so a lease store there fails it. Its readbacks also write WRAM
  ranges that come from queue entries. It moves when the tilemap queue and the scrolling
  parameters have Display owner fields; then it is the wire above plus typed queue reads.
- The grapple tiles. These make no DMA: they are native queue stores whose full-u16 cursor
  can alias the charge and animation fields. `publish` does not settle that question. The
  aliasing is in the lease's projection of the stores, not in what the hardware reads.
- The Mode 7 queue (Ceres Ridley, ×2). `publish` does not resolve it either: the lease's
  stores into $02D0-$032F still fail a persistent owned Display. It needs the queue as a
  Display owner field, as stage 4 said.
- The room entry device writes (`device_word`, `device_byte`, `device_vram_address`): the
  observer case swaps the loaded image there (lane `fc10`).
  - `publish` gives these the exported RAM.
  - They still need the Samus re-adoption that `foreign` does after the swap.
  - `copy_vram` meets the same swap in room background case 5, at its first write. Case 5
    is the row to watch: on the lease, Samus is not re-adopted after the swap.

**Rows.**
- Every room load and door (the decompressions and the VRAM copy): `door-transition`,
  `door-loading-lag-nmi`, `tas-ceres-four-doors` and the `area-*` routes.
- Animated tile rooms: `area-norfair`, `area-maridia`, `area-wrecked-ship`, `area-brinstar`.
- The DMA commands: `pause-menu-tour` (pause and map), `save-memory`, `samus-gunship-save`
  and `save-death-reload` (message boxes), `samus-scripted-intro`, and the title and intro
  frames of every route.
- The verifier's room background cases (case 5) and `room_level_copy`.
- The SFX-loss rows `item-speed-booster`, `enemy-fire-arc` and `music-fir-selector10`.
  - No port writes the audio request bytes.
  - `publish` exports them from the Session in the export's own order. A release does the
    same.

### Stage 5: HDMA on the lease

- **Change.**
  - The pre-actions and `Call`s run on the lease, with the host bounds of section 2.4.
  - Spawn and free run on the lease.
  - `Bridged` and the RAM path of `run_objects` are deleted once stage 9 holds
    admissibility; until then they are kept for the persistent path.
- **Delta.**
  - `sm_game_ram_sites` about −16: `liquid_effects` 11, `explosion_window` 3,
    `room_visuals` 1, `xray_background` 1.
  - `foreign_calls` about −20.
- **Rows.**
  - Liquids: `volcano-room-lavaquake`, `area-norfair`, `area-maridia`, `terrain-quicksand`.
  - X-ray: the four `xray-scan-*` rows, `xray-activate-release`.
  - Power bomb and crystal flash: `power-bomb-room-entry`, `power-bomb-warehouse-elevator-entry`, `samus-crystal-flash`.
  - Statues: `area-tourian`.
  - Boss HDMA: `boss-phantoon`, `boss-phantoon-second-round`, `boss-crocomire`,
    `mother-brain-phase2`.
  - Sky scroll: `area-crateria`.
  - The HDMA table check (`SM_IRQ_TABLE`) on the nightly route.

### Stage 6: the gameplay-pass phases take the lease

- **Change.**
  - The sm/ entries `advance_player` (`player_update_runtime::advance_frame`),
    `run_room_mechanisms`, `advance_room`, `player_projectiles_hit_samus`, the low health
    warning and Mother Brain's unpause take `&mut Runtime`.
  - `gameplay_frame` calls them on a `game.runtime()` it already opens, merging their inner
    `Runtime::new`s into one: player_movement_runtime alone has 22.
  - Each phase is its own commit.
- **Delta.** Strict measures move little. This is the speed stage: `runtime_opens` falls and
  leases per tick fall, which stage 0 measures.
- **Rows.**
  - Movement: the `samus-*` movement rows and `samus-shinespark-*`.
  - Mechanisms: `door-transition`, `elevator-traversal`.
  - Items: `item-*`.
  - Grapple: `samus-grapple-*`.
  - Death: `death`.
- **Done in part (lane `tick-s6`, on `ea7b09e0b`).** Not yet landed.
  - Samus's frame (`player_update_runtime::advance_frame`), its special movement handlers and
    gamma actions, `player_projectiles_hit_samus`, the room mechanisms and the room's main
    code run on the lease, through `gameplay_frame::PassLease`.
  - Without a queued message box the whole gameplay pass runs on one lease; it opened about
    15 on a plain frame. A message box, or a resumed mechanism yield, splits it in two.
  - The intro's `advance_intro_player` runs on one lease; `initialize_player` already did.
  - `Runtime::new` outside verification fell from 152 to 120; `foreign_calls` stayed at 183.
  - Stage 0's counters landed with it: `lease counters:` lines on stderr in verify builds.
  - Left: the low health warning (death phase) and Mother Brain's unpause each open one
    lease with no neighbour to merge; `finish_health_and_time` and `finish_frame` run on Game
    after the pass's lease (`gameplay_rules::Host`); `player_movement_runtime`'s
    `ceres_movement` and the crystal flash's HDMA spawn keep a `foreign` call each.

### Stage 7: cinematics and menus on the lease

- **Change.**
  - `advance_native_state` (title, intro, ending) and the menu states run with the lease as
    host: `menus::host::Host` and `MenuRuntime` for `Runtime`.
  - The cohort copies become field access.
  - The title gradient (`MODE7_ZOOM`) and the intro crossfade (`EPROJ_X_POS`) read their
    fields.
- **Delta.** About −19:
  - 11 cohort copies;
  - the `cinematic_objects` and `pause_controls` view calls;
  - two `hdma_objects` named words;
  - the `word_transfers!` in `title_sequence`, `cinematic`, `ending_screens` and
    `file_menu`.
- **Rows.**
  - Credits: `credits`, `credits-ending` and the 13 `credits-*` spawn rows.
  - Intro and Ceres: `samus-scripted-intro`, `area-ceres`, `tas-ceres-four-doors`,
    `tas-ceres-cross-frame-read`.
  - Demo: `demo-attract-setup`.
  - Menus: `options-menu-tour`, `pause-menu-tour`, `options-menu-return-parameter`,
    `file-copy-clear`.
  - Saves: `save-memory`.
- **Attempted out of order (lane `tick-s7`, on `6ab13f6fb`, after stages 1, 2 and 3a).**
  Not landed. Ratchet then: `sm_game_ram_sites` 122, `foreign_calls` 183. The stage
  needs stages 3b, 4, 5 and parts of 6 and 9 first:
  - **No frame lease spans the dispatch.** The only lease before step 7 is step 6's
    `with_session(clear_samus_requests)`, closed before `GAME_STATE` is read. Keeping it
    open through `advance_native_state` would run the whole cinematic state on it.
  - **The host.** The states are generic over `cinematic_host::Host` =
    `menus::host::Host` + `display_refresh::Host`. `Runtime` lacks 16 of their
    supertraits: `Mode7Queue`, `TilemapQueue`, `BackgroundBlocks`, `LevelData`,
    `PlayerPosition`, `HudPlayer`, `WorkRamAccess`, `LoadedRom`, `Video`,
    `DisplayRefresh`, `HdmaTable`, `SpriteHost`, `DecompressionHost`, `MenuRuntime`,
    `ResetRequest`, `RuntimeAccess`, plus `ActorImage` (an excluded facade).
  - **Hardware (stage 4).** `Registers for Runtime` is a `foreign` per write.
    `Video::produced` reads WRAM for its uploads. `load_vram_words`,
    `write_dma_register`, `disable_scanline_irq` and `advance_ending_tile_upload`
    (Mode 7 DMA) have no lease path.
  - **HDMA (stages 4 and 5).** The intro pages and `spawn_power_bomb` spawn HDMA
    objects through `place_hdma`, which writes the channel registers on Game.
    `MenuRuntime::run_hdma_objects` (`update_hdma`) opens `hdma_lease()`, so from a
    lease it would be a `Runtime::new` inside `foreign`.
  - **Samus (stage 6).** `advance_intro_player` (`player_projectiles::prepare` +
    `player_update_runtime::advance_frame`) and `initialize_player` are Game-hosted.
    `advance_frame` opens its own `Runtime::new` on every path, and also uses the
    special-movement and gamma callback tables on Game.
  - **Audio (stage 3b).** `MenuRuntime::handle_music_queue` runs on Game.
  - **Nested `.runtime()` (stage 9).** The palette effects, the shooting stars and
    `cinematic.rs` (3) open `game.runtime()` inside the frame. On a lease host,
    `RuntimeAccess::runtime()` has to hand out the lease itself.
  - **The split cohort.** `intro` copies the `Cinematic` twice around Samus's frame
    because her frame reads the table as enemy projectiles. That split goes only when her
    frame runs on the same lease.

  Without these, the cinematic host on `Runtime` needs about 20 new `foreign`
  delegations (`foreign_calls` is strict). Two of them (`run_hdma_objects` and
  `advance_intro_player`) would open a lease inside `foreign`: the section 4 failure mode. A Game-held `Cinematic`, fade plane or palette owner would be a
  second state owner outside the plan. A Game-side accessor would be an excluded facade.
- **Attempted again (lane `tick-s7b`, on `ba7cfe351`, `collab/tick-s4`).** Not landed; no
  engine change. Ratchet then: `sm_game_ram_sites` 11, `foreign_calls` 62,
  `game_host_impls` 83.
  - **Where the 11 copies run.** All of them run inside `title_sequence`, `intro` and
    `ending_and_credits` (`cinematic_states.rs`) or in helpers those states reach. The
    five `cinematic.rs` palette helpers are called from `cinematic_objects`,
    `ending_sequence`, `ending_screens`, `intro_pages` and `title_sequence`.
    - A copy can only become field access once the whole state runs on a lease. The
      state's one lease is the frame's first (step 6), kept open through the dispatch.
      Splitting a state across a lease and Game would open a lease mid-frame.
    - `title_sequence::load_graphics` also runs from the demo phase on Game
      (`load_title_sequence_graphics` in `gameplay.rs`). Its palette helper calls keep a
      Game caller until the demo moves too.
  - **What is now covered.** A compile probe checked `Runtime<'_>` against each supertrait
    of `cinematic_host::Host`, `menus::host::Host`, `display_refresh::Host` and
    `sprite_output::SpriteHost`. `Runtime` already implements:
    - display: `Video`, `HdmaTable`, `DisplayOwner`, `ReadbackQueue`, `Scrolling` and
      `TilemapQueue`;
    - player and audio: `PlayerPosition`, `PlayerFacts`, `HudPlayer`, `SoundEffects` and
      `AudioHost`;
    - everything else: `LoadedRom`, `DecompressionHost`, `SessionRam`, `MapFacts`, the
      presentation and FX access traits, `MapHost`, `NativeImage`, `ObjectAccess`,
      `escape_countdown::Access` and `screen_fade::Owner`.

    Stage 6 already runs `advance_intro_player` and `initialize_player` on one lease.
    `spawn_power_bomb` and `spawn_hdma` are generic over hosts `Runtime` implements
    (`explosion_window::Host`, `hdma_objects::Allocator`). `update_beam_tiles_and_palette`
    is generic over a lease (`player_drawing`).
  - **Still missing on `Runtime` (16).**
    - Typed Session access, independent of other lanes:
      - `LevelData` (the Session terrain; `cinematic.rs` reads the size and level words);
      - `BackgroundBlocks` (`QueuedTilemaps`);
      - `menus::state::Access`, `menu_window_state::Access` and `ArrowAccess`;
      - `InventoryAccess`;
      - `SpriteWords` (after which `SpriteHost` is an empty impl);
      - `ResetRequest` (the `GAME_STATE` word);
      - `ActorImage` (the lease's actor records).
    - `Mode7Queue` and `cinematic_host::Host::advance_ending_tile_upload`: Game writes
      $02D0..$032F as raw RAM. On a Legacy lease that is the same raw write. Under an owned
      Display the bytes still need their semantic counterpart (stage 4's note).
    - `effects::DisplayRefresh`. `display_refresh::fade` and `set_blank` are generic but
      bounded on `display_refresh::Host`. Their bodies use only the screen fade, the display
      mirrors and `Video`. Narrowing those bounds, and `initialize_io_and_oam`'s, leaves
      the cinematic graph needing `display_refresh::Host` only for `GraphicsAsset::load`.
    - `RuntimeAccess`. The cinematic graph makes 14 nested `.runtime()` or `with_session`
      calls: `cinematic.rs` 3, `intro_pages` 4, `ending_sequence` 4,
      `shooting_star_motion` 2 and `cinematic_states` 1, plus
      `palette_effects::execution::leased`. On the lease they take the lease itself, in
      one of two ways:
      - `RuntimeAccess for Runtime` hands out a child lease that returns its Session to
        the parent on drop (stage 9's rule);
      - once nothing on Game calls the graph, those helpers take the lease directly.
    - `MenuRuntime`. The cinematic graph calls only `cinematic_sprite_*`, `spawn_hdma`,
      `rtl_unreachable`, `assert_ram_pointer` and `update_beam_tiles_and_palette`.
      Splitting the trait, as fc7 and fc8 narrowed theirs, avoids implementing
      `run_hdma_objects` and `handle_music_queue` on the lease. Implementing them there
      would nest the HDMA lease or run the music on Game.
    - `display_refresh::Host`: **blocked on lane `tick-s4b`**.
      - 83 `GraphicsAsset` loads decompress into WRAM ($7E:2000.., $7F:0000..), then DMA
        from it: `ending_sequence` 50, `intro_sequence` 26, `title_sequence` 5 and
        `cinematic_states` 2. The DMA is `start_graphics_dma`, plus `fill_vram_low_bytes`
        in `load_planes`.
      - On the lease the decompression goes to the Session, so the DMA first needs
        `Runtime::publish()` and a DMA start port.
      - The cinematic states do not reach the NMI methods of `display_refresh::Host`
        (`upload_samus`, `upload_animated_tiles`, `process_mode7`, `process_readbacks`,
        `flush_tilemaps`). A narrow graphics-load trait keeps them off the lease.
  - **Order once `tick-s4b` lands.**
    1. Land the typed impls and the bound narrowing above, each `[exact]`.
    2. Put `GraphicsAsset::load` behind a graphics-load trait, with its DMA start on
       s4b's port.
    3. In `gameplay_frame::frame`, keep step 6's lease open for `Native` states and run
       the state on it through a `PassLease` method. The lease drops before
       `advance_sound_effects`, as it now drops before the dispatch, so no lease is added.
    4. Turn the copies into `lease.session_mut()` fields. `intro`'s split copy goes,
       because Samus's frame runs on the same lease.
  - **Rejected.**
    - A host trait that returns a `FadePlanes` or `Cinematic` copy, with a Game impl in sm/
      that calls `session_cohort`. It would lower the count without changing what the
      engine reads.
    - `foreign` delegations for the DMA starts and the NMI-only methods. They would raise
      `foreign_calls` by about 12.

- **Done (lane `tick-s7c`, on `87e62488b`).** Not yet replayed. Ratchet:
  `sm_game_ram_sites` 11 → 0, `sm_game_coupled_files` 4 → 0, `session_entries` 88 → 77,
  `game_host_impls` 83 → 82, `legacy_engine_ram_sites` 444 → 441, `foreign_calls` 55.
  - `gameplay_frame::frame` keeps step 6's lease open for the `Native` states and runs
    them through `PassLease::advance_native_state`; the frame ends on Game after it
    drops. A state resuming after a yield (`wait_for_nmis`) has no step 6 lease, so it
    opens one there, in place of the several its helpers opened on Game.
  - `cinematic_host` is two traits: `Scene`, which the boot Nintendo logo still runs on
    Game, and `Host`, the lease. Runtime's impls are in `sm/src/game/cinematic_lease.rs`.
    `MenuRuntime` split off `CommonRuntime` and `CinematicRuntime`, so the lease never
    implements `run_hdma_objects` or `handle_music_queue`. New supertraits that Game
    also needs (`GraphicsLoad`, `LevelReads`, `Tables`, `CommonRuntime`) are blanket
    impls over Game's existing hosts, so no Game impl was added.
  - `GraphicsAsset::load` takes `display_refresh::GraphicsLoad`. On the lease its DMA
    publishes, then starts on #217's wire, and its decompression is
    `Runtime::decompress_to_ram`, which takes the decoded bytes back. A WRAM-port load's
    written bytes come back through `start_dma`. The VRAM fill is device writes only.
  - The copies: the palette helpers and the two reads (`ending_screens` fade plane,
    `cinematic_objects` shot words) are Session field access. The `Cinematic` table is
    lent from the lease (`Cinematic::lend`): the functions take it beside the lease,
    and the verify build still holds its words.
  - The demo's `load_title_sequence_graphics` runs on one lease (`Runtime::new` in
    `gameplay.rs`), which replaces the leases its palette effects opened.
  - **Left.**
    - `intro`'s split loan stays. Samus's frame reads the table as enemy projectiles
      straight from the Session, so the lent copy must be stored before her frame. It
      goes when the cinematic functions borrow the table from the Session in place of
      a copy.
    - The Mode 7 queue ($02D0..$032F) and its tail ($0334) still have no Display owner
      field. On a Legacy lease `set_byte` writes the same raw RAM Game wrote. Under a
      persistent owned Display a store there marks the Display failed, so an Owned
      verification start inside the title or the ending would fail there. Adding the
      field needs a format bump.
    - The level data size word ($7F:0000) has no Session owner; `LevelReads` reads it
      through the lease's projection.
  - **Rows.** `samus-scripted-intro`, `demo-attract-setup`, title and logo,
    file select, `credits`, `credits-ending`, the `credits-*` rows, `area-ceres`,
    `boss-ceres-ridley`, `pause-menu-tour`, and the SFX-loss rows `item-speed-booster`,
    `enemy-fire-arc` and `music-fir-selector10`.

### Stage 8: pause hooks, save slots and demo on owners

- **Change.**
  - Kraid's pause counters (`kraid_pause.rs`: 4 named words and a raw RAM import).
  - The save slot block copies ($0A00 player data, $D7C0, $CD52, the explored map; 8 in
    `save_slots.rs`).
  - The demo's $D8F0 and explored map clears (2 in `demo_rooms.rs`), and
    `gameplay_frame.rs`'s $0996 clear.
  - Each needs a Session owner and all its writers. The checkpoint formats bump (D5).
- **Delta.** About −16.
- **Rows.**
  - Kraid: the four `boss-kraid-pause-*` rows.
  - Saves: `save-death-reload`, `save-memory`, `samus-gunship-save`, `file-copy-clear`.
  - Demo: `demo-attract-setup`.

### Stage 9: the inversion, one lease per tick

- **Change.**
  - `with_frame_session` opens one `Runtime` (the tick lease) and calls `gameplay_frame::tick`
    with it. `gameplay_frame::Host` and its supertraits move to `Runtime`.
  - `RuntimeAccess::runtime()` on the tick lease is the lease itself. The sm_game call sites
    drop `game.runtime()` for the host they already hold.
  - In verify builds, opening a lease while the tick lease is alive panics.
  - Game's domains (section 1) stay in the Session for the tick, so `begin_session_interval`
    and `take_game_owners` run once.
  - Within the tick, the kept Session machinery (`KeptSession`, the per-release clone,
    `take_written_bytes`) goes. It stays across the tick boundary, for settlement's RAM
    writes and the lockstep rewind.
  - The two retention slots, the Owned `AggregateSlot` and the Legacy `frame_session.kept`,
    merge into one tick Session slot (D4).
- **Delta.**
  - `session_entries` 116 → about 30.
  - `game_host_impls` −1 to −6: gameplay_frame, hud, gameplay_rules, display_refresh,
    samus_death and menus on Game.
  - Speed: PERFORMANCE target 3.
- **Rows.** The full PR Verify set, plus a dispatched nightly full route before landing.
- **First slice (lane `tick-s9`, on `tick-s6`).** Not yet landed.
  - A main gameplay frame (state 8, its pass from the start) holds one lease from the
    random number step to the frame's end: the random step, the sprite table resets, the
    Samus request clear, the game state read, the pass, health and time, the frame timers,
    the resume point, the sound effect advance, the sprite finish, the joypad filter word
    and the continuation check. An enemy's message box splits it in two, as before.
  - `finish_health_and_time` and `finish_frame` run on the pass's last lease in every pass
    state (`gameplay_rules::Host` is now a lease bound). Their own leases go: the frame
    counter's, the quake's enemy shake and the pose command on death.
  - The sound effects leave through a cue port on the lease (`Runtime::emit_cue`); the APU
    side reads only the upload flags from RAM there, which verify builds check against the
    lease's audio owner.
  - Leases on a plain main gameplay frame with HDMA running: 4 to 2 (the HDMA
    interpreter's and the frame's). With a message box: 5 to 3. `foreign_calls` stays at
    168: none of the moved code had a foreign wrapper. `session_entries` 100 to 99.
  - Left: `handle_music_queue` (step 4) opens `update_hdma` on Game, before any lease;
    the next lease (the HDMA interpreter's) opens after the explosion spawn and FX
    publication on Game, so joining needs stage 5 or a reorder. The NMI and its
    `NmiPending` store stay on Game (stage 10). Other pass states, resumed passes and the
    demo's pass still end the frame on Game.
- **Second slice (lane `tick-s9b`, on `3dd546550`).** Not yet replayed. Ratchet:
  `session_entries` 70 → 50, `game_host_impls` 81 → 78, `foreign_calls` 43.
  - `update_hdma` runs on the frame's first lease through `UpdateHost for Runtime`, as
    message boxes already did: the music queue steps on the Session's audio owner and
    emits through `emit_cue`; the time-frozen flag, the explosion status and the FX
    blending are the lease's; the HDMA programs run on it (`run_objects_on`) and the
    layer blending reads its room key from it. A main gameplay frame with HDMA running
    now holds one lease (was 2, or 3 when the blending read the room key).
  - The pass states with no pre-pass work (fade-in, unpause, demo start, pause fade-out,
    Zebes escape, time-up, prepare-death, death blackout) run their pass from its start
    on that lease; their finish and the frame's end stay on Game. The door states 9-11
    resume the room entry on it (`PassLease::resume_room_entry`).
  - Consecutive leases merged: the room load's enables and liquid physics (5 → 1),
    the demo stop's clears (3 → 1), the unpause reload (4 → 1) and resume (2 → 1), the
    death steps (pose, palettes, explosion table), and the demo room load (5 → 2).
    Samus's position in the death pose and the demo load is read from the Session.
  - Deleted with no callers: five `PlayerRuntime` entries, `progress::leased::boss_word_at`,
    and Game's `TouchHost`, `ShotHost` and `RoomDma` impls.
  - **Left.** The other Game actor hosts (`BossState`, `BossLease`, `ActorRuntime`,
    `EnemyHost`, `Deaths` and others) are bound by the actor leases' associated `Game`
    type, by `room_state` and by verification cases (`room_mechanisms.rs`,
    `inventory_restore_cases.rs`, `flags_candidate.rs`); deleting them together leaves
    18 unsatisfied bounds. The `progress::leased` and `animated_tiles::execution::leased`
    facades are re-exported to sm/ Game callers and verification candidates. Menus,
    pause, game over, the demo's play and the per-state pass finishes still run on Game.
  - **Rows.** The chain rooms, `door-transition`, `death`, `save-death-reload`,
    `pause-menu-tour`, `options-menu-tour`, file select, `demo-attract-setup`, the intro,
    `credits`, and the SFX-loss rows `item-speed-booster`, `enemy-fire-arc` and
    `music-fir-selector10`. The `lease counters:` lines should show `leases=` falling
    against main at the same `ticks=` and `nested=` unchanged.
- **Third slice (lane `tick-s9c`, on `1fc085dca`, `collab/tick-s9b`).** Not yet
  replayed. Ratchet: `session_entries` 50 → 48, `game_host_impls` 78 → 60,
  `foreign_calls` 43.
  - Every pass state's frame runs on the frame's first lease, from the HDMA handler to
    the frame's end: the pre-pass work (the Ceres escape timer, the reserve tanks'
    refill and its pose command), the pass from any resume point, an enemy's message
    box (`message_boxes::display` on the lease, as the room mechanisms' boxes already
    ran), the per-state finish (`finish_pass`, now generic over `PassLease`) and the
    frame's end (`close_frame`). A resumed pass and the resume-skip byte take one lease
    in `advance_gameplay`; the demo's play loop runs its pass there without the frame's
    end. `Host` loses `set_resume_point`, `update_hdma` and
    `advance_low_health_warning` (a `PassLease` method now, Samus's lease rule).
  - The last three foreign calls in sm_game (UFCS, which the ratchet's `.foreign(`
    pattern did not count) went to the lease: Draygon's breath bubbles
    (`lease_owned_step`), Draygon's BG layers (`owned_step` over
    `room_visuals::initialize_draygon_layer`, now generic over the HDMA allocator and
    the Session) and the species tables' unknown routine (`Fatal::unreachable`).
  - With no foreign call left, `BossLease`, `ActorLease` and
    `mother_brain_fight::Lease` lose their associated `Game` type and `foreign`, with
    about 40 `BossLease<Game: ...>` bounds. The bosses' Game hosts then had no
    instantiation and went: `BossState`, `BossLease`, `AiHost` (trait too),
    `torizo::Ai` (trait too), `DrawHookHost`, `LeaseActors`, `BossEffects`,
    `ActorRuntime`, `EnemyHost`, `SpeciesHost`, `Deaths`, `phantoon::Dispatch`,
    `mother_brain_fight::Host` and `OwnedWords`, crocomire's and Magdollite's hosts,
    `DisplayObjects` and the overflow `SlotBytes`. The species `touch`, `shot` and
    `draw_hook` dispatchers had no caller and went with their hosts. Verification
    needed one edit (`inventory_restore_cases.rs` leases with `Runtime::new`, as
    `LeaseScope` on Game did); `room_mechanisms.rs` and `flags_candidate.rs` needed none.
  - **Left.**
    - Menus, pause (setup, frame, unpause, the fade-in and fade-outs), game over, the
      time-up exit, the death states, the demo reset, stop, next and play loop, and
      the initial room load still run on Game, then `finish_state`. They are generic
      over `menus::host::Host` (and `display_refresh::Host` for the death states),
      which Runtime lacks: `LevelData`, `SpriteWords` (so `SpriteHost`),
      `InventoryAccess`, `menu_window_state::Access` and `ArrowAccess`, `ActorImage`
      (an excluded facade: narrow it out first), `RuntimeAccess` (the lease itself),
      `MenuRuntime` (`load_vram_words`, `write_dma_register`, `handle_music_queue` and
      `run_hdma_objects` exist on the lease as `DialogueHost`'s music, sounds and HDMA
      update; `flush_sram`, the observers and the diagnostics are Game-only), and
      Host's own tables and controller words (`sram`, the menu, slot, demo and palette
      tables, `nmi_controller_word`, `controller_word`), which the lease can answer
      from `samus_game()` as `Tables` does. Few of the menu modules open nested leases
      (`demo_rooms` 9, `samus_death` 7, `save_slots` 2), so the work is the host
      surface: narrow each state's bounds (as `refill_auto` and `reset_gameplay_layers`
      were here), implement the rest on Runtime, then run the state on the frame's
      first lease as `Native` states do.
    - Game hosts still needed by the Game-side states: `actors::host::Random`
      (`kraid_pause`, the pause hooks), `hdma_program::LeaseHdma` (Game's
      `UpdateHost`, the menus' `run_hdma_objects`), `escape_countdown::Access`,
      `NativeImage`, `SamusSources`, `grapple_swing::TileAccess` and `ObjectAccess`
      (all through `gameplay_frame::Host` or `menus::host::Host` on Game).
  - **Rows.** Every pass state: the chain rooms, `door-transition`, `death`,
    `save-death-reload`, the Ceres rows (`area-ceres`, `tas-ceres-four-doors`, the
    escape), the reserve tank rows, `demo-attract-setup`, `pause-menu-tour`; the
    bosses (`boss-*`, Draygon's in particular, for the bubbles and the layers); the
    `item-*` rows if an enemy queues a box; and the SFX-loss rows
    `item-speed-booster`, `enemy-fire-arc` and `music-fir-selector10`. The
    `lease counters:` lines should show `leases=` falling against main at the same
    `ticks=`, with `nested=` unchanged.
- **Fourth slice (lane `tick-s9d`, on `f2d2d7a86`, `collab/tick-s9c`).** Not yet
  replayed. Ratchet: `session_entries` 48 → 29, `game_host_impls` 60 → 60,
  `foreign_calls` 43.
  - On the frame's first lease, to the frame's end (`lease_frame`: the state, then
    `close_frame` on the same lease): the death states (`samus_death::Lease`; its
    six nested leases go, and `samus_death::Host`), the time-up exit, the demo's
    reset, play loop (its pass is `run_pass` on the frame's lease), stop and next
    step, the initial room load (`initialize`, with `demo_rooms::load`,
    `finish_setup` and `run_room_hook`), and the game over fade-out. The cinematic
    and door states end their frame on the lease too, in place of `finish_state` on
    Game. A resumed lease state opens the frame's lease once in the dispatch.
  - New lease ports: `PassLease::load_title_sequence_graphics` and
    `initialize_player` (both left `gameplay_frame::Host`), `demo_rooms::Tables`
    (a blanket over the menus host for Game; the Runtime answers from
    `samus_game()`), and the supertraits `samus_death::Lease`,
    `demo_rooms::TableReads`, `NativeDiagnostics`, `projection::Views`,
    `CommonRuntime`, `GraphicsLoad` and `save_slots::MapHost`.
    `display_refresh::initialize_gameplay` is bound on `GraphicsLoad`. The demo
    hook's actor edit is the lease's `edit_actor`, not `ActorImage`.
  - Deleted with no callers: `progress::leased::set_boss_byte_at`,
    `set_boss_word_le` and `hdma_effects::apply_leased`. `flags_candidate.rs` runs
    the demo load and hook on `Runtime::new`.
  - **Left.** Every state still on Game reaches `menus::host::Host`: the file,
    map, options and game over menus, the pause fade-in, setup, frame and unpause
    (and the unpause fade's pause screen draw). Until they all run on the lease,
    no Game host impl loses its last caller; `grapple_swing::TileAccess` is the
    exception, now called only by `verification/grapple_tile_alias_checks.rs`.
    Implementing `menus::host::Host` for Runtime, the probe found:
    - `RuntimeAccess` comes in through `MenuRuntime: PlayerRuntime` and
      `SpriteHost`. Dropping it leaves 68 sites in the menu graph: `with_session`
      (35), `screen_fade::Owner` (14, add it as a supertrait), `with_projectiles`
      and `with_file_select`, `session_cohort::{store, with}`, the
      `progress::leased` and `animated_tiles::execution::leased` facades, and
      `map_menu`'s room entry. Each needs a port that runs directly on the lease
      and opens one on Game, or the whole graph moves at once.
    - Eight Runtime impls collide with blankets over `menus::host::Host`
      (`BeamHost`, `DialogueHost`, `GradientHost`, `SampleHost`, `SetupHost`,
      `SlotHost`, `Tables`, `save_slots::MapHost`): bound those blankets on
      `Host + RuntimeAccess`.
    - Still missing on Runtime: `ActorImage` (narrow it out: only `game_over`'s
      baby and the demo hook used it), `LevelData`, `SpriteWords`,
      `InventoryAccess`, `menu_window_state::Access`, `ArrowAccess`, `MenuRuntime`
      and Host's own tables and controller words.
  - **Rows.** `death`, `save-death-reload`, game over, the time-up exit (the Ceres
    escape rows), `demo-attract-setup`, the initial power-on and file loads, the
    chain rooms, the intro and `credits` (their frame end moved), and the SFX-loss
    rows `item-speed-booster`, `enemy-fire-arc` and `music-fir-selector10`. The
    `lease counters:` lines should show `leases=` falling against the base at the
    same `ticks=`, with `nested=` unchanged.
- **Fifth slice (lane `tick-s9e`, on `2523e1fd1`, `collab/tick-s9d`).** Not yet
  replayed. Ratchet: `game_host_impls` 60 → 60, `session_entries` 29 → 6,
  `foreign_calls` 43.
  - Every game state's frame now runs on the frame's first lease, to the frame's
    end: the options, file select and file select map menus, game over, the pause
    fade-in, setup, frame, unpause and unpause fade-out, and the null, unused and
    debug states. Game's `finish_state` is gone; Game's frame keeps only the boot
    resume, the NMI (`display_refresh::run`) and the latched controller sample.
  - `runtime_access::Lend` lends the owned-Session view for one call: a blanket
    over `RuntimeAccess` opens a fresh lease on Game; the lease lends itself inside
    `owned_player_call`. `with_session`, `with_projectiles`, the
    `progress::leased` and `animated_tiles::execution::leased` facades,
    `PlayerRuntime` and the FX setup's store bind on it. Part of the
    `session_entries` fall is these facades' `.runtime()` calls collapsing into
    the blanket's one; the moved states' own leases (the unpause's two, the map
    menu's room entry, the pause's animated tiles) are the rest.
  - `menus::host::Host` takes `Lend`, `screen_fade::Owner`, `SlotHost`, `MapHost`
    and `SampleHost`, and loses `RuntimeAccess`, `ActorImage`, `LevelData`, `sram`,
    `save_slot_offsets` and the controller words (now `SampleHost`'s
    `latched_controller_word`). SRAM flushes are `SlotHost::flush_slots`. The
    lease implements the menus host and `MenuRuntime`; the blankets over the menus
    host bind on `Host + RuntimeAccess` (Game alone) beside the lease's own impls.
  - The pause and unpause hooks are `PassLease` ports. Kraid's bind on the
    graphics, readback, tilemap and actor ports; Draygon's next IRQ handler goes
    through `write_external`. The baby and Kraid read actor 0 through
    `ActorAccess`; `ActorImage` and Game's `Random` are deleted. Game gained
    `SlotHost` and `SampleHost` impls with its former blankets' bodies.
  - **Left.** `menus::host::Host for Game` still has callers outside the frame, so
    it and its supertraits' Game impls (`MenuRuntime`, `SpriteHost`,
    `SpriteWords`, `NativeImage`, `ObjectAccess`, `escape_countdown::Access`,
    `LeaseHdma` through `spawn_hdma_owned`, `MapFacts`, `InventoryAccess`, the
    window and arrow access) stay:
    - boot: `nintendo_logo` (`cinematic_host::Scene` needs `menus::host::Tables`,
      Game's through the blanket) and `save_slots::verify_boot`, which loads the
      three slots on Game before any frame lease;
    - verification: `player_legacy_edges`'s map check runs `render_map` on Game and
      asserts the physical RAM bytes between rotations (on a lease they are not
      published until it closes, so the check has to read the lease's projection
      to keep its strength), and `flags_candidate` / `flags_runtime_checks` run
      `save_slots::initialize` and `load` on Game.
    Narrowing `load`, `initialize` and `verify_boot` to `SlotHost` plus the state
    ports, giving Scene its own tables port, and moving the three checks onto
    `Runtime::new` (D7) would let `Host for Game` and about a dozen impls go.
    `SamusSources`, `LevelData` and `grapple_swing::TileAccess` have callers
    outside the menus (Samus animation, room loads, display refresh and the
    grapple alias check).
  - **Rows.** `options-menu-tour`, `options-menu-return-parameter`,
    `pause-menu-tour`, `file-copy-clear`, the file select and save rows
    (`save-memory`, `save-death-reload`), the map rows, game over,
    `boss-kraid-pause-*` and the Draygon and Mother Brain rooms' pauses, the chain
    rooms, and the SFX-loss rows `item-speed-booster`, `enemy-fire-arc` and
    `music-fir-selector10`. `lease counters:` should show `leases=` falling on the
    menu and pause rows at the same `ticks=`, with `nested=` unchanged.
- **Sixth slice (lane `hosts-1`, on `13e0f2318`, `collab/tick-s9e`).** Not yet
  replayed. Ratchet: `game_host_impls` 60 → 50, `session_entries` 6 → 5 (the
  deleted `UpdateHost` blanket's lease), `legacy_engine_ram_sites` 393 → 387,
  `foreign_calls` 43.
  - `menus::host::Host` and `MenuRuntime` for Game are deleted, with
    `SpriteHost`, `SpriteWords`, `InventoryAccess`, `menu_window_state::Access`
    and `ArrowAccess`, `MapFacts`, `escape_countdown::Access`, `LeaseHdma` and
    `grapple_swing::TileAccess`. So are the blankets over `Host + RuntimeAccess`
    and `MenuRuntime + RuntimeAccess` (`BeamHost`, the demo `Tables`,
    `Allocator`, `GradientHost`, `CommonRuntime`, `save_slots::MapHost`,
    `DialogueHost`, the FX `SetupHost`) and `UpdateHost`'s blanket: only the
    lease implements them now.
  - Boot stays on Game, on narrow ports. `save_slots::load`, `initialize` and
    `verify_boot` bind on `LoadHost` (`SlotHost`, the menu and sequence state,
    the display history, `Lend`). `cinematic_host::Scene`'s
    `menus::host::Tables` is a direct Game impl (+1), not the menus host's
    blanket.
  - Verification on the lease (D6, D7):
    - `player_legacy_edges`'s map check renders on `Runtime::new`. The lease's
      `MenuRuntime::observe_map_write` checks each rotation on the lease's
      projection (the exploration byte, and the destination tile through the
      workspace). Before the lease closes the check asserts it holds the whole
      rendered map; after, the published physical RAM, as before.
    - `flags_candidate` (cases 60, 63..66) and `flags_runtime_checks` run the
      new game and the load on `Runtime::new`; in the runtime checks that lease
      opens inside the existing `foreign` call.
    - `grapple_tile_alias_checks` runs each prefix and tail case on a lease,
      asserting the lease's projection (after the panic, for the two failing
      tails) and then physical RAM. Its Game comparison is now the bare update on
      a lease against the lease's owned-call `update_grapple_tiles`.
  - **Census: the 50 Game impls left**, from deleting each in turn and
    compiling (sm in both cfgs and the detached player). "Logo" means the only
    caller is `boot::resume`'s `nintendo_logo` through `Scene`.
    - Logo only (10): `TilemapQueue`, `menus::host::Tables`, `Mode7Queue`,
      `palette_effects::fading::Host`, `BackgroundBlocks`, `fx_lighting::Access`,
      `presentation_workspace::{Access, PaletteAccess, QueueAccess}`,
      `dialogue_ram::Access`. Running the logo on the frame's lease, or giving it
      a typed boot host, retires all ten.
    - Logo plus a little more: `fx_domain::Sources` (Game's own FX encoding),
      `PlayerPosition` (`sm_a9`), `ObjectAccess` and `sprite_output::MapHost`
      (boot's high table and `lease_drawing`'s forwarding to Game; the sprite
      effect observer), `menus::state::Access` (boot's `verify_boot`, Game's own
      menu commit), `SlotHost` (`verify_boot`, `flags_save_candidate`),
      `LevelData` (boot's `LevelReads` through the logo, `room_state::
      decompress_level` and the lease's `decompress_level` on the published
      image).
    - The Game frame (`gameplay_frame::Host` for Game, called from `cpu_infra`):
      `gameplay_frame::Host`, `display_refresh::{Host, DisplayOwner,
      ReadbackQueue}`, `Video`, `QueueAccess`, `Scrolling`, `HdmaTable`,
      `PlayerFacts`, `HudPlayer`, `HudFrameHost`, `ResetRequest` and `SampleHost`
      (the latched controller sample), `ContinuationAccess`, `fx_domain::
      Controls`, `SoundEffects`, `DisplayRefresh`, `Records`, `LoadedRom`,
      `SessionRam`, `screen_fade::Owner`. These go with stage 10 (the NMI inside
      the tick).
    - Broad (frame, boot, sm/ runtime and verification): `RuntimeAccess`
      (`flags_candidate`, `animated_tiles`, `enemy_projectiles_candidate`,
      `room_mechanisms`), `CompatibilityRam`, `display::preparation::Access`
      (`scanline_irq`, `rtl`, `room_world_cases`), `sequence_state::Access`,
      `gameplay_context::Access`, `display_scroll::Access` (`scroll_runtime`),
      `AudioHost` (`music`, `session_audio_checkpoint_cases`), `LoadedImage`
      (`lease_drawing`, Crocomire, Magdollite), `NativeImage`,
      `NativeDiagnostics`, `DecompressionHost` (room loads on the published
      image), and `SamusSources` (the persistent Session's body drawing on Game,
      `display_transfers`, `samus_animation`, the Samus binding and persistent
      Session cases).
  - **Rows.** Any power-on row (boot and the logo), the file select and save
    rows (`save-memory`, `save-death-reload`, `file-copy-clear`), the map rows,
    game over, and the verifier's flags (cases 60, 63..66 and the runtime
    guard), save, map (`player_legacy_edges`) and grapple alias cases. The
    engine paths are unchanged, so the replays should match the base exactly;
    the three verification checks are the ones to watch.
- **Seventh slice (lane `hosts-2`, on `39b853c05`, `collab/hosts-1`).** Not yet
  replayed. Ratchet: `game_host_impls` 50 → 37, `legacy_engine_ram_sites`
  387 → 381, `foreign_calls` 43, `session_entries` 5.
  - **The logo on a lease (option (b)).** `boot::resume` runs `nintendo_logo` on
    one `Runtime::new` per boot frame that draws the logo, in place of the leases
    its Game hosts opened there (the stage-0 `with_session`, the screen fade's
    `Runtime::new` on each fade frame). Those frames have no other lease: they
    yield before the frame's own lease. The logo's last frame (stage 6) is the
    one boot's own work and the frame's lease follow, so it opens no lease:
    `end_nintendo_logo` resumes the NMI and blanks the screen on Game's frame
    ports, as the loop's stage 6 did. `nintendo_logo` is generic over
    `cinematic_host::Host`; `Scene` has no Game implementer now.
    - **Lease counts to check.** On a power-on row, `lease counters:` should
      show `nested=` unchanged at the same `ticks=`. Not measured here: the
      hold frames (stages 2 and 3) may now open one lease where they opened
      none, and the stage-0 and fade frames one in place of theirs.
  - **Deleted for Game (13).** The ten logo-only impls (`TilemapQueue`,
    `menus::host::Tables`, `Mode7Queue`, `fading::Host`, `BackgroundBlocks`,
    `fx_lighting::Access`, the workspace `Access`/`PaletteAccess`/`QueueAccess`,
    `dialogue_ram::Access`), plus:
    - `fx_domain::Sources`, with Game's parallax helpers. Its only other caller
      was itself.
    - `PlayerPosition`, with `sm_a9`'s two Mother Brain wrappers. The push
      checks (`player_collision_ownership`, cases 76..87) call
      `mother_brain_fight`'s moves on `Runtime::new`, the lease the fight runs
      them on, and assert physical RAM after it closes (D7).
    - `LevelData`. The room load's plane split on the published image
      (`Runtime::decompress_level`, and `room_state::decompress_level` for the
      room copy check) is Game's own step, `Game::copy_level_planes` with the
      same body. Game's `level_word`, `level_behavior` and `clear_level_data`
      had no caller.
  - **Boot's tile-buffer fill** ($7E:3000, $4000, $6000) stores its bytes on
    Game in `boot.rs`, as the deleted workspace impl did.
  - **Moved but kept.**
    - `flags_save_candidate` saves on `Runtime::new`, as the engine does; its
      observer takes each stage from the lease's projection.
    - The lease's spritemap warnings call Game's diagnostics directly
      (`warn_invalid_sprite_rom`, `set_sprite_failure`), not Game's `MapHost`.
  - **Left of the 7, and why.**
    - `ObjectAccess`: boot's `clear_high_table` and `finish_frame`, between
      boot's Game work and before the frame's lease. On a lease they would be a
      mid-frame lease. `MapHost` for Game also needs it.
    - `sprite_output::MapHost`: the sprite effect observer's draw inside
      `foreign` (cases 111..116). Its `draw` hook seeds RAM mid-draw and its
      diagnostic hook swaps the loaded image, both on Game. Drawing on the lease
      after a `foreign` seed would change what cases 111..113 capture.
    - `SlotHost` and `menus::state::Access`: boot's `verify_boot`. On a Legacy
      Session it opens no lease today (Game's menu commit leases only a current
      persistent Session), so a lease for it would be mid-frame. These go when
      boot's own work runs on the frame's lease.
  - **Census: the 37 Game impls left** (`ram_ratchet.py`; the three `crate::`
    traits `Wire`, `FlushHost` and `ProjectedBytes` are not counted).
    - Boot (4): `ObjectAccess`, `MapHost` (and the sprite observer),
      `SlotHost`, `menus::state::Access`.
    - The Game frame (`gameplay_frame::Host` for Game), as the sixth slice lists
      them. A deletion probe of `HudPlayer`, `PlayerFacts`, `Records`,
      `fx_domain::Controls`, `NativeDiagnostics` and `display_scroll::Access`
      fails first at `gameplay.rs` (`gameplay_frame::Host for Game`). These go
      with stage 10.
    - Broad, unchanged from the sixth slice: `RuntimeAccess`,
      `CompatibilityRam`, `display::preparation::Access`,
      `sequence_state::Access`, `gameplay_context::Access`,
      `display_scroll::Access`, `AudioHost`, `LoadedImage`, `NativeImage`,
      `NativeDiagnostics`, `DecompressionHost`, `SamusSources`.
  - **Rows.** Every power-on and boot row (the logo), `demo-attract-setup`, the
    title, file select and the save rows (`save-memory`, `save-death-reload`,
    `file-copy-clear`), the chain rooms (room loads), and the verifier's room
    world cases 76..87 (Mother Brain pushes), the room copy check, the flags save
    cases and the sprite effect cases 111..116.

### Stage 10: NMI and settlement inside the tick

- **Goal.** The NMI, the controller latch and settlement run as tick-scheduled
  steps on the frame's lease, at the point the emulated NMI fires, with the same
  writes in the same order. Removing the register writes from sm_game was done
  under D9 (lane `d9`, 2026-11-04): sm_game makes no register write; see
  owned-display-registers.md, "Retirement step 1 under D9".
- **What ran on Game in a frame (base `947f8db62`), in order.**

  | # | Step | Where | Writes |
  | --- | --- | --- | --- |
  | 1 | `latch_controller` | Game (device) | the joypad register the auto-read latches |
  | 2 | `begin_session_interval` | Game | Game's held domains adopted |
  | 3 | Boot (`$0611` 1..3, `boot::resume`): `clear_work_ram`, the logo on one `Runtime::new` per logo frame, then boot's work: the audio init cue, blank, `clear_work_ram(0x10000)`, NMI off, `initialize_registers`, the tile buffers at $7E:3000/4000/6000, the APU queue bytes, the OAM pointers, `clear_high_table` and `finish_frame` (`ObjectAccess`), the Samus upload words, NMI on, the APU ports, `RANDOM_NUMBER`, `reset_delays`, `ENABLE_DEBUG`, `save_slots::verify_boot` (`LoadHost`), the sound words, `$0611 = 2` | Game, before any frame lease | RAM, registers, APU |
  | 4 | `clear_pass_continuations` | Game | $0612..0615 |
  | 5 | The frame's lease: `update_hdma`, `start_frame`, the state, `close_frame` (stage 9) | lease | Session |
  | 6 | The lease closes (deferred export) | | |
  | 7 | `NmiPending = 1` when the state completed; `display_refresh::run` (`scene::commit::refresh`): palette and OAM DMA ($0370, $7E:C000), Samus tiles ($071D..0722 descriptors, cartridge source), animated tiles (a fresh lease through `Lend`), the register mirrors, HDMA channel binds, the Mode 7 queue ($02D0, tail $0334), the tilemap flush (queue end word, tail, scrolling enables), the VRAM readbacks into WRAM ($0340, tail $0360), HDMA enable, the NMI counters; or, with no request, the missed-NMI counters | Game | registers, VRAM/OAM/CGRAM, WRAM (readbacks, queue stores, counters) |
  | 8 | `controller_input::sample_latched` when the NMI was requested | Game | `control[0].input.ports[0]` |
  | 9 | Soft reset check, `$0611` | Game | $0611 |
  | 10 | `with_frame_session` ends | | |
  | 11 | `draw_frame_to_ppu` (`cpu_infra.rs`): `samus_mutation`, then `scene::settle::settle` over `NativeSettlement(Game)`: per line the four beam events (`advance_beam`: raster, HDMA reading its tables from WRAM), and at `vTimer` the IRQ (`scanline_irq::advance`: `scene::irq::advance` on `ScanlineIrq(Game)`, display mirrors and history words, registers; the door resume poll `follow_transition`, one `Runtime::new` each); `finish_current`, `reconcile_current` | Game | registers, WRAM (IRQ stores, the door resume's lease) |

- **Design.**
  - **One lease per frame.** `gameplay_frame::frame` decides how the frame
    starts (a yielding boot frame, a frame that begins, a resumed state), opens
    the frame's lease once, runs the state on it and then `end_frame` on it: the
    NMI request, `display_refresh::run`, the latched sample, the soft reset
    check and the next frame state. Done in the first slice (below).
  - **The NMI on the lease.** `display_refresh::run` binds on the `Nmi` port:
    the frame display, the scroll words, `Video`, the HDMA objects, and the
    refresh's five uploads. `scene::commit::refresh` is unchanged, so the order
    is the original's; only the host changes. On the lease:
    - palette, OAM and the register commits go through the lease's `Video`
      (publish, then the transfer port; device-only commands in a device
      interval), as every other lease display command does;
    - Samus's tiles (`Runtime::upload_samus_tiles`) publish, then run the
      native upload on Game's RAM: it reads the descriptors and the cartridge
      and writes registers and VRAM alone;
    - the animated tiles run on the lease itself (it lends itself), where
      Game's NMI opened a fresh lease;
    - the Mode 7 and readback drains are generic over `DrainHost` and run on
      the lease's transfer port after a publish: each DMA start reads the
      published bytes (verify builds assert it), a readback's work RAM bytes
      return to the Session (`start_dma`), and the sentinels and tails are the
      lease's stores;
    - the tilemap flush is `flush_tilemaps_on_lease`, which the room scrolls
      already ran.
  - **Why byte-exact.** Each upload makes the same register writes in the same
    order, and each DMA reads the bytes it read on Game: Game read the lease's
    export (deferred, settled at the DMA's first RAM read), the lease reads its
    own publish of the same Session. RAM stores land in the Session and reach
    RAM at the release instead of in RAM and the next lease's write log. No
    emulated CPU runs inside the tick, and the beam starts at `settle`'s
    `reset_beam`, so moving Rust work from after the lease to inside it moves
    nothing relative to the emulated CPU.
  - **Lag frames.** `end_frame` keeps the original's logic: a state that yields
    (`returned != 0`, as door loading does) stores no request, so the refresh
    only counts the missed NMI, and the controller is latched only when an
    earlier request is still pending. `door-loading-lag-nmi` covers it.
  - **Boot's work (next slice).** Split `boot::resume` into its yielding part
    (Game and the logo's lease) and boot's work after `clear_work_ram(0x10000)`,
    and run the work as the frame's lease's first step, before `update_hdma`,
    in the same order. Ports: the workspace fill (`presentation_workspace`),
    the APU queue bytes and sound words (the audio owner), the OAM pointer
    words, `ObjectAccess` (the lease has it), NMI enable (`display_refresh::
    enable`), the APU ports (device), `RANDOM_NUMBER` and `ENABLE_DEBUG`
    (gameplay words), `reset_delays`, and `verify_boot` on `LoadHost` (the
    lease has `SlotHost` and the menu state). Then Game's `ObjectAccess`,
    `sprite_output::MapHost`'s boot use, `SlotHost` and `menus::state::Access`
    lose boot. Not in this slice: the lease would open on RAM cleared by
    `clear_work_ram(0x10000)` but before `initialize_registers` and boot's raw
    stores, and each of those raw `game.ram()` stores needs its lease form; the
    soft reset (`$0611` = 3) takes the same path.
  - **Settlement and the IRQ stay after the lease** (until the third slice,
    below, which moves them). Not provably byte-exact
    yet, for four reasons:
    1. HDMA reads its tables from WRAM on every line inside the emulated beam
       (`advance_beam`), between IRQ handlers whose RAM stores (the display
       history words, the door resume's Session writes) it reads on later
       lines. On the lease every IRQ that stores would need a publish before
       the next `advance_at`. That publish (`encode_spans`) would have to equal
       the release's export for every lane an IRQ writes, including Game's
       held frame display and scroll domains, which `ScanlineIrq(Game)` writes
       through Game's projection. Nobody has shown that.
    2. The door resume (`follow_transition`) opens `Runtime::new` per poll.
       Inside the tick lease that is a nested lease. On the tick lease it
       would be `room_entry::protocol::resume(&mut lease, ScrollIrq)` with a
       publish after each poll; the polls fall on `vTimer` lines, so its
       equivalence rests on (1).
    3. Settlement holds `samus_mutation()` across the whole beam and finishes
       it after; the lease takes Samus authority too. Nesting one inside the
       other changes ownership; it is not a move.
    4. `scanline_irq::table_check` (`SM_IRQ_TABLE`) plans each frame's
       interrupts from Game's controls when settlement begins.
    So the Session reaches RAM at the lease's release (the deferred export
    settles at HDMA's first table read), as today. The path: give settlement a
    WRAM view HDMA reads through (the lease's published image), make the IRQ's
    stores lease stores with a publish when dirty, and run the door resume on
    the tick lease; or, under D9, the native renderer's own HDMA replaces the
    register-level one.
- **Rows.**
  - Missed NMI: `door-loading-lag-nmi`.
  - IRQ door resume: `door-transition`, `elevator-up`.
  - Message boxes: `item-*`.
  - Mode 7: `boss-ceres-ridley`, `area-ceres`.
  - `credits`.
  - The nightly `SM_NATIVE_LINES` route.
- **First slice (lane `tick-s10`, on `947f8db62`, `collab/hosts-2`).** Not yet
  replayed. Ratchet: `game_host_impls` 37 → 25, `session_entries` 5 → 3,
  `legacy_engine_ram_sites` 381 → 342, `foreign_calls` 43.
  - Every frame opens one lease and ends on it (`frame`, `end_frame`): the
    state's frame, the NMI and the latched sample, the soft reset check and
    `$0611`. That covers a resumed pass (it opened its own lease in `pass` and
    then ran the NMI on Game) and a boot frame that yields after the logo's
    lease. `advance` runs only the frame; `pass` and `advance_gameplay` go.
  - The NMI runs on the lease through `display_refresh::Nmi`, as the design
    says. `gameplay_frame::Host` binds on `ContinuationAccess` and
    `RuntimeAccess` alone.
  - The Mode 7 and readback drains (`mode7_transfers`, `vram_read_queue`)
    take a `DrainHost` (the lease's transfer port). The record transfers
    (`upload_mode7`, `read_vram`) go through `Wire` with the same byte and word
    writes, in the same order, as `write_reg`/`write_reg_word`/`read_reg_word`.
  - Verification (D7): the routes that step frames by hand
    (`actor_composed_route`, `player_collision_composed_route`,
    `room_world_cases`) run the NMI on `Runtime::new`.
  - **Lease counts to check.** On `lease counters:` at the same `ticks=`,
    `nested=` should be unchanged. `leases=` should fall by one on every frame
    whose NMI was requested: Game's NMI opened a lease for the animated tiles,
    and a resumed pass's NMI ran after its lease. On a yielding boot frame
    with no request it rises by one, since the NMI now has a lease there.
  - **"Session written after the last lease".** The NMI's tilemap flush and the
    latch now come before the last lease ends, so the line should read
    `0 ticks` on the rows below. Where it could still show: settlement, which
    is after the lease. Its IRQ display stores are frame display words, held
    outside the Session in Legacy. The door resume's `Runtime::new` per poll
    is the tick's last lease on `door-transition`, `elevator-up` and
    `door-loading-lag-nmi`. A field there names a settlement writer.
  - **Census: the 25 Game impls left.** Found by deleting each in turn and
    compiling sm in the verify cfg (the detached player and `cargo check` at
    the end).
    - The Game frame (2): `gameplay_frame::Host`, and `screen_fade::Owner`
      (`RuntimeAccess`'s supertrait).
    - Boot, before the frame's lease (11): `ContinuationAccess` (the logo's
      `nintendo_logo_ending`), `Video`, `DisplayOwner` and
      `display::preparation::Access` (`end_nintendo_logo`, boot's display
      enable), `ObjectAccess` (`clear_high_table`, `finish_frame`), `SlotHost`
      and `menus::state::Access` (`verify_boot`), and `LoadHost`'s other
      supertraits, `LoadedImage`, `NativeImage`, `NativeDiagnostics` and
      `Records`. Some of these have callers outside boot as well (below).
      These go with boot's work on the frame's lease.
    - The oracle checkpoint resume (`verify/checkpoint.rs`,
      `resume_original_main_loop`) latches the next frame's controller on
      Game: `SampleHost`, `ResetRequest`, `CompatibilityRam`,
      `display_scroll::Access`, `sequence_state::Access` and
      `gameplay_context::Access`. The lease's own `latched_controller_word`
      also forwards to Game's. The resume runs on the oracle's RAM in the
      middle of snesrev's main loop. A lease there would decode and export
      that image, and the codec's round trip on a mid-loop oracle image is
      not known to be the identity, so it stays on Game.
    - Broad: `RuntimeAccess`, `SamusSources` (the Samus animation, and the
      NMI's Samus upload, which runs on Game's published RAM),
      `sprite_output::MapHost` (the sprite effect observer, cases 111..116),
      `DecompressionHost` (`room_state`'s plane split), `AudioHost` (`music`,
      the audio checkpoint cases), `SessionRam`.
  - **Left, and why.** Settlement and the IRQ: see the design above. Boot's
    work: the next slice. The NMI's Samus upload still runs native code on
    Game's RAM after a publish. A lease host for `samus_tile_upload` (the
    descriptors from the Session, `capture_table` on `samus_game()`) would
    retire that.
  - **Rows.** `door-loading-lag-nmi` (missed NMI), `door-transition` and
    `elevator-up` (the IRQ door resume, after the lease), pause and unpause
    (`pause-menu-tour`, `boss-kraid-pause-*`), the `item-*` message boxes,
    `credits`, the Mode 7 rows (`boss-ceres-ridley`, `area-ceres`: the Mode 7
    queue on the lease), the X-ray and Kraid pause readbacks (the readback
    queue on the lease), any power-on row (the logo's frames now end on a
    lease), the chain rooms, the SFX-loss rows `item-speed-booster`,
    `enemy-fire-arc` and `music-fir-selector10`, and the nightly
    `SM_NATIVE_LINES` route. RAM lockstep must stay at 0 failing frames.
    The verifier's room world cases and the two composed routes should match
    exactly.

- **Second slice (lane `tick-s10b`, on `2a5535115`, `collab/tick-s10`).** Not yet
  replayed. Ratchet: `game_host_impls` 25 → 19, `session_entries` 3 (unchanged),
  `legacy_engine_ram_sites` 342 → 336, `foreign_calls` 43.
  - **Samus's NMI tile upload** reads its descriptors ($7E:071D..0722) from the
    lease (`Runtime::upload_samus_tiles`), not from Game's RAM. The cartridge
    and the device are still Game's (`NativeSamusUpload`). The publish before it
    stays, so RAM is where the NMI's later uploads found it.
  - **The oracle checkpoint resume** (`verify/checkpoint.rs`) latches the next
    frame's controller through `OracleRam`, a verification host over the
    oracle's work RAM (D6, D7). It decodes nothing and exports nothing, so there
    is no round trip. Why it is exact:
    - The checkpoint holds no Session (`require_legacy_binary`), and the
      restore has just written every RAM byte, which moves every held
      domain's epoch. So each byte Game's held images served is the RAM byte.
    - Game's stores were byte stores to RAM (`compatibility_write_byte`). The
      host makes the same stores, in the same order.
    - The controllers and the soft reset stay Game's inherent methods
      (`latched_controller_word`, `sampled_controller_word`, `request_reset`).
    - Game's `SampleHost` and `ResetRequest` are deleted.
  - **Boot.** Game's part of `boot::resume` now ends after the Samus copy words.
    The rest is `gameplay_frame::finish_boot`, the frame lease's first step
    (`Start::Booted`), in the original order: the NMI enable
    (`display_refresh::enable`), the APU ports' clear (`clear_apu_ports`, a
    device port that reads only the upload flags, checked as at a cue),
    `RANDOM_NUMBER` (`Field::RandomNumber`), the music delays
    (`music_requests::reset_delays`), `ENABLE_DEBUG`, `verify_boot` on the
    lease's `LoadHost`, `DEBUG_DISABLE_SOUNDS`, the downtime
    (`music_requests::clear_downtime`) and `$0611 = 2`.
    - The logo's last frame (`end_nintendo_logo`) runs on its own lease, as
      the other logo frames do. The leases follow each other and are never
      nested.
    - Why it is exact: the frame lease opens on the RAM Game left after its
      device work, and the lease's typed stores are the projection of the old
      raw stores. The frame's `clear_pass_continuations` ($0612..0615) now
      comes before these stores; none of them touches those bytes.
    - With nothing left to call them, Game's `SlotHost`, `Records`,
      `menus::state::Access` and `DisplayOwner` are deleted, as is Game's NMI
      enable. Game's menu state is now an inherent method that only its own
      byte projection uses.
    - **Lease counts.** The logo's last frame gains a lease. On boot's work
      frame, `verify_boot`'s `Lend` calls borrow the frame lease instead of
      opening fresh ones, so `leases=` falls by that many. `nested=` is
      unchanged.
  - **Boot, left on Game, and the lease form each step needs.** These come
    after `clear_work_ram(0x10000)`, in order:
    - NMITIMEN off and its mirror: `apply_mirrored` with
      `Control::Interrupts`.
    - The `$51` brightness mirror store: a frame display field store.
    - `initialize_registers`. Today it is `register_writes!` on Game. On a
      lease it would be `apply_mirrored` or `apply_controls` per register,
      which needs a `Control` for every register written: BGnSC, BGnNBA, the
      doubled scroll writes, VMAIN, M7A..M7Y, the window registers, TM/TS and
      their window masks, CGWSEL/CGADSUB, COLDATA and SETINI.
      `initialize_io_and_oam` is already generic. The raw `$64`, `$71` and
      `$74..76` stores are frame display mirror fields.
    - The tile buffers: a presentation workspace fill on the lease.
    - The APU queue bytes ($0643.., $0656/66/76): the audio owner's
      `audio_store_byte`.
    - The OAM pointer words: a Views field.
    - `clear_high_table` and `finish_frame`: the lease already has
      `ObjectAccess`.
    - The Samus copy words ($071D..0722): player Session fields.

    Once those move, Game's `ObjectAccess` is left with one other user,
    `sprite_output::MapHost` for Game. Only the sprite effect observer
    (`sprite_effect_observer::draw_on_lease`, cases 111..116) uses that, inside
    `foreign`, so both go when that observer draws on the lease.
  - **Settlement, `ScanlineIrq` and the IRQ door resume (investigated; not
    moved).** None of these is a byte-exact move yet. Each needs the
    following:
    1. *Publish equals the release export for the held lanes.* An IRQ handler
       stores these lanes:
       - `irq::Store`: the pending and current interrupt words ($A7, $AB) and
         the enables ($84/$85).
       - `irq::Output`: registers.
       - The door resume: Session state.

       On a Legacy lease the frame display ranges (`frame_display_ranges`) and
       $B1..$C0 go to Game's held domains through the lease's byte path, so
       those stores need no publish. With a display-owned Session they live in
       the display state, and HDMA reads its tables from WRAM on the next line
       (`advance_beam`).

       The proof needed: a verify-build check that, after each handler that
       stores, `encode_spans` of the lease writes the same bytes, for those
       lanes, as today's Game path leaves in RAM. Run it on `door-transition`,
       `elevator-up` and the nightly `SM_NATIVE_LINES` route.
    2. *An IRQ resume that does not nest a lease.* Today `follow_transition`
       opens a `Runtime::new` per poll after the frame lease has closed. These
       leases come one after another, but they are extra leases. Settlement
       would have to run inside the frame lease, after `end_frame`, as
       `ScanlineIrq(lease)` with `room_entry::protocol::resume(lease,
       ScrollIrq)`, and with a `publish` after each poll that stored, before
       the next `advance_at`.

       `draw_frame_to_ppu` holds `samus_mutation()` across the beam and then
       runs `finish_current` and `reconcile_current`. The lease already holds
       Samus authority, so those would fold into the lease's release instead
       of wrapping it.
    3. *Plan `table_check` from the Session.* `table_check::begin_frame` reads
       the six interrupt controls and five mirrors through
       `ScanlineIrq(Game)`. Planning at the end of `end_frame` on the lease,
       through `ScanlineIrq(lease)` plus the device's `vTimer` and
       `vIrqEnabled`, reads the same values. Between the lease's end and
       `begin_frame` only the controller latch, the soft reset check and
       `$0611` are written. It needs a verify-only hook from `end_frame`.
  - **Census: the 19 Game impls left.** Found by deleting each impl in turn
    and compiling sm in the verify cfg.

    | Impl | Callers |
    | --- | --- |
    | `gameplay_frame::Host` | the tick entries in `cpu_infra.rs` |
    | `RuntimeAccess` | `Host`'s supertrait, and `Lend` for Game: `room_mechanisms`, and the verification candidates (`flags_candidate`, `animated_tiles`, `enemy_projectiles_candidate`) |
    | `screen_fade::Owner` | `RuntimeAccess`'s supertrait, and `screen_fade_runtime`'s Game entries |
    | `ContinuationAccess` | `Host`'s supertrait alone (`outer_continuation` and `clear_pass_continuations` before the lease) |
    | `Video` | settlement's `ScanlineIrq(Game)` (`scanline_irq.rs`), `display_refresh::initialize_io_and_oam` (boot), `room_world_cases` |
    | `display::preparation::Access` | `ScanlineIrq(Game)`, `rtl::write_register_and_mirror` (boot), `initialize_io_and_oam`, `room_world_cases` |
    | `CompatibilityRam` | `ScanlineIrq(Game)`, `ProjectedBytes` for Game, `room_world_cases` |
    | `display_scroll::Access` | `CompatibilityRam`'s supertrait, `scroll_runtime`, Game's own scroll bytes |
    | `sequence_state::Access` | `CompatibilityRam`'s supertrait, Game's `gameplay_context` grapple words |
    | `gameplay_context::Access` | `CompatibilityRam`'s supertrait, and the lease's fallbacks to Game (`session_runtime.rs` `gameplay_context` for Runtime) |
    | `ObjectAccess` | boot (`clear_high_table`, `finish_frame`), `MapHost`'s supertrait |
    | `sprite_output::MapHost` | `sprite_effect_observer` (verification, inside `foreign`) |
    | `LoadedImage` | `crocomire_reactions` and `magdollite` tables on Game, `MapHost`'s supertrait |
    | `NativeImage`, `NativeDiagnostics` | `SamusSources`' supertraits, `samus_animation`, the Samus draws on the lease (`session_runtime.rs`), the persistent-session and Samus binding verification cases |
    | `SamusSources` | the Samus upload's tables (`NativeSamusUpload::table`, `capture_table`), `samus_animation`, the lease's Samus draws |
    | `DecompressionHost` | `room_state::decompress_level`, the lease's `decompress_to_ram` and `decompress_level` |
    | `SessionRam` | `DecompressionHost`'s supertrait alone |
    | `persistent_session::AudioHost` | `music.rs` (the music queue on Game), the audio checkpoint cases, the palette and animated runtime checks |
  - **`session_entries` (3) is unchanged.** The three are the frame's lease
    (`gameplay_frame::frame`), `Lend`'s blanket for a `RuntimeAccess` host
    (`runtime_access.rs`) and `session_cohort.rs`'s lease. The second goes when
    Game stops being `RuntimeAccess`.
  - **Rows.** Every power-on and boot row (the logo's last frame, boot's
    work on the lease); file select and the other save-slot rows
    (`verify_boot`); the checkpoint and oracle-sample rows (`OracleRam`, which
    every checkpoint resume runs under an oracle that reads the controller in
    NMI); `door-transition`, `door-loading-lag-nmi` and `elevator-up` (Samus's
    upload in every NMI, settlement unchanged); the chain rooms; and the
    SFX-loss rows `item-speed-booster`, `enemy-fire-arc` and
    `music-fir-selector10` (the APU ports, music delays and downtime at boot).
    RAM lockstep must stay at 0 failing frames.

- **Third slice (lane `tick-s10c`, on `5bdb96711`, after #236).** Not yet
  replayed. Ratchet: `game_host_impls` 19 → 18, `session_entries` 3
  (unchanged), `legacy_engine_ram_sites` 336, `foreign_calls` 43.
  - **Order to land: the check first.** The first commit (`SM_IRQ_PUBLISH_CHECK`,
    `scanline_irq::publish_check`) is the proof the settlement move needs: it
    replays each interrupt pass of Game's path on a copy of the lease's
    Session and compares the publish with RAM. It hooks Game's interrupt
    pass, which the engine's frames leave in the next commit, so its engine
    numbers come only from a run of that first commit: `door-transition`,
    `elevator-up` and the nightly `SM_NATIVE_LINES` route, with
    `lane_mismatches=0 read_mismatches=0 other_mismatches=0` wanted. If
    lockstep fails on the move, drop the move and read the check.
  - **Settlement on the frame's lease.** `gameplay_frame::frame` ends with
    `PassLease::settle_frame` after `end_frame`, on the same lease
    (`scanline_irq::settle_on_lease`):
    - the lease publishes, so HDMA's first table read finds the bytes the
      release's deferred export would have put there;
    - the beam is `SettlementHost` for `Runtime`, over the device steps
      Game's settlement uses too (`cpu_infra::beam`);
    - each interrupt pass runs on `ScanlineIrq(lease)` (`advance_on_lease`),
      and publishes after a control store or a resume poll, before the next
      beam event reads work RAM;
    - the IRQ door resume (`follow_transition`) runs
      `room_entry::protocol::resume(lease, ScrollIrq)` inside
      `owned_player_call` on the frame's lease. Its observations are
      refreshed as the fresh lease Game's poll opened left them. No lease
      opens inside the frame's lease, and none nests;
    - `table_check` plans from the lease (`begin_frame_on`).
    - The engine's callers (`run_one_frame_of_game_both`, `RM_MINE`) end with
      `finish_settled_frame`: the Samus mutation and Session adoption that
      followed Game's settlement, without the beam. The oracle's frames and
      the hand-stepped verification routes keep `draw_frame_to_ppu` on Game.
  - **What changes besides the host.** A door resume frame's next tick now
    takes the kept frame Session up. Game's poll lease released outside the
    tick and exported without keeping, so that tick decoded RAM. `leases=`
    falls by one per resume poll; `nested=` is unchanged. The "Session
    written after the last lease" line should read `0 ticks` on
    `door-transition`, `elevator-up` and `door-loading-lag-nmi`.
  - **Game's `ContinuationAccess` is deleted.** Its only caller was
    `gameplay_frame::Host`'s supertrait. The frame reads $0611 and clears
    $0612..0615 before its lease through `Host::frame_state` and
    `Host::clear_pass_continuations`, which make the same byte reads and
    stores.
  - **Boot.** `clear_high_table` and `finish_frame` run first in `finish_boot`,
    on the frame's lease, through its `ObjectAccess`. Their bytes (the object
    table and its high bits) and the Samus upload words boot still clears on Game are separate, so the
    stores only trade places. Game's `ObjectAccess` stays: `MapHost` for Game
    (the sprite effect observer) needs it.
  - **Left, and why.**
    - Game's `Video`, `display::preparation::Access` and `CompatibilityRam`
      (and its three supertraits): the oracle's settlement still runs
      `ScanlineIrq(Game)`. It needs a verification host over the oracle's
      RAM, as `OracleRam` does for the checkpoint latch, that makes the same
      held-domain reads and stores. Boot's `initialize_registers`,
      `write_register_and_mirror` and `initialize_io_and_oam`, and
      `room_world_cases`, use them as well.
    - Boot's remaining Game work (NMITIMEN off, `$51`, `initialize_registers`,
      the tile buffers, the APU queue bytes, the OAM pointer and Samus upload
      words) needs the lease forms the second slice lists. Moving it also
      opens the frame's lease on RAM before those raw stores, which nobody
      has checked.
    - `session_entries` (3): see the second slice. None of this slice's moves
      touches the three.
  - **Rows.** `door-transition`, `elevator-up` and `door-loading-lag-nmi` (the
    resume poll on the lease); the Mode 7 and HDMA rows (`boss-ceres-ridley`,
    `area-ceres`, X-ray, the Kraid pause, the FX rooms); the `item-*` message
    boxes; `credits`; every power-on and boot row (the object table's clear on
    the lease); the chain rooms; the SFX-loss rows `item-speed-booster`,
    `enemy-fire-arc` and `music-fir-selector10`; and the nightly
    `SM_NATIVE_LINES` route with `SM_IRQ_TABLE` and `SM_HDMA_TABLE`. RAM
    lockstep must stay at 0 failing frames.

- **Fourth slice (lane `game-hosts`, on `117522061`, `collab/tick-s10c`).** Not
  yet replayed. Ratchet: `game_host_impls` 18 → 7, `session_entries` 3
  (unchanged), `foreign_calls` 43 → 42.
  - **Boot's display setup on the frame's lease.** NMITIMEN off, the `$51`
    store, `initialize_registers` and its raw `$64`, `$71`, `$74..76` stores are
    `display_refresh::initialize_boot_registers`, a typed control table and the
    first step of `finish_boot`. Why it is exact: those steps store only the
    display mirrors ($51..$86) and no register write reads RAM. Boot's other
    stores on Game (the tile buffers, APU queue bytes, OAM pointer and Samus
    upload words) are other bytes, so each byte keeps its store order. The
    lease now opens on cleared mirrors and stores each one.
  - **The oracle's settlement on a verification host.** `oracle_host::
    OracleHost` (sm/src/verification, D6) serves `Video`,
    `display::preparation::Access`, `CompatibilityRam` and its supertraits, and
    `AudioHost`, by forwarding to the projection Game's impls made, which are
    now Game's inherent methods. `scanline_irq::advance`, `table_check`'s plan,
    `room_world_cases` and the audio, palette and animated checks run on it.
    The lease's fallbacks (scroll, gameplay context, screen fade) call Game's
    inherent methods.
  - **The sprite effect observer (cases 111..116)** stores its slot changes
    through the lease's byte path and draws with `draw_objects`, not inside
    `foreign`. Game's `MapHost` and `ObjectAccess` go.
  - **Also gone:** Game's `LoadedImage` (a `RomImage` is its own), its
    `screen_fade::Owner` (`RuntimeAccess` and `Lend` no longer require it), and
    music.rs's Game wrappers, which nothing called.
  - **The 7 left.** Engine-needed, each for a stated reason:
    - `gameplay_frame::Host` and `RuntimeAccess`: the tick entry and the frame's
      lease (`game.runtime()`). They go when the runtime owns the tick.
      `RuntimeAccess` also gives `Lend` for Game to `room_mechanisms` and the
      verification candidates.
    - `DecompressionHost` and `SessionRam` (its supertrait): the lease's
      `decompress_to_ram` and `decompress_level` run the decoder on the
      published image and take back the bytes it wrote. The decoder could run on
      the lease's byte path instead, but that needs a replay of the room loads.
      `room_state::decompress_level` on Game is used only by `room_level_copy`.
    - `SamusSources`, `NativeImage` and `NativeDiagnostics`: the Samus content
      catalogs Game holds, which the lease's Samus draws, `samus_animation` and
      the NMI's Samus upload read. They need a content view that is not Game,
      because the lease lends its state and Game at the same time.
  - **`session_entries` (3)** is unchanged: the frame's lease, `Lend`'s
    blanket for `RuntimeAccess` and `session_cohort`'s lease all need the
    runtime to own the tick.
  - **Rows.** Every power-on and soft reset row (boot's registers on the
    lease); the oracle frames and checkpoint rows, `door-transition`,
    `elevator-up` and the nightly `SM_NATIVE_LINES` route with `SM_IRQ_TABLE`
    (the oracle's settlement on `OracleHost`); the verifier's room world cases,
    sprite effect cases 111..116, the audio checkpoint cases and the palette
    and animated runtime checks; and the Crocomire and Magdollite rows. RAM
    lockstep must stay at 0 failing frames.

- **Fifth slice (lane `no-foreign`, on `b9da49f22`).** Not yet replayed.
  Ratchet: `game_host_impls` 7 → 0, `session_entries` 3 → 0, `foreign_calls`
  42 (unchanged), `engine_ram_export_lines` 4692 (unchanged).
  - **The runtime owns the tick** (`4d04e08f5`, exact). sm/'s `gameplay::tick`
    makes the device's steps before the lease (the controller latch,
    `begin_session_interval`, `boot::resume`, the invalid-state failure, the
    pass continuations' clear: the same Game calls in the same order), then
    opens the one lease and calls `gameplay_frame::frame(start, &mut lease)`.
    `Continuation::of` classifies $0611; `Start` is public. The frame is
    generic over `FrameLease` (the bounds `RuntimeAccess::Runtime` had).
    `gameplay_frame::Host` and Game's impl go. The out-of-table panic no
    longer drops the lease first.
  - **No `RuntimeAccess`** (`cc1ab06d3`, exact). Game's impl goes and with it
    the trait, `Lend`'s blanket, `lend_admissible`, `with_projectiles`'
    projection fallback and `session_cohort::{store, with, with_projected}`,
    none of which had another caller. The sm/ entries that leased Game (the
    flag entries, the animated tile cases, the projectile room spawn and
    `spawn_hardcoded`) call the engine function on `Runtime::new(game)`, which
    is what Game's `Lend` ran; the engine's `leased` facades for them go.
  - **Samus content view** (`7ae0215f1`, exact). `samus_binding::SamusContent`
    (the owner and its images) serves `SamusSources`; `samus_drawing::Host::
    samus_sources` returns it by value. `owned_samus::SamusDrawing` serves the
    three owned draws and records their native diagnostics, which the lease
    makes on Game in order once the draw returns (a draw reads only the view
    and the Session). Game's `SamusSources`, `NativeImage` and
    `NativeDiagnostics` go.
  - **The decoder on the lease's byte path** (`f9391a493`, behaviour: lease
    merge). The room load's `decompress_to_ram` and `decompress_level` run on
    the lease's existing `DecompressionHost` (as the boss, options and display
    refresh decodes did) with the plane split on the lease's bytes, instead of
    publish, decode on Game, take back. Game's `DecompressionHost` and
    `SessionRam` go. Not exact by construction: back-references read the
    lease's bytes, where Game read its projection (its held menu bytes at
    $7E:9E20.. fall inside the CRE tile buffer). The room level copy check
    runs on `Runtime::new`; its case 2 panics mid-copy on the lease.
  - **Left: `foreign` (42, all verification, D7).** Classified by reading
    each site:
    - 36 check `foreign`'s own contract (RAM equals the Session at entry, the
      callback's writes visible after, the unwind keeps the prefix or
      quarantines, the world and identity mismatch rejections, Samus authority
      revoked inside): `actor_owner_checks` 2, `actor_runtime_checks` 3 (one
      nests a lease inside `foreign`), `animated_runtime_checks` 1,
      `enemy_native_runtime_cases` 2, `flags_runtime_checks` 5 (one,
      `next_random`, runs Game's RNG), `palette_runtime_checks` 1,
      `player_collision_owned_checks` 1, `player_collision_ownership` 3,
      `player_projectiles` 3, `room_mechanism_ownership` 3,
      `room_runtime_checks` 3, `samus_game_binding_cases` 5,
      `sprite_runtime_checks` 2, `visual_runtime_checks` 2. Their subject goes
      with `foreign`; the same properties for the lease are its open (decode)
      and drop (export), which a rewrite would assert as drop, act on Game,
      `Runtime::new`.
    - 6 run verification hooks on Game mid-lease and need a port that reads
      the lease (as the sprite effect observer's did):
      `actor_population_animation_entry` 1, `actor_population_animation_
      observer` 2, `room_mechanism_callback_edges` 1,
      `room_mechanism_qualified` 2.
  - **The RAM export stays.** The lease's drop still exports the Session to
    RAM: the device reads it (the NMI's DMA after `publish`, settlement's
    HDMA tables, the Samus upload), as do lockstep's RAM compare, the
    checkpoints and every check above. It goes after the Session-space
    compare and the native renderer (end state steps 1 and 3).
  - **Rows.** Every row runs the new tick entry; watch the boot and power-on
    rows, `door-transition`, `door-loading-lag-nmi` and `elevator-up` (the
    frame's start), the room loads (the chain rooms, `door-transition`, the
    save and death reloads, `demo-attract-setup`: the decoder), the Samus rows
    and the verifier's persistent Session and Samus binding cases (the
    content view), the flags, animated tile, enemy projectile and room level
    copy cases (their new entries), and the SFX-loss rows
    `item-speed-booster`, `enemy-fire-arc` and `music-fir-selector10`. RAM
    lockstep must stay at 0 failing frames.
- **Sixth slice (lane `delete-foreign`, on `2c228ee77`, `collab/no-foreign`).** Not
  yet replayed. Ratchet: `foreign_calls` 42 → 0, `engine_ram_export_lines` 4692 →
  4620, `legacy_engine_ram_sites` 325 → 320.
  - **The six hooks on the lease** (`5b12f0252`, exact). The population stage,
    opcode and $A2:804C observers run over a detached image of the lease's
    projected RAM and the Session's native spawn words (`observe_hook`) and store
    back what they change; the initializer check reads the payload the lease
    holds, and both nested spawns run on the same lease. The native read trace
    registers its replacement through the lease; the qualified mechanism calls
    store through its byte path. Each keeps the observation refresh
    (`owned_player_call`).
  - **`foreign` deleted** (`e6a49972c`, exact: no replay called it). Under D7 the
    36 contract checks go with it, keeping what still exists as drop, act on
    Game, `Runtime::new` (the publication and import, the mismatched world's
    silent drop, the unwind after a hand-off); the unwind and Samus revocation
    cases run on `owned_player_call`, and the returning-Die load runs under the
    Samus mutation. Gone with it: the resume path, the carried presentation
    stack and the suspended-display reads, the source interval and the lease
    counters' nested and foreign counts.
  - **The codec's foreign-only helpers** (`dee38dbd6`, exact):
    `physical_source_spans` and `suspended_domain_byte`, and the lease traits'
    docs. Game's source publications are now never tracked (only foreign
    tracked them); capture and restore remain and can go next.
  - **The RAM export stays** for the device (the NMI's DMA after `publish`,
    settlement's HDMA, the Samus upload), lockstep's RAM compare, checkpoints and
    the verification checks that read Game after a drop.

### Stage 11: the rest

- **Change.**
  - Family lease merges of the remaining about 160 `foreign` calls, as in #184/#187: player
    commands (17), projectile spawns, ROM records, palettes and events.
  - The 39 verification-case `foreign` calls retire with the path they test (D7).
  - The sm_game machinery (`ram.rs` 22, `word_transfers.rs` 18, `projection.rs` 3) goes when
    sm/ stops calling those macros (D6).
  - Then `Runtime::foreign` is deleted.
- **End state.** `Game` implements only device ports: registers and DMA, the cue sink, the
  RAM image publish for comparison, and diagnostics.

### Where the 189 go

| Stage | Sites |
| --- | ---: |
| 5 | 16 |
| 7 | 19 |
| 8 | 16 |
| 3b | 1 |
| 10 | 77 |
| 11 (stragglers: Crocomire, morph ball eye, Torizo; leftover `RawRam` imports) | about 12 |
| 11 (machinery) | 43 |
| Unattributed | about 5, the `menu_state.rs` doc-comment hit among them |

## 4. Risks

1. **A mid-frame lease (the SFX loss).** Commit `65b617006` opened fresh leases from
   Game-hosted code (`session_part`). It lost a queued sound effect at $0643..$0665 on
   `item-speed-booster`, `enemy-fire-arc` and `music-fir-selector10`; CI bisect run
   41643239578 isolated it (typed-slot-views.md).
   - **Cause.** Not isolated. The machinery has several places a lease boundary can
     reorder state: the single shared write log (`open_write_log` drops an earlier log), the
     deferred export settling at first access, and nested `Runtime::new` inside `foreign`
     (which imports fresh and encodes at once).
   - **Mitigation.**
     - No stage adds a lease. Stages 6 and 9 only remove them.
     - Stage 0 counts nested leases.
     - Stage 9 makes them a verify-build panic.
     - Stage 3a asserts the audio bytes are outside every export layout, and proves itself
       on those three rows first.
2. **Verifier normalization.**
   - **What is masked.** `NORMALIZED_RAM` (`cpu_infra.rs:440`) masks 31 ranges in both the
     RAM compare and the lock digests (`machine.rs` zeroes them). Among them:
     - $0000..$0051, direct page, including `pointer_scratch`;
     - $0611..$0617, the coroutine state that `frame()` writes;
     - $0E54.., the current enemy;
     - $186E..$1886.
   - **The risk.** A stage that moves a writer of a masked byte onto the Session can change
     it unseen until it propagates.
   - **Mitigation.**
     - For stages 6, 9 and 10, which touch $0611..$0615 and direct page, run lockstep
       locally with the mask dropped (`verifier-normalized-ranges`).
     - Keep the typed observation, which compares Session fields, on.
   - **The observed Session changes.** `engine_projection` (`observation/mod.rs:186`)
     observes "the Session of the Tick's last frame lease". Stage 9 makes that the tick
     lease, and stage 10 moves it after NMI. Check that the observation is captured after
     every in-tick writer.
3. **Fewer kept-Session checks.** `assert_projects_ram` runs only when the export is not
   deferred. With one deferred export per tick, it runs rarely. Keep a periodic full decode
   check, every N ticks in the nightly run.
4. **Lockstep rewind.** The tick Session must never survive `restore_snapshot`
   (ENGINE_STATE_MIGRATION: "import the selected normalized state"). Today the write log's
   overflow forces a fresh import. Stage 9 adds an explicit invalidation at restore.
5. **Writers after gameplay.** Settlement's HDMA and IRQ write RAM after the tick lease. Until
   stage 10, they stay outside the lease, and the next tick takes their bytes through the
   write log.
6. **The persistent (Owned) path.** Verification starts use `AggregateSlot::Ready` and epoch
   checks. A direct RAM write on a persistent lease poisons it (`persistent_is_current`).
   Stages 3 and 4 keep the existing Owned branches.
7. **Ratchet monotonicity.** `foreign_calls` and `game_host_impls` may not rise. Any
   delegation a stage adds is removed in the same landing.

## 5. How the verifier and oracle keep working

- **Emulated backend.** The emulated backend stays the comparison path (migration policy).
  - `RM_THEIRS` and the oracle half of the paired frame are untouched.
  - The engine still produces the whole RAM image at the same point, after settlement. The
    snapshot, `verify_snapshots_eq`, `NORMALIZED_RAM`, the APU queue compare and the lock's
    machine digests read it as today.
- **RAM image and observation.** The image is the Session's encoding, published once per
  tick: "the byte image is an encoding during typed ownership"
  (ENGINE_STATE_MIGRATION). The typed observation compares the tick lease's Session.
  Whole-RAM comparison stays while the representations agree (README).
- **Comparison paths.** No comparison path is removed. Free run, check-lock, lockstep and
  the faithful oracle keep their entry points. `SM_NATIVE_LINES` keeps the native renderer's
  check.
- **Headless.** Headless runs still publish. Settlement's HDMA reads WRAM, so skipping the
  publish is a later speed step: publish only the HDMA source spans
  (`fx_encoding::physical_source_spans`). It is outside this plan.

## 6. Stage 1, scoped for one 2-3 hour lane

**Title.** Diagnostics: the leases report unreachable states without exporting the Session
`[exact]`.

**Scope.** The 62 `foreign` closures whose whole body is a diverging diagnostic:
- `unreachable`, `report_unreachable`, `rtl_unreachable`, `invalid_interrupt`,
  `legacy_assertion`;
- in 43 files: 29 in `crates/sm_game/src` (27 actor files, `hdma_program.rs`,
  `hdma_dispatch.rs`) and 14 in `sm/src/game` (`session_runtime.rs` 4, `hosts.rs` 2,
  `player_transitions_runtime.rs` 2, and one each in the rest).

Regenerate the list with a closure-body scan (whole body = one of those calls).
`native_rom_diagnostic` and `invalid_window` calls that also capture the ROM, and sound
checks with an `unreachable!` guard, are out of scope.

**Change.**
1. In `sm/src/rtl.rs`, expose the existing failure path without a `Game`:
   `pub fn unreachable() -> !` and `pub fn invalid_interrupt_failure() -> !`. Keep the
   current `report_unreachable(&mut Game)` for Game-hosted callers.
2. In sm_game, add `fn unreachable(&mut self) -> !` to the lease traits that the call sites
   hold. These are the actor lease (`actors::host`), the boss lease (`boss_lease`),
   `space_pirates`, Mother Brain's lease and `hdma_program::Interpreter`. Prefer one small
   shared trait, for example `crate::diagnostics::Fatal`, as a supertrait.
   - Implement it for `Runtime` by calling `crate::rtl::unreachable()` directly.
   - `Bridged` keeps calling the Game method.
   - The detached verification hosts (`player_detached/enemy_host.rs`) keep their panic.
3. Replace each `x.foreign(|game| game.unreachable())` (and the variants) with
   `x.unreachable()`.

**Checks.**
- `bash tools/lane/gate.sh` and clippy on the four packages.
- `python tools/ram_ratchet.py --update`: expect `foreign_calls` 274. The other measures do
  not change. `game_host_impls` may fall if a Game-only `unreachable` impl goes.
- PR Verify. No row can reach these paths and pass, so no extra rows are needed.

**Out of scope.** Everything else in section 3.

## 7. Against the target architecture: what Jim decides

The plan follows the migration policy: it keeps the emulated backend and the oracle, and
moves complete responsibilities with one owner per interval. Where it departs from the
destination, it proposes the following:

- **D1. The interim tick owner is sm/'s `Runtime`.** CONTRACTS: "Session has no emulator ...
  foreign gameplay callback". REVIEW: owners built in the RAM-compatible runtime "are not the
  goal".
  - *Proposal.* The tick entry stays `sm_game::gameplay_frame::tick<H>`. `Runtime` is only
    its migration host. Its removal condition is "H implements only device ports", after
    which `Session::step` wraps the tick.
- **D2. Typed device ports replace `foreign`.** Zero `foreign_calls` is reached partly by
  replacing calls with narrow ports: diagnostics (stage 1), registers and DMA with an
  explicit publish (stage 4), and the cue sink (stage 3b). This follows the `owned-pres`
  typed-command precedent.
  - *Decide.* That a port carrying no gameplay state, and touching WRAM only through
    `publish`, is a retirement and not a rename. The textual ratchet cannot tell the
    difference.
- **D3. Audio requests become a Session owner for every presentation (stage 3b).** This means
  a checkpoint format bump. CONTRACTS also wants cues as `TickOutput` effects, delivered
  after the step.
  - *Proposal.* Cues stay immediate, at the original call points, through the port, so the
    APU queue compare is unchanged. Batching them at the tick end is a separate, separately
    proved change.
- **D4. One retained Session slot (stage 9).** The Owned `AggregateSlot` and the Legacy
  frame `kept` Session merge into one. This touches checkpoint and adoption semantics
  (`session_native_adoption`, `SessionSnapshotPayload` is `None` in Legacy).
- **D5. New owners in stage 8**, each with its own format bump: the save block $0A00.., $D7C0,
  $CD52, $D8F0 and Kraid's pause counters.
- **D6. The machinery.** `ram.rs`, `word_transfers.rs` and `projection.rs` (17 sites since `collab/rs-infra`) serve
  sm/ callers. Moving them into sm/ would zero the sm_game count without retiring any access.
  - *Proposal.* They count until their sm/ callers go.
- **D7. The 39 `foreign` calls in `sm/src/verification`** test `foreign`'s export and import
  contract.
  - *Proposal.* Retire them with `foreign`, and do not port them.
- **D8. Two reported measures.** Add `runtime_opens` and the stage 0 per-tick counters to the
  ratchet output (reported, not strict), so that stages 6 and 9 are visible.
- **D9. The Owned presentation, or the native renderer, as the replay authority.** This is
  retirement step 1 in `owned-display-registers.md`, and still open. Stage 10's register
  writes leave sm_game only after it.
