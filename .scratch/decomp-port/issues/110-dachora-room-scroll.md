# Dachora room scrolling
Status: resolved
Blocked by: None
Triage: ready-for-agent

Translate native RoomCode_ScrollRightDachora with exact unsigned camera gates
and scroll byte comparison. Verify boundary behavior against actual C full WRAM,
then tests, cleanup and authentication. Live ownership remains separate.

## Completion evidence

Native unsigned camera gates and exact scroll byte implemented. All 96 actual-C
full-WRAM cases pass at out/port/dachora-room-scroll/20261011T095859.415077Z/verdict.json.
Independent test preserves native three-pixel overshoot from 037F to 0382 then
stops. All 15 room tests and strict Clippy pass. Ordinary cleanup zero; dedup
24379392 bytes; all cases authenticate afterward. Free disk 269063241728 bytes.
Retained binary/source/pinned ROM dependencies; no worktree created. Offline
only; live ownership separate. Integration branch decomp-port-integration.
