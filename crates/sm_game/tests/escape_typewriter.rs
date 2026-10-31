//! The escape typewriter, checked against bank $A6's description of it
//! ($A6:C23F set up, $A6:C2B1 handle typewriter text). Expected values are worked
//! out by hand from the bank log, not read back from the code:
//! - its state is slot 0's $7E:8034 tilemap entry, $8036 text pointer, $8038 VRAM
//!   tilemap address, $803A instruction timer, $803C timer reset value and $803E
//!   stroke timer;
//! - text is a byte stream in bank $A6: word 0 ends it, word 1 sets the timer reset
//!   value and word Dh the VRAM address (each with a word argument); any other byte
//!   is a character, drawn as the base tilemap value + (character - 'A'), with '!'
//!   drawn as character 5Bh, and a space only advancing the VRAM address;
//! - each character queues a 2-byte VRAM transfer from $7E:8034 to the VRAM address;
//! - every second character sounds the stroke: sound 45h on library 2 in Ceres
//!   (area 6), else sound Dh on library 3, at most 3 queued.
use sm_data::cartridge::RomImage;
use sm_game::actors::session_slots::SlotAccess;
use sm_game::cartridge::{NativeDiagnostics, NativeImage};
use sm_game::display::transfer::TilemapQueue;
use sm_game::effects::SoundEffects;
use sm_game::escape_typewriter::{Host, handle_text, set_up_zebes};
use sm_game::ram::CompatibilityRam;

const TILEMAP_ENTRY: usize = 0x8034;
const TEXT: usize = 0x8036;
const VRAM_ADDRESS: usize = 0x8038;
const TIMER: usize = 0x803a;
const TIMER_RESET: usize = 0x803c;
const STROKE_TIMER: usize = 0x803e;
const QUEUE_TAIL: usize = 0x330;
const QUEUE: usize = 0xd0;
/// $A6:8000, where the test's text starts, in the LoROM image.
const TEXT_START: u16 = 0x8000;
const TEXT_OFFSET: usize = 0x13_0000;
const TILES: u16 = 0x2610;

struct World {
    ram: Vec<u8>,
    rom: RomImage,
    colours: [u16; 256],
    area: u16,
    sounds: Vec<(usize, u8, u8)>,
}

impl World {
    fn new(text: &[u8]) -> Self {
        let mut rom = vec![0; 0x20_0000];
        rom[TEXT_OFFSET..TEXT_OFFSET + text.len()].copy_from_slice(text);
        let rom = RomImage::from(rom);
        let mut world = Self {
            ram: vec![0; 0x2_0000],
            rom,
            colours: [0; 256],
            area: 0,
            sounds: Vec::new(),
        };
        world.set_word(TEXT, TEXT_START);
        world
    }
    fn word(&self, at: usize) -> u16 {
        u16::from_le_bytes([self.ram[at], self.ram[at + 1]])
    }
    fn set_word(&mut self, at: usize, value: u16) {
        self.ram[at..at + 2].copy_from_slice(&value.to_le_bytes());
    }
}

impl SlotAccess for World {
    fn slot_word(&self, at: usize) -> u16 {
        self.word(at)
    }
    fn set_slot_word(&mut self, at: usize, value: u16) {
        self.set_word(at, value);
    }
    fn slot_byte(&self, at: usize) -> u8 {
        self.ram[at]
    }
    fn set_slot_byte(&mut self, at: usize, value: u8) {
        self.ram[at] = value;
    }
}
impl SoundEffects for World {
    fn queue_sound(&mut self, channel: usize, sound: u8, max_queued: u8) {
        self.sounds.push((channel, sound, max_queued));
    }
}
impl NativeImage for World {
    fn native_image(&self) -> RomImage {
        self.rom.clone()
    }
}
impl NativeDiagnostics for World {
    fn native_rom_diagnostic(&mut self, address: u32) {
        panic!("text read below $8000: {address:#x}");
    }
}
impl CompatibilityRam for World {
    fn compatibility_byte(&self, at: usize) -> u8 {
        self.ram[at]
    }
    fn compatibility_write_byte(&mut self, at: usize, value: u8) {
        self.ram[at] = value;
    }
}
impl TilemapQueue for World {
    fn tilemap_queue_tail(&self) -> u16 {
        self.word(QUEUE_TAIL)
    }
    fn set_tilemap_queue_tail(&mut self, tail: u16) {
        self.set_word(QUEUE_TAIL, tail);
    }
    fn set_tilemap_queue_byte(&mut self, at: usize, value: u8) {
        self.ram[at] = value;
    }
}
impl Host for World {
    fn colour(&self, index: usize) -> u16 {
        self.colours[index]
    }
    fn set_colour(&mut self, index: usize, value: u16) {
        self.colours[index] = value;
    }
    fn area(&self) -> u16 {
        self.area
    }
}

/// Timer reset 3, VRAM address 5000h, then "A !" and the end.
const TEXT_BYTES: [u8; 13] = [1, 0, 3, 0, 13, 0, 0x00, 0x50, b'A', b' ', b'!', 0, 0];

/// The queue record at byte `entry`: size, source, bank, destination.
fn record(world: &World, entry: usize) -> (u16, u16, u8, u16) {
    let at = QUEUE + entry;
    (
        world.word(at),
        world.word(at + 2),
        world.ram[at + 4],
        world.word(at + 5),
    )
}

#[test]
fn zebes_setup_copies_the_text_colours_and_clears_the_state() {
    let mut world = World::new(&[]);
    world.colours[125] = 0x1234;
    world.colours[126] = 0x5678;
    for at in [VRAM_ADDRESS, TIMER, TIMER_RESET, STROKE_TIMER, 0xb5, 0xb7] {
        world.set_word(at, 0xffff);
    }
    set_up_zebes(&mut world);
    assert_eq!((world.colours[157], world.colours[158]), (0x1234, 0x5678));
    assert_eq!(
        world.word(TEXT),
        0xc49c,
        "Typewriter text - Zebes escape timer"
    );
    for at in [VRAM_ADDRESS, TIMER, TIMER_RESET, STROKE_TIMER] {
        assert_eq!(world.word(at), 0, "{at:#x}");
    }
    assert_eq!(
        (world.word(0xb5), world.word(0xb7)),
        (0, 0),
        "BG2 X and Y scroll"
    );
}

#[test]
fn types_each_character_after_the_timer_runs_out() {
    let mut world = World::new(&TEXT_BYTES);
    world.area = 6;
    // Frame 1: the timer reloads from the old reset value (0) before the instructions
    // set the new one, then 'A' is typed as the base tilemap value.
    assert!(!handle_text(&mut world, TILES));
    assert_eq!(world.word(TIMER_RESET), 3);
    assert_eq!(world.word(TIMER), 0);
    assert_eq!(world.word(TILEMAP_ENTRY), TILES);
    assert_eq!(record(&world, 0), (2, 0x8034, 0x7e, 0x5000));
    assert_eq!(world.word(QUEUE_TAIL), 7);
    assert_eq!(world.word(VRAM_ADDRESS), 0x5001);
    assert_eq!(
        world.word(TEXT),
        TEXT_START + 9,
        "past the instructions and one byte"
    );
    assert_eq!(world.word(STROKE_TIMER), 1);
    assert!(world.sounds.is_empty(), "the first stroke is silent");

    // Frame 2: a space advances the VRAM address without a transfer.
    assert!(!handle_text(&mut world, TILES));
    assert_eq!(world.word(TIMER), 3);
    assert_eq!(world.word(VRAM_ADDRESS), 0x5002);
    assert_eq!(world.word(QUEUE_TAIL), 7);
    assert_eq!(world.word(STROKE_TIMER), 1);

    // Frames 3 to 5 count the timer down.
    for timer in [2, 1, 0] {
        assert!(!handle_text(&mut world, TILES));
        assert_eq!(world.word(TIMER), timer);
    }
    assert_eq!(world.word(TEXT), TEXT_START + 10);

    // Frame 6: '!' is drawn as character 5Bh, and the second stroke sounds.
    assert!(!handle_text(&mut world, TILES));
    assert_eq!(world.word(TILEMAP_ENTRY), TILES + 0x5b - 0x41);
    assert_eq!(record(&world, 7), (2, 0x8034, 0x7e, 0x5002));
    assert_eq!(world.word(QUEUE_TAIL), 14);
    assert_eq!(world.word(STROKE_TIMER), 0);
    assert_eq!(world.sounds, [(1, 0x45, 3)], "Ceres: library 2 sound 45h");

    for _ in 0..3 {
        assert!(!handle_text(&mut world, TILES));
    }
    assert!(
        handle_text(&mut world, TILES),
        "the end word finishes the text"
    );
    assert_eq!(
        world.word(TEXT),
        TEXT_START + 11,
        "the end leaves the pointer on it"
    );
}

#[test]
fn zebes_strokes_use_library_three() {
    let mut world = World::new(&[b'A', b'B', 0, 0]);
    assert!(!handle_text(&mut world, 0));
    assert!(!handle_text(&mut world, 0));
    assert_eq!(world.word(TILEMAP_ENTRY), 1, "'B' is one past the base");
    assert_eq!(world.sounds, [(2, 0x0d, 3)]);
}

#[test]
fn a_vram_address_with_bit_15_keeps_it_in_the_record() {
    let mut world = World::new(&[13, 0, 0x00, 0x80, b'A', 0, 0]);
    assert!(!handle_text(&mut world, 0));
    assert_eq!(record(&world, 0).3, 0x8000);
    assert_eq!(world.word(VRAM_ADDRESS), 0x8001);
}
