# Independent review: energy-route bounded gap

Reviewer: `01a145c8-9bc8-78a2-b0f9-8519ad5f75f5`, 2026-10-16.
Disposition: **the four-attempt navigation gap is supported; the Terminator
energy milestone remains incomplete.** No fifth author run, successful strict
replay, new collector live stage or original-ROM claim follows from this review.

Evidence root: `/home/jim/sm-decomp-restart/restart/out/energy-tank/`.
Final candidate: `d2d6ddcf3dff7e0f68ad8352eb9ac6da2c6b842b`.
Review used retained artifacts and frozen source only; no producer was launched.

## Observed results and cost

| Author | Frozen candidate | Steps | Invocation wall seconds | Final normal Construction position |
| --- | --- | ---: | ---: | --- |
| 01 | `84dc09e90a45bcaa3769cb5f4ca0734704231e3d` | 41,216 | 42.998051807 | 130 / 409 |
| 02 | `67409e3d939fc2ecbee15b23cd15c3d5afb53af6` | 41,216 | 43.132416193 | 153 / 292 |
| 03 | `ec3943ac2b97c958e5cacb3a5669226eb9f0e4bb` | 26,155 | 29.307332687 | 164 / 316 |
| 04 | `d2d6ddcf3dff7e0f68ad8352eb9ac6da2c6b842b` | 26,176 | 27.609582270 | 198 / 297 |

Total invocation wall time is **143.047382957 seconds**, all failed authoring.
This excludes compilation, diagnosis, cleanup and publication. All four use the
declared 90-second deadline and 41,216-step ceiling. Distinct source/binary
identities and retained red regressions demonstrate changed attempts, not four
retries of an unchanged candidate. Policy logs retain 6/7/9/10 passing tests;
the corridor, ledge and clearance regressions retain their expected failures.
Those are observed existing logs, not tests rerun by this reviewer.

Raw input counts match every result. Hashing the first 24,832 input lines of
each run independently gives the accepted prefix
`43d67592cb3da2a9dd9c1568f609aa19b60bcb555a28b643453427677908be69`.
All four `energy-prefix.wram` files match accepted endpoint
`cb3362278173187dcda825bea8c4ec5c06952c1db7a5a948affb1c8d96a97483`.
Full input-log and energy-CSV hashes match their result records. CSV steps are
contiguous from 24,832 through each final step, including the initial observation.

Every result says `milestone-not-reached`, observer valid, health/max 99,
stage 1 and zero tank callbacks/new collector completions. Raw energy rows keep
presence/acquisition false. Only normal A107 and 9F11 have been reached in the
new tail; no energy pickup/message/resumption claim is supported. Existing
Morph/missile completion belongs to the preserved prefix.

The parent `invocation.toml` failure records retain zero completed-prefix/tail
summary fields. Actual completion counts come from child `result.toml`, raw
inputs and CSV. Do not read those parent failure defaults as an unconsumed
prefix or discard the useful failed-run evidence.

## Diagnosis is bounded to what was observed

The material report `evidence/author-04-gap.md` matches the inspected CSV events:
RIGHT+Jump at 25,126–25,128, feet above the row16 top at 25,148, left steering
at 25,149, and a grounded landing at 156/235 at 25,174. The next jump proceeds
right, falls to 198/315, and repeats vertical arcs with the first 243 peak at
25,247. Final state is 198/297 at 26,176.

The exact normal-room minimum Y is **220 at step 25,155**, versus **291 at
25,133** in author03. Frozen `energy.rs:375` updates `construction_best_y` only
after an improvement of at least eight pixels. Its 226/297 values are guard
checkpoints, not exact minima. The material report and attempts note now preserve
that distinction; no source change or new run is needed for this clarification.

Frozen `ledge_above` uses rounded row limits, two empty cells above a candidate
and distance scoring. It does not establish a traversable path through intervening
terrain. The report explicitly labels target152/128 as a source reconstruction:
the CSV does not record `climb_target`. The final retained WRAM has room width16
and row13/column12 word `81e9`, consistent with the described right-side ceiling
pocket. Keep the report's distinction between observed obstruction and inferred
target selection. These failures establish this policy's remaining upper-approach
and recovery gap; they do not prove vanilla traversal impossible.

## Retention and cleanup

All eight candidate source-archive/binary manifest entries pass. Each binary hash
matches its invocation. The three declared input/native dependencies pass. Final
source and WRAM match the material report. Key reviewed identities:

| Artifact under evidence root | SHA-256 |
| --- | --- |
| `author-04/result.toml` | `e19e9d9628ad858cdb4a21928d654676bce374939b183ee99646710e185d5549` |
| `author-04/energy-observations.csv` | `202accfa5e3fb22bfd96f75aa45d219352b927ed7f0b2ef46d690a70573f3388` |
| `author-04/final.wram` | `f89b4f40331d4d061e43bd8b65633259969fee412becafa6856ea4d9a053e219` |
| `author-04/source/energy.rs` | `2ca9cc0df5cfc44665a0a67355640af9cd3cc3aa006160d61724ff88429441b7` |
| `evidence/author-04-gap.md` | `fe48c2e1099715de3526bc133cb4fac15bae4ecf32ea38b3a0f243902e76f5af` |

Both cleanup utility passes for each finished author report zero disposable
bytes. Producer retirement records reduce the two energy worker directories
from 444,354,560 to 172,032 allocated bytes: **444,182,528 bytes reclaimed**.
They are absent from `git worktree list` and contain no `.git`; retained
`energy.rs` and `runtime_cli.rs` exactly match their `worker-source/` copies.
Available disk in the producer's after record: **99,362,590,720 bytes**.

After retirement, the final candidate source/binary, three input/native
dependencies, final result and material report hashes still pass. Preserve all
four failures, source snapshots, binary/native dependencies and private ROM
dependencies. Root owns the final encompassing seal after copying this review;
this report does not claim an as-yet-unpublished full-directory manifest passed.

This review generated only this Markdown and local tracking updates. No build,
gameplay, verification process, temporary executable or run directory was created;
reviewer cleanup is 0 bytes. The review is complete for the bounded failed-attempt
handoff. Producer22's gameplay objective remains open, while the independent
SRAM codec and prepared player-motion work retain their existing owners.
