# Progress report layout comparison

Status: completed comparison; selected template publication belongs to coordinator
Triage: ready-for-agent
Ticket: `.scratch/decomp-progress/issues/02-report-layout.md`
Date: 2026-10-13

Prefer **progress first with collapsed Report details**. It shows the stage totals and bank grid promptly, preserves a wide routine explorer, and keeps diagnostics accessible without making their length the page’s initial focus. The maintained template already implements this choice; this task only completed the missing comparison and retained prototypes.

Three structural alternatives use the existing viewer and the same embedded registry snapshot. Open `layout-comparison/prototype.html` locally and switch with the floating arrows, keyboard arrows, or `?variant=progress`, `?variant=sidebar`, and `?variant=tabs`. The production report is not replaced by these throwaway variants.

| Layout | Structure | Desktop assessment | Phone assessment | Decision |
|---|---|---|---|---|
| Progress first | Full-width metrics, bank grid, and routine explorer; collapsed Report details below | Fast scan; longest space devoted to progress; all 165 rendered diagnostic notices remain available in one disclosure | Two-column metric cards, readable bank grid; no horizontal overflow | Selected |
| Diagnostics sidebar | Progress and explorer on left; open, scrollable diagnostics column on right; stacks and collapses below 1,051 px | Diagnostic context is continually available, but its long text competes with the bank grid and forces metric labels to wrap | Stacks into one column, so the sidebar advantage disappears; no horizontal overflow | Useful for investigation, weak default |
| Overview/details tabs | Overview contains progress and explorer; Report details occupies a separate full-width tab; errors remain outside tabs | Clear diagnostic workspace, but adds navigation and hides progress while reading details | Tab strip adds height before the metrics; no horizontal overflow | More interaction than the page needs |

All variants preserve the existing script and data model. Browser checks confirmed independent translated/verified/integrated/live-exercised counts across all three named evidence contexts and the combined view, filtering/search, routine evidence contexts and SHA-256 references, and every diagnostic notice. Errors sit outside disclosures and tab panels. The offline prototype embeds the exact snapshot; the served prototype keeps the existing seven-second `registry.json` refresh path. File-picker and malformed-JSON checks were already completed in `context-browser/file-picker/verification.json` and were not rerun for these structure-only variants.

The snapshot contains 6,053 native definitions, 217 observations, 164 registry notices, and 16 stale observation IDs. The viewer adds one stale-observation diagnostic, yielding 165 rendered notices. The snapshot SHA-256 is `5bc75ca6f0741c2382d99019e46838aee1d9ee137e3e6eea255991df690894e0`; the embedded prototype SHA-256 is `1e7ed171901bbc330c82480b2c19ad4a6e6b53b7b4d388fc66e80c5d20134b68`. A snapshot records report state at comparison time; it is not a fresh gameplay authentication.

Chromium inspected 1,360 × 1,000 desktop and 390 × 844 phone viewports. Measurements include the throwaway banner/switcher, so they describe relative prototype hierarchy, not the production layout’s absolute offsets:

| Layout | Desktop metrics top | Phone metrics top | Phone document width |
|---|---:|---:|---:|
| Progress first | 314 px | 484 px | 390 px |
| Sidebar | 336 px | 506 px | 390 px |
| Tabs | 377 px | 547 px | 390 px |

Screenshots were visually inspected: progress-first and sidebar desktop, and tabs phone. Existing maintained-template checks provide the baseline without prototype chrome: 258 px desktop metrics top and 410 px phone metrics top, with phone document width exactly 390 px. Its retained checks already cover notices, context labels, evidence details, and visible errors.

Evidence pointers:

- `layout-comparison/{progress,sidebar,tabs,served}/inspection.json`: viewport measurements and route.
- `layout-comparison/{progress,sidebar,tabs,served}/checks.json`: successful structural/data behavior checks.
- `layout-comparison/{progress,sidebar,tabs,served}/{desktop,phone,details}.png`: retained screenshots.
- `layout-comparison/prototype.html.in` and `prototype.html`: switchable variants, current viewer behavior and embedded snapshot.
- `layout-comparison/generate.rs`: std-only Rust prototype generator; the checks expression is generated from Rust and runs in Chromium.
- `layout-comparison/reviewer/`: retained Rust CDP inspector source and lockfile copied from the existing layout reviewer.
- `context-browser/report.md`, `context-browser/served/{inspection,checks}.json`, and `context-browser/file-picker/verification.json`: existing selected-layout validation.

The maintained template SHA-256 was `8083b62b808cf59a305422d57e1a98db2403eda70f59804af4dfac1f22939c78` before and after this task. This task did not edit it, the issue ticket, shared registry, or generated maintained `index.html`. No gameplay replay, fixture verification, or cargo build was performed. The existing compiled Rust CDP inspector was reused; the only new compiled executable was the disposable std-only generator.

Cleanup completed after all Chromium instances finished and the temporary preview server at 127.0.0.1:4191 was stopped. The four new Chromium profiles were identified as unsealed browser intermediates and removed; their measured sizes totaled 16,584,837 bytes. The 4,567,752-byte generator executable and 2,303,031-byte temporary fetched registry were also removed, for 23,455,620 identified bytes reclaimed. Screenshots, logs, checks, embedded data and Rust source remain. Available filesystem space after cleanup was 231,290,871,808 bytes (about 215 GiB). No game-run cleanup format or evidence reauthentication applied to these browser artifacts. Existing reviewer builds and other agents’ files were preserved.

The retained throwaway branch and commit are recorded in `layout-comparison/branch.txt` after capture. The branch contains this comparison, prototype variants, inspector/generator source, and browser evidence only. The coordinator can use this pointer in ticket completion evidence and commit the selected maintained template separately.

The temporary capture executable was also removed (4638488 bytes); total identified cleanup is 28094108 bytes. The temporary capture index is removed immediately after branch creation.
