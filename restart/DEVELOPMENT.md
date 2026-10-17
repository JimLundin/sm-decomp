# Small translation checks and combined gameplay

Translate a complete scheduled operation into its gameplay crate, check it
against pinned recovered C, then install that same implementation in `Game`.
Keep private helpers together with their operation. Isolate an individual
routine when diagnosing a difference; its passing case does not complete the
whole Translation scope.

## Current local execution prerequisites

Run builds, tests, verification and artifact production locally, as required by
[AGENTS.md](../AGENTS.md). Historical hosted results retain their original
source and test scope; they do not verify a later candidate. Do not dispatch
hosted workflows or change target paths to evade a local execution policy.

Python may run the DLL host under the project-language exception in
[AGENTS.md](../AGENTS.md). Keep the launcher, Python interpreter, actual DLL,
compiled-source identity and native/private inputs separately authenticated.

The production Windows DLL now builds locally without Serde, TOML or
`crc32fast`. The earlier [build-script denial](../.scratch/agent-blocker-support/issues/01-local-cargo-denial.md)
is retained as diagnosis. Frozen source `1016c326` uses a bounded Rust evidence
codec and `flate2`'s `zlib-rs` archive backend. With the recorded Windows GNU
toolchain and `restart/private/toolchain/w64devkit/bin` on PATH, run:

```sh
cargo rustc --manifest-path restart/Cargo.toml -p sm-restart-dll-host --features private-checks,native-preparation --locked --offline --jobs 1 -- -C link-self-contained=yes
```

The final DLL passed 141 assertions through dedicated Python host processes:
39 evidence, 81 Return, one adapter failure control, seven energy-prefix and
13 item assertions. It also prepared a native library and executed the Return
route to the intentional first-full-span review stop. See
[the local DLL audit](../.scratch/restart-parallel-loop/final-local-dll-execution-audit.md).
The DLL writes PPM screenshots and binary `.sme` reports; legacy readers and
optional CLI PNG have separate dependency and compatibility limits.

Later checkpoints add executed refusal/native-integrity/receipt controls
(`d83d456e`), PPM and complete counter mapping controls (`f916a36f`), and
maintained native preparation/smoke controls (`96632949`). External commands are
`prepare-control ARCHIVE FRESH_OUTPUT RUNTIME PRODUCER_DLL` and
`native-smoke ARCHIVE ROM OUTPUT`, using absolute paths in dedicated DLL children.
The producer argument must match the actual loaded DLL. See
[the native checkpoint](../.scratch/local-execution/binary-migration-final/native-controls.md)
for exact DLL/source/run identities and remaining provenance/behavioral limits.

Ordinary Cargo test executables, Clippy and the standalone result-reader launch
remain denied on this host. DLL assertions establish their named scope;
complete workspace/native/original-ROM acceptance and cleanup remain open.
Root Unix-specific tools retain a separate platform requirement. The local DLL
route requires no hosted runner or endpoint-policy change.

Use [the current local verification queue](../.scratch/restart-parallel-loop/local-verification-queue.md)
for achieved checks, held scopes and the owner of each next action. Coordinate
the existing producer slot before any verification command.

## Focused edit loop

From the repository root, select one existing package:

```sh
just restart-focused sm-rio
```

The recipe runs package formatting, Clippy with warnings denied, all-target
tests and documentation tests with all features, locked offline dependencies and one
Cargo build job. It selects one package rather than the whole workspace. These
gameplay packages compile without native-host. All except sm-item-instructions
are dependency-free; its fixture/test support uses sha2:

| Package | Operation |
| --- | --- |
| `sm-projectile` | Ordinary projectile lifecycle |
| `sm-room-scroll` | Room scrolling |
| `sm-ceres-door` | Ceres door operations |
| `sm-room-transition` | Room transitions |
| `sm-door-scrolling` | Directional door motion |
| `sm-camera` | Normal camera |
| `sm-item-instructions` | Owned collectible instructions and pickup |
| `sm-player-motion` | Vertical motion and solid-enemy contact |
| `sm-persistence` | Bounded Save/Load and SRAM codec |
| `sm-waver` | Waver actor bodies |
| `sm-ripper` | Ripper and JetPowerRipper patrol |
| `sm-rio` | Ordinary Rio actor bodies |

Without `just`, run the recipe's Cargo commands directly. For example:

```sh
cargo test --manifest-path restart/Cargo.toml -p sm-rio --all-targets --all-features --locked --offline --jobs 1
```

Unit tests call the production gameplay implementation. For Rio,
`rio/src/tests.rs`, the actual-C verifier and `native-host/src/rio_hooks.rs`
all call `sm_rio::execute`. Scripted native responses exercise the declared
transport seam; real native terrain still requires its own comparison.

Keep each scope's incremental target during active edits. A recorded Windows
measurement of the same focused host `cargo check --all-targets` command took
1089 ms cold and 181 ms warm with identical 62-file inventories. This measures
unchanged-source compilation, not executed tests or Rust-versus-C parity.
See [the measured evidence](../.scratch/agent-code-quality/evidence/feedback/measurement.md).
Measure an actual edit-to-result loop before claiming a further improvement.

## Verification grows with the scope

| Check | Existing entry point | What it establishes |
| --- | --- | --- |
| Gameplay edit feedback | `just restart-focused PACKAGE` | Selected Rust package's formatting, lints and executed tests |
| Rio host transport | `just standalone-check restart/native-host/rio-checks/Cargo.toml` | Exact production transport/preparation modules and their declared controls |
| Rio actual-C actor bodies | [rio-verify/README.md](rio-verify/README.md) | Full canonical WRAM, outputs, ordered requests and script consumption against compiled recovered C |
| Real native services and failure | [rio-host-control.rs](native-host/examples/rio-host-control.rs) and the scope ticket | Native terrain, admission, installed refusal, terminal failures and exact ownership accounting |
| Combined installation | [rio-runtime-control.rs](examples/rio-runtime-control.rs) and the scope ticket | Production Game installers with twelve owners; four actor C/R pairs, full WRAM/SRAM/effects and sibling counters |
| Merged workspace | `just restart-check` | Complete restart workspace checks including fixtures/examples |
| Reached gameplay | Fresh C/mixed runs through `sm-restart run` on frozen ordinary inputs | Named postcondition, causal state/effect agreement and material Rust ownership on the reached route |
| Original-ROM agreement | `original/` commands and the corresponding scope ticket | Only the explicitly compared original-ROM domain |

The combined installation control registers player and persistence owners but
does not invoke them. Keep their own affected controls. A natural Rio encounter
and attack/recovery require ordinary route execution; actor fixtures or a
successful installed hook do not establish that reach. Consult each ticket for
its exact arguments, fresh output paths, source identity and complete gates.

## Recorded composition and shared private assertions

For complete recorded comparison, add `--trace yes` to the maintained `run`
command with explicit `--inputs`, exact `--steps` and `--present no`. Supply a
fresh output directory for each process. The tape must contain exactly that
many inputs; authored tails and presentation are rejected. First produce a C
reference with `--gameplay c`, then seal its `trace.bin` externally. A separate
C replay and mixed replay both add these options:

```text
--trace-reference ABSOLUTE_REFERENCE_TRACE --trace-reference-sha256 EXACT_LOWERCASE_SHA256
```

Comparison authenticates reference identity/count before Game startup and checks
every completed step's full 128 KiB WRAM, SMNOBS02 external state including SRAM,
and ordered effects. Failed acceptance or mismatch leaves the trace unsealed;
the first mismatch retains raw diagnostic bytes. Compare declared ownership as
well as trace equality. Ceres evidence covers 16,991 recorded steps, with material
Rust camera/scroll/transition/projectile/Load calls; refused or unexercised
owners retain their own gates. See [the authenticated composition audit](../.scratch/restart-parallel-loop/fulltrace-e71729a-audit.md).

The opt-in `private-checks` feature exposes the same assertion functions that
ordinary Cargo tests register. Through the existing once-per-process supervised
DLL export, the command is `private-checks --suite SUITE --out ABSOLUTE_PATH`.
The retained corrected07 DLL provides `item`, `energy-prefix`, `evidence`,
`return`, `adapter` and `author41-result`. Consult `src/private_checks.rs` for
the current source's registration; later gameplay edits add suites separately.
For item or energy-prefix, set exact absolute
`SM_ITEM_FIXTURE_SEED`/`SM_ITEM_FIXTURE_ROM` or `SM_ENERGY_PREFIX_WRAM` in the
dedicated child environment before loading the DLL. The input-bearing
`author41-result` check requires `SM_AUTHOR41_EVIDENCE_ROOT` pointing to its exact
retained result/WRAM directory. The adapter validates declared inputs and writes
named assertion logs and a terminal count/failure report. It does not require
Game or a native-library argument. The corrected result check's passed and
missing-input runs use the retained DLL `2b097d73`; see
[the independent review](../.scratch/verification-support/author41-corrected-07-review.md).

Separate processes executed 13 item assertions (12 private plus 1 input-free) and
7 energy-prefix assertions. Three Construction capture regressions remain
registered in ordinary Cargo tests with missing prerequisites; the feature
runner does not execute them. These Rust assertions do not replace compiled
actual-C Fixture pairs, live ownership or complete workspace checks. See
[the execution audit](../.scratch/restart-parallel-loop/private-assertions-6d3b8b2-execution-audit.md)
for exact source/DLL/input identity and acceptance limits.

## Parallel ownership

Use the current restart ticket and real-owner inspection before dispatch.
Source workers use separate worktrees with exact disjoint writable crate files
and pinned baselines. A worker translating gameplay may work alongside an
independent verifier preparing the recovered-C corpus and auditing admission.
Shared Cargo registration, Game/native/CLI changes, source freezing, producer
commands, integration and stage publication belong to the integration owner.

The current restart limit is two active source scopes and one Cargo producer
job. `--jobs 1` limits one Cargo invocation; launching several invocations does
not enforce that shared limit. Coordinate the producer slot before running
these commands. Independent source editing and read-only review can continue
while a gate runs. New concurrency requires an explicit ownership and resource
decision, rather than competing writes to the same target or index.

Prepared horizontal movement and fresh-process persistence work live in
[the support queue](../.scratch/restart-support/map.md). They remain unclaimed
until their current dependencies and writable ownership are checked. Retained
original production claims remain under their own fenced authority.

After integration, freeze merged source and run every affected scope's complete
gates. Publish translated, verified, integrated and live-exercised separately.
Keep full state, ordered effects, unchanged refusal and terminal failure rules
at every scale. Finish with [run cleanup](../docs/agents/run-cleanup.md), retained
source/evidence authentication and removal of integrated worker worktrees.
