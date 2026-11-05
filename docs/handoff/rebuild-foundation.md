# Rebuild on a new foundation: handoff (2026-11-05)

For the next coordinator session. Jim approved rebuilding the engine on a new foundation
(docs/REVIEW.md, "Rebuild on a new foundation (Jim, 2026-11-05)", on main since `52c2117`).
The foundation's design comes first and Jim reviews it before any spine code is written.

## Your first task: Jim's question list

Jim asked for **one list of the questions he needs to answer, each with its context**, presented
so it is easy to digest. Compile it; this session did not.

1. Collect every open question from the documents below (each ends with its open questions
   or "questions for Jim"), plus the issue comments that ask Jim something (#252, #253, #242).
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
| **The foundation design** (in progress when this was written) | `research/foundation-design` | `docs/foundation/FOUNDATION.md` |
| Game data audit (#253) | on main | `docs/handoff/game-data-audit.md` |
| Mutation sample design (#242) | on main | `docs/handoff/mutation-sample.md` |
| Native renderer in `sm_host` (paused) | `gate1/render` | see below |

The design lane was asked to: trace every requirement in the three reports to a design element
or an explicit deferral; give each decision with its alternatives; check the core types with a
compile-checked scratch sketch; collect Jim's decisions with recommendations; and propose the
first slice and the build order. If `research/foundation-design` is missing or incomplete,
redo that step from the three reports before the question list.

## One finding to carry into the lag question

Each oracle frame runs the game's main loop until it waits for the NMI
(`run_cpu_until_pc(0x808343, ...)` in `sm/src/cpu_infra.rs`). So the oracle itself appears not
to lag from CPU overrun: one main-loop iteration is one Tick. If confirmed, "no overrun lag"
matches the oracle exactly, and real-hardware lag is a Reference-profile question rather than a
foundation requirement. The frame report's overrun-lag options (cost model, recorded lag, none)
should be weighed with this.

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
