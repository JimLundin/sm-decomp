# Fresh drill reproducibility corrections

Status: audit complete; implementation not started
Owner: preparation coordinator `01a13500-1ef8-73d3-9a0d-3446b6050eba`

Use a new versioned Rust harness/helper/store and a new isolated fixture root.
Preserve old `drill/`, `drill-final/` sources/seals and diagnostics as historical
subjects. The next run must bind the corrected retry engine's exact frozen
source, build, helpers and typed requests before launching independent sessions.

## Minimum changes before launch

1. Create fixture tickets after setup and before the first `next`: inherited
   setup authors scopes with explicit `.scratch/EFFORT/issues/{a,b,c}.md`
   references but no tickets. Create `a`, `b`, `c`, `overlap` with open status,
   no blockers, ready-for-agent and task type, using the Rust helper. Verify
   each scope's referenced ticket exists and its path/status matches. Existing
   `setup-repair.md` documents the initial safe `referenced ticket disappeared`
   failures; carry this required step into the successor recipe.
2. Generate typed integration Start with `handoff.submitted_commit` from the
   exact stored linked submission. `source_commit` is the separately frozen
   shared HEAD, baseline stays the original approved base and writable files
   stay exact submitted ownership. The inherited `base_handoff` omits the field;
   v2's reviewed `start-request` adds it, refreshes current request caller and
   `remember` preserves the returned handoff. Make this the normal successor
   path before Start rather than another planned rejection/repair.
3. Build/seal the actual reviewed helper versions and name their absolute
   binaries consistently in every prompt/recipe. Current recipe builds original
   `evidence-tools/`, but corrected commands live in `evidence-tools-v2/` at a
   separate target. Freeze one successor build manifest with source hashes,
   argv and binary hashes; preflight that each prompt command exists and its
   claim/token/generation arguments are separated and current. Use bounded waits
   of at most 50 seconds with explicit retry handling.
4. Prepare the duplicate-ticket control as a known ready candidate before
   retargeting it to B's held ticket. A newly discovered id naming an already
   managed claimed ticket is suppressed before deferral. Seed the exact control
   id with an open reviewed ticket and disjoint writable file under zero slots,
   then seal successor scope pointing to B's exact held ticket and run zero-slot
   pure-next. Assert no assignment, B claim unchanged/nonstale, disjoint candidate
   paths and exact `ticket is claimed`. Keep writable-overlap refusal a separate
   control. Current `b-controls-repair.md` retains the successful two-stage recipe.
5. After both integrations and implementer retirement, replay both original
   integrated updates at the final accumulated shared HEAD. Record unchanged
   admitted publication and gate artifact counts/hashes; repeat A after B to
   exercise the new defect directly. Source rollback or retrying only latest B
   would leave this behavior unproved. New execution evidence should include
   both completed replays, while all eleven existing controls remain required.

Root reviews helper changes before freezing them and starting new actual A/B
sessions. Helper adaptation has its own source/build/review provenance; earlier
engine source-equivalence or v2 review does not attest a future helper. Namespace
readiness uses a sealed successor with current root UUID and exact new drill,
then measured cleanup/retirement/reauthentication. Preparation docs stay pending
until the actual final receipts are complete.

This audit only read source and historical/current receipts and authored local
Markdown. It created no build, gate, fixture, harness or production mutation;
no-run cleanup reclaimed 0 bytes.
