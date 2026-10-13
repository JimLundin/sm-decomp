# Reporting run cleanup

Both Rust pipeline cleanup passes succeeded with zero eligible bytes and zero snapshots.
This report/browser run has no gameplay verdict; only these identified unsealed outputs are removed.

| Path | Logical file bytes removed |
| --- | ---: |
| `/home/jim/sm-decomp/.scratch/decomp-progress/latest-submissions/pipeline-target` | 1918771855 |
| `/home/jim/sm-decomp/.scratch/decomp-progress/latest-submissions/reviewer-target` | 155647498 |
| `/home/jim/sm-decomp/.scratch/decomp-progress/latest-submissions/browser-refresh/chromium-profile` | 4081146 |
| `/home/jim/sm-decomp/.scratch/decomp-progress/latest-submissions/browser-served/chromium-profile` | 4090969 |
| `/home/jim/sm-decomp/.scratch/decomp-progress/latest-submissions/browser-served-final/chromium-profile` | 4155348 |
| `/home/jim/sm-decomp/.scratch/decomp-progress/latest-submissions/browser-preview/chromium-profile` | 3901994 |
| `/home/jim/sm-decomp/.scratch/decomp-progress/latest-submissions/browser-preview-final/chromium-profile` | 3874024 |
| `/home/jim/sm-decomp/.scratch/decomp-progress/latest-submissions/browser-refresh-polled/chromium-profile` | 4155388 |

Total logical file bytes removed: 2098678222.

Retained: exact source, browser captures/checks and failure diagnostics, Rust test/publication logs, registry/HTML snapshots, Rust reviewer source/lockfile and the exact sm-evidence reader. The intended progress server PID803313 and its sm-pipeline binary remain available. Shared out/evidence packs are preserved.

Remaining disk space at cleanup:

```text
       Avail
172619423744
```
