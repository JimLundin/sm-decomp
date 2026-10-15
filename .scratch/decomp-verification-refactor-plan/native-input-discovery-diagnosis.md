# Native first-compiler input discovery diagnosis

Status: read-only diagnosis complete; no runtime fix or current-source reproduction claimed.
Owner: `/root/bulk_context_resume`, parent session `01a13e7f-ab01-7413-a1c2-288b08a4b719`.
Source examined: retained `7b27ea1` evidence and combined `b1f8691` source.

The historical failure occurred while importing the first C compiler's complete
runtime identities, before any compiler or Fixture process launched. The
largest observed interval surrounds `/usr/include`: 8,072 entries and
104,253,693 bytes. Captured gameplay inputs and ordinary shared build preparation
do not change that path. The trace locates the expensive stage, but cannot
separate its traversal, hashing, serialization and durable-publication costs.
Run the existing complete current-source control before implementing a fix.

## Exact evidence

Let `P` be
`/home/jim/sm-decomp-performance-native-proof/7b27ea1-20261015`
and `O` be
`P/full-real/operations/4764f8a884b920dd1b25cca3c515bdef69d838a17267d12def5bc0629d70d55b`.

| Observation | Retained evidence |
| --- | --- |
| Full test failed in 112.16 s; external wall 113.01 s, user 30.28 s, system 55.76 s, peak RSS 66,996 KiB | `P/logs/full-real.log`, `full-real.time`, `full-real-command.txt` |
| First C stage: 73.306093968 s, lease wait 0.022704437 s | `P/full-real/run/family/c-build.timing.toml` |
| Compiler timeout exactly 60 s; writable 128 MiB; stdout 4 KiB; stderr 256 KiB | `O/intent.json`, `P/full-real/prepared.toml` |
| No runner, sandbox, stdout/stderr result or output; one failed operation | `O/result.json`, `P/full-real/run/native-operations/index.json` |
| Actual rejection authenticated: zero Fixture pairs, one operation | `P/full-real/observed-authentication.toml`, `P/logs/full-real-observed-authentication.log` |
| Original expectation remains 16 pairs / 34 operations | `P/full-real/retained-authentication.toml`; this is an expected-pass descriptor, not a passed outcome |

The original diagnostic is exactly
`operation total deadline exhausted during declared input discovery`.

The immutable pre-cleanup timestamp inventory,
`P/logs/full-real-before-cleanup.stat`, bounds the relevant intervals:

- Operation lock: 19:26:41.511239875; failed result: 19:27:41.668561357
  (+02:00). This is about 60.157 s. The 73.306 s outer C timer also includes
  earlier caller reservation/framing; it is not a 13-second operation overrun.
- GCC include identity published at 19:26:56.124564197, followed by the
  `/usr/include` identity at 19:27:41.340561391: a 45.216-second interval.
- The next published identity is `crtbegin.o` at 19:27:41.464561378. The
  intent contains exactly 50 completed identity references; import then
  refused at the local deadline. The outer family still had about 104 seconds.

The `/usr/include` identity is
`P/full-real/operations/installed-runtime/def59499b858ea89fa5aac92d71a9905e9f4e44c9f56863a35b2f6f8d2cd0d0e.json`:
2,207,990 encoded bytes, 8,072 entries, 104,253,693 input bytes. All 50
completed identities total 148,089,483 input bytes and 8,226 entries. Of those
bytes, 140,111,344 are runtime; the 33 C source/header/oracle inputs total only
845,371 bytes. The Program declares 33 inputs and 69 runtime bindings.

Read-only SHA-256 checks:

| Artifact | SHA-256 |
| --- | --- |
| `O/intent.json` | `5e26bb1aebc34551818174149737ff8e7ce2b6aa6a829bd3b2d74ef2808eb201` |
| `O/result.json` | `91345f3fe2ec6de8e1a66961efa9cd7342b5b6cd899bcbff955a014ca9f6ffdb` |
| `/usr/include` identity above | `f25fad7b815df6b582e142009b9c9264313dccbe33d0468a983efbef440cfe94` |
| C timing above | `3370263763cbbc487544ad1023ac59533e328424e57bcfe34446295763ca3034` |

## Responsible code and ranked explanations

1. **Cold runtime import: supported.** A broad runtime predicts large runtime
   records and a stop before compiler launch. Both are present. In
   `translation-execution/src/operation.rs`, `run_with_deadline` calls
   `reserve_bytes_with_deadline`, then `import_identities`. The latter performs
   a metadata walk for its key, then `identity` / `collect_entries` /
   `input_hash_until` for a miss, serializes it and uses
   `retention::write_once` followed by `retention::seal_file`.
2. **Gameplay/Fixture volume: not supported for this stop.** That explanation
   predicts Fixture producers or a large candidate-input closure. No Fixtures
   ran, and the C inputs are less than 1 MiB. The runtime dominates the
   completed imported bytes.
3. **Worker wait or wrongly shortened deadline: not supported.** Lease wait is
   0.023 s, the saved local limit is 60 s, and the outer family deadline was
   later. The lock/result interval agrees with the local deadline.
4. **Host load, filesystem/cache state or build profile: unresolved confounder.**
   The trace has no per-phase CPU/I/O profiler. The later `b1f8691` three-pair
   Native control completed its first C stage in 14.700830818 s, with the same
   8,072-entry `/usr/include` identity. Its 180-second declaration, different
   source/helper and observation time make it an insufficient reproduction.

In `b1f8691`, the relevant operation source locations are
`reserve_bytes_with_deadline` (239), `collect_entries` (365), `identity` (430),
`verify_identity` (439), `import_identities` (455) and `run_with_deadline` (816).
`offline-verification/src/builds.rs::producer_reservation_bytes` (100) also
performs a metadata reservation walk before the operation starts. Its
`ConfinedBuild::execute` timer (283) includes that work.

`translation-execution/src/compiler_runtime.rs::discover_bounded` binds all
of `/usr/include`, GCC headers/link support and the Rust toolchain `lib`
directory. `offline-verification/src/builds.rs::compiler_program` gives the
same runtime vector to both C and Rust. The Rust library tree is another
778,772,978 bytes / 115 entries in the later control; the historical failed
import never reached it. Removing runtime bindings would change the declared
capability contract and is not the proposed narrow fix.

The `7b27ea1..b1f8691` diff leaves these discovery/reservation/import helpers
and `compiler_runtime.rs` unchanged. Root's optimized operation admission
changes retained-result reads after execution; preserve those changes, but do
not credit them with fixing cold runtime import. `BuildContext` applies to
ordinary compilation. Native capture changes candidate input locations and
adds a bound mapping; it neither captures nor removes this installed runtime.

## Smallest current-source gate

Use the existing ignored test
`translation_native::tests::full_real_family_retains_authenticates_and_reuses_every_producer`
on a frozen current assembly and its matching retained helper. Its exact test
arguments are `--exact` plus the name, `--ignored --nocapture --test-threads=1`.
No test patch is needed to select the original budgets: this control already
has a 60-second compiler limit, five-second Fixture limit, 180-second family
limit, workers four, all 16 declared pairs / 34 producers and the negative
admission/replay checks.

Use a new private `SM_NATIVE_PROOF_NAME` and matching operation root under the
chosen proof root, current `SM_NATIVE_FULL_SOURCE`, current sandbox/rustc,
`SM_NATIVE_FIXTURE_JOBS=4`, shared pipeline settings and cleanup reader.
Leave `SM_NATIVE_PERFORMANCE_ONLY` absent. Keep the complete declaration;
do not trim Fixtures, prepopulate `installed-runtime`, retry the old failed
operation key, or extend either deadline. The existing C timing/result files
appear after the first stage, so root can observe that stage without another
test-only entry point. If it passes, allow the complete control to finish.

If the first C stage reproduces, add bounded phase counters/timers at the
existing operation seam: metadata reservation; per-unique-source cache lookup;
content hashing; encoding; `write_once`/sealing. These should measure existing
work, not reread inputs. Keep any temporary instrumentation out of acceptance.
The present trace does not justify implementing a runtime optimization yet.

## Bounded follow-up design if the current gate reproduces

A small candidate is private invocation-owned metadata inventories inside
`operation.rs`: retain the metadata already collected by the operation's own
reservation and reuse it to form cache lookup keys. On a miss, independently
walk/hash the complete current tree and compare it to the reserved inventory;
on a hit, independently authenticate the retained record and validate the full
current metadata. Keep the runner's independent checks. This can remove the
extra lookup walk and duplicate metadata serialization without moving cold
hashing outside the original BOOTTIME deadline. Its benefit must be measured;
it cannot be assumed to eliminate the 45-second interval.

The inventory must be private to one invocation, bound to the exact Program,
and charged under existing entry/byte/metadata limits. Preserve complete file,
directory and symlink membership, source freshness, hash checks, reservations,
atomic import publication, pending-file recovery, operation result formats and
independent receipt admission. Do not replace fresh checks with metadata from
another invocation or a caller-supplied digest. Keep original failure/replay
semantics when a deadline expires mid-import.

Potential ownership is strictly
`crates/translation-execution/src/operation.rs` discovery helpers and
`crates/translation-execution/tests/operation.rs` (or a new focused test file).
No scheduler/graph, profile/startup, Ceres recipe, installed policy or deadline
edits are needed. Start from root's final assembly and preserve its admission
implementation. Existing deadline, reservation, cache mutation/symlink and
interrupted-operation controls remain mandatory; add a many-header real-runtime
case and partial-publication/retry controls before accepting changed discovery.

## Scope and cleanup

This task only read retained artifacts and source, and authored this handoff.
The diagnosis workflow's new reproduction/instrumentation phases were deferred
because root explicitly requested read-only work. No producer, build, budget,
runtime or installed file was changed. The original failed run's retained
cleanup logs report zero reclaimed bytes for both passes and its observed
failure was independently authenticated; this diagnosis created no run to clean.
