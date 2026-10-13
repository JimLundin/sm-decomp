# Whole Ridley live integration design

Status: read-only design; no live admission granted
Owner: `/root/integration_audit`
Date: 2026-10-13

The recovered whole subsystem can replace native lifecycle boundaries using its
public immutable selectors and typed host. The existing controlled ABI is not
the live ABI: `port_ridley_subsystem_call` calls `dispatch_fixture`, which
deliberately permits states outside public admission. There is no
`ridley_subsystem::dispatch_rom`. Use `supports_call` followed by `dispatch_host`,
and use `process_instructions` when the script report is needed.

Authoritative evidence remains the recovered completion report
`ridley-recovery-completion.md`, the controlled 6,457-case family and whole
129,712-input native inventory. Controlled parity, native inventory and captured
supplement establish separate domains. Original-ROM parity remains untested.

## Boundary and source transformation map

All admitted lifecycle roots have native signature `void SYMBOL(void)`. The
following roots already have public selector/dispatch support and controlled
fixtures. Admission is conditional on the exact native header/state/ROM, not
automatic permission for every call bearing the symbol.

| Native root | Address | Caller / admission role |
| --- | --- | --- |
| CeresRidley_Init | A6A0F5 | CallEnemyAi init; headers E13F/E17F |
| CeresRidley_Main | A6A288 | CallEnemyAi main, header E13F |
| CeresRidley_Hurt | A6A2D3 | CallEnemyAi hurt, header E13F |
| Ridley_Main | A6B227 | CallEnemyAi main, header E17F |
| Ridley_Func_2 | A6B28A | CallEnemyAi frozen handler, header E17F |
| Ridley_Hurt | A6B297 | CallEnemyAi hurt, header E17F |
| Ridley_Shot | A6DF8A | CallEnemyAi/CallHitboxShot; native root takes no projectile argument |
| Ridley_Powerbomb | A6DFB2 | CallEnemyAi power-bomb |
| sub_A6DF59 | A6DF59 | CallHitboxTouch / touch; terminal NormalTouch service |
| sub_A6DF60 | A6DF60 | Native direct damage/knockback |
| Ridley_A2F2 | A6A2F2 | CallEnemyGfxDrawHook and same-bank Ceres draw calls |
| HandleCeresRidleyGetawayCutscene | A6AAAF | Native getaway boundary |
| RidleysExplosion_Init | A6C696 | CallEnemyAi init, explosion header E1BF |
| RidleysExplosion_Main | A6C8D4 | CallEnemyAi main, explosion header E1BF |
| Ridley_Func_70 | A6CAF5 | Tail boundary, called from Ceres/real-boss main/hurt |
| ProcessEnemyInstructions | A0C26A | Complete scheduled interpreter; E13F/E17F bankA6 script selector |

Rename exactly these definition tokens to `PortC_SYMBOL`, preserving all public
call sites and prototypes. Append public wrappers with the original native
signatures. Root's existing `live::rename` handles unique definitions and
can add declarations where needed. Transform bankA6 once on top of any existing
transformed unit, and bankA0 once on top of its existing transformations.

Prefer named native root wrappers over replacing the entire CallEnemyAi switch.
This preserves every unrelated AI case, reaches same-bank calls, and gives each
root a clear signature. CallHitboxShot's `j` argument is consumed by its caller;
native Ridley_Shot takes none. Do not route `j` into a fictitious Ridley ABI.

Do not also replace CallRidleyFunc, CallRidleyFunc_var40, CallBabyMetroidInstr or
the 220 internal routines as separately admitted live roots. Their behavior is
owned by the accepted complete lifecycle call. `CallRidleyFunc(uint32)` exists
only in the controlled fixture dispatcher; public supports_call does not admit
it. Internal state/baby coverage can be reported through source closure and
material endpoint evidence, or measured with explicit Rust observations later.

Nested roots need explicit accounting. Ceres_Main/Hurt and Ridley_Main/Hurt call
wrapped tail/draw roots; Rust composition performs these internally. Counting
both C child calls as independent roots makes C/Rust totals incomparable. Use a
Ridley root depth guard: admit/count/capture only external outer calls, and
execute nested C roots through their retained native definitions. Report tail
or draw execution within accepted parent closures separately. Require nonzero
Rust counts only for roots actually observed externally in the selected movie.
An unexercised real-boss/explosion root stays translated/offline-verified.

The bounded attack and broad script seams overlap at ProcessEnemyInstructions.
Require exactly one installed owner. For a broad gate replace the bounded attack
interpreter wrapper with the broad selector/dispatcher; retain attack-regression
opcode and allocation requirements inside the broad observed domain. Reject two
predecessors claiming the same root instead of applying two renames.

## Live ABI and transaction lifecycle

Add a separately sealed live template (for example `live_ridley_subsystem.rs.in`)
instead of weakening the controlled ABI. A numeric root ID avoids an unbounded
C string read. Suggested entry signature:

```text
int32 port_ridley_subsystem_live(
  uint8* ram, size_t ram_len, const uint8* rom, size_t rom_len,
  uint16* host, size_t host_count, uint32 root_id, uint32 execute);
```

Validate null/extent/overflow/alignment and pairwise disjoint WRAM/ROM/host
buffers, root ID, and pinned ROM identity. Copy full WRAM and eight host words
into a private transaction. Catch selector panics separately. Selector false
returns -2 without any canonical write, host callback, hardware transaction or
effect. Invalid/error returns -1. `execute=0` is a read-only support probe.
After support true, execute the public method on the private transaction. Any
error/panic terminates the process with exit4 and preserved diagnostics; it never
retries C. Success publishes the full private WRAM and all eight host words.

The C wrapper must probe support in both baselines and mixed subjects so its
selected capture keys are independent of ownership. The baseline executes C;
mixed executes Rust only for accepted calls. Maintain total/C/Rust/refused,
supported, timer-due, frozen and material-change counts by outer root and native
header/area/window. Compare duplicate C subjects and duplicate mixed subjects.
Across modes compare traversal totals and admitted domains, not ownership bits.

Public admission already refuses any lifecycle call reaching SpawnEnemy or
EnemyItemDrop on its private preflight. It also refuses unreviewed Samus codes,
queue/OAM/table extents and unknown native scripts. Do not replace those services
with no-ops, broaden the selector to fit a replay, or use fixture dispatch on a
refused frame. They remain whole-call C ownership until continuation footprints
are translated/reviewed. A foreign service error after support is fatal.

## External service and hardware bridge

Reuse the controlled bridge's exact seven service IDs, signatures and state
publication contract. Rename definitions, not only linker-wrap, in native
bankA0 and bank90 so native same-unit service calls are observed too:

| ID | Native service | Signature |
| --- | --- | --- |
| 0 | SpawnEnemy | uint16(uint8 bank, uint16 population) |
| 1 | EnemyDeathAnimation | void(uint16 slot, uint16 animation) |
| 2 | Enemy_ItemDrop_Ridley | void(uint16 slot) |
| 3 | NormalEnemyTouchAiSkipDeathAnim_CurEnemy | void(void) |
| 4 | NormalEnemyShotAiSkipDeathAnim_CurEnemy | void(void) |
| 5 | NormalEnemyPowerBombAiSkipDeathAnim_CurEnemy | void(void) |
| 6 | CallSomeSamusCode | uint16(uint16 code) |

Before each Rust service publish its current private full WRAM and eight host
words to native canonical globals. Call the observed public native service,
then copy full WRAM/host back into the transaction. Record balanced entry/return
snapshots exactly as the adapter: five LE u32 words (phase, kind, a, b, return),
131,072 WRAM bytes, sixteen host bytes. Keep nested native services in order and
count them as C ownership. Check terminal contact/death and Samus0/1 continuation
contracts; the live bridge must not let an arbitrary host violate their reviewed
write footprints.

The controlled host's RidleyHostLoad/Save duplicates the existing projectile
bridge. In live use PortProjectileHostLoad/Save once, so both seams share the same
eight globals. Do not copy the controlled standalone main, fixture worker or
SpawnEprojWithRoomGfx definition into live, where those symbols already exist.

Mode7 must use real WriteReg/ReadReg on the actual `g_snes->ppu`: M7A low, M7A high, M7B
factor, MPYM low then high reads. Retain the twenty-byte PPU transaction state:
eight LE matrix words, m7prev, ppu1openBus, ppu2openBus, snes openBus. Capture this
before/after each material outer root and periodically/terminally. Existing live
snapshots already serialize PPU state periodically, but there is no complete
per-root PPU/open-bus capture or ordered SNES-read log. The current inventory and
fixture adapters capture/restore `my_ppu`; native WriteReg/ReadReg route through
snes_read/snes_write to `g_snes->ppu`, initialized to `snes_ppu`. The captured
field may therefore be a different PPU from the multiply transaction. Seal the
active PPU identity (or both PPU states plus selection) and restore the exact
active state before independent fixtures. Existing captures cannot establish
that stronger claim without checking which PPU was selected at each boundary.

Add the native inventory's deterministic snes_read/snes_write definition seams
or the controlled adapter's tested link wraps. Log resulting real reads/writes
in exact order through the packed effect writer. Keep the existing ppu/APU log;
use distinct event tags to prevent interpreting duplicate observation formats
as additional hardware operations. Read values and buses are material evidence,
not arithmetic-equivalent substitutes.

Audio/music/projectile request observers must preserve native order. Rename all
fifteen QueueSfxN_Max{1,3,6,9,15} definitions plus QueueMusic_Delayed8 in bank80.
The Rust subsystem already updates audio/music WRAM before ordered_audio/music;
its callback should log the request only, never invoke C queue logic again.
Services that internally queue requests still execute observed C queue routines.

Current Host signatures pass WRAM/projectile state to `service`, but not to
mode7_multiply or ordered_audio/music/projectile. A logging-only callback can use
those existing methods, and a pure Mode7 callback can use native PPU globals.
However, current interfaces cannot provide exact before/after projectile WRAM
captures or publish current private WRAM before every observer/hardware call.
For that stronger contract, make an explicit observer-context extension to the
owned Host/module, or carefully review an ABI-owned nonretaining raw transaction
view with compiled alias/lifetime controls. Do not claim canonical g_ram is
current merely because an effect callback has run: it may still contain the last
service publication. The safest extension passes ephemeral WRAM/host views at
each operation; it requires a new source freeze and affected-family refresh.

## Native137 allocation and opcode accounting gaps

Whole Rust `Context::spawn` directly executes the maintained allocator; it
bypasses the native live SpawnEprojWithRoomGfx wrapper. Its current
ordered_projectile(definition, parameter, slot) callback occurs only after
allocation and lacks before WRAM/host. Therefore the existing native137 counters
and first-two allocation capture gates cannot remain correct by merely appending
that log line. The same gap affects script execution. For definition9642,
record pre-allocation F/source state, free-pool status, full WRAM/eight host words,
the native allocation invocation and its result exactly once. Reuse/generalize
the bounded attack PortRidleyAllocationBefore/After accounting, adding a checked
before/after observer to whole Context::spawn. Track other definitions honestly;
E509 dust is outside the nine admitted native137 scheduler IDs.

Broad E509 births can change the next EprojRunAll support decision. Validate this
against the final movie before requiring native137 zero scheduler refusals; do
not quietly drop its existing gate. If the observed broad route produces an
unknown scheduler definition, either extend/verify the complete native137
closure with owner approval or retain that affected broad caller under C until
its compositional support is known. Source coverage alone does not prove the
new pool is supported.

Broad process_instructions returns a ScriptReport containing ordered opcode
address, argument, canonical return and pointer kind. Logging the report after
the entire call preserves opcode order but does not preserve its interleaving
with sound/projectile/hardware effects. Keep a separate opcode stream when only
opcode order is required, or add an immediate per-opcode observer at the callback
completion point for a unified stream. Never duplicate the interpreter in a new
ABI without differential controls; use the maintained public method.

The existing bounded admission hardcodes eight attack opcode IDs and full-route
2393/36 counts. Broad admission needs a separate typed contract for eighteen
supported opcodes (sixteen bankA6 callbacks plus generic Goto/Sleep), preserving
the attack subset as a regression requirement. Full-script naturally observed
totals must come from the native inventory/captured checker, not from the old
attack constants. Keep raw pointer kind (null/bank offset/ROM pointer) alongside
canonical return. Generic Goto/Sleep actual callback addresses can use bankA6
dispatch aliases; preserve the inventory's address conventions.

## Measured ROM/admission blockers

`/root/ridley_captures` independently checked 670 fixture-supported natural
samples against public admission. With original3MiB ROM there are574 admitted
and96 refused:92 CallRidleyFunc fixture-only entries and4 getaway samples. The
other sampled roots admit: Ceres init3/main92, draw92, tail66, scripts230,
opcodes75, shot4, contact8; four additional getaway samples admit. These are
sample counts, not full live ownership totals. The captured fixture owner also
reports a natural differential mismatch, so admitted sample counts grant no
execution parity or live acceptance.

Against the actual SnesInit patched/padded rom-view.bin, all670 samples refuse
the broad FNV ROM admission guards. Existing live_ridley.rs.in passes `g_rom`
(the native view) to the selector, so directly generalizing that ABI would own
zero broad Rust calls. Retain both exact original-ROM and native-view identities.
Use an explicit original admission view, and either execute the Rust translation
against original bytes after proving relevant patched/native data equal, or
separate selector/execution views under an explicit checked transformation
contract. Passing original bytes to selectors while dispatch_host silently
rechecks the native view will still refuse. Do not relax/hash-replace the guards
merely to make patched code bytes pass. Narrow runtime data equality and source
identity must be established for the complete accepted closure.

## Registration, authentication and verification sequence

Root-owned changes required after this source freeze:

1. New explicit predecessor field/catalog live kind for the whole adapter.
   Require `rust-ridley-subsystem-v1`, callback_adapter ridley-subsystem,
   gameplay_translation true, exact installed module and complete required
   native roots. Current manifest live_eligible=false cannot silently grant
   admission. Add an explicit new live-root contract while preserving controlled
   fixtures. Require the native137 predecessor; reject overlapping attack owner.
2. Generate the new ABI/host/bank templates, deterministic transformed units,
   root symbols and dependencies. Extend typed live verdict/counters with whole
   lifecycle installed state, root order and measured C services. Seal new
   templates and module/host extensions as compiler inputs.
3. Match captures by native outer-root/header/area/window/state keys selected
   using the same selector in all four subjects. Retain full WRAM, eight host
   words, PPU state and ordered effects/services. Packed inventories must compare
   byte-for-byte; logical ownership counters may differ C/Rust by design.
4. Extend retained live authentication to reconstruct/check source transforms,
   reparse stdout counters, validate new capture completeness/nesting/effects
   and rerun the whole-root and preserved native137 admission contracts.
5. Run independent compiled ABI/support-refusal controls, accepted-error no-C-
   retry, native same-bank service controls, Mode7 read/order/bus controls,
   allocation instrumentation controls and nesting/composition controls. Run
   appropriate port/pipeline focused tests and Clippy; refresh all affected
   Offline families for the exact final freeze. The public-captured supplement
   owner establishes accepted/refused natural sample domains separately.
6. One combined four-process129712 live gate after the current bounded gate has
   completed and shared changes are unfrozen. Compare state/effects/captures,
   zero CPU fallback, actual nonzero outer Rust ownership, explicit refused/C
   scope, measured services and native137 regression counts. Do not require
   absent E17F/area2 or explosion exercise without a movie demonstrating them.
7. Scoped cleanup/dedup and post-cleanup authentication, exact-source retention,
   integrated worktree retirement, then publish stages actually achieved.

Estimated verification bounds: initial compiled seam controls should be capped
at60seconds. The prior native full inventory took roughly140seconds; four live
subjects run within the existing worker capacity, with each native live timeout
currently max(240,frames/40), about3243seconds at129712. Integration gates cap
at3600seconds. These are planning bounds, not measured broad-live timings. A
fresh affected-family batch may dominate wall time; use native cache and the
existing fork fixture executor, preserving actual-C isolation.

## Retaining the old controlled family after batch.rs changed

Old controlled verdict:
`out/port/ridley-subsystem/recovery-controlled/1791858747222202-2-0/verdict.toml`.
Its batch source seal and retained pipeline-batch.rs are
`c35d808ab379df15d72a262c3781da85cf8936bf1a6ebc9deec3067953c05be5`.
Current batch.rs is
`4aa716117c3cfd0fae0597a13f0fdfac7a0520c0cf99f4ef8905325763201219`.
The old verdict is historical frozen evidence and cannot authenticate against
the current main checkout. Do not rewrite its source map or verdict hashes.

Build a small source-context snapshot from every implementation.rust_dependencies
and harness_sources entry, using retained dependency-*.rs, pipeline-* files and
candidate.rs when current bytes differ. Verify each against the recorded digest.
Include pinned tools/dependencies.toml and the exact source/archive links needed
by family::verify_source, plus every original absolute captured asset/ROM path.
The old module and manifest identities are absolute paths under
`/home/jim/sm-decomp`, so `family-authenticate --checkout SNAPSHOT` alone is not
sufficient after those live files change: family::source_path keeps absolute
paths and the manifest digest is checked at its original path.

Concrete retention options, preserving unchanged sealed evidence:

- Keep the original absolute candidate/manifest files intact and run retained
  authentication with the snapshot for relative tooling dependencies while
  those absolute files remain exact. This is sufficient immediately after the
  batch-only change if candidate/manifest bytes still match.
- For later candidate/manifest changes, run the reader in a private read-only
  mount namespace binding the frozen source context at its original absolute
  checkout path, with retained run/source/archive/ROM paths available. `bwrap`
  and `unshare` binaries exist here; namespace permission has not been tested.
  This virtualizes paths without restoring old bytes into the active checkout.
  Alternatively add a separately sealed exact-source mapping authentication
  receipt/schema that explicitly resolves original paths to hash-checked source
  snapshots; this requires reviewed reader work, not editing the old seal.
- Freshly verify the controlled family on the final new source freeze to obtain
  current-source admission. That does not replace retention of the old record.

Do not temporarily overwrite shared main files during an active source freeze.
Retain native inventory's transformed source, rom-view.bin, all captured
companions and service logs; preserve existing evidence store/readers.

## Work done by this design lane

Only this Markdown design file was added. An unused admission-probe source was
created under /tmp, then deleted before any build or execution once the capture
owner was identified. No shared sources, captures, verdicts or stores were
changed; no verification run was launched. Cleanup scope contains no run and
reclaims no verification artifacts. Measured natural public-admission sample
counts are attributed to `/root/ridley_captures`; no second capture probe ran.
