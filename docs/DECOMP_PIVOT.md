# Pivot to readable Rust game source

Agreed on 2026-10-10 after a planning interview. The destination is a playable Super Metroid source port with **full ownership of the gameplay logic in readable Rust**, with SNES hardware emulated. It does not require generating a byte-identical SNES ROM. Get vanilla Super Metroid working first; integrate Map Rando afterward. This is the agreed direction and first implementation plan, not evidence that a decompilation runner is ready. Existing recomp acceptance records remain specific to their original contract.

## Agreed scope

- Stop expanding instruction recompilation as a development strategy. Keep the existing runner as an oracle and diagnostic tool; fix it when a concrete decomp task requires it.
- Start with the existing **any% TAS** as the first live integration target. Do not prescribe low%, 100%, or an exhaustive scenario suite now; reassess coverage after any% works.
- Original lag frames and exact video-frame synchronization are not requirements. Ignore or compensate for lag where practical, while diagnosing actual gameplay divergence. The replay adapter is an engineering investigation, not a settled timing contract.
- Temporary C gameplay and original CPU fallback are acceptable during development, but must be visible and measured. A movie pass with fallback is an integration milestone, not completion of Rust gameplay ownership.
- Preserve original RAM layout initially through safe, named accessors. Make routines readable before undertaking broad state restructuring.
- Prove an offline translation, then prove a live Rust replacement before translating a large batch.
- Map Rando seed support is the later destination, including its custom gameplay behavior. Preserve trick-sensitive quirks by default and make a best effort at supported techniques. Costly exceptions must identify the affected trick, implementation cost and compatibility consequence; exceptions that can make supported seeds unbeatable need explicit user review.

The vanilla ownership phase ultimately requires a playable game from boot through credits without gameplay fallback. Passing any% is the first target, not proof that all game logic is implemented or a commitment to a particular later coverage suite.

## Recommendation

Translate recovered Super Metroid gameplay into maintained Rust. The primary source is **not selected yet**: favor a practical runnable reference with demonstrated gameplay coverage over declaration counts or headline completeness claims. Evaluate whether `MrJackSpade/SuperMetroidDecomp`'s reported lag-compensated replay can work for this project; otherwise use the strongest practical baseline, including `snesrev/sm` or `testyourmine/sm_rewrite`. Do not turn selection into a prolonged survey. Use named assembly/bank logs to settle discrepancies against the original ROM. The C# pin currently has no located project license, so source copying/adaptation requires established permission; inspection can still guide independently verified work. Choose source per routine, recording the choice; do not merge an entire fork's behavioral changes without review.

Current evidence does not establish a coverage winner. Both inspected C candidates retain CPU execution fallback, including the `Bang_Main` path. The C# project reports lag-adjusted full-movie passes, but its developer status also lists continuous-play and subsystem gaps. Its reported passes use different movies from this repository. Reproduce useful claims and record source identity, fallback, input conversion and comparison adjustments before relying on them. See the community audit for pinned evidence.

The important work is preserving game state, integer behavior, data access, dispatch and frame scheduling. Extending the timed 65816 instruction whitelist is no longer the primary route to this goal. Retain that runner as an oracle and diagnostic tool.

See [DECOMP_SOURCES.md](DECOMP_SOURCES.md) for upstream pins, sources, licensing, extraction material and concrete limitations, the [current community audit](DECOMP_COMMUNITY_AUDIT.md) for newer efforts and unmerged corrections, and the pre-existing [project comparison](DECOMP_PROJECT_COMPARISON.md) for candidate context. Refresh selected branches, PRs and source identities before preparing the first translation task; stable pins and awareness of newer work serve different purposes.

## What is already available

The local workspace contains:

- The 3 MiB ROM at `rom/sm.sfc`, pinned upstream C source at `deps/snesrev-sm`, and original any%, low% and 100% movies plus imported inputs.
- 32 `sm_??.c` bank source files in the pinned upstream tree. `funcs.h` has approximately 4,226 declaration lines, `variables.h` has 1,110 macro declarations, and `assets/names.txt` has 19,217 address/name lines. These are inventories, **not completion percentages** or distinct translated-function counts.
- ROM-backed execution, deterministic inputs, original-lsnes checkpoint capture, Mesen tooling, hardware probes, local snapshots and immutable run evidence.
- An accepted recompilation pointer to `20261010T151922.381522Z-f49d430ddf6c`, for Japan/USA any%, 18,000 input frames. The pointed verdict's SHA-256 matches the pointer and records all mandatory gates passing. Its progress evidence reports 954,736 generated instructions, approximately 256.8 million interpreted instructions and 17 validated compiled address/width keys: about 0.37% generated execution.

The last item is an inspection of recorded evidence, not a fresh rerun or a complete re-verification of every sealed artifact. It establishes neither full-game correctness nor a decompilation baseline. The recorded later-domain timing disagreement remains debt; a full original capture is not a full candidate pass.

## Reuse map

| Existing component | Reuse | Required adaptation |
| --- | --- | --- |
| `movie`, movie manifest and imported corpus | Keep input parsing, identity checks and replay inputs | Establish the C/decomp frame and lag-frame convention; do not assume imported pads imply TAS synchronization |
| `snes` reference, patched RustySNES, lsnes/Mesen tools | Keep original-ROM execution, hardware knowledge and diagnostic oracles | Add a game-level hardware/scheduling adapter; first prove its lifecycle, DMA, NMI/IRQ and input semantics |
| `diff::Console` and region comparator | Reuse the interface and first-divergence concept | Add a decomp subject and versioned semantic snapshots, SRAM/video/hardware effects; the production CPU/clock comparison is unsuitable unchanged |
| Checkpoints, focused diagnostics and controls | Reuse evidence design and original-side capture | Serialize Rust game state, coroutine state, hardware and input position; create new caches and compare corresponding routine/frame boundaries |
| Immutable runs, source snapshots, digests, acceptance lineage, failure statuses | Reuse generic machinery | Introduce a separate decomp contract and acceptance pointer; current mandatory gates and reuse policy encode CPU-step execution |
| `recomp` LoROM reader, decoder and CFG | Keep as investigation tools | Use to confirm routines, branches and pointer behavior rather than generating the final readable source |
| `trace`, `config/functions.toml`, symbols | Keep addresses, names and observed execution as inventory seeds | Add C/Rust routine mapping, call graph, data dependencies and semantic coverage; address/MX keys are not the new progress unit |
| `runtime` operations and SingleStepTests | Keep useful low-level semantics and interpreter regressions | Translate C integer promotions/truncation explicitly; readable routines do not need opcode-by-opcode corpus acceptance |
| `game` crate | Possible home for maintained game logic | Current module only includes generated source and an empty override table; its routine table is not wired into `Subject::with_steps` as readable gameplay execution |
| Generated instruction tables | Keep locally as an oracle/archived implementation | They retain CPU mechanics and do not provide the desired readable game source |

Local evidence: [architecture](ARCHITECTURE.md), [subject runner](../crates/snes/src/subject.rs), [game library](../crates/game/src/lib.rs), [overrides](../crates/game/src/overrides.rs), [console comparator](../crates/diff/src/lib.rs), [readiness contract](../tools/readiness_contract.py), [reuse policy](READINESS_REUSE.md), [paired checkpoints](PAIRED_CHECKPOINTS.md), [dependency pins](../tools/dependencies.json), [symbol converter](../tools/symbols.py).

## Missing foundations

| Foundation | Current gap | Minimum useful deliverable |
| --- | --- | --- |
| Runnable source baseline | Candidates are locally available, but no independently verified reference build/replay baseline is established here | Select and pin a practical reference; build against the same ROM and record replay progress, fallback and comparison adjustments; assess the C# lag-compensated approach |
| State and numeric contract | C uses global `g_ram`, overlapping pointer casts, packed structures, bank pointers and decompiler macros | One canonical byte-addressed WRAM/SRAM representation with little-endian accessors and typed views; explicit wrapping, signed comparisons and pointer/address conversions |
| Asset/data interface | Original code reads numerous tables and instruction lists directly from ROM; existing setup only converts names | Validated ROM data access initially; schema/extent/digest records for extracted assets when needed; distinguish graphics/tables/game bytecode from CPU fallback |
| Executable Rust routine harness | No routine-level C/Rust differential entry point | Identical input fixtures, isolated state on each side, output/write-set comparison and machine-readable failures |
| Integration dispatch | Existing real-ROM subject dispatches instructions, not C-equivalent functions | A measured C-to-Rust replacement seam, or a Rust game-level host with explicit C fallback; prove callbacks, ownership, shared memory and error propagation |
| Scheduler and hardware contract | Existing host expects instruction fetches and cycles; recovered C schedules NMI/frame progression differently | Define frame/input/lag semantics, vblank work, IRQ/HDMA, DMA queues, audio ports, reset and save lifecycle; test at the adapter boundary |
| Coverage and acceptance | Current gates demand compiled address/MX execution and CPU state/timing parity | Routine/branch/data coverage, explicit untranslated inventory, semantic comparisons, exercised-path metrics and separate decomp acceptance lineage |
| Agent work inventory | No bounded source-translation jobs/dependency graph | Each job names pinned C source, ROM addresses, callees, RAM reads/writes, tables, fixture coverage, exclusions and a concrete acceptance command |

At inspection, `cc`, Cargo and `just` are available. `cmake`, `clang` and `sdl2-config` are absent. The upstream GUI build needs SDL development files; a purpose-built headless routine harness can start with `cc` and avoid that dependency. CMake is needed if choosing the rewrite's documented CMake build. These are setup tasks, not missing user files.

Initially preserve RAM layout with safe accessors rather than inventing a comprehensive idiomatic object model. For example, enemy indices may be byte offsets instead of element indices, and different C views can alias the same bytes. Separate Rust structs with independent copies would silently change behavior. Readable function names and control flow can improve immediately; deeper state restructuring can follow verified translation.

Keep the supplied ROM as a local data source initially. Complete extraction of all graphics, rooms and scripts is not a prerequisite for translating a pure routine. Extraction should be reproducible and validated before replacing the corresponding ROM reads.

## Validation that fits this goal

Use two complementary comparisons:

1. **C versus Rust translation:** identical routine inputs and initial memory, compare return values and affected state. This detects mistranslation quickly, including transient writes where relevant. Instrument fallback so a pass cannot come from silently executing the C/original path.
2. **Original ROM versus port behavior:** compare corresponding gameplay boundaries and effects, including persistent state, collision outcomes, resources, RNG, SRAM, rendering and audio interfaces as applicable. This detects upstream C bugs and differences inherited by a faithful Rust translation.

Do not demand identical CPU PC/registers, emulated stacks, instruction counts or all scratch WRAM from a high-level port. Do not silently exempt everything that differs either. Define each excluded implementation detail with a narrow scope and evidence that future observable behavior is unaffected. Keep raw mismatches available. The upstream comparator copies many regions between snapshots and patches original execution; its successful comparison alone is insufficient proof of vanilla equivalence.

For pure routines, compare the complete declared output plus unchanged surrounding memory. For MMIO routines, compare ordered externally visible effects and relevant pending hardware state. Frame-level screenshots alone miss state errors and audio differences. Deliberate wrong-output and transient-write fixtures should prove the harness rejects incorrect candidates; uncovered Rust replacements must fail coverage.

Exact TAS synchronization and original lag preservation are not acceptance requirements for this pivot. The existing importer preserves video-frame inputs; it does not yet derive a lag-independent gameplay input timeline. Investigate the C# replay approach and original-side input consumption to determine a practical adapter. Record skipped/retimed inputs and the reason for each adjustment so timing compensation does not conceal gameplay errors. A replay repaired through corrective state injection is not evidence of equivalent gameplay. If lag-only adaptation cannot carry the movie, preserve the first mismatch and report what remains unresolved rather than silently modifying the gameplay route.

Label evidence according to what it proves: routine equivalence, mixed C/Rust integration, adapted any% completion, or original-frame synchronization. No state injection during candidate replay or hidden fallback may turn a failed replacement into a pass. Do not rename an old recomp pass into a decomp pass.

## First implementation increments

Start with **offline routine translation** before attempting a complete live Rust game. This needs fewer foundations than whole-game integration.

### Preparation increment

1. Run a bounded source-baseline investigation. Check the C# lag-compensated replay's applicability first; select a runnable reference using actual progress and visible fallback. Pin selected sources and notices, and record relevant fork changes and baseline comparison patches. The local C decompression routine can support the pilot even if another project becomes the integration reference.
2. Specify the minimal state/ROM/numeric interface and a small routine manifest. Compile the actual selected C routine with only the required harness shims; record that source identity rather than retyping a second reference implementation.
3. Build deterministic C/Rust fixture execution, complete write-set reporting, explicit errors/timeouts, branch coverage and preserved failure evidence. Local commercial-derived fixtures stay ignored.
4. Prove one correct Rust routine passes, an intentionally incorrect routine fails, and an unexecuted replacement cannot count as accepted progress. Restore/replay fixtures deterministically.
5. Freeze the minimal translation contract and expose one command that prepares, builds, compares and writes a verdict for one job.

This is the launch gate for a **bounded offline agent loop**. It does not require all game assets, all hardware accuracy debt, a credits replay, or complete Rust frame scheduling to be resolved first.

### First pilot: RAM decompression

`DecompressToMem` at `$80:B119` is a strong candidate. It already has reconstructed C in `src/sm_80.c` and real-ROM execution coverage in the accepted recomp domain. The algorithm has literal, byte/word fill, incrementing fill, absolute and relative backreferences, XOR variants, short/extended lengths and a terminator. Its `DecompNextByte` moves from a bank's `$FFFF` to the next bank's `$8000`.

Use synthetic streams with independently specified expected bytes, captured real-ROM calls/data, and C-versus-Rust comparison. Cover overlapping backreferences (sequential copy, not a bulk non-overlapping copy), `u8` wrap in incrementing fill, odd word-fill lengths, extended headers, termination and bank crossing. Declare handling of malformed streams separately; do not infer new behavior from the C routine's unchecked pointer accesses. Do not include `DecompressToVRAM` in this first task because it adds MMIO and PPU latch semantics.

Passing the pilot means the Rust routine is correct within its tested contract. Full game integration still needs a scheduler-aware seam; merely dropping an untimed replacement into the current CPU runner would change hardware progress.

### Live replacement before scaling

After the pilot, wire one exercised Rust routine into the running reference game. A headless C host calling Rust through an explicit ABI is the default bridge; if the chosen reference offers a more practical seam, document that choice. Prove shared RAM ownership, callbacks, reset, error propagation and relevant hardware effects. A passing offline fixture alone does not prove this seam.

Replay the any% prefix with the reference and mixed implementation, recording Rust routine hits and remaining fallback. Require exercised Rust behavior and comparable gameplay results before expanding the translation queue. Do not simultaneously replace the hardware host merely to establish the first live seam.

### Routine loop after live integration

For each job: select a dependency-ready routine with observed fixtures; preserve the accepted predecessor; give the agent pinned C/assembly context and exact state/data contracts; translate one routine or tightly coupled family; run relevant checks and differential cases; diagnose the first mismatch; accept only exercised passing paths; record unknown branches and data dependencies; continue to the requested count.

Progress should report accepted routines **with exercised coverage**, replaced original execution where measurable, and remaining C/65816 fallback. Number of source lines or named functions is inadequate. Frame metrics become meaningful after live integration exists. Unknown routines must remain visible and cannot silently return defaults.

Before parallel translation, stabilize shared state/asset/hardware interfaces and allocate disjoint module ownership. Use one integration owner to serialize mutations to shared build/evidence artifacts. Parallel banks often touch the same global definitions and dispatch tables; a dependency-aware queue is safer than assigning one whole bank per agent.

### Any% as the first gameplay target

Expand the live replay toward the existing Sniq any% movie's expected completion, using lag adjustment where practical. Its archive contains 129,712 original video frames; the adapted port need not take that number of frames. Track the furthest verified gameplay checkpoint, first divergence, exercised Rust routines and C/65816 fallback separately. The existing full original-lsnes capture is a reference, not a candidate pass.

Keeping RustySNES as the final hardware backend is reasonable, but requires a new game-level host/adapter. Prove it independently using an unchanged game implementation before attributing failures to Rust translations. C frame checkpoints and RustySNES CPU checkpoints are distinct formats and boundaries.

Fix the gameplay, rendering, audio and lifecycle problems encountered along the any% route. Delay timing-sensitive IRQ/HDMA, NMI waits, complex coroutine/stack dispatch and VRAM decompression until their contracts are established. After any% works, reassess untranslated logic and choose the next coverage increment. Low%, 100% and an exhaustive trick suite are deliberately not scheduled by this plan.

### Later Map Rando integration

Vanilla Super Metroid comes first. Preserve trick-sensitive behavior during translation; do not implement randomizer adjustments as part of the first pilot or any% milestone.

Map Rando generates modified ROMs with gameplay code patches, settings-dependent mechanics and per-seed door/setup assembly, not only shuffled tables. Reading its ROM data through vanilla recovered routines cannot automatically reproduce those changes. Later integration must account for those behaviors in owned Rust, select a supported Map Rando version and seed interface, and define compatibility coverage then. Those implementation decisions remain open until the vanilla foundation exists.

The final goal is actual seed play with topology, items, settings and custom behavior. Navigation logic and technique inventories can guide testing but are not executable behavior oracles. Document unsupported tricks and their seed consequences rather than promising every technique or silently treating intended vanilla quirks as bugs. This later scope supersedes the older source assessment's blanket exclusion of randomizer behavior; that exclusion still applies when using a vanilla ROM as the immediate reference.

## Status and next work

The approved [eight-ticket foundation](../.scratch/decomp-foundation/spec.md) is
implemented on `decomp-foundation-integration`. [DECOMP_BASELINE.md](DECOMP_BASELINE.md)
records the pinned executable C reference. [DECOMP_ROUTINE.md](DECOMP_ROUTINE.md)
documents the accepted Rust RAM decompressor: 40 synthetic fixtures, 23 captured
calls, 38 exercised cases, complete ordered stores and 11 rejecting fault controls.
Three captured calls also match bounded original-ROM instruction execution with
explicit scratch and stack exclusions. Acceptance consumers verify the retained
artifact inventory and current source identities before linking.

[DECOMP_NEXT_RANDOM.md](DECOMP_NEXT_RANDOM.md) records the second routine: all
65,536 seeds match pinned C, and 16 captured calls verify complete WRAM effects.
Four independent C/mixed replay processes each run two reset epochs through the
9,000-frame prefix. Every mixed epoch executes 23 Rust decompression calls and
8,783 Rust RNG calls; checkpoints and hardware effects match C with zero measured
gameplay CPU fallback in this prefix. Other gameplay remains C-owned.

[DECOMP_PREFIX.md](DECOMP_PREFIX.md) preserves the nonaccepting original-ROM input
adapter investigation: the first unsupported prelude poll is at source frame 19,
and the first diagnostic controller-state mismatch is at original frame 272
against C step 151. A separate changed-RNG control produces an additional C/C
discrepancy. This result establishes neither adapted movie completion nor
original-ROM gameplay parity.

Follow-on bounded work should address that input boundary, expand verified
gameplay coverage toward any%, and select further exercised routines. Full Rust
ownership, hardware-host migration and Map Rando remain outside this completed
foundation batch. Existing recomp acceptance remains separate.
