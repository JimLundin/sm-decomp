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

## Oracle digest lock: scenarios kept on lockstep

The engine-only gate (`lock-gate.yml`) checks every seeded and boot corpus row
and every route room against the faithful oracle's digest lock
(`scenarios/lock.toml`). These scenarios are not locked; they stay on the patched
lockstep corpus and route jobs until the cause is fixed. Each is listed in
`UNLOCKED` in `.github/scripts/lock_gate.py`, which fails if a listed scenario
locks or an unlisted one is missing; the list only shrinks. Ticks are the first
divergence of an engine free run against the faithful oracle's trace (the
localization `check-lock` prints), recorded 2026-11-01 on `land/batch13`.

| Cause | Scenarios and first free-run divergence | Lockstep over the span |
| --- | --- | --- |
| The faithful-oracle divergences above | `samus-scripted-intro` @8256 vram, `pause-menu-tour` @1378, `boss-kraid-pause-400` @588, `-950` @1138, `-1030` @1218 (wram, vram, ppu), `enemy-fire-arc` @119 (wram, actors), route `chain-0011-DF45` @2054, `chain-0211-C98E` @368, `chain-any-0011-DF45` @2055 | fails at the listed frame |
| Not yet identified: WRAM drift that lockstep masks, in all seeded item and pickup rows but `item-wave` | `item-bombs` @437, `-grapple` @408, `-gravity` @402, `-hi-jump` @458, `-ice` @411, `-morph-ball` @972, `-plasma` @526, `-screw-attack` @514, `-space-jump` @401, `-spazer` @440, `-speed-booster` @421, `-spring-ball` @780, `-varia` @403, `-x-ray` @541; `pickup-energy` @424, `-missile` @405, `-power-bomb` @500, `-reserve` @480, `-super` @578; `samus-grapple-connect-swing` @408, `space-jump-bubble-word-counter` @401, `xray-activate-release` @541 | passes: masked scratch or state lockstep re-adopts |
| Not yet identified: drift lockstep masks | `boss-kraid` and `boss-kraid-pause-1960` @1580 wram; route `chain-0330-92FD` @210 wram, `chain-any-0079-ACF0` @345 wram, `chain-any-0196-92FD` @431 wram, `chain-0086-AA41` @95 ppu (one Tick), `chain-any-0074-AFA3` @316 ppu; the re-seeded `boss-crocomire` @1801, `ice-return-business` @411 and `music-fir-selector10` @646 wram | passes |
| The engine alone panics (`gameplay_frame.rs`, slice index 65535 of 45) | `file-copy-clear` | passes: adopting the oracle's state avoids it |

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
