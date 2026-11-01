# Lane L4 handoff (2026-11-01)

Branch `laneL4/cohorts`, tip `f8863ac38`, rebased cleanly onto origin/land/batch13 `170979ac2`.
sm_game_ram_sites: 1835 (base) -> 1496 (tip). Coupled files: 99 -> 93.

## Commits (oldest first; ratchet change from the first rebase, 1842 base)
1. 9026b5ab4 Menus: the pause frame holds MenuState and MenuScreens as one copy, so it opens one lease, not two. Ratchet 0 (task 3, a cost change).
2. 2ea382898 Menus: the BG scroll mirrors become a BgScroll cohort (-58).
3. a9de544e8 Cinematics: BgScroll copies (-17).
4. bcb8d006e Bosses and room entry: BgScroll copies; the scroll view and scroll! go [exact] (-23).
5. 471faae91 Cinematics: room_entry::clear_exit Session API for DoorExit; the cinematic! view goes (-3).
6. ffdc291a9 Display: a ScreenFade cohort and screen_fade::store (-66).
7. d3b21af27 Camera: Camera and Layer1 cohorts; camera::current goes [exact] (-25).
8. a93899f49 Rooms: RoomGeometry and RoomEffects cohorts; the room_view accessors go (-69).
9. 7fdda391e Crocomire: a Melt cohort; melt! goes [exact] (-48).
10. f06626e73 Liquids: a LiquidWords cohort; read, set and write go [exact] (-17).
11. ec0c759a4 Message boxes: a MessageBox cohort; the message_box view goes (-7).
12. dab93bf10 Tests: check_encoding for the new cohorts [exact] (0).
13. f8863ac38 Kraid, scrolling sky, demo loader: BgScroll instead of REG_BG word!s (-6).

## Qualified (on the pre-rebase hashes; each patch is unchanged by the rebases)
- GATE OK: every commit, at its own hash, and at the 377fb815f-based tip 67ca3e145. Commit 13 has GATE OK at e4f0379fa.
- Lockstep, all PASS with 0 failing frames and no guard panics:
  - At ed3e569a7 (commits 1-7), 18 rows: intro, credits-ending, pause, options, file-copy, save-memory, save-death-reload, boss-crocomire plus 2 melt rows, ceres-ridley, draygon, phantoon, mb-escape-door, door-transition, tas-ceres-four-doors, pb-room-entry, kraid-pause-400.
  - At 67ca3e145 (commits 1-12), 18 rows: intro, credits-ending, pause, file-copy, both save rows, crocomire plus 2 melt rows, volcano-lavaquake, area-maridia, area-tourian, item-morph-ball, item-wave, elevator-traversal, xray-scan-91f8, draygon, mb-escape-door.
- Check-lock at ed3e569a7: 21/21 PASS. Rows: samus-ceres-push, door-transition, ridley-fireballs, file-copy, gunship-save, options, phantoon-2nd, pb-room-entry, area-ceres, ceres-ridley, credits-ending, credits, death, draygon-grab, draygon, area-norfair, mb-escape-door, boss-mother-brain, phantoon, ridley, square-slope-reserve.
- Earlier check-lock PASS at b6a22256e and at b32698a89 (the menus and death rows).

## Pending (not run, or stopped by the wind-down)
- GATE on the rebased tip f8863ac38. qc (sm_game in both cfgs) and fmt pass; clippy and the sm crate have not run.
- Check-lock at 67ca3e145 was stopped before its first result. Rows: area-*, item-wave, elevator, pb-warehouse, xray-scan-91f8/96ba, draygon, mother-brain, mb-glass, credits, gunship-save, file-copy, options, death, golden-original-wait, botwoon.
- Commit 13 has no replays: boss-kraid-pause-*, demo-attract-setup, area-crateria (sky).
- Back-to-back intro/credits/pause timing, base vs tip: not run.
  - Single-run walls are within noise. pause-menu-tour: 31.6s base, 32.6s, 28.2s.
  - Watch cost: BgScroll loads 16 projected bytes. The cinematic transform loads it once a frame (it used to store 4 bytes).
- Corpus: --corpus <root>/scenarios. My root $TEMP/collab/laneL4/corpus has movies from laneL3-base and seeds from laneF3. Scripts: replay.sh, checklock.sh, timing.sh.

## Mechanism change (session_cohort::store)
- The retained-lease store matches Session words by RAM address.
- Words the Session does not hold right now go through the lease's SessionRam bytes, which keep it current. BgScroll's session_words visits nothing unless the presentation is Owned.
- Hosts without RuntimeAccess use with_projected: bosses, the typewriter, room-effect and liquid init, the dialogue publish.

## Next
- boss! (224): RAM outside the Session. It belongs with the boss lease work.
- Other REG_BG word!s: liquid_effects HDMA reads, room_scrolls (lease), xray_background. A per-layer scroll cohort would keep the per-frame loads small.
- Optional: drop the deleted views' names from the cycle_report labels.

## Traps
- Never hold a copy across code that writes its words by another path. The room selection writes BG1's scroll through the lease's camera Mirror, so the map menu's initialize_room opens its scroll copy only after selection::begin. If it is held across, a store of an unchanged value is lost.
- The pause save_graphics and restore_graphics read the scroll mirrors with raw ram(). Never hold BgScroll across them.
- PIXELS_TO_ERASE ($068C) is inside the melt range but is reached by slot byte. Melt leaves it out.
- The mutation anchors in session_mutations.py follow the code: swap_room_statue_words now targets room_view.rs.
- Heredocs collapse backslashes. Write scripts that contain \n with the Write tool.
