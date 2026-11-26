//! Native qualification of the attached door's prepared and uploaded OAM entries.

use super::{open_rows, read_lock};
use crate::replay::{ceres, landing_site};
use data::rom::Rom;
use game::exact::ridley_door;
use game::state::video::{Oam, SpriteCount, Video};

#[test]
#[ignore = "pinned record cache and player ROM"]
fn attached_ridley_door_appends_and_uploads_the_native_entries() {
    let sources = super::tests::sources();
    let rom = Rom::new(std::fs::read(&sources.rom).unwrap()).unwrap();
    let art = import::actors::attached_ridley_door(rom.lorom()).unwrap();
    let lock = read_lock(&sources).unwrap();
    let mut rows = open_rows(&sources, &lock, "chain-any-0006-E0B5").unwrap();

    // These component samples admit the incoming cursor from the native writer trace.
    // The full room draw produces it by appending each earlier sprite in order.
    for (cut, cursor, shake_x) in [
        (7, 5, 0),
        (682, 59, 0),
        (702, 58, 0),
        (1262, 42, 0),
        (1263, 39, -1),
        (1264, 41, -4),
    ] {
        let record = rows.record(cut).unwrap();
        let door = ceres::doors(&record.wram)
            .unwrap()
            .into_iter()
            .map(|slot| ceres::seed(&record.wram, slot).unwrap())
            .find(|door| door.kind == boss_ceres_ridley::DoorKind::RidleyDoor)
            .unwrap();
        assert!(door.phase.drawn_with_ridley());

        let mut video = Video::default();
        video.batch.sprites = SpriteCount(cursor);
        ridley_door::append(&mut video.batch, &art, door.position, shake_x);
        let first = usize::from(cursor);
        let end = first + 8;
        assert_eq!(video.batch.sprites, SpriteCount(cursor + 8), "cut {cut}");
        assert!(
            (first..end).all(|slot| {
                let low = video.batch.oam.low[slot];
                let start = 0x0370 + slot * 4;
                record.wram[start..start + 4] == [low.x, low.y, low.tile, low.attributes]
                    && high_bits(&video.batch.oam, slot)
                        == bits(record.wram[0x0570 + slot / 4], slot)
            }),
            "prepared door OAM at cut {cut}"
        );

        video.upload_oam_and_cgram();
        let native = landing_site::oam(&record).unwrap();
        assert!(
            video.memory.oam.low[first..end] == native.low[first..end]
                && (first..end)
                    .all(|slot| { high_bits(&video.memory.oam, slot) == high_bits(&native, slot) }),
            "physical door OAM at cut {cut}"
        );
    }
}

fn high_bits(oam: &Oam, slot: usize) -> u8 {
    bits(oam.high[slot / 4], slot)
}

fn bits(byte: u8, slot: usize) -> u8 {
    (byte >> ((slot % 4).wrapping_mul(2))) & 3
}
