# Reporting WIP receipt checkpoint, 2026-10-13

Status: completed WIP receipt preservation; reporting checks remain in progress
Owner: `/root/checkpoint_review`
Reporting owner: session `01a13394-29b8-76a3-aab7-5ab8eff60694`

The report owner declared a coherent WIP source and published-audit boundary.
Tickets 04 and 05 remain claimed while final browser checks and cleanup finish.
This checkpoint preserves completed report metadata and raw command receipts
with their original bytes and whitespace; it grants no gameplay acceptance or
completed reporting verification.

The archive contains three generated metadata files at their exact original
repository-relative paths: `registry-before.json`, `source-coverage-map.json`
and `source-audit-review.txt` under this directory. It also retains the completed
source audit, refresh, publication and retry stdout files enumerated in the hash
manifest, plus the completed
`.scratch/decomp-progress/compact-velocity/offline/chromium.log`.
The empty first natural-observation stdout is preserved alongside its completed
retry. No active served/synthetic output, browser profile, build, executable,
ROM or gameplay state capture is included. Originals remain in place.

The main Git owner explicitly force-added these reviewed authored TSV inputs
because the audit Rust command reads their exact paths:

- `.scratch/decomp-progress/state-audit/source-aliases.tsv`
- `.scratch/decomp-progress/state-audit/source-numeric-scopes.tsv`
- `.scratch/decomp-progress/state-audit/source-forwarders.tsv`
- `.scratch/decomp-progress/state-audit/source-module-roles.tsv`

They are source inputs, separate from the archived generated receipts.

## Restore and authenticate

The sibling `checkpoint-receipts-20261013.sha256` records the archive hash and
every member hash using original repository-relative paths. From the repository
root, verify the archive before restoring any files:

```sh
rg '  \.scratch/decomp-progress/state-audit/checkpoint-receipts-20261013\.tar\.gz$' .scratch/decomp-progress/state-audit/checkpoint-receipts-20261013.sha256 | sha256sum -c -
tar -tzf .scratch/decomp-progress/state-audit/checkpoint-receipts-20261013.tar.gz
tar -xzf .scratch/decomp-progress/state-audit/checkpoint-receipts-20261013.tar.gz -C /path/to/restore-root
```

The restore root must be the intended checkout root, with no active writer at
the selected original paths. For a private extracted copy, exclude the archive
line and check member hashes from that restore root. The complete manifest can
also verify the archive and unchanged originals from the current main checkout:

```sh
sha256sum -c .scratch/decomp-progress/state-audit/checkpoint-receipts-20261013.sha256
```

## Completion evidence

`checkpoint-receipts-20261013.tar.gz` retains **13 regular files** totaling
**4,304,815 original bytes**, compressed to **671,044 bytes**. Archive SHA-256:
`12e4a070c49e394d1f37f9411d9ef61729bebbc13d6d7d047ec60059857d8270`.
The sibling manifest records that archive hash and each exact member hash.
All originals pass the manifest before and after archive creation; the archive
member listing matches the explicit completed-file selection. No wildcard was
used to select active logs.

Private extraction to `/tmp/sm-progress-receipt-check-20261013.ShFIqN` passes
all 13 member hashes. The member check excludes only the archive's own hash
line and runs from that private restore root. `tar`, `gzip` via `tar -z`,
`sha256sum`, `rg` and filesystem metadata tools performed the preservation;
no Python, new source build, gameplay run or claim mutation was used.

After all validation writers finished, exact private extracted files were
removed with individual `unlink` calls and their empty directories with
`rmdir`. `du -sb` measured **4,329,391 apparent bytes reclaimed**, including
private directory metadata. The validation directory is gone. No production
run/build tree, evidence reader, browser profile or original receipt was cleaned.
This archive-copy check has no gameplay verdict or project cleanup target.
Final filesystem available-space snapshot: **224,305,602,560 bytes**; concurrent
report work can change that value independently.

The three individually archived generated files have exact ignore rules in
`.gitignore`; authored issues, Rust, stage-update TOML and the four reviewed TSV
inputs remain independently staged by the main Git owner. This avoids rewriting
their exact generated whitespace or duplicating large metadata diffs. Final
browser evidence and reporting cleanup stay with the report session; tickets
04 and 05 remain claimed at this WIP checkpoint.
