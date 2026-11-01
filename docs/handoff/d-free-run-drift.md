# Lane D handoff: free-run drift against the faithful oracle lock (2026-11-01)

Branch `laneD/free-run-drift` on origin/land/batch13 377fb815f; worktree
`C:\Users\Jim.Lundin\AppData\Local\sm-decomp-experiments\20261031\laneD`.
GATE OK at tip 9f055e9cf. Ratchets: ram sites 1842 and coupled files 99, both at baseline.
Not yet run: default and faithful lockstep over the corpus and routes for these
commits (only free run and check-lock were run). Run them before landing.

## Commits (engine only, so no lock entry moves)
- 5d15924eb Message boxes: a held button dismisses the box once its delay ends.
  LOOP_INPUT $85:84A3 reads the raw joypad ($4218/9). The engine tested NEWKEYS
  whenever snesrev's bug-fix counter $C2 was 0. Lockstep hid it, because it adopts
  the oracle's RAM, where $C2 is 1. This was the shared item/pickup drift.
- 3b77e7212 Escape timer draw leaves $16=0A00 ($80:9FCA). A door-closing PLM takes
  $16 as its room argument (PLM room-arg high byte, $1E0C).
- 9f055e9cf Raise/lower FX ($88:868C) leaves $16=(velocity lo)<<8, for the same
  door-closing argument ($1E10).

## Rows now matching the faithful lock running alone (check-lock, local)
These rows still carry their `unlocked` note. Remove the note and run
`tools/scenario.py lock --record`:
- item: bombs, grapple, ice, morph-ball, plasma, screw-attack, space-jump, spazer,
  speed-booster, spring-ball, varia, x-ray
- pickup: energy, missile, power-bomb, super, reserve
- samus-grapple-connect-swing, space-jump-bubble-word-counter, xray-activate-release
- boss-kraid, ice-return-business, music-fir-selector10
- route: chain-0330-92FD, chain-any-0196-92FD (timer fix); chain-any-0079-ACF0 (FX fix)

## item-gravity: WIP oracle fix, not committed
Patch: `wip-faithful-oracle-keeps-c2.patch` (sm/src/rtl.rs). The faithful oracle
stops raising $C2 to 1, raising it only while $85:9650 keeps snesrev's variant
(`!follows_original(Rom, 0x859650)`). Cause: at tick 721 a stale HDMA ch3 table
(free space at $88:A233, count FF, indirect $FFFF..) reads $7E:00C1/C2 into
BG3HOFS. Hardware has the seed's $C2 (0), and the oracle has 1.
With the patch, item-gravity matched (old-format lock in lock-c2-oracle-11rows.toml).
Next (root, condition 8): in the same commit, re-record every lock entry whose
digests move. Expected: only item-gravity, because every locked entry already
matched an engine with $C2 at its seed value. A full re-record was started and then
stopped at wind-down. To redo it, shard all 630 locked ids over
`verify_dll --corpus scenarios --mode lock --oracle faithful --lock TMP` and
compare with scenarios/lock.toml ($TEMP/collab/agentD/cmp.py does the compare).
Then update LEGACY: the $00C2 row (faithful no longer writes it) and the lock table rows.

## Still open
- item-hi-jump @767, chain-0086-AA41 @95, chain-any-0074-AFA3 @316: one tick of ppu
  (WH3 $2129, window 2 right). HDMA ch5 (B=$29, indirect, bank 7E) has its table at
  $89:0000, the WRAM mirror of direct-page scratch $7E:0000-0008. That range differs
  between the engine and the oracle (lockstep normalizes 0x0000..0x0051). Ch4 B=$28
  table is at $89:A77F. Root: a real gameplay-invisible divergence, not a verifier
  artefact. Do not mask it. List it in LEGACY with the routine and tick, and keep
  these rows unlocked. Still to do: name the HDMA object that leaves ch5's table at
  $0000 (read $18B4.. HDMA object arrays at that tick).
- boss-kraid-pause-1960: the first divergence is now @2148 (wram, vram, ppu), and
  faithful lockstep fails there too (VRAM port/VRAM). Move it to the kraid-pause
  faithful class (root agreed).
- boss-crocomire @1801: not run. Re-test it with these commits.
## Tools and traps
- Do not edit .rs files while a corpus runs (the source-hash check aborts it).
- debug-dump-and-watch.patch (local only): SM_DBG_DUMP=dir with SM_DBG_FROM/TO dumps
  WRAM and PPU/DMA per tick and side; SM_DBG_WATCH=frame prints oracle writes to $16/$17.
