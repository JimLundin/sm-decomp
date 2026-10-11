# Compare the competing worktree's actual spike

Status: resolved
Blocked by: None
Triage: ready-for-agent
Type: prototype

Read competing evidence crate and retained runs without edits. Compare an equal
scope with our complete projectile C capture: hardware.log plus all sealed WRAM
and frame snapshots. Use a separate disposable store and temporary immutable
hardlinks to avoid copying source evidence. Record actual logical/allocated
storage, ingestion time, native extraction and implemented features. Verify
readback and clean only comparison outputs, leaving other agent's work untouched.

## Answer

[Comparison](../other-spike-comparison.md): on exactly the same5.26GBscope,
ours53.14MBallocated vs other612.68MBallocated (277.52MBlogicalobjectbytes).
Other172.2singest vs our approximately76.3sconversion/import/snapshots/publication,
excluding separate exact decode. Full verification approximately82sother vs
approximately88sours (slightly different validation work). Other contributes
exact burst sharing and subtree diffs; combine these with compressed packs and
compact indexes. No integrated replay-overhead measurement exists for either.

## Completion evidence

Used other existing release binary SHA-256
`953cf9c865363305ce4a424a0db26a3eb1c00712039e2def69920e9a2ab50205`.
Matched input scope comprised hardware.log,870WRAMsnapshots and130framefiles
from retained projectilec-0capture20261011T061431.667466Z. Temporary source
hardlinks under `/home/jim/sm-decomp/out/storage-spike-other-comparison` avoid
copying5GB; same-filesystem location needed (initial /tmp link attempt failed
with EXDEV before ingestion). Source bytes/verdicts untouched.

`other-matched-benchmark.json` records all1,001files/5,264,039,623bytesverified
exactly, object counts, allocated bytes, ingest and readback timings. Native
readback passes in81.9s. `other-native-read.json` records three exact frame reads
114/69/44ms, including native CLI startup and warm filesystem cache. Retained
source snapshot/context and hashes under `other-spike-source/` authenticate the
version inspected; original other worktree remained unchanged during comparison.
Existing broader other-agent benchmark log was read only, not used as matched
scope evidence. No source/build changes or tests run in other worktree.

After all readers/writers finished, project cleanup commands on our selected
comparison root succeeded with0eligiblebytes:

```sh
python3 tools/port_cleanup.py --root /home/jim/sm-decomp/out/storage-spike-other-comparison --apply
python3 tools/port_cleanup.py --root /home/jim/sm-decomp/out/storage-spike-other-comparison --deduplicate --apply
```

The utility does not support this loose-object spike format; removed only our
identified unsealed comparison store/logs and temporary aliases. Reclaimed
277,516,426logical /612,683,776allocatedobjectbytes. Removed1,001temporary
hardlinks; source payloadbytesreclaimed0. Removed1,401,335bytesofdisposable
our-CLI extraction. Empty failed /tmp scope removed. Small results/source snapshot
retained in `.scratch/trace-storage-study/`; other agent's original
`/tmp/evidence-spike`, worktree and build remain untouched. Requested research
worktree remains retained; no integrated implementer worktree introduced.

Retained evidence dependencies: comparison/source snapshots and compact JSON;
original historical capture named above; other-native-read references the other
agent's retained store/image id. Our existing working store retained with original
physical seals from ticket03. Remaining disk268,945,154,048bytesaftercleanup.
