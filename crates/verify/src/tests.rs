//! The decoder against a synthetic record whose bytes sit where the bank logs say.
//!
//! The offsets here are written out again rather than taken from `source`, so a rule moved
//! to the wrong address fails these tests.

use crate::enemy::AiVariables;
use data::enemy::SpeciesMap;
use data::enemy::program::{EnemyPrograms, Spritemap, SpritemapId};
use data::id::{Line, RoomId, SpeciesId};
use data::plm::{Extension, ItemPlm, PlmEntry, PlmKind, PlmPopulation, ScrollChangesId};
use data::pose::tiles::{SamusTiles, TileDefinition, TileDefinitionId};
use data::rom::{Lorom, RomAddress};
use data::room::{Area, MapPosition};
use game::enemy::instructions::label;
use game::state::room::{
    Block, BlockIndex, Crumble, ElevatorArming, ElevatorDirection, ElevatorRide, ElevatorStatus,
    Enemy, EnemyBody, EnemyCommon, EnemyPaletteCycle, GlowSet, GraphicsDrawnHook, ItemStep,
    LayerBlending, PLM_SLOTS, PaletteIndex, PlmBody, PlmCommon, ScrollArea, ScrollPlm, ScrollStep,
    SpritePalette, TilesIndex,
};
use game::state::room::{BlockPoint, DrawnBlocks};
use game::state::samus::{
    Acceleration, Animation, AnimationFrame, AnimationTimer, DrawingHandler, Facing, Horizontal,
    Jump, Movement, MovementType, NewPoseFrame, PoseId, PoseRecord, Speed, SuperSpecial,
    SuperSpecialCommand, Vertical, Visor, VisorShade, VisorTimer, YDirection,
};
use game::state::samus::{Fixed, Point};
use game::state::slots::{Cursor, FlagSet, Frames, Generation, Label, Occupancy, Slot};
use game::state::strand::{
    Frame, GameplayAt, IrqCommand, IrqStrand, LagAt, MainLoopAt, MainStrand, MessageBoxAt,
    MessageWrite, PpuPreparation, PpuRestoration, Side, Wait,
};
use game::state::video::{
    LayerUploads, OamEntry, SamusTileUploads, TilemapEntry, TilemapPart, UploadRequests,
    VRAM_WORDS, VramWord,
};
use game::state::{GameMode, NmiTiming, State};
use oracle_record::{CpuRegisters, Cut, FrameRecord, Nmi, NmiKind, Pad, PpuRegisters};

use crate::decode::{
    DecodeError, LayerUploadSources, Memory, Quiescence, RoomHeader, RoomMap, Rules, StateHeader,
    StateLevel, Unsupported, decode,
};
use crate::source::{SlotField, WramArray, WramBlock, WramByte, WramWord};
use crate::strand::StrandRules;

/// The Morph Ball Room's header in bank $8F (bank $8F log).
const MORPH_BALL_ROOM: RoomHeader = RoomHeader(0x9E9F);
const MORPH_BALL_ROOM_ID: RoomId = RoomId(7);
/// The synthetic room's state header, and what the content says it loads.
const MORPH_BALL_STATE: StateHeader = StateHeader(0x9ECB);
const MORPH_BALL_LEVEL: StateLevel = StateLevel {
    tileset: data::tileset::TilesetId(8),
    layer2: true,
};

/// The main thread's stack pointer in the synthetic record.
const MAIN_LOOP_S: u16 = 0x1FF0;

fn rooms() -> RoomMap {
    RoomMap {
        rooms: vec![
            (RoomHeader(0x91F8), RoomId(1)),
            (MORPH_BALL_ROOM, MORPH_BALL_ROOM_ID),
        ],
        species: SpeciesMap::slice_zero(),
        programs: programs(),
        samus_tiles: samus_tiles(),
        plms: vec![(MORPH_BALL_ROOM_ID, population())],
        states: vec![(MORPH_BALL_STATE, MORPH_BALL_LEVEL)],
    }
}

/// Three tiles definitions, at pointers 0, 7 and 14.
fn samus_tiles() -> SamusTiles {
    SamusTiles::new(Vec::new(), vec![TileDefinition::default(); 3])
}

/// The synthetic slots' instruction lists: the eye's in bank $A8, one frame then sleep, and
/// the elevator's in bank $A3, a frame then back.
fn programs() -> EnemyPrograms {
    let mut image = vec![0; 0x15_0000];
    let mut lists = |bank: u8, offset: u16, words: &[u16]| {
        let start = (usize::from(bank & 0x7F) << 15) | usize::from(offset & 0x7FFF);
        for (at, word) in (start..).step_by(2).zip(words) {
            image[at..][..2].copy_from_slice(&word.to_le_bytes());
        }
    };
    lists(0xA8, 0x9000, &[1, EYE_SPRITEMAP, 0x812F]);
    lists(0xA3, 0x9400, &[2, ELEVATOR_SPRITEMAP, 0x80ED, 0x9400]);
    EnemyPrograms::load(
        Lorom::new(&image),
        &[RomAddress::new(0xA8, 0x9000), RomAddress::new(0xA3, 0x9400)],
        &[],
    )
    .unwrap()
}

/// Slot 0's AI bank, instruction pointer (its sleep) and spritemap.
const EYE_BANK: u16 = 0x00A8;
const EYE_INSTRUCTION: u16 = 0x9004;
const EYE_SPRITEMAP: u16 = 0x9210;
/// Slot 0's activated flag (its third AI word) and function (its sixth): the eye's words
/// hold only these.
const EYE_ACTIVATED: u16 = 1;
/// `$A8:912E`, activating.
const EYE_ACTIVATING: u16 = 0x912E;
/// Slot 1's AI bank, instruction pointer (its go-to) and spritemap.
const ELEVATOR_BANK: u16 = 0x00A3;
const ELEVATOR_INSTRUCTION: u16 = 0x9404;
const ELEVATOR_SPRITEMAP: u16 = 0x962F;

/// The cursor place and spritemap the content gives a slot's pointers.
fn pointers(bank: u16, instruction: u16, spritemap: u16) -> (Label<Enemy>, SpritemapId) {
    let [bank, _] = bank.to_le_bytes();
    let programs = programs();
    (
        label(programs.target(RomAddress::new(bank, instruction)).unwrap()),
        programs
            .spritemap_id(Spritemap(RomAddress::new(bank, spritemap)))
            .unwrap(),
    )
}

/// Enemy slot 0 in the synthetic record (bank $A0 log: `enemy_data` at $0F78, $40 a slot).
const ENEMY_SLOT_0: usize = 0x0F78;
/// Enemy slot 1.
const ENEMY_SLOT_1: usize = 0x0FB8;
/// The Morph Ball Room eye's header, `SpeciesMap::slice_zero`'s species 0.
const EYE: u16 = 0xE6BF;
/// The elevator's header, species 1.
const ELEVATOR: u16 = 0xD73F;
/// Slot 1's properties: deleted ($0200) and solid to Samus ($8000).
const DELETED_PROPERTIES: u16 = 0x8200;

/// An enemy slot whose every word past the species is `base` plus its offset, so a field
/// read from a neighbouring offset, or a neighbouring slot, reads another value; but its
/// spritemap, instruction pointer and AI bank (`+16`, `+1A`, `+2E`) are `pointers`.
fn put_enemy(wram: &mut [u8], slot: usize, species: u16, base: u16, pointers: [u16; 3]) {
    put(wram, slot, species);
    for offset in (0x02..0x40_u16).step_by(2) {
        put(
            wram,
            slot.wrapping_add(usize::from(offset)),
            base.wrapping_add(offset),
        );
    }
    for (offset, word) in [0x16_usize, 0x1A, 0x2E].into_iter().zip(pointers) {
        put(wram, slot.wrapping_add(offset), word);
    }
}

/// The slot `put_enemy` writes, decoded, with `properties` in place of its properties word
/// and `pointers` its cursor place and spritemap.
fn enemy_slot(
    species: SpeciesId,
    base: u16,
    properties: u16,
    pointers: (Label<Enemy>, SpritemapId),
    occupancy: fn(EnemyBody) -> Occupancy<EnemyBody>,
    body: fn(AiVariables) -> Result<EnemyBody, crate::enemy::UnknownWord>,
) -> Slot<EnemyCommon, EnemyBody> {
    let at = |offset: u16| base.wrapping_add(offset);
    let (place, spritemap) = pointers;
    Slot {
        common: EnemyCommon {
            species,
            position: Point {
                x: Fixed {
                    pixels: at(0x02),
                    subpixels: at(0x04),
                },
                y: Fixed {
                    pixels: at(0x06),
                    subpixels: at(0x08),
                },
            },
            radius_x: at(0x0A),
            radius_y: at(0x0C),
            properties: flags(properties),
            extra_properties: flags(at(0x10)),
            ai_handlers: flags(at(0x12)),
            health: at(0x14),
            timer: at(0x18),
            cursor: Cursor {
                at: place,
                delay: Frames(at(0x1C)),
                ..Cursor::default()
            },
            spritemap,
            palette: PaletteIndex(at(0x1E)),
            tiles: TilesIndex(at(0x20)),
            layer: at(0x22),
            flash: Frames(at(0x24)),
            frozen: Frames(at(0x26)),
            invincibility: Frames(at(0x28)),
            shake: Frames(at(0x2A)),
            frame_counter: at(0x2C),
            parameter_1: at(0x3C),
            parameter_2: at(0x3E),
        },
        occupancy: occupancy(
            body(AiVariables {
                a: at(0x30),
                b: at(0x32),
                c: at(0x34),
                d: at(0x36),
                e: at(0x38),
                f: at(0x3A),
            })
            .unwrap(),
        ),
        ..Slot::default()
    }
}

/// The set a slot's flag word holds: its bits in the enum's order.
fn flags<E>(bits: u16) -> FlagSet<E> {
    FlagSet {
        bits: u32::from(bits),
        ..FlagSet::default()
    }
}

/// The enemy slots the synthetic record sets: a live eye in slot 0, a deleting elevator in
/// slot 1, the rest free and zero.
fn decodes_the_enemies(state: &State) -> bool {
    let slots = &state.room.enemies.slots;
    let eye = enemy_slot(
        SpeciesId(0),
        0x1000,
        0x100E,
        pointers(EYE_BANK, EYE_INSTRUCTION, EYE_SPRITEMAP),
        Occupancy::Live,
        |variables| {
            crate::enemy::body(
                data::enemy::Behaviour::MorphBallEye,
                AiVariables {
                    c: EYE_ACTIVATED,
                    f: EYE_ACTIVATING,
                    ..variables
                },
            )
        },
    );
    let elevator = enemy_slot(
        SpeciesId(1),
        0x2000,
        DELETED_PROPERTIES,
        pointers(ELEVATOR_BANK, ELEVATOR_INSTRUCTION, ELEVATOR_SPRITEMAP),
        Occupancy::Deleting,
        |variables| crate::enemy::body(data::enemy::Behaviour::Elevator, variables),
    );
    slots.first() == Some(&eye)
        && slots.get(1) == Some(&elevator)
        && slots.iter().skip(2).all(|slot| *slot == Slot::default())
}

fn put(wram: &mut [u8], offset: usize, word: u16) {
    wram[offset..offset.wrapping_add(2)].copy_from_slice(&word.to_le_bytes());
}

/// The synthetic room and mini-map: the Morph Ball Room, in Brinstar at map cell (13, 4), the
/// map collected, the mini-map off, and every explored byte distinct with its top bit set, so
/// a word rule moved into the bitmap reads a value out of its range.
fn put_room(wram: &mut [u8]) {
    put(wram, 0x079B, MORPH_BALL_ROOM.0); // ROOM_PTR
    put(wram, 0x079F, 0x0001); // AREA_INDEX: Brinstar
    put(wram, 0x07A1, 0x000D); // ROOM_X_COORDINATE_ON_MAP
    put(wram, 0x07A3, 0x0004); // ROOM_Y_COORDINATE_ON_MAP
    put(wram, 0x0789, 0x0001); // HAS_AREA_MAP
    put(wram, 0x05F7, 0x0001); // DEBUG_DISABLE_MINIMAP
    put(wram, 0x07BB, MORPH_BALL_STATE.0); // ROOMDEFROOMSTATE_PTR
    put_scrolling(wram);
    for (byte, value) in wram[0x07F7..0x08F7].iter_mut().zip(0..=u8::MAX) {
        *byte = value | 0x80; // MAP_TILES_EXPLORED
    }
}

/// The BG graphics update's state: each word from `$08FF` to `$098E` that the decoder reads
/// distinct, and each upload buffer word its own index.
fn put_scrolling(wram: &mut [u8]) {
    for (offset, value) in (0x08FF..0x0907).step_by(2).zip(0x0011_u16..) {
        put(wram, offset, value); // PREVIOUS_LAYER1_X_BLOCK to PREVIOUS_LAYER2_Y_BLOCK
    }
    for (offset, value) in (0x0956..0x0990).step_by(2).zip(0x0101_u16..) {
        put(wram, offset, value); // BG1_UPDATE_COL_UNWRAPPED_SIZE to SIZE_OF_BG2
    }
    for (offset, value) in (0xC8C8..0xCAD8).step_by(2).zip(0_u16..) {
        put(wram, offset, value); // the BG1 and BG2 column and row buffers
    }
}

/// The scrolling state [`put_scrolling`] sets.
fn decodes_the_scrolling(state: &State) -> bool {
    let level = &state.video.batch.level;
    let word = |offset: u16| 0x0101_u16.wrapping_add(offset.wrapping_sub(0x0956) / 2);
    let part = |to: u16, bytes: u16| TilemapPart {
        to: VramWord(word(to)),
        bytes: word(bytes),
    };
    let entries = |words: &[TilemapEntry], at: u16| {
        words
            .iter()
            .zip(at.wrapping_sub(0xC8C8) / 2..)
            .all(|(entry, word)| *entry == TilemapEntry::from_word(word))
    };
    let layer = |uploads: &LayerUploads, base: u16, buffers: u16| {
        let column = &uploads.column;
        let row = &uploads.row;
        column.first == part(base.wrapping_add(4), base)
            && column.wrapped == part(base.wrapping_add(6), base.wrapping_add(2))
            && column.requests == UploadRequests(word(base.wrapping_add(0xC)))
            && row.first == part(base.wrapping_add(0x12), base.wrapping_add(0xE))
            && row.wrapped == part(base.wrapping_add(0x14), base.wrapping_add(0x10))
            && row.requests == UploadRequests(word(base.wrapping_add(0x1A)))
            && entries(&column.left, buffers)
            && entries(&column.right, buffers.wrapping_add(0x40))
            && entries(&row.top, buffers.wrapping_add(0x80))
            && entries(&row.bottom, buffers.wrapping_add(0xC4))
    };
    state.room.camera.drawn
        == DrawnBlocks {
            layer1: BlockPoint { x: 0x11, y: 0x12 },
            layer2: BlockPoint { x: 0x13, y: 0x14 },
        }
        && layer(&level.bg1, 0x0956, 0xC8C8)
        && layer(&level.bg2, 0x0972, 0xC9D0)
        && state.room.tilemap_gap == VramWord(word(0x098E))
        && state.room.area == Area::Brinstar
        && state.room.tileset == MORPH_BALL_LEVEL.tileset
        && state.room.layer2 == Some(Vec::new())
}

fn decodes_the_room(state: &State) -> bool {
    let mini_map = state.hud.mini_map;
    state.room.room == MORPH_BALL_ROOM_ID
        && mini_map.area == Area::Brinstar
        && mini_map.room == MapPosition { x: 13, y: 4 }
        && mini_map.map_collected
        && mini_map.disabled
        && mini_map
            .explored
            .0
            .iter()
            .zip(0..=u8::MAX)
            .all(|(&byte, value)| byte == value | 0x80)
        && decodes_the_scrolling(state)
}

/// The synthetic enemy slots, a live eye in slot 0 and a deleting elevator in slot 1, and
/// the PLM slots.
fn put_enemies(wram: &mut [u8]) {
    let eye = [EYE_SPRITEMAP, EYE_INSTRUCTION, EYE_BANK];
    put_enemy(wram, ENEMY_SLOT_0, EYE, 0x1000, eye);
    put(wram, ENEMY_SLOT_0.wrapping_add(0x34), EYE_ACTIVATED);
    put(wram, ENEMY_SLOT_0.wrapping_add(0x3A), EYE_ACTIVATING);
    let elevator = [ELEVATOR_SPRITEMAP, ELEVATOR_INSTRUCTION, ELEVATOR_BANK];
    put_enemy(wram, ENEMY_SLOT_1, ELEVATOR, 0x2000, elevator);
    put(wram, 0x0FC6, DELETED_PROPERTIES); // slot 1's properties
    put_plms(wram);
}

/// Samus's tile uploads, both halves queued.
fn put_samus_tiles(wram: &mut [u8]) {
    put(wram, 0x071D, 0x0101); // NMI_COPY_SAMUS_HALVES: both flagged
    put(wram, 0x071F, 0x0007); // NMI_COPY_SAMUS_TOP_HALF_SRC: definition 1
    put(wram, 0x0721, 0x000E); // NMI_COPY_SAMUS_BOTTOM_HALF_SRC: definition 2
}

/// The elevator ride and the drawing handler that rides it, as the synthetic record sets them.
fn put_ride(wram: &mut [u8]) {
    put(wram, 0x0A5C, 0xEC14); // SAMUS_DRAW_HANDLER: using elevator
    put(wram, 0x0B00, 0x0015); // SAMUS_Y_RADIUS: the standing radius she called the ride with
    put(wram, 0x0E18, 0x0003); // ELEVATOR_STATUS: entering the room
    put(wram, 0x0E16, 0x0080); // ELEVATOR_FLAGS: armed
    put(wram, 0x0799, 0x8000); // ELEVATOR_DIRECTION: up
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
    put_room(&mut wram);
    wram[0x05B4] = 0x01; // WAITING_FOR_NMI
    put(&mut wram, 0x09A2, 0x1004); // EQUIPPED_ITEMS: morph ball, bombs
    put(&mut wram, 0x09A4, 0x9006); // COLLECTED_ITEMS: spring ball, morph ball, bombs, X-ray
    put(&mut wram, 0x09C2, 0x0063); // SAMUS_HEALTH
    put(&mut wram, 0x09C4, 0x00C7); // SAMUS_MAX_HEALTH
    put(&mut wram, 0x09C6, 0x0004); // SAMUS_MISSILES
    put(&mut wram, 0x09C8, 0x0005); // SAMUS_MAX_MISSILES
    put(&mut wram, 0x09D4, 0x0064); // SAMUS_MAX_RESERVE_HEALTH
    put(&mut wram, 0x09D6, 0x0032); // SAMUS_RESERVE_HEALTH
    // Each byte differs from the one after it, so a byte rule moved by one reads another value.
    put(&mut wram, 0x0A1C, 0x0009); // SAMUS_POSE: facing right, moving, not aiming
    wram[0x0A1E] = 0x08; // SAMUS_POSE_X_DIR: right
    wram[0x0A1F] = 0x01; // SAMUS_MOVEMENT_TYPE: running
    put(&mut wram, 0x0A20, 0x0003); // SAMUS_PREV_POSE
    wram[0x0A22] = 0x08; // SAMUS_PREV_POSE_X_DIR: right
    wram[0x0A23] = 0x00; // SAMUS_PREV_MOVEMENT_TYPE2: standing
    put(&mut wram, 0x0A24, 0x0002); // SAMUS_LAST_DIFFERENT_POSE
    wram[0x0A26] = 0x04; // SAMUS_LAST_DIFFERENT_POSE_X_DIR: left
    wram[0x0A27] = 0x05; // SAMUS_LAST_DIFFERENT_POSE_MOVEMENT_TYPE: crouching
    put(&mut wram, 0x0A2C, 0x0002); // SAMUS_NEW_POSE_TRANSITIONAL
    put(&mut wram, 0x0A32, 0x0003); // SAMUS_HURT_SWITCH_INDEX: transition animation finished
    put_ride(&mut wram);
    put(&mut wram, 0x0A94, 0x0007); // SAMUS_ANIM_FRAME_TIMER
    put_samus_tiles(&mut wram);
    put(&mut wram, 0x0A72, 0x0803); // SAMUS_VISOR_PALETTE_TIMER_INDEX: dim, 3 frames
    put(&mut wram, 0x1982, 0x002A); // FX_LAYER_BLENDING_CONFIG_A
    put_glow(&mut wram);
    put(&mut wram, 0x0A96, 0x0003); // SAMUS_ANIM_FRAME
    put(&mut wram, 0x0A9A, 0x8000); // SAMUS_ANIM_FRAME_SKIP: preserve
    put(&mut wram, 0x0B3C, 0x0001); // SAMUS_HAS_MOMENTUM_FLAG
    put(&mut wram, 0x0B42, 0x0001); // SAMUS_X_EXTRA_RUN_SPEED
    put(&mut wram, 0x0B44, 0x2000); // SAMUS_X_EXTRA_RUN_SUBSPEED
    put(&mut wram, 0x0B46, 0x0002); // SAMUS_X_BASE_SPEED
    put(&mut wram, 0x0B48, 0xC000); // SAMUS_X_BASE_SUBSPEED
    put(&mut wram, 0x0B4A, 0x0002); // SAMUS_X_ACCEL_MODE: decelerating
    put(&mut wram, 0x090F, 0x1111); // LAYER1_X_SUBPOS
    put(&mut wram, 0x0911, 0x0200); // LAYER1_X_POS
    put(&mut wram, 0x0913, 0x2222); // LAYER1_Y_SUBPOS
    put(&mut wram, 0x0915, 0x0100); // LAYER1_Y_POS
    put(&mut wram, 0x00AB, 0x0004); // CUR_IRQ_HANDLER: main gameplay, begin HUD drawing
    // The main loop's stack at Wait for NMI ($80:8338 PHP, PHB): S = $1FF0, then $1FF1 P,
    // $1FF2 DB, $1FF3-$1FF5 the JSL at $82:897A's return ($82:897D), $1FF6 the loop's P.
    wram[0x1FF3..0x1FF6].copy_from_slice(&[0x7D, 0x89, 0x82]);
    put_enemies(&mut wram);

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

/// Samus's animation and super special request, as the synthetic record sets them.
fn decodes_the_animation(state: &State) -> bool {
    state.samus.animation
        == Animation {
            frame: AnimationFrame(3),
            timer: AnimationTimer(7),
            new_pose_frame: NewPoseFrame::Preserve,
        }
        && state.samus.super_special
            == SuperSpecial {
                pose: Some(PoseId(2)),
                command: SuperSpecialCommand::TransitionAnimationFinished,
            }
        && state.samus.drawing == DrawingHandler::UsingElevator
        && state.video.batch.samus_tiles
            == SamusTileUploads {
                top: Some(TileDefinitionId(1)),
                bottom: Some(TileDefinitionId(2)),
            }
        && state.samus.y_radius == 0x15
}

/// Samus's visor cycle and the room's layer blending, as the synthetic record sets them.
fn decodes_the_visor(state: &State) -> bool {
    state.samus.visor
        == Visor {
            shade: VisorShade::Dim,
            timer: VisorTimer(3),
        }
        && state.room.layer_blending == LayerBlending(0x2a)
}

/// The face block's glow: the hook on, sprite palette 7, colour set 3 in 7 frames.
fn put_glow(wram: &mut [u8]) {
    put(wram, 0x178C, 0xE86E); // enemy_gfx_drawn_hook: the face block's glow
    put(wram, 0x178E, 0x00A8); // its bank
    put(wram, 0x1794, 0x00E0); // VARIABLES_FOR_ENEMY_GRAPHICS_DRAWN_HOOK
    put(wram, 0x1796, 0x0003);
    put(wram, 0x1798, 0x0007);
}

/// The enemy graphics drawn hook and the palette cycle, as the synthetic record sets them.
fn decodes_the_glow(state: &State) -> bool {
    state.room.drawing.hook == GraphicsDrawnHook::FaceBlockGlow
        && state.enemy_palette_cycle
            == EnemyPaletteCycle {
                palette: SpritePalette(7),
                set: GlowSet(3),
                timer: Frames(7),
            }
}

/// The elevator ride, as the synthetic record sets it.
fn decodes_the_ride(state: &State) -> bool {
    state.elevator
        == ElevatorRide {
            status: ElevatorStatus::EnteringRoom,
            armed: ElevatorArming::ElevatorDoor,
            direction: ElevatorDirection::Up,
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
        && state.samus.pose == PoseId(9)
        && state.samus.facing == Facing::Right
        && state.samus.movement
            == Movement::Running {
                speed: Horizontal {
                    base: Speed::new(0x0002, 0xC000),
                    extra_run: Speed::new(0x0001, 0x2000),
                    acceleration: Acceleration::Decelerating,
                    momentum: true,
                },
            }
        && state.samus.previous
            == PoseRecord {
                pose: PoseId(3),
                facing: Facing::Right,
                movement: MovementType::Standing,
            }
        && state.samus.last_different
            == PoseRecord {
                pose: PoseId(2),
                facing: Facing::Left,
                movement: MovementType::Crouching,
            }
        && decodes_the_animation(state)
        && decodes_the_ride(state)
        && decodes_the_visor(state)
        && decodes_the_glow(state)
        && state.timing.last == NmiTiming::OnTime
        && state.video.nmi_requested
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
        && decodes_the_enemies(state)
        && decodes_the_plms(state)
        && decodes_the_room(state)
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
    assert_eq!(state.samus.aim, defaults.samus.aim);
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
const WORD_RULES: [(&str, WordRule); 89] = [
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
    ("pose", |rules| &mut rules.pose.pose),
    ("previous_pose", |rules| &mut rules.previous_pose.pose),
    ("last_different_pose", |rules| {
        &mut rules.last_different_pose.pose
    }),
    ("horizontal.base", |rules| &mut rules.horizontal.base),
    ("horizontal.base_subpixels", |rules| {
        &mut rules.horizontal.base_subpixels
    }),
    ("horizontal.extra_run", |rules| {
        &mut rules.horizontal.extra_run
    }),
    ("horizontal.extra_run_subpixels", |rules| {
        &mut rules.horizontal.extra_run_subpixels
    }),
    ("horizontal.acceleration", |rules| {
        &mut rules.horizontal.acceleration
    }),
    ("horizontal.momentum", |rules| {
        &mut rules.horizontal.momentum
    }),
    ("animation.frame", |rules| &mut rules.animation.frame),
    ("animation.timer", |rules| &mut rules.animation.timer),
    ("animation.new_pose_frame", |rules| {
        &mut rules.animation.new_pose_frame
    }),
    ("super_special.pose", |rules| &mut rules.super_special.pose),
    ("super_special.command", |rules| {
        &mut rules.super_special.command
    }),
    ("drawing", |rules| &mut rules.drawing),
    ("samus_tiles.halves", |rules| &mut rules.samus_tiles.halves),
    ("samus_tiles.top", |rules| &mut rules.samus_tiles.top),
    ("samus_tiles.bottom", |rules| &mut rules.samus_tiles.bottom),
    ("y_radius", |rules| &mut rules.y_radius),
    ("elevator.status", |rules| &mut rules.elevator.status),
    ("elevator.armed", |rules| &mut rules.elevator.armed),
    ("elevator.direction", |rules| &mut rules.elevator.direction),
    ("visor", |rules| &mut rules.visor),
    ("layer_blending", |rules| &mut rules.layer_blending),
    ("drawn_hook.pointer", |rules| &mut rules.drawn_hook.pointer),
    ("palette_cycle.palette", |rules| {
        &mut rules.palette_cycle.palette
    }),
    ("palette_cycle.set", |rules| &mut rules.palette_cycle.set),
    ("palette_cycle.timer", |rules| {
        &mut rules.palette_cycle.timer
    }),
    ("hud.mini_map.area", |rules| &mut rules.hud.mini_map.area),
    ("hud.mini_map.room_x", |rules| {
        &mut rules.hud.mini_map.room_x
    }),
    ("hud.mini_map.room_y", |rules| {
        &mut rules.hud.mini_map.room_y
    }),
    ("hud.mini_map.map_collected", |rules| {
        &mut rules.hud.mini_map.map_collected
    }),
    ("hud.mini_map.disabled", |rules| {
        &mut rules.hud.mini_map.disabled
    }),
    ("level.state", |rules| &mut rules.level.state),
    ("area", |rules| &mut rules.area),
    ("tilemap_gap", |rules| &mut rules.tilemap_gap),
    ("drawn.layer1.x", |rules| &mut rules.scrolling.drawn[0][0]),
    ("drawn.layer1.y", |rules| &mut rules.scrolling.drawn[0][1]),
    ("drawn.layer2.x", |rules| &mut rules.scrolling.drawn[1][0]),
    ("drawn.layer2.y", |rules| &mut rules.scrolling.drawn[1][1]),
    ("uploads[0].column.unwrapped_size", |rules| {
        &mut rules.uploads[0].column.unwrapped_size
    }),
    ("uploads[0].column.wrapped_size", |rules| {
        &mut rules.uploads[0].column.wrapped_size
    }),
    ("uploads[0].column.unwrapped_dst", |rules| {
        &mut rules.uploads[0].column.unwrapped_dst
    }),
    ("uploads[0].column.wrapped_dst", |rules| {
        &mut rules.uploads[0].column.wrapped_dst
    }),
    ("uploads[0].column.enable", |rules| {
        &mut rules.uploads[0].column.enable
    }),
    ("uploads[0].row.unwrapped_size", |rules| {
        &mut rules.uploads[0].row.unwrapped_size
    }),
    ("uploads[0].row.wrapped_size", |rules| {
        &mut rules.uploads[0].row.wrapped_size
    }),
    ("uploads[0].row.unwrapped_dst", |rules| {
        &mut rules.uploads[0].row.unwrapped_dst
    }),
    ("uploads[0].row.wrapped_dst", |rules| {
        &mut rules.uploads[0].row.wrapped_dst
    }),
    ("uploads[0].row.enable", |rules| {
        &mut rules.uploads[0].row.enable
    }),
    ("uploads[1].column.unwrapped_size", |rules| {
        &mut rules.uploads[1].column.unwrapped_size
    }),
    ("uploads[1].column.wrapped_size", |rules| {
        &mut rules.uploads[1].column.wrapped_size
    }),
    ("uploads[1].column.unwrapped_dst", |rules| {
        &mut rules.uploads[1].column.unwrapped_dst
    }),
    ("uploads[1].column.wrapped_dst", |rules| {
        &mut rules.uploads[1].column.wrapped_dst
    }),
    ("uploads[1].column.enable", |rules| {
        &mut rules.uploads[1].column.enable
    }),
    ("uploads[1].row.unwrapped_size", |rules| {
        &mut rules.uploads[1].row.unwrapped_size
    }),
    ("uploads[1].row.wrapped_size", |rules| {
        &mut rules.uploads[1].row.wrapped_size
    }),
    ("uploads[1].row.unwrapped_dst", |rules| {
        &mut rules.uploads[1].row.unwrapped_dst
    }),
    ("uploads[1].row.wrapped_dst", |rules| {
        &mut rules.uploads[1].row.wrapped_dst
    }),
    ("uploads[1].row.enable", |rules| {
        &mut rules.uploads[1].row.enable
    }),
];

/// Every byte rule.
const BYTE_RULES: [(&str, ByteRule); 10] = [
    ("lag", |rules| &mut rules.lag),
    ("max_lag", |rules| &mut rules.max_lag),
    ("nmi_requested", |rules| &mut rules.nmi_requested),
    ("facing", |rules| &mut rules.pose.facing),
    ("movement", |rules| &mut rules.pose.movement),
    ("previous_facing", |rules| &mut rules.previous_pose.facing),
    ("previous_movement", |rules| {
        &mut rules.previous_pose.movement
    }),
    ("last_different_facing", |rules| {
        &mut rules.last_different_pose.facing
    }),
    ("last_different_movement", |rules| {
        &mut rules.last_different_pose.movement
    }),
    ("drawn_hook.bank", |rules| &mut rules.drawn_hook.bank),
];

#[test]
fn an_acceleration_mode_past_2_or_a_momentum_flag_past_1_is_an_error() {
    for (offset, source) in [
        (0x0B4A, "SAMUS_X_ACCEL_MODE"),
        (0x0B3C, "SAMUS_HAS_MOMENTUM_FLAG"),
    ] {
        let mut record = record();
        put(&mut record.wram, offset, 0x0003);
        assert_eq!(
            decode(&record, &rooms()),
            Err(DecodeError::Unrepresentable { source, value: 3 })
        );
    }
}

#[test]
fn a_hook_or_cycle_word_outside_the_glow_is_unrepresentable() {
    // Another hook, the glow's address in another bank, half a palette, a colour set past 7.
    for (offset, value, source) in [
        (0x178C, 0x8000, "enemy_gfx_drawn_hook"),
        (0x178E, 0x00A0, "enemy_gfx_drawn_hook"),
        (0x1794, 0x00F0, "VARIABLES_FOR_ENEMY_GRAPHICS_DRAWN_HOOK[0]"),
        (0x1794, 0x0100, "VARIABLES_FOR_ENEMY_GRAPHICS_DRAWN_HOOK[0]"),
        (0x1796, 0x0008, "VARIABLES_FOR_ENEMY_GRAPHICS_DRAWN_HOOK[1]"),
    ] {
        let mut record = record();
        put(&mut record.wram, offset, value);
        assert_eq!(
            decode(&record, &rooms()),
            Err(DecodeError::Unrepresentable { source, value }),
            "{offset:04X}"
        );
    }
}

#[test]
fn either_banks_rtl_is_no_hook() {
    for bank in [0x00A0, 0x00A8] {
        let mut record = record();
        put(&mut record.wram, 0x178C, 0x804C);
        put(&mut record.wram, 0x178E, bank);
        let state = decode(&record, &rooms()).unwrap().state;
        assert_eq!(state.room.drawing.hook, GraphicsDrawnHook::None);
    }
}

#[test]
fn a_visor_word_outside_the_cycle_is_unrepresentable() {
    // A zero timer, then the x-ray's widening colours and an odd offset.
    for value in [0x0800, 0x0405, 0x0705] {
        let mut record = record();
        put(&mut record.wram, 0x0A72, value);
        assert_eq!(
            decode(&record, &rooms()),
            Err(DecodeError::Unrepresentable {
                source: "SAMUS_VISOR_PALETTE_TIMER_INDEX",
                value
            })
        );
    }
}

#[test]
fn samus_tile_uploads_outside_the_queue_are_unrepresentable() {
    // A flag other than 0 or 1, a pointer beside a clear flag, a pointer to no definition.
    for (offset, value, source, reported) in [
        (0x071D, 0x0102, "NMI_COPY_SAMUS_HALVES", 0x0102),
        (0x071D, 0x0100, "NMI_COPY_SAMUS_TOP_HALF_SRC", 0x0007),
        (0x071F, 0x0008, "NMI_COPY_SAMUS_TOP_HALF_SRC", 0x0008),
    ] {
        let mut record = record();
        put(&mut record.wram, offset, value);
        assert_eq!(
            decode(&record, &rooms()),
            Err(DecodeError::Unrepresentable {
                source,
                value: reported
            })
        );
    }
    let mut record = record();
    put(&mut record.wram, 0x071D, 0x0000);
    put(&mut record.wram, 0x071F, 0x0000);
    put(&mut record.wram, 0x0721, 0x0000);
    let decoded = decode(&record, &rooms()).unwrap();
    assert_eq!(
        decoded.state.video.batch.samus_tiles,
        SamusTileUploads::default()
    );
}

#[test]
fn a_drawing_handler_outside_the_handlers_is_unrepresentable() {
    // $EBF2 is an RTS only unused code stores; $EB53 is inside the default handler.
    for value in [0xEBF2, 0xEB53, 0x0000] {
        let mut record = record();
        put(&mut record.wram, 0x0A5C, value);
        assert_eq!(
            decode(&record, &rooms()),
            Err(DecodeError::Unrepresentable {
                source: "SAMUS_DRAW_HANDLER",
                value
            })
        );
    }
}

#[test]
fn a_ride_word_outside_its_values_is_unrepresentable() {
    for (offset, value, source) in [
        (0x0E18, 0x0004, "ELEVATOR_STATUS"),
        (0x0E16, 0x0002, "ELEVATOR_FLAGS"),
        (0x0799, 0x0001, "ELEVATOR_DIRECTION"),
    ] {
        let mut record = record();
        put(&mut record.wram, offset, value);
        assert_eq!(
            decode(&record, &rooms()),
            Err(DecodeError::Unrepresentable { source, value })
        );
    }
}

#[test]
fn a_pose_past_the_table_or_a_movement_type_past_the_handler_is_an_error() {
    let mut past_table = record();
    put(&mut past_table.wram, 0x0A1C, 0x00FD);
    assert_eq!(
        decode(&past_table, &rooms()),
        Err(DecodeError::Unrepresentable {
            source: "SAMUS_POSE",
            value: 0xFD
        })
    );
    let mut past_handler = record();
    past_handler.wram[0x0A1F] = 0x1C;
    assert_eq!(
        decode(&past_handler, &rooms()),
        Err(DecodeError::Unrepresentable {
            source: "SAMUS_MOVEMENT_TYPE",
            value: 0x1C
        })
    );
}

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

/// A place in `Rules` holding an enemy slot field.
type SlotRule = for<'r> fn(&'r mut Rules<'static>) -> &'r mut SlotField;

/// Every enemy slot field the decoder reads.
const SLOT_RULES: [(&str, SlotRule); 32] = [
    ("species", |rules| &mut rules.enemies.species),
    ("x", |rules| &mut rules.enemies.x),
    ("x_subpixels", |rules| &mut rules.enemies.x_subpixels),
    ("y", |rules| &mut rules.enemies.y),
    ("y_subpixels", |rules| &mut rules.enemies.y_subpixels),
    ("radius_x", |rules| &mut rules.enemies.radius_x),
    ("radius_y", |rules| &mut rules.enemies.radius_y),
    ("properties", |rules| &mut rules.enemies.properties),
    ("extra_properties", |rules| {
        &mut rules.enemies.extra_properties
    }),
    ("ai_handlers", |rules| &mut rules.enemies.ai_handlers),
    ("health", |rules| &mut rules.enemies.health),
    ("timer", |rules| &mut rules.enemies.timer),
    ("spritemap", |rules| &mut rules.enemies.spritemap),
    ("instruction", |rules| &mut rules.enemies.instruction),
    ("instruction_timer", |rules| {
        &mut rules.enemies.instruction_timer
    }),
    ("bank", |rules| &mut rules.enemies.bank),
    ("palette", |rules| &mut rules.enemies.palette),
    ("tiles", |rules| &mut rules.enemies.tiles),
    ("layer", |rules| &mut rules.enemies.layer),
    ("flash", |rules| &mut rules.enemies.flash),
    ("frozen", |rules| &mut rules.enemies.frozen),
    ("invincibility", |rules| &mut rules.enemies.invincibility),
    ("shake", |rules| &mut rules.enemies.shake),
    ("frame_counter", |rules| &mut rules.enemies.frame_counter),
    ("variables[0]", |rules| &mut rules.enemies.variables[0]),
    ("variables[1]", |rules| &mut rules.enemies.variables[1]),
    ("variables[2]", |rules| &mut rules.enemies.variables[2]),
    ("variables[3]", |rules| &mut rules.enemies.variables[3]),
    ("variables[4]", |rules| &mut rules.enemies.variables[4]),
    ("variables[5]", |rules| &mut rules.enemies.variables[5]),
    ("parameter_1", |rules| &mut rules.enemies.parameter_1),
    ("parameter_2", |rules| &mut rules.enemies.parameter_2),
];

#[test]
fn every_moved_enemy_field_fails() {
    for (name, rule) in SLOT_RULES {
        let mut mutant = Rules::ORIGINAL;
        let field = rule(&mut mutant);
        field.offset = field.offset.wrapping_add(2);
        assert!(!survives(&mutant), "{name}");
    }
}

/// A place in an upload's sources holding one of its buffers.
type BufferRule = fn(&mut LayerUploadSources) -> &mut WramBlock;

#[test]
fn every_moved_upload_buffer_fails() {
    let buffers: [(&str, BufferRule); 4] = [
        ("column first", |sources| &mut sources.column.first_halves),
        ("column second", |sources| &mut sources.column.second_halves),
        ("row first", |sources| &mut sources.row.first_halves),
        ("row second", |sources| &mut sources.row.second_halves),
    ];
    for layer in 0..2 {
        for (name, buffer) in buffers {
            let mut mutant = Rules::ORIGINAL;
            let block = buffer(&mut mutant.uploads[layer]);
            block.offset = block.offset.wrapping_add(2);
            assert!(!survives(&mutant), "BG{} {name}", layer.wrapping_add(1));
        }
    }
}

#[test]
fn a_room_state_the_map_doesnt_hold_is_an_error() {
    let mut record = record();
    put(&mut record.wram, 0x07BB, 0x1234);
    assert_eq!(
        decode(&record, &rooms()),
        Err(DecodeError::UnknownRoomState(StateHeader(0x1234)))
    );
}

#[test]
fn a_moved_explored_bitmap_fails() {
    let mut moved = Rules::ORIGINAL;
    moved.hud.mini_map.explored.offset = moved.hud.mini_map.explored.offset.wrapping_add(1);
    assert!(!survives(&moved));
}

#[test]
fn a_moved_or_restrided_enemy_table_fails() {
    let mut moved = Rules::ORIGINAL;
    moved.enemies.table.offset = moved.enemies.table.offset.wrapping_add(2);
    assert!(!survives(&moved));
    let mut restrided = Rules::ORIGINAL;
    restrided.enemies.table.stride = restrided.enemies.table.stride.wrapping_add(2);
    assert!(!survives(&restrided));
}

/// The synthetic room's population: two scroll PLMs over lists `$8F:9F05` and `$8F:9F08`, an
/// extension and the Morph Ball, whose collected bit is `$1A`.
fn population() -> PlmPopulation {
    let entry = |kind| PlmEntry { kind, x: 1, y: 2 };
    PlmPopulation {
        entries: vec![
            entry(PlmKind::Scroll(ScrollChangesId(0))),
            entry(PlmKind::Extension(Extension::Up)),
            entry(PlmKind::Scroll(ScrollChangesId(1))),
            entry(PlmKind::Item(ItemPlm::MorphBall, 0x1A)),
        ],
        scroll_changes: vec![
            (RomAddress::new(0x8F, 0x9F05), Vec::new()),
            (RomAddress::new(0x8F, 0x9F08), Vec::new()),
        ],
    }
}

/// Slot `slot`'s word in the PLM array at `array` (bank $84 log: one word per slot).
fn plm_word(array: usize, slot: usize) -> usize {
    array.wrapping_add(slot.wrapping_mul(2))
}

/// The PLM arrays: `PLM_HEADER_PTR`, `PLM_BLOCK_INDICES`, `PLM_INSTR_LIST_PTRS`,
/// `PLM_ROOM_ARGUMENTS`, `PLM_VARIABLE` and `PLM_INSTRUCTION_TIMER`.
const PLM_ID: usize = 0x1C37;
const PLM_BLOCK: usize = 0x1C87;
const PLM_INSTRUCTION: usize = 0x1D27;
const PLM_ARGUMENT: usize = 0x1DC7;
const PLM_VARIABLE: usize = 0x1E17;
const PLM_TIMER: usize = 0xDE1C;

/// The synthetic PLM slots: a sleeping scroll PLM in slot 39, a triggered one in 38, the
/// Morph Ball in 37, a scroll block touch PLM in 36, an item collision detection PLM in 35 and
/// a 1x1 shot block past its second draw in 34; every slot's block and timer distinct, as a
/// free slot keeps its last occupant's.
fn put_plms(wram: &mut [u8]) {
    for slot in 0..PLM_SLOTS {
        let distinct = u16::try_from(slot).unwrap();
        put(
            wram,
            plm_word(PLM_BLOCK, slot),
            distinct.wrapping_add(0x100).wrapping_mul(2),
        );
        put(wram, plm_word(PLM_TIMER, slot), distinct.wrapping_add(0x40));
    }
    for (slot, id, argument, variable, instruction) in [
        (39, 0xB703, 0x9F05, 0, 0xAF8A),
        (38, 0xB703, 0x9F08, 0x8000, 0xAF8C),
        (37, 0xEF23, 0x001A, 0, 0xE3EF),
        (36, 0xB6FF, 0, 0, 0xAAE3),
        (35, 0xEED3, 0, 0, 0xAAE3),
        (34, 0xD074, 0, 0, 0xCBC6),
    ] {
        put(wram, plm_word(PLM_ID, slot), id);
        put(wram, plm_word(PLM_ARGUMENT, slot), argument);
        put(wram, plm_word(PLM_VARIABLE, slot), variable);
        put(wram, plm_word(PLM_INSTRUCTION, slot), instruction);
    }
}

/// The PLM slots `put_plms` writes, decoded.
fn decodes_the_plms(state: &State) -> bool {
    let scroll = |changes, triggered, step| {
        Occupancy::Live(PlmBody::Scroll(ScrollPlm {
            changes: ScrollChangesId(changes),
            triggered,
            step,
        }))
    };
    let occupancy = |slot: usize| match slot {
        39 => scroll(0, false, ScrollStep::Sleep),
        38 => scroll(1, true, ScrollStep::Update),
        37 => Occupancy::Live(PlmBody::Item {
            item: ItemPlm::MorphBall,
            collected_bit: 0x1A,
            step: ItemStep::Shown,
        }),
        36 => Occupancy::Live(PlmBody::ScrollTouch),
        35 => Occupancy::Live(PlmBody::ItemCollision),
        34 => Occupancy::Live(PlmBody::ShotBlock {
            crumble: Crumble::Third,
        }),
        _ => Occupancy::Free,
    };
    state
        .room
        .plms
        .slots
        .iter()
        .enumerate()
        .all(|(slot, decoded)| {
            let distinct = u16::try_from(slot).unwrap();
            *decoded
                == Slot {
                    generation: Generation::default(),
                    common: PlmCommon {
                        block: BlockIndex(distinct.wrapping_add(0x100)),
                        cursor: Cursor {
                            delay: Frames(distinct.wrapping_add(0x40)),
                            ..Cursor::default()
                        },
                    },
                    occupancy: occupancy(slot),
                }
        })
}

/// A place in `Rules` holding a PLM array.
type ArrayRule = for<'r> fn(&'r mut Rules<'static>) -> &'r mut WramArray;

/// Every PLM array the decoder reads.
const ARRAY_RULES: [(&str, ArrayRule); 6] = [
    ("id", |rules| &mut rules.plms.id),
    ("block", |rules| &mut rules.plms.block),
    ("timer", |rules| &mut rules.plms.timer),
    ("instruction", |rules| &mut rules.plms.instruction),
    ("argument", |rules| &mut rules.plms.argument),
    ("variable", |rules| &mut rules.plms.variable),
];

#[test]
fn every_moved_plm_array_fails() {
    for (name, rule) in ARRAY_RULES {
        // By a slot, so each slot reads the next one's word.
        let mut moved = Rules::ORIGINAL;
        let array = rule(&mut moved);
        array.offset = array.offset.wrapping_add(2);
        assert!(!survives(&moved), "{name}");
        // Onto the next array of the table, `$50` bytes on.
        let mut next = Rules::ORIGINAL;
        let array = rule(&mut next);
        array.offset = array.offset.wrapping_add(0x50);
        assert!(!survives(&next), "{name} at the next array");
    }
}

#[test]
fn a_plm_word_outside_the_rooms_plms_is_unrepresentable() {
    for (array, slot, value, source) in [
        // An ID the room's population doesn't place: an extension, which deletes itself.
        (PLM_ID, 30, 0xB647, "PLM_HEADER_PTR"),
        (PLM_ID, 30, 0xC848, "PLM_HEADER_PTR"),
        (PLM_BLOCK, 3, 0x0201, "PLM_BLOCK_INDICES"),
        (PLM_ARGUMENT, 39, 0x9F0B, "PLM_ROOM_ARGUMENTS"),
        (PLM_VARIABLE, 39, 0x0001, "PLM_VARIABLE"),
        (PLM_INSTRUCTION, 39, 0xAF86, "PLM_INSTR_LIST_PTRS"),
        // An item collision detection PLM past its delete, a shot block inside a draw.
        (PLM_INSTRUCTION, 35, 0x86BC, "PLM_INSTR_LIST_PTRS"),
        (PLM_INSTRUCTION, 34, 0xCBBA, "PLM_INSTR_LIST_PTRS"),
    ] {
        let mut record = record();
        put(&mut record.wram, plm_word(array, slot), value);
        assert_eq!(
            decode(&record, &rooms()),
            Err(DecodeError::Unrepresentable { source, value }),
            "{source}"
        );
    }
}

#[test]
fn a_room_without_scroll_plms_runs_no_scroll_block_touch_plm() {
    let mut rooms = rooms();
    rooms.plms = vec![(
        MORPH_BALL_ROOM_ID,
        PlmPopulation {
            entries: vec![PlmEntry {
                kind: PlmKind::Item(ItemPlm::MorphBall, 0x1A),
                x: 1,
                y: 2,
            }],
            scroll_changes: Vec::new(),
        },
    )];
    let mut record = record();
    for slot in [39, 38] {
        put(&mut record.wram, plm_word(PLM_ID, slot), 0);
    }
    assert_eq!(
        decode(&record, &rooms),
        Err(DecodeError::Unrepresentable {
            source: "PLM_HEADER_PTR",
            value: 0xB6FF
        })
    );
}

#[test]
fn a_room_without_items_runs_no_item_collision_detection_plm() {
    let mut rooms = rooms();
    rooms.plms = vec![(
        MORPH_BALL_ROOM_ID,
        PlmPopulation {
            entries: vec![PlmEntry {
                kind: PlmKind::Scroll(ScrollChangesId(0)),
                x: 1,
                y: 2,
            }],
            scroll_changes: vec![(RomAddress::new(0x8F, 0x9F05), Vec::new())],
        },
    )];
    let mut record = record();
    put(&mut record.wram, plm_word(PLM_ID, 37), 0);
    assert_eq!(
        decode(&record, &rooms),
        Err(DecodeError::Unrepresentable {
            source: "PLM_HEADER_PTR",
            value: 0xEED3
        })
    );
}

#[test]
fn an_unknown_species_is_unrepresentable() {
    let mut record = record();
    put(&mut record.wram, ENEMY_SLOT_1, 0xCEBF);
    assert_eq!(
        decode(&record, &rooms()),
        Err(DecodeError::Unrepresentable {
            source: "enemy_ptr",
            value: 0xCEBF
        })
    );
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

/// The synthetic record in a spin jump, rising: pose $19 (facing right, spin jump).
fn jump_record() -> FrameRecord {
    let mut record = record();
    put(&mut record.wram, 0x0A1C, 0x0019); // SAMUS_POSE: facing right, spin jump
    record.wram[0x0A1F] = 0x03; // SAMUS_MOVEMENT_TYPE: spin jumping
    put(&mut record.wram, 0x0B2C, 0xE000); // SAMUS_Y_SUBSPEED
    put(&mut record.wram, 0x0B2E, 0x0004); // SAMUS_Y_SPEED
    put(&mut record.wram, 0x0B36, 0x0001); // SAMUS_Y_DIR: up
    record
}

/// The jump's motion, as the jump record sets it: both speeds.
fn decodes_the_jump(state: &State) -> bool {
    state.samus.movement
        == Movement::Jumping {
            jump: Jump::Spin,
            speed: Horizontal {
                base: Speed::new(0x0002, 0xC000),
                extra_run: Speed::new(0x0001, 0x2000),
                acceleration: Acceleration::Decelerating,
                momentum: true,
            },
            vertical: Vertical {
                speed: Speed::new(0x0004, 0xE000),
                direction: YDirection::Up,
            },
        }
}

#[test]
fn a_jump_decodes_both_speeds() {
    let state = decode(&jump_record(), &rooms()).unwrap().state;
    assert!(decodes_the_jump(&state), "{:?}", state.samus.movement);
}

#[test]
fn every_moved_vertical_rule_fails() {
    let rules: [(&str, WordRule); 3] = [
        ("vertical.speed", |rules| &mut rules.vertical.speed),
        ("vertical.subpixels", |rules| &mut rules.vertical.subpixels),
        ("vertical.direction", |rules| &mut rules.vertical.direction),
    ];
    for (name, rule) in rules {
        let mut mutant = Rules::ORIGINAL;
        let word = rule(&mut mutant);
        word.offset = word.offset.wrapping_add(2);
        let survives = mutant
            .decode(&jump_record(), &rooms())
            .is_ok_and(|decoded| decodes_the_jump(&decoded.state));
        assert!(!survives, "{name}");
    }
}

#[test]
fn a_y_direction_past_2_is_an_error() {
    let mut record = jump_record();
    put(&mut record.wram, 0x0B36, 0x0003);
    assert_eq!(
        decode(&record, &rooms()),
        Err(DecodeError::Unrepresentable {
            source: "SAMUS_Y_DIR",
            value: 3
        })
    );
}

/// `record()` in a room one screen across and two down, each block and BTS distinct, the top
/// screen green and the bottom blue.
fn level_record() -> FrameRecord {
    let mut record = record();
    put(&mut record.wram, 0x07A5, 16); // ROOM_WIDTH_IN_BLOCKS
    put(&mut record.wram, 0x07A7, 32); // ROOM_HEIGHT_IN_BLOCKS
    for (block, bts) in (0..512_u16).zip((0..=u8::MAX).cycle()) {
        let at = usize::from(block);
        put(
            &mut record.wram,
            0x1_0002_usize.wrapping_add(at.wrapping_mul(2)),
            block.wrapping_mul(0x0123),
        ); // LEVEL_DATA
        record.wram[0x1_6402_usize.wrapping_add(at)] = bts; // BTS
    }
    record.wram[0xCD20] = 2; // SCROLLS
    record.wram[0xCD21] = 1;
    record
}

fn level(rules: &Rules) -> Option<(Vec<Block>, Vec<ScrollArea>)> {
    let state = rules.decode(&level_record(), &rooms()).ok()?.state;
    Some((state.room.blocks, state.room.scroll_areas))
}

#[test]
fn the_level_decodes_from_the_buffers_the_rooms_size_fills() {
    let (blocks, areas) = level(&Rules::ORIGINAL).unwrap();
    assert_eq!(blocks.len(), 512);
    for (block, (index, bts)) in blocks.iter().zip((0..512_u16).zip((0..=u8::MAX).cycle())) {
        assert_eq!(block.word(), index.wrapping_mul(0x0123));
        assert_eq!(block.kind.bts(), bts);
    }
    assert_eq!(areas, [ScrollArea::Green, ScrollArea::Blue]);
}

#[test]
fn every_moved_level_rule_fails() {
    let original = level(&Rules::ORIGINAL);
    let moved: [fn(&mut Rules<'static>); 5] = [
        |rules| rules.level.width.offset = rules.level.width.offset.wrapping_add(2),
        |rules| rules.level.height.offset = rules.level.height.offset.wrapping_add(2),
        |rules| rules.level.blocks.offset = rules.level.blocks.offset.wrapping_add(2),
        |rules| rules.level.bts.offset = rules.level.bts.offset.wrapping_add(1),
        |rules| rules.level.scrolls.offset = rules.level.scrolls.offset.wrapping_add(1),
    ];
    for (index, mutate) in moved.into_iter().enumerate() {
        let mut mutant = Rules::ORIGINAL;
        mutate(&mut mutant);
        assert_ne!(level(&mutant), original, "rule {index}");
    }
}

#[test]
fn a_scroll_byte_no_area_has_is_unrepresentable() {
    let mut record = level_record();
    record.wram[0xCD21] = 3;
    assert_eq!(
        decode(&record, &rooms()),
        Err(DecodeError::Unrepresentable {
            source: "SCROLLS",
            value: 3
        })
    );
}

/// Both rows' frames from `SM_CACHE`, and the Morph Ball Room as content loads it.
struct CachedRows {
    /// The room as `RoomScoped::load` loads it.
    loaded: game::state::room::RoomScoped,
    /// Each frame that decodes, by row and frame.
    frames: Vec<(String, State)>,
    /// Each frame that doesn't, with why.
    undecodable: Vec<(String, DecodeError)>,
}

/// Each row's frames decoded from `SM_CACHE`, with the Morph Ball Room's level as content
/// loads it from `SM_ROM`; `None` where either isn't present (CI's Verify job has neither).
fn cached_rows() -> Option<CachedRows> {
    use data::room::{Progress, RoomKey, Rooms, SLICE_ZERO_ROOMS};
    let cache = std::path::PathBuf::from(std::env::var_os("SM_CACHE")?);
    let image = std::fs::read(std::env::var_os("SM_ROM")?).ok()?;
    let content = crate::bundle::content(image.clone()).unwrap();
    let rooms = crate::lockstep::slice_zero_rooms()
        .with_programs(content.enemy_programs)
        .with_samus_tiles(content.samus_tiles)
        .with_plms(crate::lockstep::slice_zero_plms(&image).unwrap())
        .with_states(crate::lockstep::slice_zero_states(&image).unwrap());
    let rom = data::rom::Rom::new(image).unwrap();
    let (loaded_rooms, _) = Rooms::load(rom.lorom(), &SLICE_ZERO_ROOMS).unwrap();
    let id = loaded_rooms.map().id(RoomKey::MORPH_BALL_ROOM).unwrap();
    let morph_ball_room = loaded_rooms.get(id).unwrap();
    let state = morph_ball_room
        .state(morph_ball_room.select(&Progress::default()))
        .unwrap();
    let loaded = game::state::room::RoomScoped::load(id, morph_ball_room, state);
    let lock_path =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scenarios/locks/lsnes.toml");
    let lock = crate::lock::Lock::parse(&std::fs::read_to_string(lock_path).unwrap()).unwrap();
    let mut frames = Vec::new();
    let mut undecodable = Vec::new();
    for row in ["area-brinstar", "item-morph-ball"] {
        let row = lock.row(&crate::lock::RowId(row.to_owned())).unwrap();
        let cache = crate::cache::Cache::open(&cache, row).unwrap();
        for item in cache.records() {
            let (frame, record) = item.unwrap();
            let name = format!("{}:{}", row.id.0, frame.0);
            match decode(&record, &rooms) {
                Ok(decoded) => frames.push((name, decoded.state)),
                Err(error) => undecodable.push((name, error)),
            }
        }
    }
    Some(CachedRows {
        loaded,
        frames,
        undecodable,
    })
}

/// The shot block Samus shoots in `item-morph-ball`, a 1x1 block that doesn't reform.
const SHOT_CELL: usize = 5708;

/// The Morph Ball's block: its setup makes it air with BTS $45, then its instruction list's
/// draw (`$84:E407`, the item frames) writes the block's word, which waits for the item's
/// list (C12).
const ITEM_CELL: usize = 5317;

/// The cells the room's PLMs write at load (scroll PLM triggers, special air BTS $46, with
/// an extension beside them, and the vertical extension columns above them) are the
/// content's once `game::plm` spawns its population.
#[test]
fn the_cached_levels_are_the_loaded_rooms_but_for_the_item_shot_block_and_screen_13() {
    let Some(CachedRows { loaded, frames, .. }) = cached_rows() else {
        return;
    };
    // Every frame of both rows, now the PLM slots decode whatever PLM is live and a spin
    // jump's wall jump eligible frame has its tiles.
    assert_eq!(frames.len(), 1905);
    let mut differing = std::collections::BTreeMap::new();
    let mut screens = std::collections::BTreeSet::new();
    for (frame, state) in &frames {
        assert_eq!(state.room.blocks.len(), loaded.blocks.len(), "{frame}");
        let cells = state.room.blocks.iter().zip(&loaded.blocks).enumerate();
        for (cell, _) in cells.filter(|(_, (decoded, content))| decoded != content) {
            differing.entry(cell).or_insert_with(|| frame.clone());
        }
        let areas = state.room.scroll_areas.iter().zip(&loaded.scroll_areas);
        for (screen, _) in areas
            .enumerate()
            .filter(|(_, (decoded, content))| decoded != content)
        {
            screens.insert(screen);
        }
    }
    assert_eq!(
        differing.keys().copied().collect::<Vec<_>>(),
        [ITEM_CELL, SHOT_CELL]
    );
    // Shot at frame 1,236: the shot block crumbles there.
    assert_eq!(differing[&SHOT_CELL], "item-morph-ball:1236");
    // A scroll PLM turns screen 13 green from frame 92 of both rows: Samus touches its
    // trigger, which the special air inside reaction doesn't run yet.
    assert_eq!(screens.into_iter().collect::<Vec<_>>(), [13]);
}

#[test]
fn the_face_blocks_words_round_trip_and_an_activated_word_past_1_is_unknown() {
    let words = AiVariables {
        a: 1,
        b: 0xffc1,
        c: 3,
        d: 4,
        e: 5,
        f: 6,
    };
    let block = crate::enemy::face_block(words).unwrap();
    assert!(block.activated);
    assert_eq!(block.samus_offset, 0xffc1);
    assert_eq!(block.unused, [3, 4, 5, 6]);
    assert_eq!(crate::enemy::face_block_words(block), words);
    assert_eq!(
        crate::enemy::face_block(AiVariables { a: 2, ..words }),
        Err(crate::enemy::UnknownWord { index: 0, value: 2 })
    );
}

/// The live PLMs in `plms`: each slot's index, block and body.
fn live_plms(
    plms: &game::state::slots::SlotStore<PlmCommon, PlmBody, PLM_SLOTS>,
) -> Vec<(usize, BlockIndex, PlmBody)> {
    plms.slots
        .iter()
        .enumerate()
        .filter_map(|(slot, plm)| match plm.occupancy {
            Occupancy::Free => None,
            Occupancy::Live(body) | Occupancy::Deleting(body) => {
                Some((slot, plm.common.block, body))
            }
        })
        .collect()
}

/// Every frame of both rows decodes, its PLM slots too. The live slots are the loaded room's:
/// five scroll PLMs asleep and the item, once the extensions have freed theirs, until the
/// item is collected; beside them the item collision detection PLM (`$84:EED3`) Samus's touch
/// of the Morph Ball spawns, left live while the pickup's message box holds the PLM pass
/// (C13), and the 1x1 shot block (`$84:D074`) Samus shoots, through its crumble.
#[test]
fn the_cached_plm_slots_are_the_loaded_rooms_population() {
    let Some(rows) = cached_rows() else {
        return;
    };
    assert_eq!(rows.undecodable, Vec::new());
    assert_eq!(rows.frames.len(), 1905);

    let loaded = live_plms(&rows.loaded.plms);
    let item = |(_, _, body): &(usize, BlockIndex, PlmBody)| matches!(body, PlmBody::Item { .. });
    assert_eq!(loaded.len(), 6);
    assert_eq!(loaded.iter().filter(|plm| item(plm)).count(), 1);
    let collected: Vec<_> = loaded.iter().filter(|plm| !item(plm)).copied().collect();
    let mut first_collected = None;
    let mut touches = Vec::new();
    let mut crumbles = Vec::new();
    let mut picked_up = Vec::new();
    for (frame, state) in &rows.frames {
        let mut live = live_plms(&state.room.plms);
        // The item's place in its list past its pickup, compared below.
        for (_, _, body) in &mut live {
            if let PlmBody::Item { step, .. } = body
                && *step != ItemStep::Shown
            {
                picked_up.push((frame.as_str(), *step));
                *step = ItemStep::Shown;
            }
        }
        if let Some(at) = live
            .iter()
            .position(|plm| *plm == (33, BlockIndex(0), PlmBody::ItemCollision))
        {
            live.remove(at);
            touches.push(frame.as_str());
        }
        if let Some(at) = live.iter().position(|(slot, block, body)| {
            (*slot, *block) == (34, BlockIndex(5708)) && matches!(body, PlmBody::ShotBlock { .. })
        }) {
            live.remove(at);
            crumbles.push(frame.as_str());
        }
        if live == collected {
            first_collected.get_or_insert(frame.as_str());
        } else {
            assert_eq!(live, loaded, "{frame}");
        }
    }
    assert_eq!(first_collected, Some("item-morph-ball:1217"));
    // Inside the box from Samus's touch, the empty item for the frame after it.
    assert_eq!(
        picked_up.first(),
        Some(&("item-morph-ball:817", ItemStep::InBox))
    );
    assert_eq!(picked_up.len(), 1217 - 817);
    assert_eq!(
        picked_up.last(),
        Some(&("item-morph-ball:1216", ItemStep::Emptied))
    );
    assert_eq!(
        picked_up
            .iter()
            .filter(|(_, step)| *step == ItemStep::Emptied)
            .count(),
        1
    );
    assert_eq!(
        (touches.first().copied(), touches.len()),
        (Some("item-morph-ball:817"), 399)
    );
    assert_eq!(
        (crumbles.first().copied(), crumbles.len()),
        (Some("item-morph-ball:1236"), 13)
    );
}
