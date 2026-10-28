---
name: sm-decomp-backends
description: Define or review sm-decomp rendering, audio and asset-loading seams between our engine and replaceable backend shims, providers or plugins.
---

# Engine and backend boundaries

Read [PLAN](../../../docs/PLAN.md) first, then [REVIEW](../../../docs/REVIEW.md)'s
Milestone plan at each item's start. Read the accepted
[destination](../../../docs/target-architecture/README.md), its Windows policy,
[architecture](../../../docs/target-architecture/ARCHITECTURE.md) and
[ownership/lifecycle contracts](../../../docs/target-architecture/CONTRACTS.md)
for reached video, audio, content and storage sections.

The deterministic Session owns gameplay and emits owned Scene/effects.
Device objects and backend completion timing cannot independently mutate gameplay.

| Responsibility | Engine owner | Replaceable adapter/provider |
| --- | --- | --- |
| Visual interpretation | Our renderer: Scene IDs, ordering, palettes, masks and effects | GPU resources, submission and presentation |
| Sound semantics | Our sequencer/mixer: Tick cues, voices, pitch, envelopes, music and echo | Audio-device access and bounded delivery transport |
| Asset loading | sm_data/tools domain import, decoding and validation; runtime prepared sound banks, images and atlases | Storage, transport and platform resources |

New game/domain seams exchange typed owned records and validated asset references.
Reuse existing typed Rust asset/frame/material/cue references and immutable
validated media revisions for sound/music, sprites and tilesets. IDs stay independent
of ROM addresses, pack order, device handles or filenames; reuse existing ID types.
Session audio uses semantic Tick-stamped commands; device PCM/C registers never enter gameplay.
Confine platform types, C ABI and raw pointers to named compatibility adapters;
preserve their allocation provenance, native preconditions and borrowed lifetimes.
Inventory retained ROM, SNES, OpenGL, SDL and C-callback dependencies, with an
explicit removal condition for each reached caller/callee chain.
Windows migration may retain the working oracle/backend until replacement
evidence exists; disclose that compatibility scope.

Pack choice is owned config/request data, independent of UI: startup settings or a future in-game menu.
Validate/prepare the candidate first; install media-only replacements at committed Tick seams.
A bad candidate preserves the active revision; old GPU/voice resources live until actual retirement.
Hitboxes, animation timing or rules change gameplay fingerprint/reset policy; appearance/sound
replacement is media-only only when gameplay contracts stay compatible.

- Trace actual producers/consumers, inputs/outputs, external writers and the sole
  mutable owner; keep visual, sound and content interpretation in engine code.
- Specify owning thread, callback limits and media/resource lifetimes through
  GPU completion or acknowledged voice/command retirement.
- Preserve reset, shutdown, error prefixes, recovery and ordered delivery.
  Storage/device workers return owned results; gameplay-affecting results enter recorded Tick inputs.
- Completing a seam needs compile/type proof through the actual reachable engine
  consumer with the compatibility backend unavailable, plus required independent
  visual/sound/content fidelity evidence. Name remaining dependencies and gaps.

Wrappers or package moves alone prove neither safety nor plugin completion.
