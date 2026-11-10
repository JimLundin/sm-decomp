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

/// A gameplay record whose work RAM also holds `words` and whose frame ran the instructions at
/// `hits`.
fn record(words: &[(WramWord, u16)], hits: &[u32]) -> FrameRecord {
    let mut record = crate::harness::tests::record(0, MAIN_GAMEPLAY, [0; 4]);
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
        Ok(Some(BossCalled::InItsRoom(RoomBoss::Phantoon)))
    );
    assert_eq!(
        called(0x01),
        Ok(Some(BossCalled::InItsRoom(RoomBoss::RidleyInCeres)))
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
    let domain = HudDomain {
        content: HudContent::default(),
        rules: Rules::reference(),
    };
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
            boss: RoomBoss::Kraid,
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
