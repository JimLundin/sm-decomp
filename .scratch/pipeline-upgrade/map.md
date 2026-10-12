# Native pipeline upgrade work map

Status: claimed

Plan: `plan.md`. Visual source: `visual/plan.mdx`.

## Decisions so far

- User-directed bounded measurement replaces hour-long telemetry tuning.
  Deliver exact legacy rendering (ticket09) and default-off detailed telemetry;
  19.389% short complete-wall reduction with exact observations. Large telemetry
  2% and binary15%/5% promotion remain unproved, conditional04–08 unclaimed.
  See [short measurements](evidence/short-measurements.md).

- Reuse e820a6b infrastructure and the uncommitted native Rust migration;
  preserve sm-evidence-v1. See `optimization-progress.md` and source snapshot.
- First reconcile delivered native code/current decomp and fill measurement gaps.
- Profile/select in ticket 03; binary tickets remain conditional and unclaimed.
- New tools/tests are Rust; typed interfaces/TOML receipts, binary transport;
  historical JSON formats/readers stay compatible.
- Explicit typed PPU/APU and shared exact literal scanner; mandatory END counts.
- Current decomp chains/host ownership gates are prerequisites for promotion.
- Implementation started on `pipeline-upgrade-integration` at delivered merge
  `3ad1bf3`; see `reconciliation.md` for current native v2 evidence and retained
  historical-reader contracts. Ticket 01 is resolved with refreshed admission:
  13,624 actual-C cases and a complete 129,712-input four-subject native text
  verdict, post-cleanup/removal authentication. See `evidence/native-integration.md`.
- Absolute benchmark budgets are frozen in `benchmark-budgets.md` before
  sampling; metrics and benchmark exploration notes are retained in `/tmp/`.

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
| [09](issues/09-exact-trace-restoration.md) | task | Selected improvement |

Ticket 01's external prerequisite is satisfied by migration `c89db9b`, merge
`3ad1bf3`, resolved local migration tickets and their recorded controls/cleanup.
Ticket 01 current-source admission and cleanup are complete. Ticket 02 is claimed;
ticket 03 and all conditional transport tickets remain blocked.

Crash recovery checkpoint: [checkpoint-02.md](checkpoint-02.md). Ticket 02's
native metrics implementation and controls are integrated, with fresh admission
and cleanup complete. Its telemetry overhead gate awaits quiet-host measurement;
the interrupted first sample is explicitly excluded from performance evidence.
