# Planning evidence and handoff

Date: 2026-10-11
Status: planning complete; implementation tickets 01–06 remain open.
Integration checkout: `decomp-port-integration`.
Protected optimization ref: `e820a6b77338eb4e84a2d91baeed2dc9f33c72f1`.

## Grounding

The proposal reuses the committed optimization infrastructure documented in
[the comparison](../pipeline-review/optimization-comparison.md) and
[measurement audit](../pipeline-review/optimization-measurements.md). The
[source index](../pipeline-review/optimization-source-index.json) pins inspected
optimization objects. Read committed optimization objects from the decomp
checkout; no physical optimization worktree, source, run or cache was accessed.

The design was checked against `crates/evidence/src/trace.rs` (`Event`, rendering,
`TraceWriter::push`, `ingest_trace`, inferred frames), evidence CLI capture/import
and verified inventories, `tools/port_evidence.py` capture/publication, Rust
worker leases and current decomp generated host emitters in `tools/port_live.py`.
Their seams are named in [plan.md](plan.md). Files marked new are proposed.

Previous numerical results are historical/reported capture measurements; they
exclude complete publication and use uncontrolled load. They establish no speedup
for this proposed binary transport. The 15% wall/5% resource thresholds are
proposed adoption criteria. Current decomp chain/WithGfx reconciliation precedes
measurements that could justify promotion; original-ROM accuracy stays separate.

## Independent design review

The visual-plan skill's skeptical-review pass was performed by
`capture_design_check`. All six concrete findings were incorporated into the plan,
relevant tickets and visual source:

- Shared scanner state governs typed/literal interleaving, long-line continuation
  and END residuals; tests require stream/segment identities as well as bytes.
- Durable subject pointers follow successful capture and reopened authentication;
  accepted live verdicts follow comparison/admission. Failed comparisons retain
  sealed divergent evidence. Release follows an authenticated retained replacement.
- Collector SHA authenticates decoded/stored bytes; independent text/binary
  equivalence controls shared encoder/decoder mistakes within tested coverage.
- One lease covers a producer/collector pair; accounting helpers never reacquire
  that lease. Both processes count toward measured capacity costs.
- Promotion thresholds are explicit, with frozen absolute host budgets and
  combined-source benchmark requirements. Exceptions need a recorded owner decision.
- Metrics-only is an alternative that replaces tickets 02–06 with deferred scope.

## Artifact verification

- Fetched the public no-auth block catalog before authoring MDX; no plan content
  was sent for catalog lookup.
- `npx --yes @agent-native/core@latest plan local check --dir
  .scratch/pipeline-upgrade/visual`: offline lint passed on final source. This is
  not full renderer-schema validation.
- `plan local verify --dir .scratch/pipeline-upgrade/visual --kind plan`: private
  network preflight returned 204 and bridge payload returned 200 with `localOnly`.
  Remote validation was skipped to keep the source local.
- Chromium/Playwright opened the local bridge in the Plan UI, clicked its local
  connection control, and verified the complete document, diagram, annotated code,
  scope form and updated acceptance thresholds. No invalid-block placeholders
  remained. Visually inspected the rendered architecture diagram at 1440×1000.
  The catalog's fenced HTML/CSS Diagram shape was required for correct rendering.
- The bridge is an intentional local review dependency. `.plan-url` is ignored
  and is excluded from the documentation commit. Its URL works on this machine
  while that bridge remains active; the MDX folder and Markdown remain reusable.
- Final checks cover local Markdown citations, ticket dependency ordering,
  Markdown/visual content agreement, whitespace and the protected optimization ref.

No pipeline implementation, replay, merge, build or performance benchmark was
performed. These checks validate the planning artifact, not transport correctness
or simulation accuracy.

## Completion evidence and cleanup

No port run/batch directories were created, so there is no new scope for
`port_cleanup.py` or live-evidence reauthentication. Run bytes reclaimed: 0.
Removed only this task's temporary block catalog, schema catalog and browser
screenshot after recording verification: 264,139 logical bytes. No implementation
worktrees were created. Existing run evidence, readers, sources, shared stores,
optimization worktree and branch were preserved. The review bridge remains active.

Remaining filesystem space after cleanup (`df -B1 .`): 252,176,052,224 bytes
available (about 234.9 GiB). Implementation remains proposed under the six open tickets; the committed
planning folder is the handoff artifact.
