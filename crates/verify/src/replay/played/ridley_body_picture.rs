//! Native qualification of the ordinary extended body operator.

use boss_ceres_ridley::RidleyLook;
use core::num::Wrapping;
use data::rom::Rom;
use game::exact::ridley_picture;
use game::state::video::{Oam, SpriteCount, Video};
use types::{Pair, Side};

use super::{open_rows, read_lock};
use crate::replay::landing_site;

#[test]
#[ignore = "pinned record cache and player ROM"]
fn ordinary_ridley_body_appends_native_prepared_and_physical_entries() {
    let sources = super::tests::sources();
    let rom = Rom::new(std::fs::read(&sources.rom).unwrap()).unwrap();
    let art = import::ridley_picture::import(rom.lorom()).unwrap();
    let record_lock = read_lock(&sources).unwrap();
    let mut rows = open_rows(&sources, &record_lock, "chain-any-0006-E0B5").unwrap();

    // The native writer trace independently attributes these contiguous slot ranges.
    for (cut, first, count) in [
        (577, 22, 26),
        (675, 14, 29),
        (682, 25, 29),
        (702, 24, 29),
        (866, 21, 26),
        (867, 13, 26),
        (870, 15, 25),
        (874, 15, 25),
        (879, 13, 25),
        (880, 14, 25),
        (881, 10, 25),
        (1027, 7, 26),
        (1028, 17, 26),
    ] {
        let record = rows.record(cut).unwrap();
        let word = |at: usize| {
            let [lo, hi] = *record.wram[at..].first_chunk::<2>().unwrap();
            u16::from_le_bytes([lo, hi])
        };
        let look = match word(0x0f8e) {
            0xe983 => RidleyLook::Rest,
            0xe9c7 => RidleyLook::Open,
            0xe9e9 => RidleyLook::Wide,
            0xea4f => RidleyLook::TakeOff1,
            0xea71 => RidleyLook::TakeOff2,
            _ => panic!("unknown look at cut {cut}"),
        };
        let cycle = word(0x0e44);
        let palette = if word(0x0f9c) != 0 && cycle & 2 != 0 {
            0
        } else {
            7
        };
        let origin = Pair {
            x: i32::from(
                word(0x0f7a)
                    .wrapping_sub(word(0x0911))
                    .wrapping_add(word(0x7010))
                    .cast_signed(),
            ),
            y: i32::from(
                word(0x0f7e)
                    .wrapping_sub(word(0x0915))
                    .wrapping_add(word(0x7012))
                    .cast_signed(),
            ),
        };
        if matches!(cut, 682 | 702) {
            strict_parts_match_native(&art, &record, word, cut);
        }
        let mut video = Video::default();
        video.batch.sprites = SpriteCount(first);
        ridley_picture::body(&mut video.batch, &art.left, look, origin, palette);
        assert_eq!(video.batch.sprites, SpriteCount(first + count), "cut {cut}");

        let span = usize::from(first)..usize::from(first + count);
        let prepared = &record.wram[0x0370..0x0590];
        for slot in span.clone() {
            assert!(
                prepared_matches(prepared, &video.batch.oam, slot),
                "prepared extended body OAM at cut {cut}, slot {slot}"
            );
        }
        video.upload_oam_and_cgram();
        let physical = landing_site::oam(&record).unwrap();
        for slot in span {
            assert!(
                low(&video.memory.oam, slot) == low(&physical, slot)
                    && high(&video.memory.oam, slot) == high(&physical, slot),
                "physical extended body OAM at cut {cut}, slot {slot}"
            );
        }
    }
}

fn strict_parts_match_native(
    art: &data::assets::ridley_picture::Art,
    record: &oracle_record::FrameRecord,
    word: impl Fn(usize) -> u16,
    cut: u64,
) {
    let mut video = Video::default();
    let camera = Pair {
        x: types::px(Wrapping(word(0x0911))),
        y: types::px(Wrapping(word(0x0915))),
    };
    let tail = core::array::from_fn(|draw_index| {
        let from_base = 6_usize.wrapping_sub(draw_index);
        let x = 0x202c_usize.wrapping_add(from_base.wrapping_mul(20));
        Pair {
            x: Wrapping(word(x)),
            y: Wrapping(word(x.wrapping_add(2))),
        }
    });
    let tip = word(0x208e).wrapping_add(word(0x20a2)).wrapping_add(8);
    let tip = usize::from((tip >> 4) & 15);
    let palette = u8::try_from(word(0x7818) >> 9).unwrap();
    ridley_picture::tail(&mut video.batch, art, tip, tail, camera, palette).unwrap();
    let wing_anchor = Pair {
        x: Wrapping(word(0x8804).wrapping_add(16)),
        y: Wrapping(word(0x8808).wrapping_sub(22)),
    };
    assert_eq!(word(0x7820), 0, "left-facing sample at cut {cut}");
    ridley_picture::wings(
        &mut video.batch,
        art,
        Side::Left,
        usize::from(word(0x780e)),
        ridley_picture::relative(wing_anchor, camera),
        palette,
    )
    .unwrap();
    assert_eq!(
        video.batch.sprites,
        SpriteCount(13),
        "strict parts at cut {cut}"
    );

    let prepared = &record.wram[0x0370..0x0590];
    let physical = landing_site::oam(record).unwrap();
    video.upload_oam_and_cgram();
    for slot in 0..13 {
        assert!(
            prepared_matches(prepared, &video.batch.oam, slot),
            "prepared strict part OAM at cut {cut}, slot {slot}"
        );
        assert!(
            low(&video.memory.oam, slot) == low(&physical, slot)
                && high(&video.memory.oam, slot) == high(&physical, slot),
            "physical strict part OAM at cut {cut}, slot {slot}"
        );
    }
}

fn prepared_matches(image: &[u8], oam: &Oam, slot: usize) -> bool {
    let offset = slot.wrapping_mul(4);
    let high_table = 512_usize.wrapping_add(slot / 4);
    image[offset..offset.wrapping_add(4)] == low(oam, slot)
        && bits(image[high_table], slot) == high(oam, slot)
}

fn low(picture: &Oam, slot: usize) -> [u8; 4] {
    let entry = picture.low[slot];
    [entry.x, entry.y, entry.tile, entry.attributes]
}

fn high(picture: &Oam, slot: usize) -> u8 {
    bits(picture.high[slot / 4], slot)
}

fn bits(byte: u8, slot: usize) -> u8 {
    (byte >> (slot % 4).wrapping_mul(2)) & 3
}
