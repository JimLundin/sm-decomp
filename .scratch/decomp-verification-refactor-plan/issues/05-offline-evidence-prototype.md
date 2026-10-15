# Measure direct indexed Offline evidence

Status: resolved
Blocked by: None
Triage: ready-for-agent
Type: prototype

## Scope and ownership

Own a bounded Rust prototype and evidence under this effort, with only the minimal
isolated public-seam hooks required to exercise `offline-verification` and
`evidence`. Do not replace active production receipts during the experiment.

## Design question

Can direct indexed output reduce complete verification/authentication/cleanup cost
while retaining exact per-case observations and Native producer attribution?

Compare current directory output with direct immutable records. Reuse the existing
evidence store and compare Merkle images against larger raw/chunked payloads.
Measure compression, hashing, import, restore and independent admission as well
as execution and final disk size. Test private bounded writers; do not serialize
every case behind the global store's single writer lock.

Separate two effects: packing completed files reduces retained file count, while
a bounded binary output transport/owned arena is needed to avoid the initial
per-case write/read cycle. Exercise both where supported and state precisely
which effect the prototype measures. Initial samples fit roughly 60 seconds;
larger experiments require a predeclared bounded budget.

## Acceptance

Same before/Rust-after/C-after bytes, returns, host state, capture expectations,
ordered effects, chains, logs and complete producer membership. Reconstruct every
logical member through a fresh reader. Missing/corrupt/extra objects, equal wrong
outputs, incomplete frame and interrupted publication must refuse. Measure retained
and staging bytes/files, CPU, memory, reads and full elapsed time.

## Answer

Keep the current directory/operation-index formats and implement bounded reuse of
authenticated bytes within a case/operation admission. Packing the complete
finished ordinary run lost elapsed time in every alternating comparison, despite
about 75% lower retained allocation with raw chunks. Ticket 06, the architecture
and the spec were revised explicitly before production adoption (commit `9070f0f`).
Direct producer transport remains unmeasured and is neither selected nor ruled
out by this storage-only experiment.

## Completion evidence

Implementation: `30cd749c272a42a62c7d3a57bb7d35634291013a`; final study/handoff:
`5bb08b45b459d2a8f8b86d839a16d961545cd3a3`. Integrated on the isolated performance
assembly branch at `8b82d4af41a50105a0fc82e4164c74d3f7f9f06f`. This resolves the
prototype decision, not production verification acceptance or the full refactor.

The [retained study](/home/jim/sm-decomp-performance-evidence-proofs/20261015/evidence-study.md)
contains exact commands, source/readers, bounds, CPU/RSS, syscall-count limitations
and all measurements. It used the complete admitted 16-pair ordinary run: 333
members / 43,675,132 logical bytes. Eighteen alternating samples and 16 explicit
negative controls passed; current `sm-evidence` independently checked all three
packed strategies. Median storage lifecycle from finished staging was 0.2628 s
for directories, 0.9034 s raw, 1.0246 s Merkle and 1.8181 s private writers/import.
No host/callback/chain/Native transport or end-to-end speedup is claimed.

All writers stopped, both scoped cleanup passes succeeded for samples, controls,
failed setup and the study. Controls deduplicated 10,354,688 bytes; final study
cleanup deduplicated 20,971,520 bytes. Removing the private build reclaimed
76,251,136 allocated bytes plus a 4,096-byte TMPDIR; retained source, exact reader,
raw results and diagnostics remain. The dependency symlink was unlinked safely.

Root retired `/home/jim/sm-decomp-performance-evidence` after integration. Both
post-retirement cleanup passes succeeded with zero further bytes reclaimed and
767 snapshots checked. New readers then authenticated all 18 retained samples,
and frozen `family-authenticate` returned byte-identical original evidence
(`50cc84fd73fb6b4d14c635fcd763d3411b9051056a4f70f590258e12bde1c166`).
See study `logs/post-retirement-after-cleanup-*` and `post-retirement-cleanup*.toml`.
Available space at that check: 120,924,135,424 bytes. Original frozen source,
ROM/capture dependencies and baseline readers remain required; the retired worker
checkout and deleted build target are not dependencies.

## Comments

Keep the `.smf` execution format unchanged unless a separate measurement supports
changing it. Do not create historical-format migration tools.

2026-10-15 scheduling refinement: the representation experiment can start from
the extraction's already authenticated, completed ordinary proof outputs without
depending on ticket 02's bulk Interface. Its first slice owns only an isolated
Rust example/study and experimental output. Production hooks remain coordinated
with ticket 02 and production adoption remains ticket 06, after 02/03. Any
staging-only measurement must say that it does not test a direct producer transport.

Claimed by performance session `01a13e7f-ab01-7413-a1c2-288b08a4b719` for
`/home/jim/sm-decomp-performance-evidence`, branch
`performance/evidence-study-20261015`, from extracted commit `8bf8acf`.
Initial writable source is an isolated Rust example under `crates/evidence/examples/`
and its study/handoff; production recipe/input edits require root coordination.
