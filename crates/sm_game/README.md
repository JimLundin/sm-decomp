# sm_game

Gameplay rules and the deterministic `Session`: the package that becomes the
engine. It depends only on `sm_data`, forbids `unsafe`, and cannot reach the
emulator or devices except through host traits that `sm` implements. CI checks the dependency tree and runs
`cargo test -p sm_game`.

Modules arrive here from `sm/src/game/` once they compile without `sm`. A
module that still needs the emulated machine declares a narrow `Host` trait;
`sm` implements it for `Game` or for a Runtime session lease, and keeps a
re-export shim at the old path. Code that still reads the RAM image through
`word!`, `slot!`, `word_transfers!`, register writes or raw RAM hosts has
moved but is not yet owned; `tools/cycle_report.py` counts those sites.

## Main modules

| Module | Contents |
| --- | --- |
| `session`, `persistent_session`, `session_owners`, `session_access` | The owned Session state, its start descriptions and access shared by the runtime and test harness |
| `session_encoding` | The one projection of owned Session state into the SNES RAM layout, used by the oracle comparison and checkpoints |
| `session_checkpoint`, `checkpoint_binding` | Checkpoint components and their layout |
| `gameplay_frame`, `dispatch` | Frame dispatch, ordered gameplay phases and resumable continuations |
| `player_movement`, `player_collision`, `player_transitions`, `player_update`, `player_projectiles` | Samus: locomotion, collision, pose transitions, per-frame update and beams |
| `actors/` | Enemies and bosses, from Etecoon and Dachora to Kraid, Draygon, Crocomire, Ridley and Mother Brain |
| `enemy_projectiles/` | Enemy projectiles and their scripts |
| `room_entry/`, `room_mechanisms/`, `room_scrolls/`, `room_state` | Room loading, room objects, scrolling and camera |
| `progress`, `item_pickups`, `save_file` | Inventory, pickups, world flags and saves |
| `scene/`, `display/`, `hdma*`, `palette_effects/`, `animated_tiles/` | Display state and the Scene handed to rendering |
| `menus/`, `message_boxes`, `message_composition/`, `hud` | Pause, map, equipment, file and options menus, message boxes, HUD |
| `imported`, `records` | `Imported<T>`: `sm_data` records tied to the cartridge image they came from |
| `ram` | Names for work RAM offsets still used by compat-facing code |

See the [accepted architecture](../../docs/target-architecture/README.md) for
the destination and [state migration](../../docs/ENGINE_STATE_MIGRATION.md)
for the rules on moving state out of RAM.
