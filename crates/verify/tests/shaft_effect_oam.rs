//! Scoped proof of the effect spritemap encoder with a borrowed native origin.

use data::rom::Rom;
use game::intent::on_screen;
use game::state::video::NmiBatch;
use std::path::PathBuf;
use types::Pair;
use verify::cache::Cache;
use verify::lock::{Frame, Lock, RowId};

#[test]
#[ignore = "pinned record cache and player ROM"]
fn first_smoke_picture_matches_the_prepared_and_published_oam_prefix() {
    let path = |key| PathBuf::from(std::env::var_os(key).unwrap());
    let lock = Lock::parse(&std::fs::read_to_string(path("SM_LOCK")).unwrap()).unwrap();
    let row = lock.row(&RowId("chain-any-0010-DF8D".into())).unwrap();
    let cache = Cache::open(&path("SM_CACHE"), row).unwrap();
    let record = |cut| {
        let entry = cache
            .entries()
            .iter()
            .find(|entry| entry.frame == Frame(cut))
            .unwrap();
        cache.record(entry).unwrap()
    };
    let actor = record(241);
    let published = record(243);
    let word =
        |offset: usize| u16::from_le_bytes(actor.wram[offset..offset + 2].try_into().unwrap());
    let first = (0_usize..36)
        .step_by(2)
        .find(|slot| word(0x1997 + slot) == 0xe509)
        .unwrap();
    let origin = Pair {
        // At this cut the passage retains the actor shake of one pixel up and left.
        x: on_screen(word(0x1a4b + first), word(0x0911)).saturating_sub(1),
        y: on_screen(word(0x1a93 + first), word(0x0915)).saturating_sub(1),
    };

    let rom = Rom::new(std::fs::read(path("SM_ROM")).unwrap()).unwrap();
    let art = import::room_effects::shaft_burst(rom.lorom()).unwrap();
    let mut batch = NmiBatch::default();
    batch.draw_room_effect(&art.smoke[0].sprites, origin);
    let first = batch.oam.low[0];
    let prefix = [first.x, first.y, first.tile, first.attributes];
    assert_eq!(batch.sprites.0, 1);
    assert!(prefix == actor.wram[0x0370..0x0374], "prepared OAM");
    assert!(prefix != actor.oam[..4], "publication is still waiting");
    assert!(prefix == published.oam[..4], "published OAM");
    let high = batch.oam.high[0] & 3;
    assert!(high == actor.wram[0x0570] & 3, "prepared high OAM");
    assert!(high == published.oam[0x0200] & 3, "published high OAM");
}
