# Observed Ridley enemy instruction frame ownership

Status: resolved
Blocked by: 137
Triage: ready-for-agent

Close the naturally exercised bank-A6 Ridley attack-script boundary through
native `ProcessEnemyInstructions` ($A0C26A), including aiming, fireball spawning,
animation timers and the actual control-flow closure. This extends readable
Rust ownership from ticket137's projectile lifecycle into the enemy script that
produces it. Select whole supported enemy instruction frames before mutation;
unsupported enemy banks/scripts remain explicitly bounded C ownership.

## Execution evidence and current gap

Authenticated full129,712-input actual-C inventory:
`out/port/projectile-inventory/20261011T072333.457801Z/verdict.json`.
On2026-10-11 read-only SHA256 verification matched its sealed
`projectiles-decoded.tsv`, transformed`sm_86.c` and
`original-source/sm_a6.c`. It records sixteen initializer93CA calls, first
frame36,895/last70,802, and617 active9642 projectile calls through70,841.

Pinned`sm_a6.c:4498` Ridley_E90C is the sole actual C source caller of
SpawnEprojWithRoomGfx(9642,...). Ridley_Instr_12($A6E904) and
Ridley_Instr_13($A6E909) call it with F-source0/000E. These callbacks therefore
produce all sixteen observed initial allocations. Their individual split counts
and whole enemy interpreter counts have not been instrumented; do not invent
per-callback execution counts from the projectile inventory.

The actual ROM has four fireball attack-list variants at$A6E734/$A6E770/
$A6E7AC/$A6E7E8. Each has one E904 and three E909 spawn instructions, each
immediately preceded by E84D aiming. Pinned scripts also contain E4D2 health/
area branching, E4BE/E4CA sound-state transitions, native frame records and812F
sleep. Read-only ROM inspection establishes data contents, not hit evidence;
the first implementation step must inventory actual interpreter/callback use.

Maintained`crates/port/src/enemy_instructions.rs` provides generic handlers and
`process_generic_instructions`, but its `generic_symbol` rejects every Ridley
opcode with "unsupported bank-specific enemy instruction". Existing ticket09
proves generic synthetic interpreter frames only and explicitly excludes whole
live interpreter ownership. No maintained Ridley aiming/spawning script closure
exists; ticket137 deliberately leaves its caller in C.

`lib.rs` publicly exports enemy_instructions, enemy, audio, enemy_projectiles
and observed_projectile_ownership. Generic dispatch/frame entries are public
`enemy_instructions::dispatch_rom` and `process_generic_instructions`; the latter
is called for dispatch symbol ProcessEnemyInstructions and has no host-aware
variant. Tickets03(audio),06(enemy math) and09(generic instructions) are resolved.
Ticket137 is the only unfinished dependency for the maintained native allocator
and eight-word live host bridge.

## Native state and boundaries

- `sm_a0.c:3395` ProcessEnemyInstructions decrements the selected enemy's word
  instruction timer, executes actual banked opcodes until a frame/stop, updates
  spritemap/current instruction, and sets/clears extra-properties8000. Frozen
  ai-handler bit4 returns without a decrement. Preserve raw pointer results,
  odd script positions, wrapping timers and source/destination aliases.
- `sm_a0.c:1715` EnemyMain sets cur_enemy_index and invokes the native interpreter
  after enemy AI when properties2000 is set. Keep enemy AI ownership explicit;
  replacing this coherent instruction boundary does not claim EnemyMain or
  Ridley AI is translated.
- `sm_a6.c:4457` Ridley_Instr_11($A6E84D) always reads enemy slot0, regardless of
  supplied/current enemy; aim from X±25/Y−43 to Samus, retain eight-bit angle
  negation and exact directional clamps, then write velocities to7832/7834
  with Math_MultBySin/Cos amplitude0500. Reuse owned angle/table semantics where
  faithful; inspect native86C26C/86C272/86C27A before assuming another trig
  helper has the same width/rounding. Current pub(crate)
  enemy_projectiles::beam_component has the signed-table magnitude product
  needed here; enemy's angle_from_xy is private but the public dispatch exposes
  CalculateAngleFromXY. Verify these exact formulas against native routines
  before sharing them with the new dispatcher.
- Ridley_E90C writes1995 then performs complete host-aware native9642 allocation;
  retain the eight-word ProjectileHost and ticket137 highest-free/saturation
  semantics. Enemy0 orientation var10 aliases7820.
- Health/area branch E4D2 uses signed Samus-health−30, area2 override and writes
  enemy0 var00 at7800. E4BE writes781E=89 and queues SFX2 max6 sound59; E4CA
  clears it. Include additional genuinely observed script closure if the
  inventory reaches it; E727 orientation, E71C/E72F and D3F9 tilemap changes
  must never be replaced with no-op callbacks.
- Same-bank ProcessEnemyInstructions callers in sm_a0 and actual CallEnemyInstr
  dispatch need definition-renaming/adapters so acceptance reaches Rust before
  mutation. Publish all eight host words after success; a Rust error terminates
  rather than retrying C. Prove supported opcode/ROM/callback closure before
  accepting the entire frame, not after its timer or sound state changes.

## Required implementation and verification

1. After ticket137 verification freezes end, add a read-only actual-C diagnostic
   for native ProcessEnemyInstructions entries and banked CallEnemyInstr opcodes.
   Capture first/last frame, counts, enemy header/bank/current instruction,
   state, and full before/after WRAM plus all eight host words for Ridley attack
   frames. Run through both observed combat intervals. Preserve original ROM
   lists and source seals. This inventory may refine the closure but cannot
   shrink the target to isolated spawn leaves or empty timer traversals.
2. Implement the complete exercised Ridley attack-list closure through a
   maintained host-aware enemy instruction dispatcher/frame entry. Translate
   E84D aiming, E904/E909→E90C spawning, E4D2/E4BE/E4CA control/effects and native
   generic/frame/sleep semantics; include every other opcode required by the
   observed accepted scripts. Unknown scripts choose C before mutation. Accepted
   Rust errors retain diagnostic evidence and never trigger a C retry.
3. Compare actual compiled C against full WRAM, raw banked returns and all eight
   host words for direct callbacks, actual-ROM scheduled frames, and chained
   aim→spawn→frame→repeat→sleep execution. Cover both orientations, parameter
   0/000E, all highest-free choices/full pool, enemy0/current-enemy differences,
   coordinate wrap, all angle clamp boundaries, trig rounding/signs, health
   29/30/31/8000 boundaries, area2/default, timer0/1/FFFF, frozen dispatch and
   sound queue saturation. Controlled chained fixtures must be labeled
   separately from natural replay.
4. Add independent semantic/ABI controls for enemy0 aliasing, exact aim clamps,
   source1995-before-allocation order, full-pool behavior, pointer/slot/ROM bounds,
   support refusal preserving WRAM/host and accepted-error no-C-retry. Verify
   current Rust tests, appropriate formatting, strict all-target Clippy and
   relevant interpreter/host harness controls against the final source freeze.
5. Reseal final actual-C dependencies and run four independent C/mixed processes
   through full129,712 inputs. Require periodic/terminal state, ordered effects
   and WRAM/host capture parity; zero CPU fallback; nonzero exclusive Rust native
   interpreter counts for actual Ridley aiming/spawning frames, matching observed
   opcode counts, and all sixteen resulting9642 allocations. Report supported,
   refused, C and Rust whole-frame counts and bounded remaining C scope. Retain
   ticket137 native allocator/scheduler/contacts/rendering ownership.
6. Complete scoped cleanup/deduplication and post-cleanup offline/live evidence
   authentication. Preserve snapshots, pinned native scripts, eight-word host
   captures and shared packed-evidence dependencies.

## Relation to the unresolved input frontier

Ticket78 remains claimed and important for original-ROM parity: its current
1,097 causal completed updates through original frame1600 match semantic fields
and poll order, while full WRAM, hardware-frame/APU handshake timing remain
unproved. That is a separate adapter acceptance frontier. This ticket advances
the full readable gameplay ownership objective at a naturally observed native
caller, without treating C/mixed parity as original-ROM parity. Existing
claimed paired-projectile, sprite and animated allocators have explicit zero
movie hits and do not provide a stronger naturally exercised replacement here.

## Comments

Proposed read-only while ticket137 WithGfx final source/replay is frozen. No
gameplay/harness source was changed and no new verification run launched.
Counts above come from retained authenticated execution or are explicitly
labeled source/ROM observations; new per-enemy/opcode counts remain required.

## Completion evidence

2026-10-13 recovery: implemented at `e947e968086a15c170f58472764bf9244b403a2a`,
shared native live wiring at `3ce28ba58aba319b60bbc949153baafd4433132e`.
The historical four-list interpretation above is superseded by the authentic
entryA6:E73A,238-byte E73A..E828 script, SHA256
`04a1e79b51fa101b337ab6183269760110c936b914f81734f63564336b609580`.
Current controlled1804pairs plus90natural captures pass and authenticate after
cleanup. Seven port controls and compiled host/ABI refusal/order controls pass.

The completed129712-input four-subject composed verdict matches complete state,
ordered effects and WRAM/eight-word host captures with zero fallback. Each mixed
subject executes2393exclusive Rust attack frames,36timer-due frames and48opcodes:
E5172/E4D24/E4BE4/E4CA2/E84D18/E9044/E90912/812F2. All16resulting9642allocations
and native137 scheduler/render/contact ownership remain accepted.209673other
interpreter calls explicitly remain C-owned. These counts match both C subjects.

`../recovery-live-completion-20261013.md` gives exact verdict, retained source,
test and cleanup pointers. `../recovery-composed-post-cleanup-authentication.toml`
authenticates four subjects/1881artifacts each after ordinary cleanup48176296bytes
and dedup0. Eight stale worktree registrations were retired; exact source-only
snapshots, shared store, pointers/readers remain. Disk receipt:
`../recovery-live-final-disk.txt`. Original-ROM movie parity, other Ridley scripts
and whole lifecycle/AI remain separate; ticket141 owns the broader candidate.
