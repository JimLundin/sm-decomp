# Spike content-addressed packed storage

Status: resolved
Blocked by: None
Triage: ready-for-agent
Type: prototype

User requested a working spike and approximate disk reduction, runtime overhead,
read speed and frame-access latency. Build throwaway disk-backed compressed
objects with compact snapshot page references, canonical content identities,
run/observation seals and frame indexes. Preserve exact legacy trace/state bytes.
Use real captures and a native direct-writer benchmark; distinguish offline
conversion, native observer cost and unmeasured whole-emulator overhead.

No production migration or accepted evidence mutation. Retain a reproducible
one-command benchmark and report limitations. Clean all generated unsealed
intermediates and retain only small authenticatable spike artifacts/results.

## Answer

Working disk-backed content-addressed compressed store implemented as throwaway
Python/C tooling. [Results](../spike-results.md) records complete-capture disk,
writer, conversion, verification, memory and random-read measurements.

Two complete captures (5.73 GB logical / 5.69 GB allocated within scopes) become
54.93 MB actual / 54.94 MB allocated: about 104× smaller including pack, index,
manifests. Ten-million-event native writer median 0.220 s binary vs 2.314 s plain
and 3.519 s gzip1; not a whole-emulator overhead percentage. Python frame reads
17–30 ms median, trace-frame reads 6–7 ms, initial large-run open/index about 0.8 s.
Full report describes projection limits, warm-cache timings and missing
burst/concurrency/GC implementation.

## Completion evidence

Sources: `tools/storage_spike.py`, `tools/storage_spike_native.c`, prior codec
measurement helper. Branch `research/trace-storage-format`, following 688d7b8.
Reproduce with `PYTHONDONTWRITEBYTECODE=1 python3 tools/storage_spike.py benchmark
--output out/storage-spike/new-run --events 10000000`.

`spike-benchmark.json` contains full measured results. `final-controls.json`,
`extra-read-measurements.json`, `native-memory.json`, `reader-open-measurements.json`
and `frame-extraction-checks.json` record follow-up controls/measurements.

Independent native decoder verified all 306,019,371 events reproduce exact original
SHA-256. Store imports verify logical/physical chunk hashes. All 1,142 snapshot
members checked against original verdicts and reconstructed after store reopen.
Twenty indexed-frame reads compared against independent whole-chunk expansion.
CLI extracted frame-001000.bin exactly equals 1,401,335 original bytes; temporary
extraction removed. Source verdicts remained byte-identical.

Controls pass: duplicates, resets, partial pages, changed snapshot roots, unknown
literal records, unequal producer/run roots with equal observations, missing
object, corruption, truncation and complete-chunk/missing-footer rejection.
C compile, Python CLI and Git whitespace checks passed. No production changes.

Initial full probe failed on a room-plm record missed by earlier samples; writer
now retains exact literal unknown records. Slow sscanf converter run interrupted
and replaced with checked decimal parser; all final complete runs passed.
Cleanup of failed/interrupted scopes recorded in `probe-cleanup.json` and
`restart-cleanup.json`: 18,886,504 + 23,849,595 + 76,427 unsealed bytes removed.
Successful benchmark automatically removed 16,853,627 bytes of native builds,
samples and control files plus verified/imported temporary trace containers.
Final memory/control intermediates removed 18,027,092 bytes, recorded in
`final-cleanup.json`. Extracted frame 1,401,335 bytes removed.

After all processes finished, scoped project cleanup succeeded:

```sh
python3 tools/port_cleanup.py --root out/storage-spike --apply
python3 tools/port_cleanup.py --root out/storage-spike --deduplicate --apply
```

Both report 0 eligible bytes. No disposable binaries, raw trace samples, control
stores, conversion intermediates or extracted frames remain. Available disk
268,002,426,880 bytes at cleanup. No integrated implementer worktrees; requested
research worktree remains retained. Working store 54,933,211 bytes plus small
benchmark.json is retained under `out/storage-spike/final`.

Retained physical dependencies: `objects.pack` SHA-256
`10e894ce46c42c206869b36337c73bea28ae306ed7cb880305e0df516847631c`,
`index.sqlite` SHA-256
`0a1d248303bd6d75150fc9383136f489d9e59a7e6bd87f9681aa2dfec5d2b3ce`;
both reauthenticated after all reader/extraction operations. Root manifests are
inside retained store and authenticated against roots in spike-benchmark.json.
Original captures/verdict paths retained unchanged. Code and compact measurements
are committed on the throwaway branch; current production admission not claimed.
