# Measured internal live coverage reconciliation

Date: 2026-10-13. Read-only audit; no gameplay execution, new authentication run,
shared publication or edits to the immutable ticket/audit mapping JSON.

Publish eleven additional native symbols as `live-exercised` in the explicitly
bounded composed domain below: eight measured Ridley opcode implementations
and three successful native projectile initializers. The evidence is retained
frozen execution, not a new direct-wrapper count or whole Ridley admission.

## Missing native symbols

The current composed receipt publishes fifteen owning roots. The independent
legacy audit authenticates sixteen older runs with ninety-five native roots.
None of the following eleven symbols appears in either live routine set at the
reviewed snapshot. They already have implemented/verified provenance; their
measured internal live execution was omitted by wrapper-only enumeration.

| Exact native symbol | Native dispatch address / definition | Rust calls mixed-0 | Rust calls mixed-1 | Counter and admitted domain |
| --- | --- | ---: | ---: | --- |
| Ridley_Instr_1 | A6E517 | 2 | 2 | ridley_opcode_hits.10937623; attack-script orientation branch |
| Ridley_Instr_5 | A6E4BE | 4 | 4 | ridley_opcode_hits.10937534; attack-script sound/state write |
| Ridley_Instr_6 | A6E4CA | 2 | 2 | ridley_opcode_hits.10937546; attack-script state clear |
| Ridley_Instr_10 | A6E4D2 | 4 | 4 | ridley_opcode_hits.10937554; low-health branch |
| Ridley_Instr_11 | A6E84D | 18 | 18 | ridley_opcode_hits.10938445; native aiming/velocity writes |
| Ridley_Instr_12 | A6E904 | 4 | 4 | ridley_opcode_hits.10938628; zero-F fireball allocation opcode |
| Ridley_Instr_13 | A6E909 | 12 | 12 | ridley_opcode_hits.10938633; F14 fireball allocation opcode |
| EnemyInstr_Sleep | A6-bank dispatch alias A6812F, lexical shared body sm_a0.c:157 | 2 | 2 | ridley_opcode_hits.10912047; shared native stop/sleep body called by attack interpreter |
| EprojInit_9642_RidleysFireball | bank86 definition9642 → initializer8693CA | 16 | 16 | projectile_allocation_hits.38466; successful highest-free allocation inside accepted fireball opcode |
| EprojInit_CeresElevatorPad | bank86 definitionA387 → initializer86A2EE | 4 | 4 | projectile_allocation_hits.41863; successful native Ceres pad allocation |
| EprojInit_CeresElevatorPlatform | bank86 definitionA395 → initializer86A31B | 4 | 4 | projectile_allocation_hits.41877; successful native Ceres platform allocation |

Every listed mixed counter has C=0; every successful allocation counter has
refused=0. Both C subjects have matching native counts. The eight opcode counts
sum to48per mixed subject. These occur in2,393accepted attack interpreter calls,
of which36are timer-due;209,673other ProcessEnemyInstructions calls remain C-owned
or refused by this bounded replacement. The script domain is the authentic
A6:E73A entry and238-byte E73A..E828 attack list, SHA256
`04a1e79b51fa101b337ab6183269760110c936b914f81734f63564336b609580`.

## Retained execution and authentication

All paths in this section are relative to `/home/jim/sm-decomp` unless absolute.
The owning run is
`out/recovery-session/composed-live-release/1791861180321696848-129445/`.

| Artifact | SHA256 / role |
| --- | --- |
| verdict.toml | 5e224bda08b3f597b07dd3a03c59bf8cf136b288993459c62e453ef3fac400c2; matching state/effects, four subjects, exact counters/source seals |
| live | bb789e65341e88e49c745fa97e8993e18fcc295f40a1a6482d772911cbf08576; retained immutable live binary |
| families.rs | aaa6135ffc67f8a92d50e7895a45683b34fb8873816dce0577740bda22c20d92; actual Rust live bridge invoking native-named opcode dispatch |
| mixed-0/replay.stdout | 58ebe504e3eac6f3f70cca0d3cee779910b0300e6439e1df8060127da4005840; exact opcode/allocation lines5232–5249 |
| mixed-1/replay.stdout | 58ebe504e3eac6f3f70cca0d3cee779910b0300e6439e1df8060127da4005840; independent producer, same recorded counters |
| .scratch/decomp-port/recovery-composed-post-cleanup-authentication.toml | 9251ee29114c3ac5ea7da19279050ac7e83227cb790e624c5b6a0900372f3845; authenticated four subjects,1,881artifacts per subject, binary=true |

The run's `c-0/evidence.toml`, `c-1/evidence.toml`,
`mixed-0/evidence.toml` and `mixed-1/evidence.toml`, shared `out/evidence`
store and retained `sm-evidence` reader remain required evidence dependencies.
Capture names are logical packed artifacts in each subject, not loose files:

- `fixtures/ridley-frame-*-before.wram`, `*-after.wram`, `*-before.host`,
  `*-after.host`: complete accepted native attack frame states/eight host words.
- `fixtures/projectile-allocation-9642-001-before.wram` and `-after.wram`,
  their host sidecars and002pair: fireball allocation material evidence.
- The corresponding `projectile-allocation-a387-001/002-*` and
  `projectile-allocation-a395-001/002-*` pairs: Ceres successful allocations.

Post-cleanup authentication reparses all four retained stdout counters and
compares them with the verdict. `observations.rs:189` checks native allocation
counters and complete bounded counters; `live.rs:936` requires both independent
subjects' opcode counters to match. `live.rs:975` compares each native C count
with exclusive Rust ownership and enforces exact full-route opcode counts.
`live.rs:907` similarly checks successful allocation count parity and enforces
16/4/4for the three initializers. Native admission also requires the material
full-WRAM/host capture pairs. No new authenticator was executed by this audit.

## Exact native and Rust mapping

Native pin: `578f90b3cc49557bb70060ad033bb90b8cf8ac50`; inventory SHA256
`7842e9035e7360eef10a93a96e6abc1cca601888b8423ea92fb554bbb15abb25`.
Native opcode names/addresses are defined by `deps/snesrev-sm/src/funcs.h` and
actual definitions in `sm_a6.c`. `EnemyInstr_Sleep_A6` is a dispatch alias of
the shared actual `EnemyInstr_Sleep` definition, not a separate native function.

The run seals exact source at `/tmp/sm-recovery-bounded-source/`:

| Source | SHA256 |
| --- | --- |
| crates/port/src/ridley_enemy_instructions.rs | e393dc2d8e53d709e15262a9d52bf27c600d955de5a5447c62381db0e1c6ab10 |
| crates/port/src/enemy_projectiles.rs | 4c99175b8eb0be86554e46f7424e6ee7aa43510afebe2d555ea2dcb890e7f0de |
| crates/port/src/observed_projectile_ownership.rs | b8e77711094a2de4e0222c587d81a104f48af25fde7b60335ba409063fc54b56 |
| crates/pipeline/src/live_ridley_host.c.in | b86cc3d6fa1079c563d49550b0d4484e4a80b02e1e737a5b881abf69a460ddae |
| crates/pipeline/src/live_projectile_host.c.in | 4ae30cd01c4e76796b8afe424c94067437447cedc9b75169eb08cdedd98627aa |

These hashes were compared read-only with the retained run source seals. The
three maintained gameplay files currently match these exact frozen files;
that observation alone does not authenticate every current shared dependency.

The retained `families.rs:56` selects exact native opcode names, calls
`ridley_enemy_instructions::dispatch_host`, then calls PortRidleyOpcode only
after successful return. Thus the48opcode events count actual executed Rust
callbacks, rather than a speculative preflight plan or script decode. Native C
CallEnemyInstr observations use the same address list within accepted frames.

For initializers, `live_projectile_host.c.in:62` counts allocation-ID only when
the pool has a free slot, increments Rust ownership only after accepted Rust
allocator return, and retains before/after material state. The Ridley bridge
has equivalent before/after bookkeeping around its sixteen successful9642
allocations. Saturated calls are excluded from allocation-ID counts.

Each admitted definition is fixed against the pinned ROM by
`observed_projectile_ownership.rs:9`. EprojDef's first word is init_code_ptr;
definition9642's first word93CA, A387'sA2EE and A395'sA31B select exactly the
three native definitions above. `enemy_projectiles.rs:64` reads that word and
the successful path invokes `initialize` at:204. Rust branches at:700 implement
the corresponding native initializers. It is therefore sound to attribute the
successful16/4/4internal executions to those exact native symbols. Separate
initializer wrapper counters were not installed.

## Publication recommendation and limits

Add one explicit live observation for the eight measured opcode symbols and
one for the three measured successful initializer symbols, each with the
composed verdict, post-cleanup authentication, generated families.rs, relevant
sealed source and this mapping note as evidence. Mark the tested domain as
frozen composed internal execution under authenticated native owning roots.
Use `current-checkout` only if the integration owner also retains the required
full current source/dependency identity; otherwise use `historical-frozen`.
Evidence context must describe source identity rather than the date of the run.

The two observations should say that opcode counters are internal to accepted
ProcessEnemyInstructions and initializer counts derive from successful native
allocator-ID counters. They must not claim independent direct wrappers,
unbounded script/init domains, every Ridley lifecycle definition, original-ROM
movie parity or credits progression. Native frame traversal and zero CPU
fallback are already established by the owning receipt.

The opcode/initializer sets are disjoint from the existing15current roots and
95legacy roots, so these additions contribute eleven distinct live symbols.
No further internal coverage follows solely from active projectile-ID traversal,
a source call graph, the palette C object count or unmeasured scheduler children.
Those domains require exact additional composed evidence before publication.

This audit created only this note. It left all retained runs, packed store
dependencies and immutable audit/mapping JSON untouched. No build output,
worktree or verification-run directory was created, so scoped cleanup had no
disposable artifacts to remove.
