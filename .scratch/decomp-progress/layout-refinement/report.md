# Progress report layout refinement

Status: complete
Blocked by: None
Date: 2026-10-13
Ticket: [03](../issues/03-report-details-refinement.md)

The user selected progress first with collapsed report details. The maintained
HTML now uses a shorter heading and introduction, shows metrics first, and puts
Report details immediately after the metrics. Its summary reports the note count
and stale/corrected exclusions. Expanding it reveals four separate groups:
coverage and interpretation, fixture declarations, excluded stale reports, and
replaced reports. Long lists are bounded to 300 pixels and keyboard scrollable.
Source provenance remains in its own disclosure. Actual load/refresh errors stay
above the metrics.

All 165 exact registry notices remain available, including 142 fixture declaration
notes. Report counting, evidence contexts, independent stages, stale and immutable
correction exclusions, routine evidence and registry publication were preserved.
No gameplay observations were published by this layout task. The previous
three-layout comparison and throwaway branch remain recorded in ticket 02.

Rust CDP browser assertions pass in `served-final/checks.json` and
`offline-final/checks.json`. They compare all five stage totals in the combined
view and every evidence context, remaining counts, search, expanded SHA-256
evidence and correction history, exact notice retention, bounded keyboard
scrolling and visible errors. Live checks also preserve filters and open details
through a seven-second registry check. File selection, literal notice text and
visible malformed-file errors pass in `file-picker-final/verification.json`.
An initial file-picker assertion timed out because the inspector supplied a
relative file path; canonicalizing its fixture directory corrected the inspector.
The failed browser log and malformed fixture remain retained.

Screenshots were visually inspected at 1,360 × 1,000 desktop and 390 × 844 phone
sizes, including light and dark themes and collapsed/expanded details. Desktop
metrics start at 228 pixels and the offline phone metrics at 311 pixels; phone
document width is exactly 390 pixels. Final screenshots, inspection/check JSON
and browser logs remain under `served-final/` and `offline-final/`. Earlier
iterations are retained under `served/` and `offline/`.

`cargo build -p pipeline --locked --offline --bin sm-pipeline` succeeds. The
maintained server at http://localhost:4185/index.html serves the revised template
from `target/debug/sm-pipeline`, intentional process PID 259921 / session 70423.
The generated `.scratch/decomp-progress/index.html` was atomically republished
with the retained Rust renderer. The report was opened in the default Windows
browser with PowerShell Start-Process after verification.

All verification Chromium processes exited before cleanup. Both project cleanup
passes on this directory and the isolated Cargo target succeeded with zero
supported disposable bytes; exact receipts are in `cleanup/project-passes.toml`.
The scoped Rust cleanup removed six new browser profiles and two copied upload
fixtures, reclaiming 30,937,964 measured apparent bytes; its exact receipt is
`cleanup/browser-profiles.json`. `cargo clean` removed the isolated review build
(571 files, 146.9 MiB reported; pre-clean apparent size 139,404,875 bytes). Final
measured available filesystem space was 224,959,778,816 bytes. No implementer
worktree, gameplay evidence, source snapshot, packed store or evidence reader was
removed. Rust reviewer/assertions, lockfile, screenshots, logs and embedded
snapshot remain. Main-checkout edits and Git mutations were handed to the user's
checkpoint owner after these completion records were written.

The snapshot hash list now records the refreshed HTML and the shared registry as
found at completion; the registry contains 257 observations after independent
work. Prior 256-observation receipts retain their observation-time contents.
The HTML refinement does not explain or authenticate new gameplay coverage.
