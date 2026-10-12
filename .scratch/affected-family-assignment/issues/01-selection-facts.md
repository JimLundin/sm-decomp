# Establish selection and assignment facts

Status: resolved
Blocked by: None
Triage: ready-for-agent
Type: research

## Answer

Three read-only agents examined current selection, dependency records and local
assignment conventions at `243a414`.

- `batch.rs:362–401` chooses profile/explicit family names and applies
  `git diff --name-only REV --` before loading previous batch records at
  `:414–443`. Tracked working-tree changes appear; untracked files and change
  kinds/rename identities do not. Broad fallbacks select every family for any
  tools or port-source edit.
- `inputs.rs:110–157` records compiler dependencies, declared extra paths and
  tool identities. `family.rs:578–615` retains these in implementation records;
  callback and PLM have equivalent dependency contracts. Generated run paths
  are omitted; external source paths may be absolute.
- `family.rs:895–984` authenticates complete retained inventory, current manifest,
  pinned archive and dependency/harness contents. A historical closure cannot
  substitute current passing admission. Authentication does not independently
  rederive every dependency-map entry from retained compiler.d.
- `inputs.rs:35–64` enumerates the whole module filename set and scans all
  identifier-like words, including comments and strings. New module stems can
  affect unchanged source discovery. A changed parent adding an import is
  already affected when that parent appears in prior compiler closure.
- Fixture before/after files and ROM assets are family inputs outside Rust
  compiler reads. Pinned source authentication checks every archive member.
- `batch.rs:535` returns all accumulated batch family records, not only this
  invocation's selected names. Tests must observe actual scheduled/skipped work.
- `tests/family.rs:533–588` offers small actual-C/Rust controls for reuse,
  changed-candidate rejection and independent batch locks. Selection itself
  can be exercised without a full-route replay.
- Tracker readiness requires open status and resolved blockers; done is also
  accepted historically. Triage is separate. Current required metadata lacks
  authoritative owner, write paths or cross-worktree atomic claims.
- Existing batch workers schedule selected families internally. Separate
  worktree/batch verification shares worker capacity/cache. A shared mutable
  source checkout can change generated #[path] inputs and invalidate evidence.
- Compiler dependencies indicate affected verification, not independent edit
  ownership. New translation needs observed exercise and coherent scoped work.

## Completion evidence

Read-only source investigation; no build, verification run, new worktree or
evidence-store write. Scoped run cleanup has no new run directory; bytes
reclaimed: zero. Historical evidence dependencies remain intact.

## Comments

Facts inform the design discussion, not authorization to weaken admission.
