# Native pipeline upgrade work map

Status: open

Plan: `plan.md`. Visual source: `visual/plan.mdx`.

## Decisions so far

- Reuse e820a6b infrastructure and the uncommitted native Rust migration;
  preserve sm-evidence-v1. See `optimization-progress.md` and source snapshot.
- First reconcile delivered native code/current decomp and fill measurement gaps.
- Profile/select in ticket 03; binary tickets remain conditional and unclaimed.
- New tools/tests are Rust; typed interfaces/TOML receipts, binary transport;
  historical JSON formats/readers stay compatible.
- Explicit typed PPU/APU and shared exact literal scanner; mandatory END counts.
- Current decomp chains/host ownership gates are prerequisites for promotion.
- Planning alone leaves all implementation tickets open.

## Work

01 integration → 02 metrics → 03 native baseline/selection.
If binary selected: 04 decoder → 05 native producer → 06 compatibility →
07 benchmark/full-route → 08 promotion.

| Child | Type | Phase |
| --- | --- | --- |
| [01](issues/01-native-migration-integration.md) | task | First phase |
| [02](issues/02-complete-capture-metrics.md) | task | First phase |
| [03](issues/03-native-baseline-and-selection.md) | task | First phase |
| [04](issues/04-framed-binary-ingestion.md) | task | Conditional binary |
| [05](issues/05-native-host-and-transport.md) | task | Conditional binary |
| [06](issues/06-compatibility-and-failure-controls.md) | task | Conditional binary |
| [07](issues/07-controlled-binary-benchmarks.md) | task | Conditional binary |
| [08](issues/08-promotion-and-retention.md) | task | Conditional binary |

First ticket without local blockers: 01. Its external prerequisite is the
migration owner’s reviewed delivery and completed controls; it does not claim
or resolve that owner’s migration tickets.
