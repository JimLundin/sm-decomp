# Coordinator handoff (2026-11-06, night)

This is for the next coordinator session, and it replaces the version from the evening of 2026-11-05.
Live status is in pinned issue #177. The oracle's detail is in `docs/handoff/oracle-harness.md` and
`docs/oracle/MEASUREMENTS.md` (sections (a)-(r)) on `research/oracle-harness`.

## Branches

| Branch | What it holds |
| --- | --- |
| `rebuild/spine` | **The working trunk.** It merges `research/architecture` and `research/oracle-harness`, and adds the new crates: `data`, `game`, `runtime`, `app`, `verify`, `tools`, `canonical`, `canonical_derive` and `oracle_record`, with `crates/README.md`. Also `tools/check_crate_rules.sh` (in CI), the `Canonical` codec with a structural `SCHEMA`, the slice-0 State skeleton (`game::state`) and the decoder's two first slices (`verify`; the next slice is listed in `ed3c6ae`'s lane report: wait-site table, stack walk, `UnknownYieldSite`, `Quiescence`, `irq`). Draft PR #285 exists only so CI runs: Quality passes; Verify fails only on the stale `file-copy-clear` lock entry, which the re-lock fixes. |
| `research/oracle-harness` | The oracle (see below). Merge it into `rebuild/spine` after it moves. |
| `research/architecture` | PRINCIPLES and ARCHITECTURE, merged into `rebuild/spine`. The newest decisions are D25 (which oracle locks which stream) and D26 (dedicated short trick movies, using [sm-json-data-rs](https://github.com/JimLundin/sm-json-data-rs)). |
| `main` | Nothing from the rebuild has landed yet. It lands through `tools/lane/land.sh`, docs first. |

## The oracle, as it stands

- **lsnes** (bsnes v085, run as a separate process) keeps sync with every movie, so it is the single locking oracle.
  - **Cut:** always at the NMI's RTI (MEASUREMENTS (i)).
  - **Streams:** opt-in, and records without them stay byte-identical (MEASUREMENTS (o), (r)):
    - the picture, as lsnes's PNG decoded in Rust and compared as it shows. Over any% 0-1000 it equals LakeSnes on 1001 of 1001 frames;
    - the PPU registers;
    - the CPU-side APU port log. A resumed run's APU log is empty; a lane is fixing this.
  - **Ladders:** rungs every 10000 frames in `$TEMP/ohx/data/lsnes-ladder/{100,low,any}`; all 47 rungs check equal.
  - **Native seeds:** in `$TEMP/ohx/data/lsnes-native/`. `$TEMP/ohx/oh` runs `oracle_rti2.dll`.
- **LakeSnes** with fixes 0001 (APU clock), 0002 (exactly 32040.5 Hz) and 0004 (no HDMA overhead once every channel has terminated, after anomie) follows any% exactly to step 35199. It is the fast lockstep oracle.
  - **Paused on purpose:** the step-35175 HDMA question (MEASUREMENTS (m), (n), (p), (q)). lsnes's HDMA rule is fitted, but the residual starts at the line-0 IRQ. The next step is written in (q). Pick it up only if LakeSnes ever has to run whole movies.
- **MesenCE 2.2.1** works as a third vote, run as a released binary in a separate process (MEASUREMENTS (g)). The accuracy research, scoped to Super Metroid, is in `docs/oracle/ACCURACY.md`.

## In flight at hand-off

- `lane/oracle-apu-resume`: the lsnes APU log after a resume.
- `lane/relock-plan`: `docs/oracle/RELOCK.md`, the re-lock plan (inventory, lock format, steps).

Merge each one by fast-forward or a rebase onto its base branch, then remove its worktree and delete its remote branch.

## Next, in order

1. **The re-lock**, following RELOCK.md: cut the missing native seeds from the ladders, then `corpus --record` in the new format with lsnes streams, then record every row in parallel shards, then bundle and upload to ci-data (root only), then the reader in `verify`. Doing this clears #285's Verify failure.
2. **Slice 0 on the new engine** (ARCHITECTURE §15): the decoder's PC/stack slice, the Tick spine, then `area-brinstar` and `item-morph-ball` green at ≥1,000 Ticks/s.
3. **Land on main:** the design docs first, then the oracle crate, through `land.sh`. Retire the PLAN/REVIEW lines that contradict PRINCIPLES/ARCHITECTURE (#252).

## Working rules learned (also in memory)

- **Lanes:** one item each, about 30 minutes and 300k tokens at most. That ceiling is a scoping test: if a lane is heading past it, stop it with a handoff and split what remains. Set a one-shot `CronCreate` check about 32 minutes after each spawn.
- **Long emulator runs** are the root's background jobs (`run_in_background` plus a Monitor), never a lane's. Use ladders and parallel segments, and keep each process under about 5 minutes.
- **Oracle accuracy** means accuracy for Super Metroid only.
- **Strats** come from sm-json-data-rs (the `sm-logic` crate and its packs), never raw JSON.
- **Defender** blocks fresh test executables here at random. CI's Quality job runs the rebuild crates' tests.
- **#177 edits:** run `gh` with `-R JimLundin/sm-decomp` and check the saved body isn't empty before editing (one edit wiped it; recovered from `userContentEdits`).
- **Pushes:** don't chain cleanup after a push with `;`. One push hit a GitHub error, and the commit was lost after the branch removal.
- **Orphans:** five lsnes processes (6176, 14972, 24536, 22000, 25512) can't be killed; a reboot clears them.
