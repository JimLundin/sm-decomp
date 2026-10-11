# Native sprite object allocation
Status: claimed
Blocked by: None
Triage: ready-for-agent

Translate actual CreateSpriteAtPos four-word entry, descending free-slot search,
ROM instruction list/timer and exact writes. Extend the routine boundary to
preserve all four native arguments, verify full WRAM against actual C and prove
live ownership. Clean and authenticate evidence. Rendering remains follow-up.

## Work evidence

CreateSpriteAtPos implementation preserves descending free-slot choice, all four
native words, full-slot refusal, eight selected-slot fields and real ROM timer.
Explicit packed_u16_args=4 manifest contract maps native four uint16 arguments
losslessly into the existing two uint32 boundary in offline and live wrappers.
495 actual-C full-WRAM and return cases pass:
out/port/sprite-allocation/20261011T083712.414571Z/verdict.json.
Independent full-WRAM allocation/refusal test and strict all-target Clippy pass;
nine family harness tests pass. Cleanup --apply reclaimed zero bytes and
--deduplicate --apply reclaimed 127401984 bytes; oracle authenticates afterward.
No implementer worktree created. Disk approximately 259 GiB free.
Fresh full-movie four-process run:
out/port/sprite-allocation-live/20261011T083728.358255Z, session 90788.
Poll existing handle before restarting. Live result, cleanup and final evidence
authentication remain required. Rendering dependency identified: actual
DrawSpritemapWithBaseTile in sm_81.c writes OAM and extension bits.

Allocation live run still active, first C subject passed 105000 inputs with zero
CPU fallback. Rendering addition changed sprite_objects.rs after allocation
binary build; refreshing current-source allocation oracle. Existing live binary
remains frozen; retain original predecessor plus refreshed oracle separately.

First full-movie actual-C allocation subject completed with zero native allocator
calls. This trajectory cannot establish allocator live ownership; keep ticket
claimed. Current-source 495-case oracle:
out/port/sprite-allocation/20261011T084019.596381Z/verdict.json, authenticated
after ordinary zero-byte cleanup and dedup reclaiming 127401984 bytes.
Native callers exist in A5/A6/A7/A8/B3; targeted caller exercise is needed.
Existing four-process job remains active via session 90788; await terminal
status and clean retained diagnostic before resolving or changing admission.

Expanded actual-C allocation coverage to native caller indices including
Draygon/Ridley/Yapping Maw (3,9,21,24,29,43,44,45,46,61): 1485 cases pass.
Checked-in fixture generator tools/port_sprite_allocation_fixtures.py.
Current evidence out/port/sprite-allocation/20261011T084212.807053Z/verdict.json.
Full-movie zero allocator hits remains an explicit live coverage gap.

Expanded oracle cleanup --apply reclaimed zero bytes; --deduplicate --apply
reclaimed 390856704 bytes. All 1485 cases authenticate afterward. No worktree.
Live runs still active; final run cleanup and live scope classification pending.

## Completion evidence (offline scope; live requirement remains open)

Full-movie four-process run terminated rejected: no Rust gameplay family
executed. All four subjects have zero CreateSpriteAtPos calls; state/effect
comparison passed but cannot establish allocator ownership. Retained verdict:
out/port/sprite-allocation-live/20261011T083728.358255Z/verdict.json.
Finished run cleanup --apply and --deduplicate --apply each reclaimed zero;
automatic finished ROM/observation sharing reclaimed 4769723648 bytes.
Ticket remains claimed for targeted native caller/live allocator exercise.
Current 1485-case oracle remains authenticated. No implementer worktree created.

After cleanup all four diagnostic artifact sets authenticate (974 artifacts per
subject). Approximately 256 GiB remains free. Retain frozen binary, transformed
host/source, sealed observations and original/current allocation oracle.
