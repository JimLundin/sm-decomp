Lane L2 handoff (root, from L2's final report, 2026-11-01)
State: sm_game_ram_sites 3187 -> ~1863 (honest view counting).
- Batch 1 is on main.
- Batch 2 (menu state, inventory, flow, the lease observation refresh, the cinematics' shared words, palette fades, Mother Brain's shots) is on land/batch13 / PR slice T.
Mechanism: crates/sm_game/src/session_cohort.rs.
- A function loads one typed copy of a group of Session words (Cohort), works on the fields, then stores only the changed words once.
- With a retained Session current, the store is one lease and calls SessionAccess::refresh_inputs. Otherwise it goes to the projection as the old views did.
- Verify-only guard: session_cohort::check_not_held, a bit set, checked from projection byte access and with_projectiles.
Views deleted: death_sequence, map_screen, save_selection, options_menu, power_bomb, session_word, menu, inventory, and all but 3 cinematic! sites. flow::set/current remain only for the layout test and sm's HUD host.
Left:
- cinematic!: 3 sites, all DoorExit = 0. It is a world key, so it needs a Session API rather than a cohort field.
- scroll!: 85 sites. The BG scroll words are Session fields only while the presentation is Owned. Needs a presentation-aware cohort, or a move onto leases.
- boss!: 224 sites, mostly RAM outside the Session (extended $9000, registers, scratch). Belongs with the boss lease work.
- Untouched: melt!, room_view, camera::current, screen_fade.
- The pause screen nests two copies (MenuState outer, MenuScreens inner), costing two leases per frame retained. Fold them into one pair copy.
Traps:
- Nothing may reach a held copy's words by another path while it is held. The guard does NOT cover raw ram(), word_transfers!, or lease-side reads. Before converting, grep the function and everything it calls for raw addresses in the cohort's range. Examples so far:
  - Samus's frame reads the cinematic table as enemy projectiles.
  - The save file transfer reads the save selection.
  - The pause sprite animations work by RAM address, now mapped via MenuState::word_at.
- Cohort words that are ExternalInputs need refresh_inputs; store() already calls it.
Evidence pattern per commit:
- lockstep on the touched rows;
- engine-only check-lock (verify_dll.py --corpus movies/segments.toml --mode check-lock --lock scenarios/lock.toml --segment ...) on the locked rows touched;
- clean back-to-back timing for intro and credits when cost could change.
Scripts: $TEMP/collab/laneL2/replay.sh, checklock.sh, cin_rewrite.py, menu_rewrite.py; corpus root $TEMP/collab/laneL2/corpus.
