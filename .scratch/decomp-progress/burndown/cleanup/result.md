# Finished reporting run cleanup

No browser review or build process remains. The intended report server is
retained on `http://localhost:4185/index.html` (PID 420446 at inspection).

Both required maintained passes succeeded for `burndown` and
`compact-velocity`:

```sh
target/release/sm-pipeline cleanup --root .scratch/decomp-progress/burndown --apply
target/release/sm-pipeline cleanup --root .scratch/decomp-progress/burndown --deduplicate --apply
target/release/sm-pipeline cleanup --root .scratch/decomp-progress/compact-velocity --apply
target/release/sm-pipeline cleanup --root .scratch/decomp-progress/compact-velocity --deduplicate --apply
```

Each pass reported zero supported disposable bytes and no remaining targets.
`state-audit`'s previously completed passes also returned zero; their exact
stdout is preserved in the raw receipt archive.

Unsupported browser/build intermediates were handled by exact reviewed paths:

| Scope | Apparent bytes reclaimed |
| --- | ---: |
| Earlier compact viewer profiles and uploaded copy | 27,686,632 |
| New burndown browser profiles and uploaded copies | 86,364,265 |
| New disposable build, prototype executable and main prototype copy | 166,203,256 |
| Final disposable receipt/cleanup helper | 4,698,520 |

The new temporary Rust review target itself accounted for 157,367,199 bytes.
Earlier audit/compact cleanup additionally removed the 5,424,008-byte audit
executable and 140,329,014 apparent bytes of the compact reviewer build, as
recorded during that work. These earlier measurements are not reattributed to
the burndown redesign.

The throwaway prototype worktree was removed after commit
`6912d24a234097ea62ac7ca955b3f222707c5605` preserved its source and screenshots
on `throwaway/progress-burndown-layouts-20261013`. Main keeps the selected layout
and this branch pointer. The worktree's untracked files were copies of the
retained main reporting evidence; none was the sole retained evidence source.

Remaining free disk space measured after removal: **221,569,400,832 bytes**.
The later final helper removal and concurrent completed work left
223,380,353,024 bytes available at the final disk check; that increase is not
attributed wholly to this reporting task.
Exact new profile measurements and removal paths are in
[browser-profiles.json](browser-profiles.json) and
[disposable-outputs.md](disposable-outputs.md).

All 49 raw review/audit receipt files are retained byte-for-byte in
`../raw-review-receipts-20261013.tar.gz`, with member hashes in
`../raw-review-receipts.sha256` and archive hash in
`../raw-review-archive.sha256`. The archive SHA check passes. Original immutable
checkpoint archives were not overwritten. The original source audit, reviewed
mapping inputs, Ridley inline audit and pinned inventory all pass
`state-audit/preserved-inputs.sha256` unchanged.

Screenshots, fixtures, JSON assertions, reporting Rust source, locked dependency
manifests, registry observations, sealed fixture/live verdicts, retained evidence
readers and shared evidence stores remain. No gameplay source, run or acceptance
pointer was changed by cleanup.
