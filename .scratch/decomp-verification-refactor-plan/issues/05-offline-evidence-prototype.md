# Measure direct indexed Offline evidence

Status: claimed
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

Pending. Select granularity, writer/import layout, transport and Native contract
changes only from the comparison. If packing loses overall, propose the narrower
winning change and update ticket 06 explicitly before implementation.

## Completion evidence

Pending. Retain exact source, raw measurements and failures. Run both cleanup
passes, remove disposable builds, authenticate retained proof and record space.

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
