Lane T4 handoff (2026-11-01)
State: 6 new rows on laneT4/coverage (worktree ...\20261031\laneT4). All pass lockstep (patched and
faithful), free run and check-lock; lock_gate.py check reports 630 locked, 41 unlocked.
- Batch 1, picked into land/batch13 (bundle seeds-v1-handwings-6302cac975d0049c): terrain-wrecked-ship-hand,
  enemy-kihunter-wings, enemy-baby-metroid-remorse, enemy-etecoon-tunnel-jump.
- Batch 2: d40dfb8f3 probes, e9aa17acb rows, cdcb655a0 enable (bundle seeds-v1-acidhand-263f82ef4a26cef1):
  terrain-lower-norfair-hand, enemy-etecoon-run-left.
Facts:
- The WS hand is in C98E (Bowling Alley, seed 211, block 74,23), not CAAE.
- The LN hand trigger appears at block (4,8) of B1E5 after a power bomb clears its BTS 09 blocks. The
  morph pose must face right (0x1D); facing left (0x41) never activates.
- Scenario enemy edits work. Set the instruction list as well as the function, otherwise the oracle
  crashes (CpuOpcodeHook assert).
- Dachora: every function is covered. Its left-facing start, wall bump and left-facing spark landing
  can't happen in 9CB3 (the facing comes from room data; sparks only start facing right).
- Etecoon hit_ceiling: not reached in 4000 released frames from seed 43.
Tools in $TEMP/collab/laneT4:
- tr.sh: chain prefix plus recipe, with a positions dump.
- corp.sh / mksub.sh: subset corpus runs in each mode.
- ins.py: probe insertion by line number.
- mergelock.py: sorted union of lock entries.
- laneT4_plmscan.py / laneT4_btsscan.py (in ..\): find PLMs and special blocks in seeds.
- enem.py: enemy slots. Enemy AI vars are at +0x30; param1 is at +0x3C, param2 at +0x3E.
