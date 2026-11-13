use super::*;

#[test]
fn the_field_sets_name_the_declared_fields_at_their_places() {
    let place = |name| {
        HudDomain::FIELDS
            .iter()
            .position(|field| field.name == name)
            .map(FieldSet::single)
    };
    assert_eq!(place("status bar"), Some(STATUS_BAR));
    assert_eq!(place("values last shown"), Some(SHOWN));
    assert_eq!(place("selected item"), Some(SELECTED));
    assert_eq!(place("one-shot item"), Some(ONE_SHOT));
    assert_eq!(place("mini-map window"), Some(WINDOW_TILES));
    assert_eq!(place("explored cells"), Some(EXPLORED));
    assert_eq!(place("mini-map off"), Some(MINI_MAP_OFF));
}

/// Game state 8, gameplay.
const GAMEPLAY: u16 = 0x08;

/// A gameplay record whose work RAM also holds `words` and whose frame ran the instructions at
/// `hits`.
fn record(words: &[(WramWord, u16)], hits: &[u32]) -> FrameRecord {
    let mut record = crate::harness::tests::record(0, GAMEPLAY, [0; 4]);
    for &(source, value) in words {
        let at = usize::from(source.offset);
        record.wram[at..at + 2].copy_from_slice(&value.to_le_bytes());
    }
    record.hits = hits
        .iter()
        .map(|&pc| oracle_record::Hit {
            pc,
            beam: oracle_record::Beam { v: 0, h: 0 },
            peek: [0; 4],
        })
        .collect();
    record
}

#[test]
fn the_boss_that_blanks_the_map_is_named_by_its_id() {
    let called = |id| boss_called(&record(&[(BOSS_ID, id)], &[BOSS_BLANKS_THE_MAP]));
    assert_eq!(called(0x0A), Ok(Some(BossCalled::MotherBrain)));
    assert_eq!(
        called(0x07),
        Ok(Some(BossCalled::InItsRoom(Boss::Phantoon)))
    );
    assert_eq!(
        called(0x01),
        Ok(Some(BossCalled::InItsRoom(Boss::RidleyInCeres)))
    );
    assert!(called(0x02).is_err());
    assert_eq!(boss_called(&record(&[(BOSS_ID, 0x0A)], &[])), Ok(None));
}

#[test]
fn a_one_shot_item_is_the_selected_item_or_none() {
    let read = |selected, one_shot| {
        selection(&record(
            &[
                (source::HUD_ITEM_INDEX, selected),
                (source::SAMUS_AUTO_CANCEL_HUD_ITEM_INDEX, one_shot),
            ],
            &[],
        ))
    };
    assert_eq!(read(0, 0), Ok(Selection::None));
    assert_eq!(read(4, 0), Ok(Selection::Item(SelectableItem::GrappleBeam)));
    assert_eq!(
        read(4, 4),
        Ok(Selection::OneShot(SelectableItem::GrappleBeam))
    );
    assert_eq!(
        read(4, 2),
        Err(Skip::OneShotNotSelected {
            selected: 4,
            one_shot: 2
        })
    );
    assert!(matches!(read(0, 1), Err(Skip::OneShotNotSelected { .. })));
}

#[test]
fn a_boss_met_outside_play_is_the_boss_step_and_in_play_a_defect() {
    let domain = domain();
    let door = |words: &[(WramWord, u16)], hits| {
        let mut words = words.to_vec();
        words.push((source::GAME_STATE, 0x0B));
        record(&words, hits)
    };
    let met = domain.world(
        &door(&[], &[]),
        &door(&[(BOSS_ID, 0x03)], &[BOSS_BLANKS_THE_MAP]),
    );
    assert!(matches!(
        met.map(|world| world.turn),
        Ok(Turn::BossMet {
            boss: Boss::Kraid,
            ..
        })
    ));
    let in_play = domain.world(
        &record(&[], &[]),
        &record(
            &[(BOSS_ID, 0x03), (source::NMI_FRAME_COUNTER_WORD, 1)],
            &[BOSS_BLANKS_THE_MAP],
        ),
    );
    assert_eq!(
        in_play.map(|world| world.events),
        Err(Skip::BossInPlay(0x03))
    );
    assert!(matches!(
        domain.uncompared(&Skip::BossInPlay(0x03)),
        Uncompared::Defect(_)
    ));
}

/// The domain over no imported content.
fn domain() -> HudDomain {
    HudDomain {
        content: HudContent::default(),
        rules: Rules::reference(),
    }
}

#[test]
fn record_ns_game_state_picks_the_frame_of_play_the_step_outside_play_or_another_domain() {
    assert_eq!(Routine::of(0x08), Routine::Play);
    assert_eq!(Routine::of(0x13), Routine::Play);
    assert_eq!(Routine::of(0x24), Routine::OutsidePlay);
    assert_eq!(Routine::of(0x0B), Routine::OutsidePlay);
    assert_eq!(Routine::of(0x22), Routine::OutsidePlay);
    assert_eq!(Routine::of(0x1F), Routine::HudStartUp);
    assert_eq!(Routine::of(0x0F), Routine::Pause);
    let in_state = |code| record(&[(source::GAME_STATE, code)], &[]);
    let decoded = |code| domain().decode(&in_state(code), Interval::ending_at(&in_state(code)));
    assert_eq!(decoded(0x06).err(), Some(Skip::HudStartUp(0x06)));
    assert_eq!(decoded(0x0E).err(), Some(Skip::Paused(0x0E)));
    // Every HUD word is 0: no HUD yet, skipped by name, not a defect.
    assert_eq!(decoded(0x1E).err(), Some(Skip::NoHudYet));
}

/// A record in a door's passage (game state Ah) holding `words`, its frame running `hits`.
fn door(words: &[(WramWord, u16)], hits: &[u32]) -> FrameRecord {
    let mut words = words.to_vec();
    words.push((source::GAME_STATE, 0x0A));
    record(&words, hits)
}

/// The step outside play from `from` to `next`.
fn outside(from: &FrameRecord, next: &FrameRecord) -> Outside {
    match domain().world(from, next) {
        Ok(World {
            turn: Turn::OutOfPlay(outside),
            ..
        }) => outside,
        other => panic!("not a step outside play: {other:?}"),
    }
}

/// Sets the first byte of `block`, `at` bytes in, in `record`'s work RAM.
fn set_byte(record: &mut FrameRecord, block: WramBlock, at: usize) {
    if let Some(byte) = usize::try_from(block.offset)
        .ok()
        .and_then(|start| record.wram.get_mut(start.saturating_add(at)))
    {
        *byte = 1;
    }
}

#[test]
fn a_turn_on_read_at_05f7s_edge_is_unverified_and_a_watched_one_is_not() {
    let off = door(&[(source::DEBUG_DISABLE_MINIMAP, 1)], &[]);
    let on = door(&[], &[]);
    assert_eq!(outside(&off, &on).turned_on, TurnOn::AtTheEdge);
    assert_eq!(outside(&on, &on).turned_on, TurnOn::No);
    let watched = door(&[], &[TURNS_THE_MINI_MAP_ON[2]]);
    assert_eq!(outside(&off, &watched).turned_on, TurnOn::Watched);
    let neutral = |from, next| {
        let domain = domain();
        let world = domain.world(from, next).ok()?;
        domain.neutral(&world)
    };
    // The edge is the compared field: the stand-in without it doesn't turn the mini-map on.
    assert!(matches!(
        neutral(&off, &on).map(|world| world.turn),
        Some(Turn::OutOfPlay(Outside {
            turned_on: TurnOn::No,
            ..
        }))
    ));
    assert!(neutral(&off, &watched).is_none());
    assert!(neutral(&on, &on).is_none());
}

#[test]
fn a_new_areas_cells_are_progress_s_copy_in_record_n_not_record_n_plus_1s() {
    let on = door(&[], &[]);
    assert!(outside(&on, &on).new_area.is_none());
    let mut leaving = door(&[], &[]);
    // Brinstar's copy, area 1, holds one explored cell.
    set_byte(&mut leaving, EXPLORED_BY_AREA, MAP_BIT_BYTES);
    let mut brinstar = door(&[(source::AREA_INDEX, 1)], &[]);
    // Record N+1's current cells, the compared field, hold another.
    set_byte(&mut brinstar, source::MAP_TILES_EXPLORED, 1);
    let entered = outside(&leaving, &brinstar).new_area.map(|cells| *cells);
    assert_eq!(entered, explored_kept(&leaving, Area::Brinstar).ok());
    assert!(entered.is_some_and(|cells| cells != MapCellSet::default()));
    assert_ne!(entered, explored(&brinstar).ok());
}

#[test]
fn a_selection_cleared_by_no_decoded_writer_is_a_defect() {
    let with = |code, selected| {
        record(
            &[
                (source::GAME_STATE, code),
                (source::HUD_ITEM_INDEX, selected),
            ],
            &[],
        )
    };
    // A door's passage with the icon cancel option on drops it; that store isn't watched.
    let door = domain().world(&with(0x0B, 1), &with(0x0B, 0));
    assert_eq!(door.map(|world| world.events), Err(Skip::UndecodedDrop));
    assert!(matches!(
        domain().uncompared(&Skip::UndecodedDrop),
        Uncompared::Defect(_)
    ));
    // The station exploding with her aboard is decoded from its game state.
    let station = domain().world(&with(STATION_EXPLODES, 1), &with(STATION_EXPLODES, 0));
    assert!(matches!(
        station.map(|world| world.turn),
        Ok(Turn::OutOfPlay(Outside {
            dropped: Dropped::Selection,
            ..
        }))
    ));
}
