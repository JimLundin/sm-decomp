# Hand-off, 2026-11-01

This page is written for the next coordinator session. Start with `AGENTS.md` and
the `sm-coordinate`, `sm-land` and `sm-resources` skills. Then do the first steps below.

## Where things are

- **`main`:** `1e91c617c`; #138 and #139 landed after every exact-head gate passed.
- Complete RAM measure: **2,138 sites / 98 coupled files**, including 303 executable
  RAM-backed cohort-copy calls. The lowered baseline is queued with the next slice.
- #139 landed the scenario format, L5 batch 1, typed native-video verifier producers,
  corrected accounting and the Cue-trace dispatch option. Physical production grew
  by 372 lines; `sm_tools` is counted separately as verification.
- The rotating faithful-oracle sample is locally qualified: 30/30 scenarios,
  16,896 ticks, no digest changes. Hosted qualification is the next slice.
- Full Cue/port trace on main: run **41291863856**, pending. Its per-row results
  are required before checkpoint-policy or audio changes land.
- Fresh L5, L6, L4, D and C2 owners restarted in that order from the retained donor
  pins below, reconciling their local branches onto the explicit main pin.
- Dedicated coverage and low-movie coverage owners are active. Any% and 100% have
  room chains; low% has six focused capture rows and no committed room chain.
  The low capture has 186 room observations; boot and omitted transition scope
  need explicit coverage before full-movie completion can be claimed.

## First steps

1. Collect the locally qualified rotating-sample commits and lowered RAM baseline
   into the next small slice; run its exact-head gates and background watcher.
2. Replenish the active L5/L6/L4 implementation and D/C2 qualification owners.
   L4 retires RAM-backed copies toward sole typed Session ownership; its old donor
   savings use an incomplete measure and are not current ownership evidence.
3. Keep a dedicated gameplay coverage owner supplied with bounded tasks. Prove
   new gameplay facts against the faithful oracle and demonstrate bug sensitivity.
4. Cover the complete current any%, 100% and low% movie scope, including boot,
   omitted transitions and endgame. Passing selected room rows proves only those rows.
5. Collect the Cue run's per-row results before landing checkpoint-policy/audio work.

## Lane branches (all pushed to origin)

| Branch | State | Handoff |
| --- | --- | --- |
| `lane/l4-cohorts` (`f8863ac38`) | 13 commits, typed copies for scroll, camera, rooms, melt, liquids, message box and fade; RAM sites 1835 → 1496. Rebased on `170979ac2`. Gate on the rebased tip, check-lock, and commit 13's replays still to run | `docs/handoff/l4-cohorts.md` |
| `lane/l5-slot-access` (`22bcb8d48`) | batch 1 has landed. Batch 2 (10 families, 1809 sites) is gated but not replay-qualified. The top commit is a Crocomire WIP with 2 compile errors. 58 errors remain before `SlotAccess for Game` can go | `docs/handoff/l5-slot-access.md` |
| `lane/l6-hosts-kraid` (`910e393ba`, on L5's `0cc19efe8`) | 7 families: fire arc, boulders, statues, etecoons, dachora, Mini Kraid, Kraid. 1803 sites. The first 5 pass lockstep; Mini Kraid and Kraid not replayed; no check-lock | `docs/handoff/l6-hosts-kraid.md` |
| `lane/d-free-run-drift` (`00ef5d5b1`) | 3 engine fixes: a held button dismisses a message box; the escape timer and liquids leave `$16`. 26 listed drift rows now match the lock. Needs lockstep on both oracles, then remove their `unlocked` notes and `lock --record`. The top commit is a WIP oracle change for item-gravity, which needs a full lock re-record | `docs/handoff/d-free-run-drift.md` |
| `lane/c2-reseed` (`cd241c5b1`) | WIP: recipes and scenario seeds for the 6 stale checkpoint rows. Next: corpus entries, LEGACY rows, bundle upload, then the checkpoint policy guard `7048175e9` | `docs/handoff/c2-reseed.md` |

Native video step 5 (make the typed path the production path, deleting the `$21xx`
writes) has no branch yet: `docs/handoff/v2-native-video.md`. The finished lanes'
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
