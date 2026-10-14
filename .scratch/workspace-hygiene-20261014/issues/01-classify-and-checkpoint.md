# 01 — Classify retained artifacts and checkpoint workspace records

Status: resolved
Triage: ready-for-agent
Blocked by: None
Type: task
Owner: `/root`, session `01a13b35-079e-7151-8e5a-f759fee6bcf3`

## Scope

Review the pre-existing untracked scratch files and modified generated assignment.
Extend `.gitignore` for generated command output and known retained artifact
directories. Checkpoint authored records and the valid assignment update without
changing evidence bytes. Keep project source/issues visible; do not blanket-ignore
`.scratch`, all Markdown, all JSON or all TOML files.

## Completion evidence

The 825 original untracked files are classified as 101 records to checkpoint
and 724 generated local artifacts. Checkpointed records include 25 assignment
projections, two sealed result requests, 13 orchestration/provenance requests,
research answers/child proposals/manifests, authority-transfer observations,
completion receipts and small Rust reproduction/cleanup sources. These are
historical records, not new queue transitions or gameplay acceptance.

Ignore rules cover typed stdout/stderr/log suffixes and exact retained browser,
binary, input-snapshot and raw-diagnostic paths. Captured ROM data and extracted
ROM observation payloads remain local. Authored requests, research proposals and
wrapper source remain visible; no blanket scratch or TOML/JSON ignore was added.

The modified assignment's revision `50d3c2c…` matches the clean authoritative
ledger at lines 1435–1456; `e0c776…` occurs only in history. The job is
`not-required`, without a claim, attempt, observation or result. Independent
generator review confirms its suffix-preserving behavior explains the trailing
blank lines. Checkpoint it byte-for-byte to preserve the previously recorded
SHA-256 `4aa2f2139f6223a0864e0d674b3b74b66c25092c8796c2bb3331723eb978f87d`.
No regeneration or queue mutation was run.

All 825 original file hashes passed, as did hashes for the tracked assignment,
ledger, production registry and deterministic fence. `git check-ignore` confirms
all 724 artifact paths are ignored and representative future Rust, Markdown,
ticket and TOML request paths remain visible. Formatting checks cover the new
ignore rules and this issue/spec; historical sealed record bytes are preserved.
No gameplay/build verification producer or implementer worktree was created.

Audit evidence is under `out/workspace-hygiene-20261014`: the initial status and
file list, original/protected hashes, exact ignored/checkpoint file lists and
per-path ignore rules. Both project cleanup passes on that audit directory
succeeded with zero eligible bytes; no existing artifact was deleted or moved.
Last observed available disk space was 186,567,462,912 bytes.
