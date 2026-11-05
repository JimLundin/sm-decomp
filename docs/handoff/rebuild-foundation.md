# Rebuild on a new foundation: handoff (2026-11-05)

> Superseded on 2026-11-05 by [coordinator.md](coordinator.md).

For the next coordinator session. Jim approved rebuilding the engine on a new foundation
(docs/REVIEW.md, "Rebuild on a new foundation (Jim, 2026-11-05)", on main since `52c2117`).
The foundation's design comes first and Jim reviews it before any spine code is written.

## Your first task: Jim's question list

Jim asked for **one list of the questions he needs to answer, each with its context**, presented
so it is easy to digest. Compile it; this session did not.

1. Start from FOUNDATION.md's "Decisions for Jim" (J1-J24, each with a recommendation), then
   add every open question from the three requirement reports that J1-J24 don't already
   cover, plus the issue comments that ask Jim something (#252, #253, #242).
2. Deduplicate and group them by topic (for example: fidelity and lag, actors and handles,
   content and Map Rando, verification and gates, docs and process).
3. For each question give, in a few lines: what is being decided, why it matters for the
   foundation (what it changes if answered one way or the other), the options, the
   recommendation and its reason, and the source document.
4. Order them: questions that block the foundation's design first, then those that only
   block later slices, then process and docs.
5. Present it to Jim in the conversation as a short summary plus the grouped list, and commit
   it as `docs/foundation/QUESTIONS.md` on the design branch so he can answer there.

Do not start building the spine until Jim has answered the questions that block it.

## Where everything is

| What | Branch | Path |
| --- | --- | --- |
| Frame behaviour requirements (R1-R31: Tick, NMI and input latch, lag, per-scanline display, door transition, sound protocol, limits) | `research/foundation-frame` | `docs/foundation/frame-requirements.md` |
| Domain model and size (one instruction-list interpreter, slot store, behaviour as data; rebuilt `sm_game` estimated 57-84k lines) | `research/foundation-domains` | `docs/foundation/domain-model.md` |
| Map Rando modularity (doorway IDs, topology and media as content, typed profiles, transitions carry the Session) | `research/foundation-maprando` | `docs/foundation/map-rando-requirements.md` |
| **The foundation design** (done: decisions, traceability table, Jim's decisions J1-J24, build order, size budget) | `research/foundation-design` (head `0436208`) | `docs/foundation/FOUNDATION.md`; the three reports are copied beside it; the compile-checked type sketch is `scratch/foundation_sketch/` (its own workspace) |
| Game data audit (#253) | on main | `docs/handoff/game-data-audit.md` |
| Mutation sample design (#242) | on main | `docs/handoff/mutation-sample.md` |
| Native renderer in `sm_host` (paused) | `gate1/render` | see below |

The design maps every requirement (R1-R31, the domain decisions, the Map Rando items) to a
design element or a stated deferral, and lists ten revisions to the accepted
target-architecture docs, which wait for Jim's approval. Its build order: preparation (the
crate plan, the Checkpoint derive and state skeleton, two oracle measurements, the decoder
skeleton), then slice 0 in the Morph Ball Room (`area-brinstar`, then `item-morph-ball`), then
clusters of rows that share a room, then the room chains and the full movies. Size budget:
83-123k engine lines.

## Confirmed in the oracle code

The oracle does not model CPU-overrun lag: each frame runs the main loop to `$80:8343` or
`$85:813C`, then the NMI, with no cycles counted against the beam; the beam and IRQs are
stepped afterwards at line granularity. So overrun lag, mid-frame HDMA tearing and the IRQ's
multiplier clobber cannot occur in the oracle (FOUNDATION.md, J1). The APU ports reach the
sound driver once per frame, so the sound handshake can match exactly in Tick time. 157 of the
161 corpus rows start from a seed snapshot, so the oracle RAM → Session decoder is needed to
open rows, not only to compare them.

## State of main and CI

- Main `52c2117`: Gate 1 slice (#282), the Rust Gate 1 runner and lints/mutations/docs/audit
  batch (#283), and Jim's rebuild decision (#284). Every landing was a fast-forward onto a head
  whose CI passed.
- Gate 1 workflow: all 114 locked rows match the faithful oracle on all 8 machine streams
  (`sm_pipeline gate1`, eight shards on pushes to `gate1/**`).
- Issues: #240 and #241 closed. #248 has the Gate 1 status. #252, #253 and #242 have findings
  and questions for Jim. #239 closes after a second green nightly (2026-11-05 was green).
- Pushes to `main`, `verify/**` and `gate1/**` and PRs run CI. Research and handoff branches
  run nothing. Runs can't be cancelled or re-run from a cloud session (403); ask Jim.
- This container has no ROM key; replays run only in CI.

## Paused: the native renderer (`gate1/render`, head `615e3b2`)

`sm_runtime::rendering` draws `sm_host`'s replay frames: 114/114 rows match at `839c69a`. At
`09c2895` the hardware departures are implemented instead of listed in LEGACY, and every row it
finished matches, but credits and menu rows run about 30% slower because Mode 1 with BG3 at
normal priority takes the per-pixel path. `615e3b2` is diagnosis only and must not land. The
renderer itself (`draw_line`, the per-pixel composer) lives in `sm_runtime` and is reusable by
the new foundation; the `sm_host` glue is not. Decide with the design whether to land it.
