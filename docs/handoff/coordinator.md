# Coordinator handoff (2026-11-05, evening)

For the next coordinator session. This replaces [rebuild-foundation.md](rebuild-foundation.md):
its first task, the question list, was done and then superseded by Jim's direction below.

## Where the rebuild stands

Jim's order of work is (1) a functional oracle, (2) the engine's architecture, (3)
implementation against the oracle. (1) is in progress and (2) is designed. No engine code
has been written, and nothing from today is on `main` yet.

| Branch | Head | What it holds |
| --- | --- | --- |
| `research/architecture` | `fcf6344` | **The design.** `docs/architecture/`: PRINCIPLES (the thesis, P1-P20, K1-K10), ARCHITECTURE (D1-D24), ROBUSTNESS, QUESTIONS (answered), RETROSPECTIVE (a copy), plus `scratch/architecture_sketch` and `scratch/lint_cost` |
| `research/retrospective` | `bcc9e48` | the evidence-based retrospective of both attempts (510 lines) |
| `research/oracle-harness` | `ae33140` | `crates/oracle` (the unmodified ROM on upstream LakeSnes, no patches), `docs/oracle/HARNESS.md`, `MEASUREMENTS.md`, `docs/handoff/oracle-harness.md` |
| `research/foundation-design` | `1600c4c` | FOUNDATION.md (superseded by the architecture; kept as its input) and the superseded QUESTIONS list |
| `gate1/render` | `615e3b2` | the paused renderer. ARCHITECTURE decides: don't land it; the compositor moves into the new `runtime` in slice 0 |
| `main` | `52c2117` | Jim's rebuild decision (REVIEW). The old docs (PLAN, target-architecture) don't know about today's work |

Worktrees are under `sm-decomp-experiments/20261105/`, one per branch above, plus `handoff`.

## Jim's decisions today (binding; recorded in the docs above)

- **The thesis:** we model Super Metroid's behaviour in its entirety, as named behaviours, so
  each can be reproduced exactly, adjusted (for example the space jump timing window), added
  (respin, QoL) or removed (lag). A behaviour hidden in structure is a modelling failure.
- **The principles are approved.** The newest stable Rust, the strictest lints (no
  `unwrap`/`expect`/`panic`/indexing panics in engine code), idiomatic and readable to modern
  developers, and comments that say why, never what.
- **Crate names have no `sm_` prefix:** `data`, `game`, `runtime`, `app`, `oracle`, `verify`,
  `tools`, `canonical`. The old `sm_*` crates keep their names until deleted, and new crates
  may not depend on them.
- **Slowdown** (load times and busy-room lag) is reproduced in vanilla play, and can be
  toggled off. Every TAS movie trick must be executable in play.
- **Widescreen** isn't built now. Later, it means wide gameplay, after vanilla is solid.
- **Map Rando** is an aspiration with no roadmap. The architecture must not rule it out.
- **Oracles:** emulators, possibly several. GPL emulators are allowed only as separate binaries,
  never linked or copied, and one may be the primary oracle if necessary.
- **Questions for Jim** are goal trade-offs only, each with full context and examples.
  Engineering choices are the coordinator's, recorded with their reasons. Jim vetoes.

## The last lane's result (no lane is running)

The lsnes backend works: `research/oracle-harness` at `97ae09a`, GATE OK, with next steps in
order in `docs/handoff/oracle-harness.md`.

- **How it runs.** `crates/oracle/src/lsnes.rs` and `tools/oracle/lsnes_server.lua` drive
  `lsnes-bsnes.exe` as a separate process (`movie --oracle lsnes`), with no emulator code in
  the repo. It produces one FrameRecord per frame, cut at the NMI's RTI.
- **Sync.** any% syncs from power-on through frame 17829. All 16 any% room-boundary seeds
  match the old capture with 0 WRAM bytes differing. LakeSnes desyncs at about 8567.
- **Speed.** 45 frames/s, recording full memories.
- **Missing fields:** picture, APU log, SPC state (lsnes exposes the SMP and DSP only as raw
  structs), the write log, PpuRegisters and overscan. So lsnes is the sync and seed authority,
  and LakeSnes stays the full-record oracle.
- **Native seeds.** 16 seeds (the `.lss` state plus the raw record) are in
  `$TEMP/ohx/data/lsnes-native/any-boundary`, not in git.
- **Not done:**
  - lsnes exited at step 18068, cause unknown. Capture its stderr first.
  - `corpus` doesn't yet accept `--oracle lsnes` or native seeds, so none of the 27 stalled
    rows has been re-run.
  - Door lengths haven't been measured.

## Next steps, in order

1. **The oracle (finish (1)).** First the lsnes lane's leftovers (the exit at 18068, `corpus --oracle lsnes` with native seeds, the 27 stalled rows, door lengths), then:
   - The cross-oracle disagreement report between LakeSnes and lsnes.
   - The re-lock, recorded oracle-side by `corpus --record`, keyed by (row, oracle, version), with
     the new stream definitions (D20-D23).
   - The measurements ARCHITECTURE still waits on: the beam-sync point (D7), the IRQ dots
     (D11), the H9 load-cost hypothesis (D9) and the driver echo (D18). Some are already in
     MEASUREMENTS.md, and door lengths need the syncing core.
   - Decide which oracle is the gate's primary from these results.
2. **Land the design.** Open one PR per branch to `main` (docs only, no engine code), through
   `tools/lane/land.sh`. Retire the superseded rules: PLAN/REVIEW lines and target-architecture
   that contradict PRINCIPLES/ARCHITECTURE. This is also #252's "one current-rules file". The
   `crates/oracle` PR needs CI to build it; the zig/native toolchain question is in HARNESS.md.
3. **Slice 0** (ARCHITECTURE §15): the crate plan, then the spine built by one lane, then
   `area-brinstar` and `item-morph-ball` green on the new engine, at ≥1,000 Ticks/s.

## Working rules learned today

- **Lanes are short-lived:** one spawn = one milestone, about an hour, then a handoff in
  `docs/handoff/` and a fresh spawn. Don't relay follow-on work into a running lane. Jim
  flagged a 2-hour lane.
- Watch the context size, your own included. Brief lanes with pointers, not pasted text, and
  read excerpts.
- Defender randomly blocks built executables here (exit 126). The harness lane's workaround
  is described in its handoff and isn't committed.
- Private ROM-derived data: `$TEMP/ohx/data` (1.3 GB). Never in git.
- Keep pinned issue #177's first paragraph current, and keep `MEMORY.md` updated.
- Open from #252/#253/#242 (tier 3 of the superseded list, not yet put to Jim): one
  current-rules file, the README's "no game data" wording, the data-boundary rule, the
  mutation nightly. Bring them to Jim with context when the design lands.
