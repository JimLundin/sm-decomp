# Investigate lossless validation storage

Status: resolved
Blocked by: None
Triage: ready-for-agent
Type: research

Measure representative hardware events, WRAM fixtures and complete frame states.
Compare lossless zstd over existing bytes with binary events and page/delta
snapshots. Preserve event order, duplicate writes and every complete state byte.
Record round-trip checks, sample limitations, primary sources and an incremental
implementation recommendation. Run scoped cleanup before resolving.

## Answer

Use a versioned binary pack with independent Zstandard event chunks and bounded
snapshot page pools, indexed by original member identity and frame/event sequence.
Keep logical decoded SHA-256 separately from stored-container SHA-256.

[Recommendation](../recommendation.md) contains measured results, assumptions,
proposed contract and rollout. [Primary-source research](../format-research.md)
records format/reference facts and alternatives. Binary chunks yield 91–249×
payload reductions across sampled traces; 256-byte page pools yield 82–108×
over raw snapshot sets. Ratios are empirical samples, with proposed container
overheads described explicitly. No automatic historical conversion is authorized.

## Completion evidence

Branch `research/trace-storage-format`, baseline `8bbef1f`, retained requested
worktree `/tmp/sm-trace-storage-study`. `tools/storage_format_study.py` is the
reproducible read-only benchmark tool. It uses system libzstd 1.4.4 for measurement
only and saves small JSON results, not packed source captures.

- `projectile-measurements.json`: three 16 MiB text samples; every one of 870
  WRAM snapshots and 130 frames; first 32 ROM snapshots. Independent codec,
  grouped concatenation, XOR and 256/1024/4096-byte page-pool round trips passed.
- `room-measurements.json`: three 8 MiB text samples; first 128 WRAM snapshots,
  all 10 frames and 23 ROM snapshots. Same exact-byte round trips passed.
- `projectile-trace-chunks.json` / `room-trace-chunks.json`: independently
  compressed binary/text chunks with bounded frame grouping and chunk size;
  exact-byte round trips passed. Each contains original sample offsets/hashes,
  counts, rough Python timings and source verdict identity.
- `final-smoke.json`: final script, three 1 MiB room-trace samples, including
  source sample re-read verification; all round trips passed.
- Built-in controls passed: duplicate/reordered/changed-value events, frame
  resets, malformed and overflowing varints, compressed-frame truncation and
  deterministic same-process compressed output. These are spike controls, not
  production format acceptance tests or generated-host throughput measurements.
- `source-authentication.json`: all original sealed trace/WRAM/frame artifacts
  checked against both retained verdicts, 1,144 files / 5,734,473,474 bytes;
  verdict bytes unchanged. This authenticates historical bytes only.
- `python3 -m py_compile tools/storage_format_study.py` and CLI help passed.
  New-file whitespace checks produced no errors; `git diff --check` passed.

Cleanup completed after all benchmark/authentication processes finished:

```sh
python3 tools/port_cleanup.py --root .scratch/trace-storage-study --apply
python3 tools/port_cleanup.py --root .scratch/trace-storage-study --deduplicate --apply
```

Both returned `reclaimed: 0` (no eligible build/snapshot files). Removed only
identified disposable study bytecode, reclaiming 13,452 bytes. No generated
captures or compiled outputs remain. Measured available disk at cleanup:
270,742,257,664 bytes (about 252 GiB); the retained study notes/results occupy
roughly 100 KB. No integrated implementer worktrees were created; the requested
research worktree is retained for review and reuse.

Evidence dependencies: compact measurement JSON and primary-source notes in this
worktree, exact experiment source at `tools/storage_format_study.py`, original
historical capture directories under `/home/jim/sm-decomp/out/port/` named in the
JSON. Source captures and verdicts were neither relocated nor modified. Published
primary-source URLs and baseline repo source references are in the research note.
