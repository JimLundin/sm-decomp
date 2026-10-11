# Compare a global content-addressed store

Status: resolved
Blocked by: None
Triage: ready-for-agent
Type: research

Compare the competing proposal (256-byte Merkle snapshots, shared burst payloads,
per-frame roots and one object store across out/) with the measured bounded packs.
Clarify equality, canonical hashing, exact trace preservation and retention.
Use an in-memory packed-object estimate to evaluate global page/node sharing;
do not migrate captures or implement a production store.

## Answer

Adopt content-addressed logical artifacts backed by compact compressed packs.
Cross-run page/payload sharing is useful; Merkle identity does not determine
physical encoding efficiency. Use distinct provenance/run and observation roots,
canonical object encodings and exact expansion of observed event bursts.

[Comparison](../global-store-comparison.md) records the recommendation, claim
corrections, migration/retention implications and empirical limitations.

## Completion evidence

`tools/storage_merkle_study.py` creates only in-memory candidates and compact
`merkle-estimate.json`. Canonical type/length-prefixed SHA-256 object experiment:
256-byte leaves, fanout16, single hash-sorted Zstandard3 object pack, system
libzstd1.4.4. Same-membership global flat pool: padded256-byte pages, u32 IDs,
exact member lengths/tables, one Zstandard3 compressed frame.

Both candidate representations reconstructed all1,142WRAM/frame snapshots from
two historical C captures exactly. Simple Merkle pack6,148,639bytes; global flat
pool1,230,236bytes; indexes/catalog and actual filesystem allocation excluded.
All10room frame snapshots already occur in the longer capture and add0objects.
This demonstrates reuse; it does not benchmark concurrent production or seeking.

The reproducible script ran successfully with `PYTHONDONTWRITEBYTECODE=1`.
Prior source authentication remains retained in `source-authentication.json`.
No originals, verdicts, packed artifacts or store directories were modified.

After measurement processes finished:

```sh
python3 tools/port_cleanup.py --root .scratch/trace-storage-study --apply
python3 tools/port_cleanup.py --root .scratch/trace-storage-study --deduplicate --apply
```

Both cleanup commands succeeded, reclaiming0eligible bytes. Identified disposable
Python bytecode from the initial in-memory experiment was removed (13,480bytes).
Available disk267,940,642,816bytes (about249.5GiB). No integrated implementer
worktrees or build outputs were created. Requested research worktree remains
retained, with source, notes, compact measurements and cited historical source
capture paths as evidence dependencies.
