//! The decoder against a synthetic record whose bytes sit where the bank logs say.
//!
//! The offsets here are written out again rather than taken from `source`, so a rule moved
//! to the wrong address fails these tests.

use data::id::RoomId;
use game::state::video::{OamEntry, VRAM_WORDS};
use game::state::{GameMode, NmiTiming, State};
use oracle_record::{CpuRegisters, Cut, FrameRecord, Nmi, NmiKind, Pad, PpuRegisters};

use crate::decode::{DecodeError, Memory, RoomHeader, RoomMap, Rules, Unsupported, decode};
use crate::source::{WramByte, WramWord};

/// The Morph Ball Room's header in bank $8F (bank $8F log).
const MORPH_BALL_ROOM: RoomHeader = RoomHeader(0x9E9F);
const MORPH_BALL_ROOM_ID: RoomId = RoomId(7);

fn rooms() -> RoomMap {
    RoomMap {
        rooms: vec![
            (RoomHeader(0x91F8), RoomId(1)),
            (MORPH_BALL_ROOM, MORPH_BALL_ROOM_ID),
        ],
    }
}

fn put(wram: &mut [u8], offset: usize, word: u16) {
    wram[offset..offset.wrapping_add(2)].copy_from_slice(&word.to_le_bytes());
}

/// A record in main gameplay with a distinct value at every decoded source.
fn record() -> FrameRecord {
    let mut wram = vec![0; 0x20000];
    put(&mut wram, 0x05B8, 0x1201); // NMI_FRAME_COUNTER_INCLUDING_LAG
    put(&mut wram, 0x05B6, 0x1102); // NMI_FRAME_COUNTER_WORD
    wram[0x05BA] = 0x03; // NMI_FRAMES_MISSED
    wram[0x05BB] = 0x04; // NMI_FRAMES_MISSED_MAX
    put(&mut wram, 0x09DA, 0x0005); // GAME_TIME_FRAMES
    put(&mut wram, 0x09DC, 0x0006); // GAME_TIME_SECONDS
    put(&mut wram, 0x09DE, 0x0007); // GAME_TIME_MINUTES
    put(&mut wram, 0x09E0, 0x0008); // GAME_TIME_HOURS
    put(&mut wram, 0x05E5, 0xBEEF); // RANDOM_NUMBER
    put(&mut wram, 0x008B, 0x8000); // JOYPAD1_LASTKEYS: B
    put(&mut wram, 0x008F, 0x4000); // JOYPAD1_NEWKEYS: Y
    put(&mut wram, 0x0093, 0x0080); // JOYPAD1_NEWKEYS2_UNUSED: A
    put(&mut wram, 0x0097, 0x0010); // JOYPAD1_PREV: R
    put(&mut wram, 0x00A3, 0x0009); // JOYPAD1_KEYREPEAT_CTR
    put(&mut wram, 0x0998, 0x0008); // GAME_STATE: main gameplay
    put(&mut wram, 0x0AF6, 0x0123); // SAMUS_X_POS
    put(&mut wram, 0x0AF8, 0x4567); // SAMUS_X_SUBPOS
    put(&mut wram, 0x0AFA, 0x0089); // SAMUS_Y_POS
    put(&mut wram, 0x0AFC, 0xABCD); // SAMUS_Y_SUBPOS
    put(&mut wram, 0x079B, MORPH_BALL_ROOM.0); // ROOM_PTR
    wram[0x05B4] = 0x01; // WAITING_FOR_NMI
    put(&mut wram, 0x09A2, 0x1004); // EQUIPPED_ITEMS: morph ball, bombs
    put(&mut wram, 0x09A4, 0x9006); // COLLECTED_ITEMS: spring ball, morph ball, bombs, X-ray
    put(&mut wram, 0x09C2, 0x0063); // SAMUS_HEALTH
    put(&mut wram, 0x09C4, 0x00C7); // SAMUS_MAX_HEALTH
    put(&mut wram, 0x09C6, 0x0004); // SAMUS_MISSILES
    put(&mut wram, 0x09C8, 0x0005); // SAMUS_MAX_MISSILES
    put(&mut wram, 0x09D4, 0x0064); // SAMUS_MAX_RESERVE_HEALTH
    put(&mut wram, 0x09D6, 0x0032); // SAMUS_RESERVE_HEALTH
    put(&mut wram, 0x090F, 0x1111); // LAYER1_X_SUBPOS
    put(&mut wram, 0x0911, 0x0200); // LAYER1_X_POS
    put(&mut wram, 0x0913, 0x2222); // LAYER1_Y_SUBPOS
    put(&mut wram, 0x0915, 0x0100); // LAYER1_Y_POS

    let vram: Vec<u16> = (0..=u16::MAX).step_by(2).collect();
    let cgram: Vec<u16> = (0..256_u16).map(|colour| colour.wrapping_mul(3)).collect();
    let mut oam = vec![0_u8; 0x220];
    oam[4..8].copy_from_slice(&[10, 20, 30, 40]);
    oam[0x21F] = 0xA5;
    FrameRecord {
        index: 100,
        video_frame: 101,
        pad: Pad(0x0F00), // Up, Down, Left, Right
        cut: Cut::NmiReturn(Nmi {
            kind: NmiKind::Requested,
            interrupted_pc: 0x80_8343,
            polled: true,
        }),
        joypad_reads: 1,
        wait_entry: None,
        overscan: false,
        cpu: CpuRegisters::default(),
        wram: wram.into_boxed_slice(),
        vram: vram.into_boxed_slice(),
        cgram: cgram.into_boxed_slice(),
        oam: oam.into_boxed_slice(),
        sram: vec![0; 0x2000].into_boxed_slice(),
        ppu: PpuRegisters::default(),
        apu_log: Vec::new(),
        picture: None,
        spc: None,
        bank_instructions: None,
        writes: Vec::new(),
        hits: Vec::new(),
        legacy_ppu_digest: 0,
    }
}

/// The fields the synthetic record sets, checked against the values it holds.
fn decodes_the_record(state: &State) -> bool {
    let counters = state.clock.counters;
    let time = state.clock.game_time;
    let controller = state.controller;
    let position = state.samus.position;
    let memory = &state.video.memory;
    let equipment = state.samus.equipment;
    let camera = state.room.camera;
    [
        (counters.nmi.0, 0x1201),
        (counters.completed.0, 0x1102),
        (counters.lag.0, 3),
        (counters.max_lag.0, 4),
        (time.frames, 5),
        (time.seconds, 6),
        (time.minutes, 7),
        (time.hours, 8),
        (state.rng.word, 0xBEEF),
        (controller.repeat_delay.0, 9),
        (position.x.pixels, 0x0123),
        (position.x.subpixels, 0x4567),
        (position.y.pixels, 0x0089),
        (position.y.subpixels, 0xABCD),
        (memory.vram[1], 2),
        (memory.cgram[5].0, 15),
        (equipment.energy.current, 0x0063),
        (equipment.energy.max, 0x00C7),
        (equipment.missiles.current, 4),
        (equipment.missiles.max, 5),
        (equipment.reserve.current, 0x0032),
        (equipment.reserve.max, 0x0064),
        (camera.x.pixels, 0x0200),
        (camera.x.subpixels, 0x1111),
        (camera.y.pixels, 0x0100),
        (camera.y.subpixels, 0x2222),
    ]
    .iter()
    .all(|(decoded, expected)| decoded == expected)
        // FlagSet bit n is the nth Button: B, Y, Select, Start, Up, Down, Left, Right, A, X, L, R.
        && [
            (controller.held.bits, 1 << 0),
            (controller.pressed.bits, 1 << 1),
            (controller.repeat.bits, 1 << 8),
            (controller.previous.bits, 1 << 11),
            (controller.auto_read.bits, 0b1111 << 4),
            // Item bit n is the nth Item: Varia, spring ball, morph ball, screw attack,
            // gravity, hi-jump, space jump, bombs, speed booster, grapple, X-ray.
            (equipment.equipped.bits, (1 << 2) | (1 << 7)),
            (equipment.collected.bits, (1 << 1) | (1 << 2) | (1 << 7) | (1 << 10)),
        ]
        .iter()
        .all(|(decoded, expected)| decoded == expected)
        && state.mode == GameMode::Gameplay
        && state.timing.last == NmiTiming::OnTime
        && state.video.nmi_requested
        && state.room.room == MORPH_BALL_ROOM_ID
        && memory.vram.len() == VRAM_WORDS
        && memory.oam.low[1]
            == OamEntry {
                x: 10,
                y: 20,
                tile: 30,
                attributes: 40,
            }
        && memory.oam.high[31] == 0xA5
}

#[test]
fn a_record_decodes_from_its_named_sources() {
    let state = decode(&record(), &rooms()).unwrap();
    assert!(decodes_the_record(&state), "{state:?}");
}

#[test]
fn undecoded_fields_keep_their_defaults() {
    let state = decode(&record(), &rooms()).unwrap();
    let defaults = State::default();
    assert_eq!(state.main, defaults.main);
    assert_eq!(state.samus.movement, defaults.samus.movement);
    assert_eq!(state.samus.facing, defaults.samus.facing);
    assert_eq!(state.timing.owed, defaults.timing.owed);
    assert_eq!(state.clock.tick, defaults.clock.tick);
    assert_eq!(state.rng.calls, defaults.rng.calls);
}

#[test]
fn a_rule_at_the_wrong_address_fails() {
    // The mutant reads Samus's X from the word after it: her subpixels.
    let mutant = Rules {
        samus_x: WramWord {
            offset: 0x0AF8,
            ..Rules::ORIGINAL.samus_x
        },
        ..Rules::ORIGINAL
    };
    let state = mutant.decode(&record(), &rooms()).unwrap();
    assert!(!decodes_the_record(&state));
}

/// A place in `Rules` holding a word rule.
type WordRule = fn(&mut Rules) -> &mut WramWord;
/// A place in `Rules` holding a byte rule.
type ByteRule = fn(&mut Rules) -> &mut WramByte;

/// Every word rule the synthetic record checks; `game_state` and `room` fail by error instead.
const WORD_RULES: [(&str, WordRule); 28] = [
    ("nmi", |rules| &mut rules.nmi),
    ("completed", |rules| &mut rules.completed),
    ("hours", |rules| &mut rules.hours),
    ("minutes", |rules| &mut rules.minutes),
    ("seconds", |rules| &mut rules.seconds),
    ("frames", |rules| &mut rules.frames),
    ("rng", |rules| &mut rules.rng),
    ("held", |rules| &mut rules.held),
    ("pressed", |rules| &mut rules.pressed),
    ("previous", |rules| &mut rules.previous),
    ("repeat", |rules| &mut rules.repeat),
    ("repeat_delay", |rules| &mut rules.repeat_delay),
    ("samus_x", |rules| &mut rules.samus_x),
    ("samus_x_subpixels", |rules| &mut rules.samus_x_subpixels),
    ("samus_y", |rules| &mut rules.samus_y),
    ("samus_y_subpixels", |rules| &mut rules.samus_y_subpixels),
    ("collected", |rules| &mut rules.collected),
    ("equipped", |rules| &mut rules.equipped),
    ("energy", |rules| &mut rules.energy),
    ("max_energy", |rules| &mut rules.max_energy),
    ("reserve", |rules| &mut rules.reserve),
    ("max_reserve", |rules| &mut rules.max_reserve),
    ("missiles", |rules| &mut rules.missiles),
    ("max_missiles", |rules| &mut rules.max_missiles),
    ("camera_x", |rules| &mut rules.camera_x),
    ("camera_x_subpixels", |rules| &mut rules.camera_x_subpixels),
    ("camera_y", |rules| &mut rules.camera_y),
    ("camera_y_subpixels", |rules| &mut rules.camera_y_subpixels),
];

/// Every byte rule.
const BYTE_RULES: [(&str, ByteRule); 3] = [
    ("lag", |rules| &mut rules.lag),
    ("max_lag", |rules| &mut rules.max_lag),
    ("nmi_requested", |rules| &mut rules.nmi_requested),
];

fn survives(mutant: &Rules) -> bool {
    mutant
        .decode(&record(), &rooms())
        .is_ok_and(|state| decodes_the_record(&state))
}

#[test]
fn every_moved_rule_fails() {
    for (name, rule) in WORD_RULES {
        let mut mutant = Rules::ORIGINAL;
        let word = rule(&mut mutant);
        word.offset = word.offset.wrapping_add(0x100);
        assert!(!survives(&mutant), "{name}");
    }
    for (name, rule) in BYTE_RULES {
        let mut mutant = Rules::ORIGINAL;
        let byte = rule(&mut mutant);
        byte.offset = byte.offset.wrapping_add(1);
        assert!(!survives(&mutant), "{name}");
    }
}

#[test]
fn an_overrun_nmi_decodes_as_overrun() {
    let mut record = record();
    record.cut = Cut::NmiReturn(Nmi {
        kind: NmiKind::Overrun,
        interrupted_pc: 0x82_8B44,
        polled: true,
    });
    let state = decode(&record, &rooms()).unwrap();
    assert_eq!(state.timing.last, NmiTiming::Overrun);
}

#[test]
fn a_cut_without_an_nmi_is_unsupported() {
    for cut in [
        Cut::VblankWithoutNmi,
        Cut::NmiStillRunning {
            interrupted_pc: 0x80_8343,
        },
    ] {
        let mut record = record();
        record.cut = cut;
        assert_eq!(
            decode(&record, &rooms()),
            Err(DecodeError::Unsupported(Unsupported::NoNmi(cut)))
        );
    }
}

#[test]
fn each_item_bit_is_its_own_item() {
    // The bank $84 log's item bits, each with its Item's declaration index.
    let items = [
        (0x0001, 0),  // Varia suit
        (0x0002, 1),  // spring ball
        (0x0004, 2),  // morph ball
        (0x0008, 3),  // screw attack
        (0x0020, 4),  // gravity suit
        (0x0100, 5),  // hi-jump boots
        (0x0200, 6),  // space jump
        (0x1000, 7),  // bombs
        (0x2000, 8),  // speed booster
        (0x4000, 9),  // grapple beam
        (0x8000, 10), // X-ray scope
    ];
    for (bit, index) in items {
        let mut record = record();
        put(&mut record.wram, 0x09A4, bit); // COLLECTED_ITEMS
        put(&mut record.wram, 0x09A2, bit); // EQUIPPED_ITEMS
        let equipment = decode(&record, &rooms()).unwrap().samus.equipment;
        assert_eq!(equipment.collected.bits, 1 << index, "{bit:#06X}");
        assert_eq!(equipment.equipped.bits, 1 << index, "{bit:#06X}");
    }
}

#[test]
fn unused_item_bits_are_unrepresentable() {
    for bit in [0x0010, 0x0040, 0x0080, 0x0400, 0x0800] {
        let mut record = record();
        put(&mut record.wram, 0x09A2, 0x0004 | bit); // EQUIPPED_ITEMS: morph ball and the bit
        assert_eq!(
            decode(&record, &rooms()),
            Err(DecodeError::Unrepresentable {
                source: "EQUIPPED_ITEMS",
                value: bit,
            })
        );
    }
}

#[test]
fn an_nmi_request_other_than_0_or_1_is_unrepresentable() {
    let mut record = record();
    record.wram[0x05B4] = 0x02; // WAITING_FOR_NMI
    assert_eq!(
        decode(&record, &rooms()),
        Err(DecodeError::Unrepresentable {
            source: "WAITING_FOR_NMI",
            value: 2,
        })
    );
}

#[test]
fn another_game_state_is_unsupported() {
    let mut record = record();
    put(&mut record.wram, 0x0998, 0x0006);
    assert_eq!(
        decode(&record, &rooms()),
        Err(DecodeError::Unsupported(Unsupported::GameState(6)))
    );
}

#[test]
fn an_unknown_room_is_an_error() {
    let mut record = record();
    put(&mut record.wram, 0x079B, 0x1234);
    assert_eq!(
        decode(&record, &rooms()),
        Err(DecodeError::UnknownRoom(RoomHeader(0x1234)))
    );
}

#[test]
fn short_memories_are_errors() {
    let mut short = record();
    short.wram = vec![0; 0x100].into_boxed_slice();
    assert!(matches!(
        decode(&short, &rooms()),
        Err(DecodeError::WramTooShort { .. })
    ));
    let mut short = record();
    short.oam = vec![0; 0x200].into_boxed_slice();
    assert!(matches!(
        decode(&short, &rooms()),
        Err(DecodeError::WrongSize {
            memory: Memory::Oam,
            ..
        })
    ));
}
