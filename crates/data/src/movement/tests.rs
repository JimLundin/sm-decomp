use super::*;
use crate::rom::Rom;

/// Words as the ROM stores them.
fn bytes(words: &[u16]) -> Vec<u8> {
    words.iter().flat_map(|word| word.to_le_bytes()).collect()
}

#[test]
fn a_speed_keeps_its_pixel_and_subpixel_words() {
    let speed = Speed::new(0x0002, 0xC000);
    assert_eq!(speed, Speed(0x0002_C000));
    assert_eq!((speed.pixels(), speed.subpixels()), (0x0002, 0xC000));
}

#[test]
fn an_entry_decodes_word_by_word() {
    // Standing, then running: the bank $90 log's first two rows.
    let table = SpeedTable::decode(&bytes(&[
        0x0000, 0xC000, 0x0000, 0x0000, 0x0000, 0x8000, // standing
        0x0000, 0x3000, 0x0002, 0xC000, 0x0000, 0x8000, // running
        0x0001, // a partial entry, ignored
    ]));
    assert_eq!(
        table.of(MovementType::Running),
        Some(&SpeedEntry {
            acceleration: Speed::new(0, 0x3000),
            max: Speed::new(2, 0xC000),
            deceleration: Speed::new(0, 0x8000),
        })
    );
    assert_eq!(table.of(MovementType::NormalJumping), None);
}

#[test]
fn the_extra_run_constants_take_the_air_column() {
    let mut words = [0_u16; 18];
    // Increments, booster caps, caps: pixels for air, water, lava, then their subpixels.
    words[3] = 0x1000;
    words[6] = 7;
    words[12] = 2;
    words[4] = 0x0400; // the water column, not read
    let raw = bytes(&words);
    let extra = ExtraRun::decode(raw.first_chunk().unwrap());
    assert_eq!(
        extra,
        ExtraRun {
            acceleration: Speed::new(0, 0x1000),
            booster_max: Speed::new(7, 0),
            max: Speed::new(2, 0),
        }
    );
}

fn rom() -> Option<Rom> {
    let path = std::env::var_os("SM_ROM").map_or_else(
        || std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../rom/sm.smc"),
        std::path::PathBuf::from,
    );
    let image = std::fs::read(path).ok()?;
    Some(Rom::new(image).unwrap())
}

#[test]
fn the_speed_tables_decode_as_the_bank_log_says() {
    let Some(cartridge) = rom() else {
        return;
    };
    let speeds = HorizontalSpeeds::load(cartridge.lorom()).unwrap();
    let entry = |movement| *speeds.normal.of(movement).unwrap();
    assert_eq!(
        entry(MovementType::Running),
        SpeedEntry {
            acceleration: Speed::new(0, 0x3000),
            max: Speed::new(2, 0xC000),
            deceleration: Speed::new(0, 0x8000),
        }
    );
    assert_eq!(
        entry(MovementType::TurningAroundOnGround),
        SpeedEntry {
            acceleration: Speed::new(0, 0xC000),
            max: Speed::ZERO,
            deceleration: Speed::new(0, 0x8000),
        }
    );
    assert_eq!(
        entry(MovementType::DamageBoost).max,
        Speed::new(5, 0),
        "the last normal row"
    );
    assert_eq!(
        speeds.extra_run,
        ExtraRun {
            acceleration: Speed::new(0, 0x1000),
            booster_max: Speed::new(7, 0),
            max: Speed::new(2, 0),
        }
    );
}

/// The bank $90 log's physics constants from `$90:9EA1` to `$90:9EE9`.
const PHYSICS_WORDS: [u16; 36] = [
    0x1C00, 0x0800, 0x0900, 0x0000, 0x0000, 0x0000, // gravity: subpixels, then pixels
    0x0001, 0x0000, 0x0001, 0x0000, // the camera's turning offsets, not read
    0x0001, 0x0000, // the morph ball's bounce, not read
    0x0004, 0x0001, 0x0002, 0xE000, 0xC000, 0xC000, // jumping
    0x0006, 0x0002, 0x0003, 0x0000, 0x8000, 0x8000, // with hi-jump
    0x0004, 0x0000, 0x0002, 0xA000, 0x4000, 0xA000, // wall-jumping
    0x0005, 0x0000, 0x0003, 0x8000, 0x8000, 0x8000, // with hi-jump
];

fn the_logs_vertical_speeds() -> VerticalSpeeds {
    VerticalSpeeds {
        gravity: Speed::new(0, 0x1C00),
        jump: Speed::new(4, 0xE000),
        hi_jump: Speed::new(6, 0),
        wall_jump: Speed::new(4, 0xA000),
        wall_hi_jump: Speed::new(5, 0x8000),
    }
}

#[test]
fn the_vertical_constants_take_the_air_column() {
    let raw = bytes(&PHYSICS_WORDS);
    let speeds = VerticalSpeeds::decode(raw.first_chunk().unwrap());
    assert_eq!(speeds, the_logs_vertical_speeds());
}

#[test]
fn the_vertical_constants_decode_as_the_bank_log_says() {
    let Some(cartridge) = rom() else {
        return;
    };
    let speeds = VerticalSpeeds::load(cartridge.lorom()).unwrap();
    assert_eq!(speeds, the_logs_vertical_speeds());
}
