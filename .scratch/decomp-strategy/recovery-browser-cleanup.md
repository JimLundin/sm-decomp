# Recovered browser probe cleanup

Status: resolved
Blocked by: None
Triage: ready-for-agent
Type: task
Owner: `/root/recovery_consistency`
Date: 2026-10-13

The six prior progress-context browser probes and the standalone Chromium probe
have finished cleanup. Their verification JSON, screenshots and diagnostic logs
are retained; only identified unsealed Chromium profiles/cache were removed.
This was tooling cleanup, with no gameplay verification or shared-source change.

## Completion evidence

`recovery-browser-cleanup.rs` is the retained Rust cleanup coordinator. It was
compiled with `rustc --edition=2021` to `/tmp/sm-recovery-browser-cleanup` and
executed once. Before copying or removal, it inspected every readable `/proc`
command line, working directory, process root and open file descriptor. No
process referenced any selected probe root. It repeated that check immediately
before deleting the profiles. The preflight result is retained in
`recovery-browser-evidence/preflight-processes.txt`.

All 21 top-level evidence files from the six context probes were copied to
`recovery-browser-evidence/<original-probe-name>/` and checked byte-for-byte by
SHA256 before deletion. Original top-level files remain at their `/tmp` paths;
their profile directories are absent. The complete source/destination/size/hash
mapping is `recovery-browser-evidence/retained-artifacts.tsv`; a reusable digest
manifest is `recovery-browser-evidence/artifact-hashes.sha256`. Every retained
file passes the post-cleanup check recorded in
`post-cleanup-artifact-authentication.txt`.

| Probe | Retained top-level artifacts | Profile apparent bytes removed |
| --- | --- | ---: |
| `sm-progress-context-file-check` | `chromium.log` | 4,141,785 |
| `sm-progress-context-file-check-v2` | `verification.json`, `opened.json`, `malformed.json`, `chromium.log` | 4,420,882 |
| `sm-progress-context-inspection` | `desktop.png`, `chromium.log` | 4,117,184 |
| `sm-progress-context-inspection-v2` | `desktop.png`, `chromium.log` | 4,134,636 |
| `sm-progress-context-inspection-v3` | `checks.json`, `inspection.json`, `desktop.png`, `details.png`, `phone.png`, `chromium.log` | 4,129,520 |
| `sm-progress-context-served-inspection` | `checks.json`, `inspection.json`, `desktop.png`, `details.png`, `phone.png`, `chromium.log` | 4,184,365 |
| `sm-progress-chrome-probe` | Profile-only preflight directory listing retained | 4,089,380 |

The existing `.scratch/decomp-progress/recovery-browser/` evidence remains at
its original path. Its seven recorded artifact hashes pass before and after
this cleanup, including `verification.json`, desktop/phone screenshots,
synthetic JSON/HTML, Rust helper source and pinned inventory. That directory
also retains the historical report, server/Chromium logs, 24-writer failure
diagnostic, helper Cargo manifest/lockfile and ten original cleanup receipts.
Historical `source-hashes.sha256` describes the older verification source and
binary; it was preserved as historical identity rather than compared to the
changed current checkout.

Both required project cleanup passes succeeded for each of the seven selected
roots using the existing compiled `target/debug/sm-pipeline`:

```sh
target/debug/sm-pipeline cleanup --root /tmp/<explicit-probe-root> --apply
target/debug/sm-pipeline cleanup --root /tmp/<explicit-probe-root> --deduplicate --apply
```

The exact fourteen commands, stdout receipts and stderr files are retained in
`recovery-browser-evidence/cleanup-commands.txt` and `cleanup/`. Every receipt
reports zero bytes, because these browser profiles are outside the utility's
recognized gameplay run formats. The Rust helper then removed only each
explicit `chromium-profile` directory and the profile-only
`/tmp/sm-progress-chrome-probe`. The detailed removal receipt is
`recovery-browser-evidence/removed-profiles.tsv`: 29,217,752 apparent bytes and
25,387,008 allocated bytes across 1,620 entries. No sealed artifact, verdict,
acceptance pointer or evidence reader was in those profiles.

After successful cleanup the 4,869,120-byte disposable Rust executable was
removed and its absence checked, retaining only the source. Total identified
apparent bytes removed by this task are therefore 34,086,872. The final disk
measurement after removing that executable was 231,303,987,200 available bytes;
the profile-cleanup before/after measurements remain under the evidence root.

Current viewer/layout-review paths, shared Cargo target, registry, active
coverage publisher tools/build outputs and gameplay evidence stores were not
touched. No implementer worktree was created. Original evidence paths remain
available, retained copies authenticate, all selected disposable profiles are
absent, and no cleanup action remains for this lane.
