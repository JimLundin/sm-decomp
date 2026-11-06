//! The decoder against a synthetic record whose bytes sit where the bank logs say.
//!
//! The offsets here are written out again rather than taken from `source`, so a rule moved
//! to the wrong address fails these tests.

use data::id::{Line, RoomId};
use game::state::slots::Frames;
use game::state::strand::{
    Frame, GameplayAt, IrqCommand, IrqStrand, LagAt, MainLoopAt, MainStrand, MessageBoxAt,
    MessageWrite, PpuPreparation, PpuRestoration, Side, Wait,
};
use game::state::video::{OamEntry, VRAM_WORDS};
use game::state::{GameMode, NmiTiming, State};
use oracle_record::{CpuRegisters, Cut, FrameRecord, Nmi, NmiKind, Pad, PpuRegisters};

use crate::decode::{
    DecodeError, Memory, Quiescence, RoomHeader, RoomMap, Rules, Unsupported, decode,
};
use crate::source::{WramByte, WramWord};
use crate::strand::StrandRules;

/// The Morph Ball Room's header in bank $8F (bank $8F log).
const MORPH_BALL_ROOM: RoomHeader = RoomHeader(0x9E9F);
const MORPH_BALL_ROOM_ID: RoomId = RoomId(7);

/// The main thread's stack pointer in the synthetic record.
const MAIN_LOOP_S: u16 = 0x1FF0;

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
    put(&mut wram, 0x00AB, 0x0004); // CUR_IRQ_HANDLER: main gameplay, begin HUD drawing
    // The main loop's stack at Wait for NMI ($80:8338 PHP, PHB): S = $1FF0, then $1FF1 P,
    // $1FF2 DB, $1FF3-$1FF5 the JSL at $82:897A's return ($82:897D), $1FF6 the loop's P.
    wram[0x1FF3..0x1FF6].copy_from_slice(&[0x7D, 0x89, 0x82]);

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
        cpu: CpuRegisters {
            s: MAIN_LOOP_S,
            ..CpuRegisters::default()
        },
        wram: wram.into_boxed_slice(),
        vram: vram.into_boxed_slice(),
        cgram: cgram.into_boxed_slice(),
        oam: oam.into_boxed_slice(),
        sram: vec![0; 0x2000].into_boxed_slice(),
        ppu: PpuRegisters {
            vtime: 0x0021,
            ..PpuRegisters::default()
        },
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
        && state.main
            == MainStrand {
                frames: vec![Frame::MainLoop(MainLoopAt::Start)],
                waiting: Some(Wait::Nmi),
            }
        && state.irq
            == IrqStrand {
                command: IrqCommand::GameplayHud,
                line: Line(0x0021),
            }
}

#[test]
fn a_record_decodes_from_its_named_sources() {
    let state = decode(&record(), &rooms()).unwrap().state;
    assert!(decodes_the_record(&state), "{state:?}");
}

#[test]
fn undecoded_fields_keep_their_defaults() {
    let state = decode(&record(), &rooms()).unwrap().state;
    let defaults = State::default();
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
    let state = mutant.decode(&record(), &rooms()).unwrap().state;
    assert!(!decodes_the_record(&state));
}

/// A place in `Rules` holding a word rule.
type WordRule = for<'r> fn(&'r mut Rules<'static>) -> &'r mut WramWord;
/// A place in `Rules` holding a byte rule.
type ByteRule = for<'r> fn(&'r mut Rules<'static>) -> &'r mut WramByte;

/// Every word rule the synthetic record checks; `game_state` and `room` fail by error instead.
const WORD_RULES: [(&str, WordRule); 29] = [
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
    ("irq_command", |rules| &mut rules.irq_command),
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
        .is_ok_and(|decoded| decodes_the_record(&decoded.state))
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
    let decoded = decode(&record, &rooms()).unwrap();
    assert_eq!(decoded.quiescence, Quiescence::Busy);
    assert_eq!(decoded.state.main, MainStrand::default());
    let state = decoded.state;
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
        let equipment = decode(&record, &rooms()).unwrap().state.samus.equipment;
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

#[test]
fn a_requested_nmi_at_the_main_loop_is_quiet() {
    let decoded = decode(&record(), &rooms()).unwrap();
    assert_eq!(decoded.quiescence, Quiescence::Quiet);
}

#[test]
fn an_unknown_wait_pc_is_an_unknown_yield_site() {
    let mut record = record();
    record.cut = Cut::NmiReturn(Nmi {
        kind: NmiKind::Requested,
        interrupted_pc: 0x80_8348,
        polled: true,
    });
    assert_eq!(
        decode(&record, &rooms()),
        Err(DecodeError::UnknownYieldSite { pc: 0x80_8348 })
    );
}

#[test]
fn an_unknown_return_address_is_an_unknown_yield_site() {
    let mut record = record();
    record.wram[0x1FF3] = 0x7E; // the JSL's pushed address one byte on: $82:897E
    assert_eq!(
        decode(&record, &rooms()),
        Err(DecodeError::UnknownYieldSite { pc: 0x82_897F })
    );
}

#[test]
fn an_unknown_irq_command_is_unsupported() {
    let mut door = record();
    put(&mut door.wram, 0x00AB, 0x0008); // CUR_IRQ_HANDLER: door transition, begin HUD
    assert_eq!(
        decode(&door, &rooms()),
        Err(DecodeError::Unsupported(Unsupported::IrqCommand {
            current: 8,
            next: 0,
        }))
    );
    let mut queued = record();
    put(&mut queued.wram, 0x00A7, 0x0010); // IRQHANDLER_NEXT_HANDLER: a queued command
    assert_eq!(
        decode(&queued, &rooms()),
        Err(DecodeError::Unsupported(Unsupported::IrqCommand {
            current: 4,
            next: 0x10,
        }))
    );
}

/// A message box wait inside an item pickup, PLM slot 3, with the stack from S = $1FD0 up.
///
/// `inner` is the bytes from the wait up to the message box's own pushes, lowest first.
/// Above them, from the bank logs: display message box's PHY, PHX, PHB, PHP; the JSL at
/// $84:8912; process PLM's PEA $85ED; the JSR at $84:85CD; the PLM handler's PHB, PHP; the
/// JSL at $82:8B71; main gameplay's PHP; the JSR at $82:896B; the main loop's PHP.
fn message_box_record(pc: u32, inner: &[u8], radius: u16) -> FrameRecord {
    let mut record = record();
    let outer: [u8; 22] = [
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // PHY, PHX, PHB, PHP
        0x15, 0x89, 0x84, // JSL at $84:8912 pushed $84:8915
        0xED, 0x85, // PEA $85ED
        0xCF, 0x85, // JSR at $84:85CD pushed $85CF
        0x00, 0x00, // PHB, PHP
        0x74, 0x8B, 0x82, // JSL at $82:8B71 pushed $82:8B74
        0x00, // PHP
        0x6D, 0x89, // JSR at $82:896B pushed $896D
        0x00, // the main loop's PHP
    ];
    let start = 0x1FD1_usize;
    let middle = start.wrapping_add(inner.len());
    record.wram[start..middle].copy_from_slice(inner);
    record.wram[middle..middle.wrapping_add(outer.len())].copy_from_slice(&outer);
    put(&mut record.wram, 0x1C27, 0x0006); // PLM_ID: slot 3
    put(&mut record.wram, 0x05A2, radius); // MESSAGE_BOX_ANIMATION_Y_RADIUS
    record.cpu.s = 0x1FD0;
    record.cut = Cut::NmiReturn(Nmi {
        kind: NmiKind::Requested,
        interrupted_pc: pc,
        polled: true,
    });
    record
}

/// The input loop in $85:846D, called by the JSR at $85:80A4 (pushed $80A6).
fn shown_record() -> FrameRecord {
    message_box_record(0x85_84AA, &[0xA6, 0x80], 0x1800)
}

/// Wait for lag frame's PHP; the JSR at $85:859C (pushed $859E); $859B's PHP; then the
/// caller's JSR into $859B, then display message box's JSR.
fn animation_record(pc: u32, caller: [u8; 2], outer: [u8; 2], radius: u16) -> FrameRecord {
    let [caller_low, caller_high] = caller;
    let [outer_low, outer_high] = outer;
    message_box_record(
        pc,
        &[
            0x00,
            0x9E,
            0x85,
            0x00,
            caller_low,
            caller_high,
            outer_low,
            outer_high,
        ],
        radius,
    )
}

fn slot_3() -> game::state::slots::SlotIndex<game::state::room::Plm> {
    game::state::slots::SlotIndex(3, std::marker::PhantomData)
}

fn pickup(message_box: MessageBoxAt) -> MainStrand {
    MainStrand {
        frames: vec![
            Frame::MainLoop(MainLoopAt::AfterGameState),
            Frame::Gameplay(GameplayAt::AfterPlms),
            Frame::Plms { slot: slot_3() },
            Frame::MessageBox(message_box),
        ],
        waiting: Some(Wait::LagFrame),
    }
}

/// A wait for lag frame inside the message box: its PHP, then `chain`, the return addresses
/// and pushes up to display message box's own, with X at `x`.
fn lag_wait_record(chain: &[u8], x: u16) -> FrameRecord {
    let mut inner = vec![0x00];
    inner.extend_from_slice(chain);
    let mut record = message_box_record(0x85_813C, &inner, 0);
    record.cpu.x = x;
    record
}

/// The waits around the box's animation, each with its chain from the bank $85 log: a JSR
/// at $xxxx pushes $xxxx + 2, low byte first.
fn message_box_step_cases() -> Vec<(FrameRecord, MainStrand)> {
    let write = |step| pickup(MessageBoxAt::Writing(step));
    let lag = |at, left| {
        pickup(MessageBoxAt::Lag {
            at,
            left: Frames(left),
        })
    };
    vec![
        (
            // $85:8148 (pushed $814A) in initialise PPU, from $85:808D (pushed $808F).
            lag_wait_record(&[0x4A, 0x81, 0x8F, 0x80], 0),
            pickup(MessageBoxAt::PreparingPpu(PpuPreparation::Registers)),
        ),
        (
            // $85:81B2 (pushed $81B4).
            lag_wait_record(&[0xB4, 0x81, 0x8F, 0x80], 0),
            pickup(MessageBoxAt::PreparingPpu(PpuPreparation::TilemapBackup)),
        ),
        (
            // $85:8203 (pushed $8205) in clear tilemap, from $85:8090 (pushed $8092).
            lag_wait_record(&[0x05, 0x82, 0x92, 0x80], 0),
            pickup(MessageBoxAt::ClearingTilemap(Side::BeforeOpen)),
        ),
        (
            // The same, from $85:80AA (pushed $80AC).
            lag_wait_record(&[0x05, 0x82, 0xAC, 0x80], 0),
            pickup(MessageBoxAt::ClearingTilemap(Side::AfterClose)),
        ),
        (
            // $85:82B8 (pushed $82BA) in write message tilemap; $85:829D (pushed $829F);
            // $85:8250 (pushed $8252); initialise message box's PHX before it; $85:8093.
            lag_wait_record(
                &[0xBA, 0x82, 0x9F, 0x82, 0x52, 0x82, 0x34, 0x12, 0x95, 0x80],
                0,
            ),
            write(MessageWrite::Tilemap),
        ),
        (
            // $85:8321 (pushed $8323); $85:843D (pushed $843F); $85:8254 (pushed $8256).
            lag_wait_record(&[0x23, 0x83, 0x3F, 0x84, 0x56, 0x82, 0x95, 0x80], 0),
            write(MessageWrite::TilemapUpload),
        ),
        (
            // $85:859C (pushed $859E); $859B's PHP; $85:83BA (pushed $83BC); $85:831E (pushed
            // $8320); then as above.
            lag_wait_record(
                &[
                    0x9E, 0x85, 0x00, 0xBC, 0x83, 0x20, 0x83, 0x3F, 0x84, 0x56, 0x82, 0x95, 0x80,
                ],
                0,
            ),
            write(MessageWrite::ScrollHdma),
        ),
        (
            // $85:8578 (pushed $857A) in play 2 lag frames, from $85:8096 (pushed $8098):
            // X = 2 is the first of two.
            lag_wait_record(&[0x7A, 0x85, 0x98, 0x80], 2),
            lag(LagAt::BeforeOpen, 1),
        ),
        (
            // The same, from $85:80B4 (pushed $80B6): X = 1 is the last.
            lag_wait_record(&[0x7A, 0x85, 0xB6, 0x80], 1),
            lag(LagAt::AfterClose, 0),
        ),
        (
            // $85:8493 (pushed $8495) in the interaction, from $85:80A4 (pushed $80A6): the
            // first of 360.
            lag_wait_record(&[0x95, 0x84, 0xA6, 0x80], 360),
            lag(LagAt::Shown, 359),
        ),
        (
            // $85:861C (pushed $861E) in restore PPU, from $85:80AD (pushed $80AF).
            lag_wait_record(&[0x1E, 0x86, 0xAF, 0x80], 0),
            pickup(MessageBoxAt::RestoringPpu(PpuRestoration::Tilemap)),
        ),
        (
            // $85:8651 (pushed $8653).
            lag_wait_record(&[0x53, 0x86, 0xAF, 0x80], 0),
            pickup(MessageBoxAt::RestoringPpu(PpuRestoration::Registers)),
        ),
    ]
}

fn strand_cases() -> Vec<(FrameRecord, MainStrand)> {
    let mut cases = vec![
        (
            record(),
            MainStrand {
                frames: vec![Frame::MainLoop(MainLoopAt::Start)],
                waiting: Some(Wait::Nmi),
            },
        ),
        (shown_record(), pickup(MessageBoxAt::Shown)),
        (
            // Open's JSR at $85:8451 (pushed $8453), from the JSR at $85:8099 (pushed $809B).
            // From radius $400, nine more loop frames ($600-$1600) and the last at $1800.
            animation_record(0x85_813C, [0x53, 0x84], [0x9B, 0x80], 0x0400),
            pickup(MessageBoxAt::Opening { left: Frames(10) }),
        ),
        (
            // Open's last JSR at $85:8469 (pushed $846B).
            animation_record(0x85_813F, [0x6B, 0x84], [0x9B, 0x80], 0x1800),
            pickup(MessageBoxAt::Opening { left: Frames(0) }),
        ),
        (
            // Close's JSR at $85:858B (pushed $858D), from the JSR at $85:80A7 (pushed
            // $80A9). From radius $600: $400, $200 and 0 to come.
            animation_record(0x85_813F, [0x8D, 0x85], [0xA9, 0x80], 0x0600),
            pickup(MessageBoxAt::Closing { left: Frames(3) }),
        ),
    ];
    cases.extend(message_box_step_cases());
    cases
}

#[test]
fn each_stack_shape_decodes_to_its_strand() {
    for (record, strand) in strand_cases() {
        let decoded = decode(&record, &rooms()).unwrap();
        assert_eq!(decoded.state.main, strand);
        assert_eq!(decoded.quiescence, Quiescence::Quiet);
    }
}

fn strands_survive(mutant: &Rules) -> bool {
    strand_cases().into_iter().all(|(record, strand)| {
        mutant
            .decode(&record, &rooms())
            .is_ok_and(|decoded| decoded.state.main == strand)
    })
}

#[test]
fn every_moved_wait_site_or_call_fails() {
    let original = Rules::ORIGINAL.strand;
    for (index, site) in original.sites.iter().enumerate() {
        let moved: Vec<u32> = site.loop_pcs.iter().map(|pc| pc.wrapping_add(1)).collect();
        let mut sites = original.sites.to_vec();
        sites[index].loop_pcs = moved.leak();
        let mutant = Rules {
            strand: StrandRules {
                sites: &sites,
                ..original
            },
            ..Rules::ORIGINAL
        };
        assert!(!strands_survive(&mutant), "{}", site.routine);
    }
    for (index, call) in original.calls.iter().enumerate() {
        let mut calls = original.calls.to_vec();
        calls[index].returns_to = call.returns_to.wrapping_add(1);
        let mutant = Rules {
            strand: StrandRules {
                calls: &calls,
                ..original
            },
            ..Rules::ORIGINAL
        };
        assert!(!strands_survive(&mutant), "{}", call.routine);
    }
}

#[test]
fn a_moved_plm_index_or_radius_fails() {
    let original = Rules::ORIGINAL.strand;
    let moved = [
        StrandRules {
            plm_index: WramWord {
                offset: 0x1C29,
                ..original.plm_index
            },
            ..original
        },
        StrandRules {
            radius: WramWord {
                offset: 0x05A4,
                ..original.radius
            },
            ..original
        },
    ];
    for strand in moved {
        let mutant = Rules {
            strand,
            ..Rules::ORIGINAL
        };
        assert!(!strands_survive(&mutant));
    }
}

#[test]
fn a_radius_off_the_animation_is_unrepresentable() {
    let record = animation_record(0x85_813C, [0x53, 0x84], [0x9B, 0x80], 0x0401);
    assert_eq!(
        decode(&record, &rooms()),
        Err(DecodeError::Unrepresentable {
            source: "MESSAGE_BOX_ANIMATION_Y_RADIUS",
            value: 0x0401,
        })
    );
}

#[test]
fn a_countdown_outside_its_range_is_unrepresentable() {
    // X = 0 would count 65,536 frames and X = 361 more than the interaction's 360: no count
    // the original starts.
    for (chain, x) in [
        ([0x95, 0x84, 0xA6, 0x80], 0),
        ([0x95, 0x84, 0xA6, 0x80], 361),
        ([0x7A, 0x85, 0x98, 0x80], 3),
    ] {
        assert_eq!(
            decode(&lag_wait_record(&chain, x), &rooms()),
            Err(DecodeError::Unrepresentable {
                source: "cpu.x",
                value: x,
            })
        );
    }
}
