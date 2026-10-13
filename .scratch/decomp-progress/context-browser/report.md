# Progress context and report layout verification

Status: passed; final publication and scoped cleanup complete.

The maintained viewer now filters observations by Checkout at observation,
Retained historical source, and Unspecified / legacy. All reported contexts is
the default. Exact stages remain independent, and overlapping context counts
remain visible. Historical observations are eligible only against the matching
pinned C inventory and intact evidence references; the label does not assert
that later Rust edits have been authenticated.

The progress metrics and source banks appear before diagnostic notices. Report
details starts collapsed at the bottom and retains every notice, provenance
field and interpretation limit. Its summary displays the notice count. Load and
refresh errors stay visible above the metrics.

Sixteen focused Rust progress tests pass, including legacy serialization and
idempotent retries, conflicting context changes, overlapping context counts,
changed-evidence exclusion, concurrent publication, and HTTP contracts. Existing
six-writer and earlier full browser results remain in recovery-browser/report.md;
no new concurrency stress or gameplay replay was launched for this change.

Real Chromium CDP checks passed for both standalone embedded HTML and the served
report. They compare all four achieved stage totals across all three contexts,
check that checkout labels refer to observation time, retain exact evidence
hashes in expanded routine details, preserve all notices in the collapsed
section, and keep errors outside it. File picker loading and malformed JSON
visible-error checks also pass. Desktop and 390-pixel phone screenshots were
inspected; phone document width equals the viewport with no horizontal overflow.
Retained outputs are checks.json, inspection.json, served/* and file-picker/*.

Two initial assertion failures were verifier-only: the notice total also
includes the extra stale-observation notice, and a details toggle event needs
one browser event turn before inspecting its lazy content. A file-picker attempt
without a running local server also failed; the subsequent served attempt
passed. All diagnostics remain in recovery-context-*.log.

The final accuracy snapshot has 256 observations and counts 1,460 translated,
924 verified, 38 integrated and 121 live-exercised native definitions. Nineteen
focused progress tests now pass, including immutable supersession controls.
Fresh served and standalone browser checks also exclude superseded claims while
retaining their visible history and reviewed correction. Retained evidence is
in accuracy-final-{served,offline}/; both desktop and phone screenshots were
inspected, with phone document width exactly 390 pixels.

Both scoped project cleanup passes and disposable Cargo build cleanup completed.
26,482,694 identified unsealed browser/audit bytes were removed after copying
final screenshots/checks/logs; audit/layout Cargo clean removed 289.3MiB/146.4MiB.
The cleanup executor was removed after use. `../coverage-final-*` and
`../../decomp-strategy/coverage-audit.md` retain commands, measurements, final
source/evidence checks and remaining dependencies.
