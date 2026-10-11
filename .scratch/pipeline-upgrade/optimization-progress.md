# Native Rust migration progress and remaining upgrade work

Observed: 2026-10-11, read-only in `/tmp/sm-decomp-pipeline`.
Branch HEAD: `e820a6b77338eb4e84a2d91baeed2dc9f33c72f1`.
Source/report hashes: [optimization-source-snapshot.toml](optimization-source-snapshot.toml).

The optimization branch has substantial uncommitted Rust migration work beyond
HEAD. Its current source and local completion notes replace the old Python
implementation seams in this proposal. This is an active worktree observation,
not a committed release or an atomic source freeze. The migration owner must
provide a reviewed revision and final-source evidence before integration.

## What exists and should be reused

| Area | Inspected native implementation | Recorded evidence or limit |
| --- | --- | --- |
| Process/build | `builds::{execute_timeout,exec,cargo_build}`, shared leases/caches | Existing elapsed/wait receipts; no PID CPU/RSS or full capture-to-cleanup summary observed |
| Cleanup | `cleanup.rs` and native filesystem tests | Unreadable evidence, symlink/path escapes, unfinished runs and dedup controls; recorded 1,008,861,184 + 158,466,048 dedup bytes |
| Family/batch | `family.rs`, `batch.rs`; native compile, Rust-before-C, full WRAM/returns and host checks | Five-family batch/reuse summaries; current source changes can invalidate their seals |
| Capture/publication | `packed::{capture,publish,materialize}` | Private stores, one capture lease, retained failed staging, reopened auth and shared import; native packed tests exist |
| Live/authentication | `live.rs`, C templates, `observations.rs` | Notes report direct system and system/audio batch four-subject 1,000-frame runs; final live adversarial controls remain pending |
| Fixtures/bootstrap | Native fixture modules, standalone `tools/bootstrap`, movie corpus | Notes report all 44 fixture domains, pinned PLM ROM and 1,444 declarations, 36 APU tests and doctor/bootstrap checks |
| Decomp/readiness | Native acceptance/RNG/reset and readiness/parity modules | Notes report decompressor 40/23/3 cases + 11 controls, four 9,000-frame reset subjects and all 65,536 RNG seeds; readiness/iteration completion remains pending |

The recorded native batch covers system 475, audio 1,147, PLM 1,444, pirate host
577 and projectile script leaf 342 cases. Its copied summary entries are
`offline-verified`; the subsequent report lists all five as `reused` with zero
additional dedup bytes. The reported family `seconds` are carried into that reuse
report, so they are not measured reuse durations. They also cannot be summed into
a batch total while families overlap.

`sm-rust-live-retained-auth.json` authenticates four subjects and 108 members each,
explicitly scoped to a retained frozen binary and observations. It certifies no
current-source admission or original-ROM parity. Native live and reset run claims
come from migration ticket completion notes; this review did not rerun or
reauthenticate those large run directories. The retained test log covers 3 unit,
3 cleanup, 7 family, 4 observation and 4 process tests on its recorded source.
Other tests now exist, but that log does not establish their latest execution.

## Language and evidence contracts

The worktree's `AGENTS.md` and `.scratch/rust-tooling/spec.md` require Rust tools,
orchestration and tests, no Python runtime dependency, typed native interfaces,
TOML for small readable metadata and binary for large traces/state. Existing JSON
readers remain for external corpora or historical sealed evidence. The new native
code still uses dynamic JSON requests/receipts; the spec calls that transitional.
New upgrade work must follow the intended native contract without rewriting old
canonical objects or expanding into a second migration effort.

Migration spec is open. Tickets 01–06 are claimed and 07 is open in the inspected
state. CI/recipes still include Python commands. Final cleanup, final source
verification, adversarial live/readiness controls and commit are pending. Source
presence and completion notes are progress, not proof that migration is complete.

## Concrete remaining seams

- `effect_writer.c.in` uses `vfprintf`. `live_host.c.in` emits formatted PPU/APU
  calls. `packed::capture` invokes evidence `capture`, not a binary decoder.
  Binary transport is still a possible future optimization.
- `packed::Timing` writes elapsed/completed/worker wait. Capture's pointer timer
  stops before its subsequent subject authentication and staging release.
  `packed::publish` starts its timer before store/worker waits but records it
  before pointer replacement, final authentication and private deletion. Native
  `live::verify` adds a capture elapsed field and performs final authentication;
  a complete outer benchmark boundary still needs explicit definition.
- Native `family::verify` builds a flattened case task list; `host_bytes` reads
  explicit eight-word `host_words`. No `case_order`/`continue_from` implementation
  was found there. Native live templates lack current decomp's WithGfx bridge.
  Both newer decomp contracts remain integration prerequisites.
- Native child polling uses `Child::try_wait`; CPU accounting must share one
  process owner with that lifecycle, not race it with a second `wait4` caller.
- Snapshot/import/authentication remain real costs. No new controlled 45,000
  or 129,712 native whole-pipeline benchmark was found in the inspected reports.

## Effect on the plan

Integrate delivered native migration and current decomp semantics first. Extend
native measurements, profile the text baseline, and select the next bottleneck.
Conditional binary work targets `packed.rs`, `live.rs`, `batch.rs`, C templates
and the evidence reader. No new Python helper, duplicate packed/live port or
blanket metadata migration is proposed. Historical e820a6b capture numbers remain
historical measurements of a different pipeline/source combination.

No files, index, refs, builds, runs or evidence in the optimization worktree were
changed by this review. The source snapshot records per-file hashes while other
work may continue; recheck it against the delivered revision before implementation.
