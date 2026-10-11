# Capture upgrade work map

Status: open

Plan: `plan.md`. Visual source: `visual/plan.mdx`.

## Decisions so far

- Reuse optimization e820a6b infrastructure and preserve sm-evidence-v1.
- Start with complete telemetry; binary transport stays opt-in until verified.
- Explicit typed PPU/APU and shared exact literal scanner; mandatory END counts.
- Current decomp chains/host ownership gates are prerequisites for promotion.
- Planning alone leaves all implementation tickets open.

## Work

01 metrics → 02 decoder → 03 producer → 04 compatibility → 05 benchmark/integration → 06 promotion.
First open unblocked ticket: 01. Each child records Type: task.
