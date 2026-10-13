# Coverage and layout finish handoff

Status: complete; coordinator commits retained reporting evidence as appropriate
Date: 2026-10-13
Owner: /root/coverage_finish

Maintained viewer/server was committed at `b8c3ffc`; contexts and permanent
immutable corrections are committed at `66cde488fec48a92d28c9cf1f5394e5ab1292f1e`.
The three maintained progress files are clean. No further source edits are
pending from this agent. The intentional server is PID 193727 at localhost:4185.

Final registry: 6,053 definitions / 75,229 body lines, 834 declared native roots,
256 observations, 16 stale, one superseded, 165 registry notices. Independent
stages: translated1,460 / verified924 / integrated38 / live121 / translating220.
Current-context stages:1,460/38/38/26; historical:911/911/0/106; legacy:34/34/34/0.
Eleven internal live symbols overlap current and historical contexts; combined
live union is121. Duplicate reviewed Ridley and internal-live records are
retained immutable observations; unique symbol counts are unaffected.

Read `.scratch/decomp-strategy/coverage-audit.md` for audited scope, exact
interpretation, exclusions, unknowns and cleanup. All141 tickets,44 maintained
Rust modules and175 manifest declarations were reconciled.118 selected retained
Offline runs were authenticated read-only; additional sparks artifact checks
passed.16 accepted historical live runs and11 measured composed internals are
published with their limits. Current integration remains the38 explicit roots.
Whole Ridley live acceptance and credits/original-ROM parity remain open.

Ticket02 and progressspec are resolved. The three-layout comparison is retained
on `throwaway/progress-layout-comparison-20261013`, commit
`e574fef4fc1c5ee006cfc02d550dd815199adc00`, based at66cde48. Chosen progress-first
template preserves collapsed Report details, visible errors and independent
counts. Nineteen focused progress tests pass. Fresh served and standalone
Chromium checks pass across every stage/context, visible correction history,
evidence details and desktop/390px phone presentation; screenshots inspected.

Both scoped utility cleanup passes succeeded for audit,layout,browser and
comparison roots (zero candidates). Cargo clean removed289.3MiB audit and146.4MiB
layout builds. Rust explicit deletion removed26,482,694 unsealed file bytes and
then the4,540,024-byte cleanup executor. Independent layout comparison cleanup
removed28,094,108bytes. Remaining space231,865,810,944bytes. Final CLI status and
immutable audit hashes pass aftercleanup. No gameplay run was launched.

Suggested explicit review/staging set (do not recursively add .scratch):

- `.scratch/decomp-strategy/coverage-audit.md`
- `.scratch/decomp-strategy/source-coverage-audit.md`
- `.scratch/decomp-strategy/source-coverage-map.json`
- `.scratch/decomp-strategy/retained-coverage-authentication.json`
- `.scratch/decomp-strategy/retained-live-coverage-audit.json`
- `.scratch/decomp-strategy/ticket-provenance-note.md`
- `.scratch/decomp-strategy/ticket-authenticated-internal-mappings.json`
- `.scratch/decomp-strategy/live-internal-reconciliation.md`
- `.scratch/decomp-strategy/ridley-inlined-source-alias-review.md`
- `.scratch/decomp-progress/spec.md`
- `.scratch/decomp-progress/issues/02-report-layout.md`
- `.scratch/decomp-progress/layout-comparison.md`
- `.scratch/decomp-progress/layout-comparison/branch.txt`
- `.scratch/decomp-progress/context-browser/report.md`
- `.scratch/decomp-progress/context-layout-checks.js`
- `.scratch/decomp-progress/coverage-final-evidence.sha256`
- `.scratch/decomp-progress/coverage-final-after-cleanup-authentication.log`
- `.scratch/decomp-progress/coverage-final-status.log`
- `.scratch/decomp-progress/coverage-final-{audit,layout,browser,comparison}-{cleanup,dedup}.toml`
- `.scratch/decomp-progress/coverage-final-{cargo,layout-cargo}-clean.stderr`
- `.scratch/decomp-progress/coverage-final-remove-intermediates.{rs,log}`
- `.scratch/decomp-progress/sparks-retained-coverage-authentication.json`
- `.scratch/decomp-progress/ticket-authenticated-internal-mappings-published.json`
- `.scratch/decomp-progress/ridley-inline-translation-audit.md`
- `.scratch/decomp-progress/recovery-accuracy-final-{tests,build,browser,offline-browser}.{log,stderr}`
- `.scratch/decomp-progress/context-browser/accuracy-final-{served,offline}/{checks,inspection}.json`

Publication helpers, individual TOML observations/stdout/stderr, final registry,
standalone HTML, screenshots, other audit JSON and compile logs remain locally
reviewable. The retained throwaway branch contains full comparison/prototype
browser evidence. Exact source snapshots, packed object stores, verdicts and
readers were preserved; no unbounded evidence staging is needed.
