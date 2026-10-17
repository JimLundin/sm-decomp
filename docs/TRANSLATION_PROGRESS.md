# Translation progress

Updated 2026-10-17. This tracks new recovered-C-to-Rust actor source batches.
These counts are additional source work, not a percentage of the complete game.
Existing root/restart translations and their historical verification are separate.

## Landed source

Eight complete scopes, **159 recovered-C definitions**, are published on `main`.
All eight have restart workspace library targets. Library registration enables checks; production runtime dispatch,
native adapters and gameplay parity remain unfinished.

| Scope | Definitions | Source | Restart library target | Gameplay verification |
| --- | ---: | --- | --- | --- |
| Skree | 10 | [Published](../translations/skree/enemy_skree.rs) | Registered | Pending |
| Beetom | 43 | [Published](../translations/beetom/lib.rs) | Registered | Pending |
| Rising/Falling Platform | 19 | [Published](../translations/platform/lib.rs) | Registered | Pending |
| Rinka | 26 | [Published](../translations/rinka/enemy_rinka.rs) | Registered | Pending |
| Fireflea | 12 | [Published](../translations/fireflea/src/lib.rs) | Registered | Pending |
| Bouncing Goofball | 10 | [Published](../translations/bouncing-goofball/src/lib.rs) | Registered | Pending |
| Mini Crocomire | 14 | [Published](../translations/mini-crocomire/src/lib.rs) | Registered | Pending |
| MiniDraygon/Evir | 25 | [Published](../translations/minidraygon/src/lib.rs) | Registered | Pending |

Source batch commits: `ca9522f` (six scopes, 120 definitions), then `1a71f28`
(eight scopes, 159 definitions and six library targets). The latter preserves
the reviewed gameplay source bytes. See [scope and acceptance details](../translations/README.md).

## Active source work

| Scope | Full boundary | Status |
| --- | ---: | --- |
| Yapping Maw | 32 definitions | Translation in progress |
| KiHunter and linked Wings | 30 definitions | Translation in progress |

The active 62 definitions are excluded from completed totals until complete
source is frozen, independently reviewed and published. Shared projectile and
native-service implementations reused by these scopes are not counted again.

## Burndown and acceptance

The new source backlog moved from zero to six published scopes (120 definitions),
then eight (159). There is no reconciled count of all remaining unimplemented
gameplay scopes, so a whole-game percentage or completion date is not supported.

The current process translates whole scopes first and verifies coherent batches
later using GitHub Actions. Formatting and dependency metadata checks do not
establish actual-C agreement. Independent source review can catch arithmetic,
addressing and effect-order defects, but execution remains a separate gate.
Whole-entry actual-C comparisons, failure controls, runtime integration, live
replay and original-ROM acceptance remain required. Missing private fixture
inputs must stay explicit failures; private inputs and outputs are not published.

Progress reports are requested every 15 minutes and at completed milestones.
This document records landed source and active boundaries, while local tickets
retain exact ownership, source hashes, review findings and pending gates.
