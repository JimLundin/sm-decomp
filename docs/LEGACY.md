# Legacy behavior register

Behavior we keep only because a check depends on it, not because the game
needs it. Each entry is marked in the code with a `LEGACY(<kind>)` comment, so
the register and the code cannot drift:

```sh
grep -rn 'LEGACY(' --include=*.rs sm crates
```

Do not add new dependencies of these kinds. When a check is changed so it no
longer needs an entry, remove the held behavior, its tags and its row here in
the same change.

## `LEGACY(message-text)`: checks that compare failure text

The verifier asserts on the exact text of panics and `fatal_error` calls. That ties the
engine to wording it does not care about: production code has to spell out
`index out of bounds: the len is N but the index is i` by hand, and a file read
still reports `fread failed` after the C stream calls were replaced by
`std::fs`. The checks should assert the kind of failure (which table or read
rejected, and where execution stopped), not the words.

| Held in production | What is kept |
| --- | --- |
| `crates/sm_data/src/table.rs` (`check_index`) | slice-index panic text for jump-table selectors |
| `crates/sm_game/src/player_projectiles/{weapons,specials,bomb_terrain}.rs` | 11 hand-written slice-index panics |
| `crates/sm_game/src/actors/mother_brain_fight.rs` | 1 hand-written slice-index panic |
| `sm/src/rtl.rs` (`read_file_block`) | `fatal_error("fread failed\n")` on a short state-file read |

| Checks that compare the text | Tags |
| --- | --- |
| `sm/src/verification/native_content_checks.rs` | 7 |
| `sm/src/verification/player_movement_edges.rs` | 4 |
| `sm/src/verification/{actor_population_animation_cases,collision_mechanism_checks,player_minimap_edges}.rs` | 2 each |
| `sm/src/verify.rs` | 1 |
| `sm/src/verification/{actor_movement_cases,player_minimap_boss,room_background_cases,room_contact_start_cases,room_reference_cases,room_world_cases,room_world_dll,samus_game_binding_cases}.rs` | 1 each |

To retire an entry: change the check to assert the failure kind, then replace
the held text with whatever reads best (a plain slice index, an `assert!` with
its own message, or a typed error) and delete the tags.

## Faithful-oracle known divergences

`SM_ORACLE=faithful` runs the original game without snesrev's patches. Some
original behaviour copies uninitialised low-WRAM scratch (the direct page
$00-$50, the stack) into VRAM or RAM tables. The engine does not model that
scratch, and the effect has no gameplay consequence, so these patches stay
installed (verdict "a" in `tools/snesrev_patches.py`) and these faithful rows
are expected to fail at the listed frame. Owning the direct-page scratch in the
Session would retire the list.

| Patch | Faithful rows and first failure | What hardware shows |
| --- | --- | --- |
| `$82:831E` BG3/sprite tile DMA size | `samus-scripted-intro` @8256, `demo-attract-setup` @2619, route `chain-0011` @2054, any% chain room 11 (`chain-any-0011-DF45`) @2055 | WRAM $0000-$11FF in sprite VRAM $7700-$7FFF, overwritten before display |
| `$84:8243`, `$84:EFD3` unpause item-GFX load | `pause-menu-tour` @1378, `boss-kraid-pause-400` @588, `-950` @1138, `-1030` @1218, `-1960` @2148, route `chain-0211` @368 | garbage transfers into empty item-GFX slots |
| `$80:9AF3` broken mini-map init | `samus-scripted-intro` @8316, any% chain room 11 (`chain-any-0011-DF45`) @2115 | one frame of HUD mini-map tiles from scratch, while blanked |
| not yet identified (gameplay-visible) | `enemy-fire-arc` @119, in B3A5 before the fire arc's room | an enemy the engine kills survives (NUM_ENEMIES_KILLED_IN_ROOM 1, not 2): a snesrev patch decides the kill and the engine follows it, pending an oracle-fidelity fix |

## `LEGACY(verifier)`: RAM the frame comparison does not check

`normalize_snapshot_ram` (`sm/src/cpu_infra.rs`) copies these ranges from one
side to the other before each frame's RAM comparison, so a difference there
never fails a frame. "Theirs" copies the original's bytes over the engine's
(the engine does not model them); "mine" copies the engine's over the
original's (the engine keeps its own values there). Each range is a blind
spot: a wrong engine value in it goes unseen.

"Differs" counts, from the 67 chain rooms 1, 6, ..., 331 in the default mode
(batch 12), the rooms and frames where the two sides differ before the copy.
$0641, $0D1E and $1790 never differed there and are no longer normalized.

| Range | Copy | What it holds | Differs (rooms / frames) | What would retire it | Owner |
| --- | --- | --- | --- | --- | --- |
| $0000-$0050 | theirs | direct-page scratch | 67 / 19718 | the Session owns each scratch word its last writer leaves, as `PointerScratch` ($00-$03) and `SpawnScratch` ($12-$17) already do | Session, with each routine's module |
| $1F5B-$1FFF | theirs | the stack: return addresses and pushed registers | 67 / 14221 | none short of emulating the 65816 stack; permanent | verifier |
| $00AD-$00B0 | theirs | pointer to return-address parameters (the door VRAM update reads through it); region animated palette FX ($89:AC6A) | 48 / 99 | storing both where the original does | room entry (doors), room FX |
| $05BC-$05C4 | theirs | door transition VRAM update flag, destination, source and size | 47 / 98 | the deferred door VRAM update (`$80:D000` flip group) | room entry (doors) |
| $05ED-$05F4 | theirs | the multiplication product ($05F1-$05F4); the original never touches $05ED-$05F0 | 15 / 280 | `MultiplyScratch` covering the product, as it covers $05E9/$05EB | player collision |
| $0B24-$0B27 | theirs | Samus tiles temporaries (top, bottom) | 67 / 9144 | storing the tiles definitions' halves where `$92:8000` does | Samus drawing |
| $1966-$196B | theirs | current FX entry pointer and offset, palette FX bitset | 42 / 44 | loading the FX entry on the frame the original does (one frame per room load) | room FX |
| $060B-$0610 | mine | drop count, enemy hitbox entry counters, projectile counter copy | 67 / 15449 | storing the counters the loops leave | actors, enemy drops |
| $0611-$0616 | mine | coroutine states (snesrev's bookkeeping in RAM the original never writes) | 66 / 280 | moving the coroutine state out of game RAM; permanent while it lives there | runtime |
| $077E-$0782 | mine | `MY_COUNTER` (snesrev's counters in RAM the original never writes) | 66 / 384 | as above | runtime |
| $078F-$0790 | mine | door BTS | 24 / 41 | storing the door BTS where the original does | room entry (doors) |
| $07B7-$07B8 | mine | room state condition pointer | 5 / 5 | as above, for the state selection | room entry |
| $0933-$093C | mine | scrolling and tilemap update temporaries | 67 / 16689 | storing the temporaries the scroll update leaves | scrolling |
| $0D82-$0D8F | mine | saved button configuration (and grapple temporaries at $0D82) | 5 / 382 | storing them where the demo and grapple code does | demo, grapple |
| $0D9C-$0D9D | mine | total grapple swing angular velocity | 4 / 174 | as above | grapple |
| $0DD2-$0DD7 | mine | collision temporaries (slope index, distance) | 67 / 16758 | storing the temporaries the block collision leaves | player collision |
| $0E02-$0E03 | mine | Samus bottom boundary position | 66 / 16128 | as above | player collision |
| $0E20-$0E45 | mine | enemy temporaries | 58 / 12992 | widening the actors' compatibility scratch ($0E20-$0E2A) to the whole range | actors |
| $0E4A-$0E4B | mine | new enemy index | 1 / 1 | storing it where enemy spawning does | actors |
| $0E54-$0E55 | mine | current enemy index | 4 / 211 | as above, for the enemy loop | actors |
| $0E56-$0E59 | mine | enemy index backup, enemy data pointer | 51 / 7763 | as above | actors |
| $1784-$178B | mine | enemy AI pointer and backup | 60 / 7731 | storing the species' AI pointer the dispatch would jump through | actors |
| $17A8-$17AB | mine | active and interactive enemy list indices | 60 / 15948 | storing the indices the list loops leave | actors |
| $1834-$183B | mine | Samus-to-solid-enemy collision distances | 8 / 46 | storing them in the solid enemy collision | player collision |
| $184A-$185B | mine | colliding-solid-enemy positions and deltas | 13 / 251 | as above | player collision |
| $186E-$1885 | mine | enemy spritemap and border collision temporaries | 36 / 4260 | storing them in the enemy collision | actors |
| $189A-$18A5 | mine | Samus target positions and radius mirrors | 58 / 13225 | storing them in the Samus/enemy collision | player collision |
| $18A6-$18A7 | mine | collision detection (projectile) index | 65 / 11946 | storing the index the projectile loops leave | projectiles |
| $EF74-$EF77 | mine | next enemy GFX tiles index and VRAM pointer | 42 / 43 | loading the enemy graphics on the frame the original does (one frame per room load) | actors (room load) |
| $F37A-$F37F | mine | debug copies of an enemy's instruction list, index and ID (never read) | 55 / 9907 | storing them where the enemy processing does | actors |

To retire a row: model the range so it stops differing on the chain and
corpus, then delete it from `normalize_snapshot_ram` and from this table in
the same change.

Free run (docs/SCENARIO_HARNESS.md) masks the same ranges in its WRAM digest, and
one more, `ORACLE_RAM` in `sm/src/verification/observation/machine.rs`:

| Range | What it holds | Why | What would retire it | Owner |
| --- | --- | --- | --- | --- |
| $00C2-$00C3 | snesrev's bug-fix counter, the oracle's patch selector; the original never uses it | only an oracle frame writes it; lockstep's sides share RAM, so only free run sees it (from the first Tick of a seeded row) | the oracle keeping its patch selector outside game RAM | verifier |
