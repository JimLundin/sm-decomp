# Put translation progress before report diagnostics

Status: resolved
Blocked by: None
Triage: ready-for-agent
Type: task
Owner: /root/coverage_finish (progress HTML and isolated layout review only)

## Required work

- Inspect the currently served report and the maintained HTML template.
- Compare three structural layouts with the existing report data: progress first,
  a diagnostics sidebar, and separate overview/details tabs.
- Prefer progress first with a collapsed Report details section unless the user
  chooses another layout. Keep every notice and stale-observation limitation
  accessible, and keep load/refresh errors visible.
- Preserve independent stage counts, filters, evidence details, offline loading
  and automatic registry refresh. Validate desktop and phone presentation.
- Publish the chosen template and generated local HTML, retain the layout
  comparison on a throwaway branch, and perform scoped run cleanup.

## Comments

2026-10-13: the currently published registry contains 149 notices and four stale
updates. The HTML renders all notices in an expanded warning block before any
progress metrics. The user is viewing http://localhost:4185/index.html and asks
for a better layout. An optional layout preference was requested while work
continues. The implementation is confined to progress presentation and does not
publish gameplay stages or alter the shared registry.

## Completion evidence

The maintained progress-first template and generated report are published.
Metrics and routine exploration precede collapsed Report details; every notice,
stale/corrected observation, provenance field and limit remains accessible.
Load/refresh errors stay outside the disclosure. Maintained viewer/server is
committed at `b8c3ffc`; context and immutable correction support at `66cde48`.

`../layout-comparison.md` compares progress first, diagnostics sidebar and
overview/details tabs using the same existing report snapshot. All three
desktop/phone variants pass browser behavior and overflow checks. Throwaway
branch `throwaway/progress-layout-comparison-20261013`, commit
`e574fef4fc1c5ee006cfc02d550dd815199adc00`, retains prototypes, comparison,
Rust generators/inspector, screenshots and checks; its base is `66cde48`.

Nineteen focused Rust progress tests pass in `../recovery-accuracy-final-tests.log`.
Final served and standalone checks and inspected desktop/390px phone screenshots
are in `../context-browser/accuracy-final-{served,offline}/`. They compare all
stage/context totals, preserve evidence hashes and correction history, and show
zero phone horizontal overflow. Earlier JSON picker/malformed-file visible error
checks remain at `../context-browser/file-picker/verification.json`.

Both project cleanup passes for isolated audit/layout roots succeeded with zero
utility candidates (`coverage-final-{audit,layout}-{cleanup,dedup}.toml`).
Disposable audit/layout Cargo targets were cleaned (289.3MiB / 146.4MiB);
26,482,694 identified unsealed audit/browser bytes and the 4,540,024-byte cleanup
executor were removed. Separate three-layout comparison cleanup reclaimed
28,094,108 bytes. Required screenshots/source/logs, verdicts, packed store,
source snapshots and readers remain. Measured remaining space:
231,865,810,944 bytes. Exact logs and final counts are recorded in
`../../decomp-strategy/coverage-audit.md` and `../coverage-final-*`.
