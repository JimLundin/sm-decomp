# Coordinator handoff (2026-11-06, morning)

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

## Done since the night version

- **The re-lock (RELOCK steps a-f):** `scenarios/locks/lsnes.toml` (`f42c70c`) holds 635 rows in format 2, recorded by lsnes from per-frame pads.
  - Every entry is agreed by two of three runs. Waves A and B agreed on 619 rows; the other 16 differed on one video frame each, the screenshot race (13 of them in the more loaded wave A), and wave C settled them.
  - Bundle: `relock-v2-lsnes-5db3282dbb5aaf7a.tar.gz.enc` in ci-data, pinned by its ciphertext SHA-256. It holds the seeds, the pads, the ladders and the new `file-copy-clear` inputs.
  - Left out: the three Golden Torizo rows. Their source is a snes9x movie, and the fix is a dedicated movie (D26).
  - The working files are private, under `$TEMP/ohx/relock-d/`: waves a, b and c, their sidecars, `compare.txt`, `merge.txt`, and the wave C PNG dumps for diagnosing the race.
- **Engine:**
  - the decoder's three slices;
  - the lock reader (`verify::lock`);
  - the shared span fold (`data::span`);
  - B1, the Session and Tick: 18.2M empty Ticks/s in CI. `Quality` runs the `empty_ticks` example.

## Next, in order

1. **Slice 0** (`docs/architecture/SLICE0.md`):
   - B2, lockstep, with A2, the record cache;
   - B3-B7: content, blocks, video, the line program, the interpreter;
   - the C domain lanes;
   - D1, the gate on the lsnes lock.

   A1 (census), A4 (typed facts), A5 (the compositor from `gate1/render`) and A6 (size) can run alongside.
2. **The video race:** find it with the wave-C dumps against the odd runs' sidecars. It looks load-dependent: watch lsnes's frame hand-off.
3. **Land on main:** the design docs first, then the oracle crate, through `land.sh`. Retire the old lock's Verify path and the PLAN/REVIEW lines that contradict the design (#252).

## Working rules learned (also in memory)

- **Lanes:** one item each, about 30 minutes and 300k tokens at most. That ceiling is a scoping test: if a lane is heading past it, stop it with a handoff and split what remains. Set a one-shot `CronCreate` check about 32 minutes after each spawn.
- **Long emulator runs** are the root's background jobs (`run_in_background` plus a Monitor), never a lane's. Use ladders and parallel segments, and keep each process under about 5 minutes.
- **Oracle accuracy** means accuracy for Super Metroid only.
- **Strats** come from sm-json-data-rs (the `sm-logic` crate and its packs), never raw JSON.
- **Defender** blocks fresh test executables here at random. CI's Quality job runs the rebuild crates' tests.
- **#177 edits:** run `gh` with `-R JimLundin/sm-decomp` and check the saved body isn't empty before editing (one edit wiped it; recovered from `userContentEdits`).
- **YAML step names:** quote any step name with `: ` in it. One broke the workflow file.
- **Encoding:** some lanes write Latin-1 (`§`, `×`). Check edited docs with `file`.
- **Pushes:** don't chain cleanup after a push with `;`. One push hit a GitHub error, and the commit was lost after the branch removal.
- **Orphans:** five lsnes processes (6176, 14972, 24536, 22000, 25512) can't be killed; a reboot clears them.
