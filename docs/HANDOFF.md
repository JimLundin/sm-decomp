# Hand-off, 2026-11-01

This page is written for the next coordinator session. Start with `AGENTS.md` and
the `sm-coordinate`, `sm-land` and `sm-resources` skills. Then do the first steps below.

## Where things are

- **`main`:** `74a984d5e`; #138 through #142 landed with their exact-head gates green.
- Complete RAM measure: **2,109 sites / 96 coupled files**. The lowered baseline
  is queued beside the qualified Metroid coverage pair in the next slice.
- #139 landed the scenario format, L5 batch 1, typed native-video verifier producers,
  corrected accounting and the Cue-trace dispatch option. Physical production grew
  by 372 lines; `sm_tools` is counted separately as verification.
- #140 landed the rotating faithful sample; #141 landed Metroid latch/drain and
  Botwoon spit. Both coverage scenarios caught deliberate gameplay mutants.
- **#142 landed at 12:24:02 UTC:** twelve qualified L5 commits, head `74a984d5e`.
  Local gate, CI **41303910705**, corpus **41303916057**, route **41303910279**
  and lock **41303910244** passed. Watcher **88871** is terminal EXIT0; do not
  restart it. This slice measures **2,109 sites / 96 files**, physical
  production **179,299** (+8), ratchet production **178,892** (+8).
- Next coverage commits are picked after #142 on the landing branch: `d725ed328`,
  `55f557469`, `e581ce2f9`. Metroid ice/missile shatter and latch/bomb release pass
  both oracles, faithful free run and engine locks. Mutants fail at ticks 49 and
  706. Encrypted bundle `coverage-v2-metroid-interactions.tar.gz.enc` is uploaded;
  ciphertext SHA `851545e4084779da2b662a4311c681a45d303bb66176dbd5fffbf09a68140d1b`.
  Its 18 restored members match. Composition gate/hosted qualification remain.
- Fresh dedicated coverage owner `verification_grapple_qualification` is reviewing
  repaired guards and preparing proof of airborne zero-speed terrain selection.
  Genuine reach and natural apex are unproved; its predecessor
  brief is `$TEMP/collab/verification-next/SUCCESSOR.md`. Keep this lane supplied
  with fresh bounded owners; report reached gameplay and bug sensitivity separately.
- Full Cue/port run **41291863856**: 154/160 rows passed; six failed after zero
  differing frames. Safe diagnostic patches are reconciled onto `74a984d5e` in
  `20261101/cue-diag-qual`; gate/build/focused physical-audio replay remain pending.
- Full any/100 inputs are genuinely imported and hash-qualified: **115,542** and
  **199,259** native ticks. Full any patched replay is live under owner
  `full_any_replay`, exec **81000**, corpus PID **20200**, supervisor **15140**;
  20,000 ticks passed, no differences. Frozen source is `33519e8e...`, matching
  qualified #141 DLL `834065f5...`. Full EOF/facts/continuation proof remains open.
- Low% has a full **153,266**-tick input and 186 captured room observations.
  Faithful boot faults at 7892/7952 require hardware fixes. Fresh owner
  `hardware_writer_qualification` is qualifying the existing last-writer diagnostic;
  source `3ed035ca...`, direct DLL exec **44613**. Preserve original inputs/ROM.
- L6 source/DLL are frozen at `747bdb51...` / `0734dcb...`: 68 patched rows passed;
  faithful locked-remaining 57 is live (exec **71281**, PID **23836**), then 62
  engine locks. Kraid pause1030 failed at 1218 identically on pre-L6 L5, matching
  LEGACY; excluded pause rows are not green faithful proof. L4/D/C2/V2 await grants.
- Botwoon and Draygon lease source patches are ready, not qualified. Their exact
  bases are `74a984d5e`; handoffs are `$TEMP/collab/l5-{botwoon,draygon}/HANDOFF.md`.
  Botwoon source review found no concrete regression; Draygon review is active.

## First steps

1. Land the ready Metroid pair and lowered RAM baseline with a fresh exact-head
   composition gate, hosted gates and background watcher. #142 is already landed;
   its ratchet update completed. Report each landing to Jim.
2. Replenish the active L5/L6/L4 implementation and D/C2 qualification owners.
   L4 retires RAM-backed copies toward sole typed Session ownership; its old donor
   savings use an incomplete measure and are not current ownership evidence.
3. Keep a dedicated gameplay coverage owner supplied with bounded tasks. Prove
   new gameplay facts against the faithful oracle and demonstrate bug sensitivity.
4. Cover the complete current any%, 100% and low% movie scope, including boot,
   omitted transitions and endgame. Passing selected room rows proves only those rows.
5. Qualify the prepared safe Cue diagnostics and rerun the six failed rows before
   landing checkpoint-policy/audio work. Preserve the completed per-row inventory.

## Resource recovery and current handles

Root removed 5,313 explicit regenerable compiler files from 54 completed historical
roots, recovering 21.16 GiB; source, DLLs, native archives, seeds and receipts remain.
Removal exec **92172** is terminal EXIT0; receipt is
`$TEMP/collab/resource-cache-inventory/removed-qualified-cache.csv`. Disk reached
54.08 GiB and is about 53.6 GiB after new work. PLAN's 50 GiB reserve applies.
Do not repeat cleanup or delete external hardlinks/junctions excluded by its manifest.

Hardware has the sole new DLL compiler grant: 3.25 GiB family RAM, 1.25 GiB combined
new disk; full-any replay is 200 MiB / 64 MiB new outputs, L6 replay 768 MiB / 16 MiB
logs. All coverage mutant/final-gate families are terminal. Its first final gate
stopped at the 300 MiB disk allowance; expanded gate **87690/24884** passed.
Fresh-sample actual handles/resources before successor admission; files alone do not
prove liveness. `.ps1` file execution is disabled; use ordinary native commands or
the permitted Python launchers without changing security settings.

## Retained origin/lane donors

These are the original retained donor pins; use the current owners and frozen
handoffs above for live qualification and new source, preserving donor patches.

| Branch | State | Handoff |
| --- | --- | --- |
| `lane/l4-cohorts` (`f8863ac38`) | 13 commits, typed copies for scroll, camera, rooms, melt, liquids, message box and fade; RAM sites 1835 → 1496. Rebased on `170979ac2`. Gate on the rebased tip, check-lock, and commit 13's replays still to run | `docs/handoff/l4-cohorts.md` |
| `lane/l5-slot-access` (`22bcb8d48`) | batch 1 has landed. Batch 2 (10 families, 1809 sites) is gated but not replay-qualified. The top commit is a Crocomire WIP with 2 compile errors. 58 errors remain before `SlotAccess for Game` can go | `docs/handoff/l5-slot-access.md` |
| `lane/l6-hosts-kraid` (`910e393ba`, on L5's `0cc19efe8`) | 7 families: fire arc, boulders, statues, etecoons, dachora, Mini Kraid, Kraid. 1803 sites. The first 5 pass lockstep; Mini Kraid and Kraid not replayed; no check-lock | `docs/handoff/l6-hosts-kraid.md` |
| `lane/d-free-run-drift` (`00ef5d5b1`) | 3 engine fixes: a held button dismisses a message box; the escape timer and liquids leave `$16`. 26 listed drift rows now match the lock. Needs lockstep on both oracles, then remove their `unlocked` notes and `lock --record`. The top commit is a WIP oracle change for item-gravity, which needs a full lock re-record | `docs/handoff/d-free-run-drift.md` |
| `lane/c2-reseed` (`cd241c5b1`) | WIP: recipes and scenario seeds for the 6 stale checkpoint rows. Next: corpus entries, LEGACY rows, bundle upload, then the checkpoint policy guard `7048175e9` | `docs/handoff/c2-reseed.md` |

Native video step 5 has source commit `38051cfd9` in `20261101/v2-native-video`;
its Samus typed VRAM sink still needs compilation/replays and remaining consumers.
See `docs/handoff/v2-native-video.md` and `$TEMP/collab/v2-native-video-20261101/HANDOFF.md`.
The finished lanes'
handoffs in `docs/handoff/*-done.md`, `f1-`, `o-` and `t-` explain those subsystems.

## Open goals, in priority order

1. **Real decoupling.** Finish moving the actor families onto the lease (L5, L6), delete
   `SlotAccess for Game`, and land L4's typed copies. Measure: `ram_ratchet.py`.
2. **Verification.** Land D's fixes and unlock their rows; aim for every scenario locked.
   Harness step 5: retire whole-RAM comparison domain by domain (REVIEW lane S).
3. **Native video step 5,** then audio and the remaining endgame items in
   `docs/ENDGAME_SURVEY.md`.
4. **Coverage.** Run a coverage pass (`coverage.yml`, full_route), then add rows (`sm-scenario`).
5. **Later:** the sm-json-data importer (REVIEW lane S, condition 9), which also supplies the
   community room names Jim wants shown next to hex ids.

## Jim's standing decisions

- The real game (hardware) is the authority. snesrev patches move to `follows`. A
  difference is fixed, or listed in `docs/LEGACY.md`, with no tolerances.
- Engine-only free run against the oracle digest lock is the frequent gate. Lockstep
  stays for locating faults (REVIEW condition 8).
- Production line count is advisory for now; the RAM ratchets are strict.
- Idiomatic Rust, official names, no lint `allow`s.
- Never commit ROM-derived bytes or JSON. Encrypted bundles on ci-data are
  pre-approved; unencrypted ROM data is never uploaded.
- Land small: at most about 25 commits per slice, and land the green prefix.
- Use fresh agents per task; retire them at task end with a handoff.
- The coordinator may edit REVIEW and PLAN to record Jim's decisions. Ask Jim before
  posting to #6, except result reports.

## Machine

- Repo: `C:\Users\Jim.Lundin\Documents\Projects\sm-decomp`.
- Worktrees: `C:\Users\Jim.Lundin\AppData\Local\sm-decomp-experiments\<date>\<lane>`.
- The landing worktree is `...\20261029\collab-8b`.
- Python: `C:\Users\Jim.Lundin\AppData\Local\sm-decomp-tools\python-3.13.7\python.exe`.
- Toolchain: rustup `stable-x86_64-pc-windows-gnu`.
- ROM: `rom\sm.smc` in the repo (gitignored).
- Test executables can't run locally; CI runs them.
- Disk: about 50 GB free; see `sm-resources`.
- `$TEMP\collab\` holds lane scratch: replay and tally scripts, restored inputs and
  seeds. It isn't needed to continue, but saves time.
