# Ridley attack evidence and whole subsystem expansion

Lane: `/root/ridley_instructions`.
Worktree: `/tmp/sm-ridley-instructions`, branch `decomp-ridley-instructions`.
Shared registration, CLI, live ABI and final acceptance: root integration owner.

## Current attack closure

Actual entry is `$A6:E73A`; the ticket's four-list interpretation at E734,
E770, E7AC and E7E8 is incorrect. The pinned script spans E73A..E828, 238 bytes,
SHA256 `04a1e79b51fa101b337ab6183269760110c936b914f81734f63564336b609580`.
Both directional branches execute two volleys with four spawns each. Enemy AI
remains separately owned.

Maintained module SHA256 (identical in worktree and main):
`e393dc2d8e53d709e15262a9d52bf27c600d955de5a5447c62381db0e1c6ab10`.
Gameplay commit integrated by root as e947e96; owned fixture/inventory commits
bb421c5, 17dd3a4, 060e0ac, 8450a58. Natural capture extension: 685b706.

## Measured evidence

Full native diagnostic:
`out/port/ridley-instructions/inventory/1791838371843885184-2/verdict.toml`.
129,712 inputs completed, zero CPU fallback, 419 decompressions. Wall 138.173s,
replay 127.290s, cached C link/build 0.183s, worker wait below 0.001s.
All 2,353 sealed artifacts were authenticated by the Rust capture generator.
This diagnostic is actual-C observation, not live Rust acceptance.

| Accepted attack frames | First combat | Later combat | Total |
| --- | ---: | ---: | ---: |
| Complete native entries | 2,075 | 318 | 2,393 |
| First / last frame | 36,877 / 38,951 | 70,656 / 70,973 | |
| Timer-due entries | 18 | 18 | 36 |
| Frozen entries | 0 | 0 | 0 |
| Retained complete-frame captures | 35 | 35 | 70 |
| Retained relevant callback captures | 10 | 10 | 20 |

Every accepted entry has slot 0, header E13F, bank A6, orientation 0 and area 6.
The long frame tail is native sleep/timer wrap, not thousands of attack callbacks.

| Opcode in accepted frame | Each combat interval | Total |
| --- | ---: | ---: |
| E517 orientation branch | 1 | 2 |
| E4D2 health/area branch | 2 | 4 |
| E4BE sound-state / SFX | 2 | 4 |
| E4CA sound-state clear | 1 | 2 |
| E84D aiming | 9 | 18 |
| E904 spawn F=0000 | 2 | 4 |
| E909 spawn F=000E | 6 | 12 |
| 812F sleep | 1 | 2 |
| Total executed opcodes | 24 | 48 |

Offline controlled family passed 1,804 cases / 8 routines:
`out/port/ridley-instructions/offline/1791838472346671-2-0/verdict.toml`.
898 direct callbacks, 906 complete interpreter frames, four unpatched
221-frame chains. Original-ROM parity and live ownership remain unclaimed.

New exact natural batch:
`tools/port/observed-ridley-instructions-captured.toml` (90 cases).
Rust capture check matched every full 131,072-byte WRAM, every captured host word,
canonical callback returns, and captured complete-frame opcode
address/argument/return/raw pointer kind/order. Verification and artifact
authentication took 4.957s excluding compile:
`parallel-138-scope-natural-capture-verification.json`.
Two fixture-generator tests and isolated strict Clippy passed.

Root's live gate must require 2,393 exclusive Rust whole-frame entries,
matching the 48-opcode sequence/counts and all 16 allocations, four independent
C/mixed processes, complete periodic/terminal state and ordered-effect parity.
It must report refused/C/Rust counts; all other enemy banks and Ridley scripts
remain bounded C ownership until subsystem expansion is accepted.

## Whole Ridley subsystem scope

Recovered source is sufficient. Source closure is 220 native definitions,
3,129 function-body lines, selected from these subsystem boundary roots:

- CeresRidley_Init, CeresRidley_Main, CeresRidley_Hurt.
- Ridley_Main, Ridley_Hurt, Ridley_Func_2, Ridley_Shot, Ridley_Powerbomb.
- sub_A6DF59, sub_A6DF60 and Ridley_A2F2.
- HandleCeresRidleyGetawayCutscene.
- RidleysExplosion_Init and RidleysExplosion_Main.
- All 16 Ridley bank-A6 instruction callbacks.

The closure includes every CallRidleyFunc / var40 state, the baby-metroid script,
fixed-point movement, seven-segment tail simulation, palettes, OAM drawing,
VRAM/Mode7 transfer queues, Ceres escape text/cutscene, contact/grab/release,
death and explosion scheduling. Assembly is needed only if differential
execution exposes a discrepancy in recovered C.

Owned writable files for the expansion:

- `crates/port/src/ridley_subsystem.rs`: public subsystem boundary / typed host
  dependencies, shared state and ROM access, orchestration and lifecycle states.
- `crates/port/src/ridley_movement.rs`: movement and tail mechanics.
- `crates/port/src/ridley_cutscene.rs`: Ceres appearance/escape and death states.
- `crates/port/src/ridley_rendering.rs`: palette, transfer queues, OAM and baby
  metroid animation.
- `crates/port/tests/ridley_subsystem.rs`: subsystem semantic/ABI controls.
- `crates/pipeline/src/ridley_subsystem_fixtures.rs` and
  `tools/port/ridley-subsystem.toml`: one coherent fixture batch.
- Existing owned `ridley_enemy_instructions.rs` may delegate to the broad module
  after root's ticket138 source freeze ends. Shared lib/main/family/live files
  remain root-owned.

Dependencies already available in maintained Rust: audio queues, enemy angle,
beam components/random, native 9642 allocator, system boss-bit state. Palette
copying, VRAM and Mode7 queue writes, OAM emission and Ridley's block/movement
probes can be translated here directly because their state is canonical WRAM.

The current eight-word ProjectileHost alone cannot represent these native
external services: SpawnEnemy (Ceres doors/steam and 12 death pieces), non-9642
dust E509 allocation, enemy item drops, generic touch/shot/powerbomb AI, Samus
grab/release and Ceres push-out, and minimap/map marking. Expose them through
typed synchronous Host calls with explicit ownership; never replace them with
no-ops. Implement local simple shared service semantics or reuse validated Rust
where possible. Root's live seam needs an ordered callback bridge for services
that remain C-owned. This allows whole Ridley orchestration/AI ownership while
keeping allocator/contact/map ownership honest.

Tail ComputeSinMult/ComputeCosMult also writes hardware M7A twice and M7B once,
then reads MPYM. Preserve these ordered register effects through the host,
including the actual PPU multiply state, rather than using only an arithmetic
helper with matching return values.

Existing inventory shows the natural E13F / area6 script closure, including
E501, E51F and E969 outside the current attack scope. It does not inventory AI
state/init/contact/death entries, and it proves no E17F / area2 execution.
Extend the existing producer once at the whole CallEnemyAi/frame seam and run
one completed native subsystem inventory. Batch captured + controlled
subsystem comparisons and use one four-process acceptance replay; isolate a
routine only when a mismatch requires diagnosis.

## Identities and retention

Source revision: `578f90b3cc49557bb70060ad033bb90b8cf8ac50`.
ROM: `12b77c4bc9c1832cee8881244659065ee1d84c70c3d29e6eaf92e6798cc2ca72`.
Inputs: `cc643d751554ad6d30b5d7e3a41a4259525517ca7614cf73abec9fac30176c0e`.
Native sine table starts A0B3C3 / ROM offset 1033C3.

Preserve successful inventory and its captured .host/.wram inputs, transformed
C, original-source snapshots and rom-view.bin. Preserve failed 60s diagnostic
`out/port/ridley-instructions/inventory/1791838109064773582-2` as sealed failure
evidence. The controlled manifest depends on 512 independent expected WRAM
files in `out/port/ridley-instructions/expected`.

Final run cleanup/deduplication, offline/live authentication and integrated
worktree removal belong to root. Temporary isolated Cargo verification projects
are `/tmp/sm-ridley-generator-check`, `/tmp/sm-ridley-source-clippy` and
`/tmp/sm-ridley-checks`; retain only source checker snapshots if referenced
before removing disposable targets. At the latest check the filesystem had
194 GiB available. Ticket138 stays claimed until live acceptance and required
cleanup finish.
