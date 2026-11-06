//! The decoder against a synthetic record whose bytes sit where the bank logs say.
//!
//! The offsets here are written out again rather than taken from `source`, so a rule moved
//! to the wrong address fails these tests.

use data::id::RoomId;
use game::state::video::{OamEntry, VRAM_WORDS};
use game::state::{GameMode, State};
use oracle_record::{CpuRegisters, Cut, FrameRecord, Nmi, NmiKind, Pad, PpuRegisters};

use crate::decode::{DecodeError, Memory, RoomHeader, RoomMap, Rules, Unsupported, decode};
use crate::source::WramWord;

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
        ]
        .iter()
        .all(|(decoded, expected)| decoded == expected)
        && state.mode == GameMode::Gameplay
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
    assert_eq!(state.room.camera, defaults.room.camera);
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

#[test]
fn every_moved_rule_fails() {
    let original = Rules::ORIGINAL;
    let moved = |word: WramWord| WramWord {
        offset: word.offset.wrapping_add(0x100),
        ..word
    };
    let mutants = [
        Rules {
            nmi: moved(original.nmi),
            ..original
        },
        Rules {
            completed: moved(original.completed),
            ..original
        },
        Rules {
            hours: moved(original.hours),
            ..original
        },
        Rules {
            minutes: moved(original.minutes),
            ..original
        },
        Rules {
            seconds: moved(original.seconds),
            ..original
        },
        Rules {
            frames: moved(original.frames),
            ..original
        },
        Rules {
            rng: moved(original.rng),
            ..original
        },
        Rules {
            held: moved(original.held),
            ..original
        },
        Rules {
            pressed: moved(original.pressed),
            ..original
        },
        Rules {
            previous: moved(original.previous),
            ..original
        },
        Rules {
            repeat: moved(original.repeat),
            ..original
        },
        Rules {
            repeat_delay: moved(original.repeat_delay),
            ..original
        },
        Rules {
            samus_x: moved(original.samus_x),
            ..original
        },
        Rules {
            samus_x_subpixels: moved(original.samus_x_subpixels),
            ..original
        },
        Rules {
            samus_y: moved(original.samus_y),
            ..original
        },
        Rules {
            samus_y_subpixels: moved(original.samus_y_subpixels),
            ..original
        },
    ];
    for mutant in mutants {
        let survived = mutant
            .decode(&record(), &rooms())
            .is_ok_and(|state| decodes_the_record(&state));
        assert!(!survived, "{mutant:?}");
    }
    let lag = Rules {
        lag: crate::source::WramByte {
            offset: original.lag.offset.wrapping_add(1),
            ..original.lag
        },
        ..original
    };
    assert!(!decodes_the_record(
        &lag.decode(&record(), &rooms()).unwrap()
    ));
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
