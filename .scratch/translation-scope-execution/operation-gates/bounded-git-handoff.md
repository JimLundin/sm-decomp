# Bounded trusted Git lifecycle and scoped source publication

Owner: `/root/bounded_git` in the replacement deterministic Rust session.
Status: implementation and focused verification complete; root owns final
checkpoint and managed pipeline integration. No production queue, authority,
progress, main checkout source or Git/index mutation by this worker.

The preexisting checkout/revision recovery implementation remains intact.
Owned changes are `crates/translation-execution/src/checkout.rs`,
`crates/translation-execution/tests/git.rs` and this handoff/evidence scope.

## Public interfaces

Existing wrappers remain supported, with a single 120-second operation cap.
The following variants accept `deadline_unix_ms: u64` as their final argument:

- `checkout_bytes_until`, `create_until`, `snapshot_until`
- `apply_commit_until`, `integrate_until`
- `retain_source_until`, `retain_revision_until`, `restore_revision_until`
- `authenticate_revision_until`

Every nested read, copy, archive/import and Git process inherits the earliest
deadline. A nested wrapper cannot reset the budget. Bounded pipe collectors
replace every unbounded Git `.output()` in this module. Read-only Git children
run without an inherited writer lease; mutating children inherit the exact
controller flock. The controller and `pre_exec` both check the inherited
monotonic expiry, and in-flight process groups are stopped at the same cap.
Source reads/writes and seals check the deadline between bounded blocks.
Successful operation return also checks the inherited deadline.

`read_only_paths_until(repository, argv, max_bytes, deadline)` returns raw
NUL-delimited bytes, including non-UTF8 names. It accepts only the fixed
affected-planning `diff --name-only --no-renames -z REV --` and two `ls-files`
queries. Revisions are bounded non-option selectors, preserving `HEAD` CLI
compatibility; the supervisor still passes its frozen full commit. The seam
inserts `--no-ext-diff`, disables optional index locking, and caps stdout at
4 MiB. Other Git commands and mutating flags are rejected.

`publish_revisions(repository, candidate, integrated, context, approved_paths,
publication_root, deadline) -> Result<Vec<Reference>>` wraps an empty controller
projection. `context.phase` must be `Publication`.

Managed integration uses:

```rust
pub struct ControllerProjection {
    pub paths: Vec<PathBuf>,
    pub references: Vec<retention::Reference>,
}

publish_revisions_with_controller_until(
    repository, candidate, integrated, context, approved_paths,
    publication_root, &controller_projection, deadline_unix_ms,
) -> Result<Vec<retention::Reference>>;

authenticate_shared_revision_until(
    repository, integrated, approved_paths,
    &controller_projection, deadline_unix_ms,
) -> Result<()>;
```

The managed caller holds the shared assignment mutation lock, validates current
role/claim ownership and semantically authenticates current controller refs
before each call. The exact controller path inventory is frozen in the source
publication intent. Each existing path needs a current exact reference;
predeclared absent future files are permitted. Changed current hashes are
accepted only for that same inventory. Paths are bounded regular files under
the repository's `.scratch`, disjoint from approved source and Git metadata.
Aliases, hidden source flags, foreign modes, missing hashes, broadened paths
and unrelated dirty files fail closed. The initial refs remain sealed in the
publication intent as historical identities; queue/result validators own the
current projection semantics. The index always retains the exact committed
metadata entries; controller changes stay in the worktree projection.

## Publication and recovery contract

Before publishing source, the function authenticates immutable archives and
approved ownership, requires the original attached branch, validates exact
baseline/index/source, and seals an intent. Raw Git objects are imported with
their original OIDs. Linear scoped commit history, exact object inventory,
source modes and complete committed SourceSnapshots are rechecked.

The function holds the actual shared `index.lock` via a hardlink to its exact
immutable marker. It reuses only the same marker inode after controller death;
a foreign or surviving index writer returns `ExecutionPending`. Mutating Git
children retain the outer publication flock while the shared index is locked.
Only approved files are copied, with old/new hash checks immediately before
replacement. An alternate index stages only approved paths, validates all
other entries, and replaces the shared index under its lock.

One `update-ref --stdin` transaction atomically CASes the shared branch from
the integrated baseline to the exact verified integration commit and retains:

```text
refs/translation-execution/<original-run-id>/attempt-<evaluation>/candidate
refs/translation-execution/<original-run-id>/attempt-<evaluation>/integrated
```

The shared branch remains attached; no reset or detach is used. Git 2.25.1
compatibility is covered (`--absolute-git-dir`, implicit stdin ref transaction).
Lost response and partial old/new source/index states recover from the exact
sealed intent. Completed retries authenticate instead of importing objects.
Publisher-owned controller paths are finite (`intent.json`, `result.json`,
`index-lock.marker`, zero-byte `lock`) with exact expected pending-file recovery;
the enclosing directory receives no blanket exception. Returned refs retain
intent, marker and result. The managed reader includes these current files and
the deterministic zero lock in its exact controller projection when needed.

The separate shared authenticator is read-only: it creates no lock, intent,
source/index/ref update, requires exact current HEAD, and authenticates the
immutable integration commit, exact index and source with the supplied current
controller projection.

## Completion evidence

Final commands used the shared workers4 environment and isolated
`/tmp/sm-translation-bounded-git-target`:

```sh
SM_PIPELINE_ROOT=/home/jim/sm-decomp/out/pipeline SM_PIPELINE_WORKERS=4 \
cargo test -p translation-execution --locked --offline \
  --target-dir /tmp/sm-translation-bounded-git-target \
  --test git --test checkout -- --nocapture --test-threads=1

SM_PIPELINE_ROOT=/home/jim/sm-decomp/out/pipeline SM_PIPELINE_WORKERS=4 \
cargo clippy -p translation-execution --locked --offline \
  --target-dir /tmp/sm-translation-bounded-git-target \
  --lib --test git --test checkout -- -D clippy::all
```

- `bounded-git/test-final-03.log`: 6 existing checkout tests and 14 Git tests
  pass. New tests cover expired cap before clone/source/index/ref mutations,
  archive reachability after producer retirement, exact branch CAS,
  lost-response and partial scoped-copy/index-lock recovery, foreign dirty
  source/HEAD/index writer refusal, original linked branch, raw NUL/non-UTF8
  path scans, tracked ledger hash evolution, predeclared future files, stale
  refs/inventory broadening/unowned metadata refusal, and interrupted own
  intent/result writes. Shared baseline remains unchanged until explicit
  publication.
- `bounded-git/clippy-final-04.log`: scoped `-D clippy::all` passes.
- `bounded-git/cleanup-ordinary-02.toml` and
  `bounded-git/cleanup-deduplicate-02.toml`: both required project cleanup
  passes succeed, zero eligible bytes. They use the retained maintained
  `native-gates/tools/sm-pipeline-cleanup`; no active writer remained.
- `bounded-git/build-cleanup-final.log`: final isolated target cleanup removed
  958 files / 441.0 MiB. Earlier scoped cleanup reclaimed 333.6 + 131.6 MiB
  before the final hardening/test run. The target is absent.
- Available disk bytes after cleanup: 188,981,653,504.
- `bounded-git/retained-sha256.txt` reauthenticates exact source and final
  compact logs/cleanup receipts after target retirement; all seven checks pass.

Fixture `Drop` removes only each exact isolated temporary repository. No test
producer Git checkout or binary is a retained evidence dependency. Root may
stage exact source/handoff/compact receipts; broad logs remain outside its
default Git inventory and need explicit selection if desired.

Remaining integration responsibility: root passes persisted whole-stage caps
through the new APIs and binds managed current controller projections to its
immutable original queue/result/progress evidence. This work provides a
source-publication mechanism, not production-switch authorization.
