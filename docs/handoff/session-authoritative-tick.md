# Session authoritative for the whole tick: a staged plan (2026-11-03)

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

### Stage 4: the lease's hardware port

- **Change.**
  - A `Runtime::publish()` writes the Session's spans to RAM, keeping the lease current. It is
    the same path as `relinquish`'s persistent publish (`session_runtime.rs:528-535`), without
    the release.
  - `Registers for Runtime` writes registers that touch no WRAM straight to the backend: the
    PPU registers other than the data ports, the DMA channel parameters, the multiplier and
    divider, HDMAEN, the IRQ timers.
  - It publishes, then writes, for an MDMAEN whose sources are in WRAM.
  - It falls back to `foreign` only for DMA or WRAM-port destinations in WRAM.
- **Verify builds.** They also publish before any write that `room_background_observer` or
  the shadow PPU observes, because they read RAM.
- **Delta.** About −12: the four `Registers` delegations, the room entry register writes,
  Yapping Maw's multiplicand, and `place_hdma`'s channel writes, which let spawns run on the
  lease.
- **Rows.** `enemy-yapping-maw`, `door-transition`, `area-crateria`, `area-maridia`,
  `boss-crocomire` (HDMA spawn), `mother-brain-beam-spawn-parameters`, `xray-activate-release`,
  `door-loading-lag-nmi`.

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

### Stage 10: NMI and settlement inside the tick

- **Change.**
  - `display_refresh::run` runs on the tick lease through stage 4's port; readbacks are taken
    back.
  - The single publish moves to just before `settle`, and `scanline_irq::advance` runs on
    `ScanlineIrq(&mut lease)`.
  - The NMI and IRQ register writes become typed display commands, encoded in sm/ (lane
    `owned-pres`).
  - Removing them from sm_game also needs D9.
- **Delta.** `display_refresh` 27, `hdma_objects` 4, `gameplay_frame` 3. With the
  message box, cinematic display and transfer slices, up to −77.
- **Rows.**
  - Missed NMI: `door-loading-lag-nmi`.
  - IRQ door resume: `door-transition`, `elevator-up`.
  - Message boxes: `item-*`.
  - Mode 7: `boss-ceres-ridley`, `area-ceres`.
  - `credits`.
  - The nightly `SM_NATIVE_LINES` route.

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
- **D6. The machinery.** `ram.rs`, `word_transfers.rs` and `projection.rs` (43 sites) serve
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
