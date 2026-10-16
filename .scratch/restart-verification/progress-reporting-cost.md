# Progress reporting: bounded cost finding

2026-10-16. Read-only follow-up to completed door milestone `ca64dfc`.
This is a future improvement candidate, not a gameplay prerequisite.

**Leading candidate:** each status or update command repeats a whole-registry
evidence-hashing pass and current source-history checks. Three separate stage
updates repeat that work three times and rewrite both report files. This is
supported by the current code; the reported 12.265-second status and 40.855-second
three-update timings do not identify which phase dominates.

## Code facts

- `progress::cli` routes `status` through `read_registry`, `check_current`,
  `observe_evidence`, then printing. `update` takes the same path after parsing
  the new observation and acquiring the registry lock, then persists JSON and
  HTML. `read_registry` parses/validates all stored routines and updates;
  `check_current` hashes the dependency-pin file and retained source inventory.
  Sources: [progress.rs](../../crates/pipeline/src/progress.rs), `cli` (line 332),
  `read_registry` (601), `check_current` (442), `update` (824).
- `observe_evidence` iterates all updates and hashes referenced evidence, including
  superseded updates. Its path-to-hash map avoids duplicate reads **within one
  call only**. A stale update may stop at its first bad pointer. It recomputes
  stage summaries and invokes `history::observe`. That function recursively
  reads/hashes `.rs` files under `crates/port/src`, then checks mapping inputs
  and retained pipeline timing/audit evidence when history exists. Sources:
  [progress.rs](../../crates/pipeline/src/progress.rs), `observe_evidence` (1076);
  [history.rs](../../crates/pipeline/src/progress/history.rs), `current_files`
  (169), `observe` (313).
- At this inspection the existing registry contained 6,053 routines, 287 updates,
  1,220 evidence references and 400 distinct evidence paths. It had source
  history with six input records, 46 recorded source-file records and three
  pipeline samples. Those are metadata counts, not measured I/O or timings.
  Registry size was 3,398,191 bytes, HTML 2,676,981 bytes, retained inventory
  3,256,608 bytes. Source: [registry.json](../decomp-progress/registry.json);
  counts obtained with read-only `jq`, sizes with `stat`.
- New evidence is hashed when constructing an update and rechecked under the
  lock; the global observation pass can read it again. `persist` renders the
  full embedded HTML snapshot, serializes the full registry and atomically
  writes/syncs both files, including on identical retries. Sources:
  [progress.rs](../../crates/pipeline/src/progress.rs), `input_update`, `update`,
  `persist` (423), `atomic_write`.

These paths do **not** call `load_inventory`, regenerate the C inventory, parse
all Fixture declarations, reconstruct Git coverage history, or invoke the
family/live acceptance authenticators. Those operations belong to other paths.
Calling this a fresh decompilation-verification run would misdescribe the code.
The recurring work here is report validation and file hashing. Without profiling,
its share of the reported time, evidence byte volume, filesystem effects and
possible update-lock contention remain unknown.

## Existing supported ways to avoid repeated inspection work

For routine reading, open the already-published
[index.html](../decomp-progress/index.html) or read
[registry.json](../decomp-progress/registry.json). The runbook explicitly supports
the embedded snapshot and loading a newer registry through its file picker.
That reads the last publication without freshly checking every evidence file.
Copying source identity from this registry is also the documented update workflow.
Source: [translation-progress.md](../../docs/agents/translation-progress.md),
“Update the shared inventory” and “Inspect progress.”

`progress update` already prints status, so an immediate extra `progress status`
is unnecessary when that update output answers the question. `progress serve`
is **not** a cached substitute: `respond` repeats registry/current/evidence
checks on every request. The public CLI accepts one `--update` file and exposes
no batch-update or snapshot-only status flag. The internal three-stage
`prepare_admitted`/`publish_admitted` APIs belong to trusted result admission;
they are not a supported generic route for publishing restart observations.
Sources: [progress.rs](../../crates/pipeline/src/progress.rs), `cli`, `options`,
`respond`, `prepare_admitted`, `publish_admitted`.

## Version and timing limits

Source findings above refer to main
`193cad941713b3539d23e08238a79e7fb09dde61`. The used executable was reportedly
`/home/jim/sm-decomp-production-execution/bin/sm-pipeline`.
Its [preparation receipt](/home/jim/sm-decomp-production-execution/receipt.json)
records 303,275,816 bytes and SHA-256
`4d870401e5465344b31f1085de6a9987365e44cf90381562a3f88c0c66f8898c`;
the [intent](/home/jim/sm-decomp-production-execution/intent.json) agrees.
These records identify the installation but do not establish an exact controller
source commit. The referenced preparation checkout currently at `808efe2` has
the same status/update/global-observation control flow; that is corroboration,
not proof of the installed binary's build inputs. No binary authentication,
rebuild or benchmark was performed.

The continuity owner subsequently supplied the exact retained timing pointers:
integrator turn `01a1443c-d9fb-79e3-a109-5f890c63ba86`, command
`exec-fcc732da-cca0-4c26-9fe4-f0075e321149` (`progress status`, durationMs 12265),
and `exec-d55cbfb8-1b03-4327-a227-7e0dbf43c1eb` (three updates, durationMs 40855).
These are observed command/group wall times reported by that owner, not phase
profiles. Our bounded thread read did not independently recover those items.
Stop at the repeated observation-pass candidate rather than opening implementation
or optimization work from this note.

## Cleanup

No builds, tests, benchmarks, captures, recurring processes or verification
artifacts were created. Only this note was written. Cleanup reclaimed 0 bytes; filesystem free
space at inspection was 106,139,090,944 bytes. No evidence dependencies added.
