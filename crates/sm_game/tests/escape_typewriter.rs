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
use sm_game::actors::slot_view::SlotState;
use sm_game::cartridge::{NativeDiagnostics, NativeImage};
use sm_game::display::transfer::TilemapQueue;
use sm_game::effects::SoundEffects;
use sm_game::escape_typewriter::{Host, Typewriter, handle_text, set_up_zebes, timer_reset};
use sm_game::external_inputs::ExternalInputs;
use sm_game::projection::{Decode, Field, Layout, Views};
use sm_game::ram::CompatibilityRam;
use sm_game::room_entry::world::WorldBinding;
use sm_game::session::Session;
use sm_game::session_access::SessionAccess;

const QUEUE_TAIL: usize = 0x330;
const QUEUE: usize = 0xd0;
/// $A6:8000, where the test's text starts, in the LoROM image.
const TEXT_START: u16 = 0x8000;
const TEXT_OFFSET: usize = 0x13_0000;
const TILES: u16 = 0x2610;

/// The typewriter's state is slot 0's, in the Session; the rest of the world
/// (scroll, tilemap queue) is a RAM image.
struct World {
    session: Session,
    inputs: ExternalInputs,
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
            session: Session::with_world(WorldBinding::canonical_legacy()),
            inputs: ExternalInputs::default(),
            ram: vec![0; 0x2_0000],
            rom,
            colours: [0; 256],
            area: 0,
            sounds: Vec::new(),
        };
        world.set_state(|state| state.text = TEXT_START);
        world
    }
    /// The typewriter's state.
    fn state(&self) -> State {
        let view = Typewriter::of(self, 0);
        State {
            tilemap_entry: view.tilemap_entry(),
            text: view.text(),
            vram_address: view.vram_address(),
            timer: view.timer(),
            timer_reset: timer_reset(self),
            stroke_timer: view.stroke_timer(),
        }
    }
    /// Store the typewriter's state as `change` leaves it.
    fn set_state(&mut self, change: impl FnOnce(&mut State)) {
        let mut state = self.state();
        change(&mut state);
        let mut edit = sm_game::escape_typewriter::TypewriterEdit::of(self, 0);
        *edit.tilemap_entry() = state.tilemap_entry;
        *edit.text() = state.text;
        *edit.vram_address() = state.vram_address;
        *edit.timer() = state.timer;
        *edit.stroke_timer() = state.stroke_timer;
        drop(edit);
        sm_game::escape_typewriter::set_timer_reset(self, state.timer_reset);
    }
    fn word(&self, at: usize) -> u16 {
        u16::from_le_bytes([self.ram[at], self.ram[at + 1]])
    }
    fn set_word(&mut self, at: usize, value: u16) {
        self.ram[at..at + 2].copy_from_slice(&value.to_le_bytes());
    }
}

/// Slot 0's typewriter words ($7E:8034..$7E:803F).
#[derive(Clone, Copy, Debug, PartialEq)]
struct State {
    tilemap_entry: u16,
    text: u16,
    vram_address: u16,
    timer: u16,
    timer_reset: u16,
    stroke_timer: u16,
}

impl SessionAccess for World {
    fn session(&self) -> &Session {
        &self.session
    }
    fn session_mut(&mut self) -> &mut Session {
        &mut self.session
    }
    fn inputs(&self) -> &ExternalInputs {
        &self.inputs
    }
    fn probe(&self, _: &'static str) {}
}
impl SlotState for World {
    fn record_stored(&mut self) {}
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
impl sm_game::display_scroll::Access for World {
    fn scroll_word(&self, word: sm_game::display_scroll::Word) -> u16 {
        let at = 0xb1 + 2 * usize::from(word);
        u16::from_le_bytes([self.ram[at], self.ram[at + 1]])
    }
    fn set_scroll_word(&mut self, word: sm_game::display_scroll::Word, value: u16) {
        let at = 0xb1 + 2 * usize::from(word);
        self.ram[at..at + 2].copy_from_slice(&value.to_le_bytes());
    }
    fn physical_scroll_byte(&self, word: sm_game::display_scroll::Word, high: bool) -> u8 {
        self.ram[0xb1 + 2 * usize::from(word) + usize::from(high)]
    }
    fn write_physical_scroll_byte(
        &mut self,
        word: sm_game::display_scroll::Word,
        high: bool,
        value: u8,
    ) {
        self.ram[0xb1 + 2 * usize::from(word) + usize::from(high)] = value;
    }
}
impl sm_game::gameplay_context::Access for World {
    fn gameplay_word(&self, _: sm_game::gameplay_context::Word) -> u16 {
        panic!("input and game-loop owner was not supplied")
    }
    fn set_gameplay_byte(&mut self, _: sm_game::gameplay_context::Word, _: usize, _: u8) {
        panic!("input and game-loop owner was not supplied")
    }
}
impl CompatibilityRam for World {
    fn observed_inputs(&self) -> ExternalInputs {
        sm_game::session_encoding::inputs(&|at| self.ram[at])
    }
}
impl Views for World {
    fn view_word(&self, field: Field) -> u16 {
        self.word(field.address())
    }
    fn set_view_word(&mut self, field: Field, value: u16) {
        self.set_word(field.address(), value);
    }
    fn set_view_low_byte(&mut self, field: Field, value: u8) {
        self.ram[field.address()] = value;
    }
    fn decode_view<T: Decode>(&self) -> T {
        T::decode(&|at| self.ram[at])
    }
    fn load_words(&self, words: &mut impl Layout) {
        words.visit(|at, word| *word = self.word(at));
    }
    fn store_words(&mut self, words: &mut impl Layout, mut select: impl FnMut(usize) -> bool) {
        let mut index = 0;
        words.visit(|at, word| {
            if select(index) {
                self.set_word(at, *word);
            }
            index += 1;
        });
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
    fn area_index(&self) -> u16 {
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
    world.set_state(|state| {
        state.vram_address = 0xffff;
        state.timer = 0xffff;
        state.timer_reset = 0xffff;
        state.stroke_timer = 0xffff;
    });
    for at in [0xb5, 0xb7] {
        world.set_word(at, 0xffff);
    }
    set_up_zebes(&mut world);
    assert_eq!((world.colours[157], world.colours[158]), (0x1234, 0x5678));
    assert_eq!(
        world.state().text,
        0xc49c,
        "Typewriter text - Zebes escape timer"
    );
    let state = world.state();
    assert_eq!(
        (
            state.vram_address,
            state.timer,
            state.timer_reset,
            state.stroke_timer
        ),
        (0, 0, 0, 0)
    );
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
    assert_eq!(world.state().timer_reset, 3);
    assert_eq!(world.state().timer, 0);
    assert_eq!(world.state().tilemap_entry, TILES);
    assert_eq!(record(&world, 0), (2, 0x8034, 0x7e, 0x5000));
    assert_eq!(world.word(QUEUE_TAIL), 7);
    assert_eq!(world.state().vram_address, 0x5001);
    assert_eq!(
        world.state().text,
        TEXT_START + 9,
        "past the instructions and one byte"
    );
    assert_eq!(world.state().stroke_timer, 1);
    assert!(world.sounds.is_empty(), "the first stroke is silent");

    // Frame 2: a space advances the VRAM address without a transfer.
    assert!(!handle_text(&mut world, TILES));
    assert_eq!(world.state().timer, 3);
    assert_eq!(world.state().vram_address, 0x5002);
    assert_eq!(world.word(QUEUE_TAIL), 7);
    assert_eq!(world.state().stroke_timer, 1);

    // Frames 3 to 5 count the timer down.
    for timer in [2, 1, 0] {
        assert!(!handle_text(&mut world, TILES));
        assert_eq!(world.state().timer, timer);
    }
    assert_eq!(world.state().text, TEXT_START + 10);

    // Frame 6: '!' is drawn as character 5Bh, and the second stroke sounds.
    assert!(!handle_text(&mut world, TILES));
    assert_eq!(world.state().tilemap_entry, TILES + 0x5b - 0x41);
    assert_eq!(record(&world, 7), (2, 0x8034, 0x7e, 0x5002));
    assert_eq!(world.word(QUEUE_TAIL), 14);
    assert_eq!(world.state().stroke_timer, 0);
    assert_eq!(world.sounds, [(1, 0x45, 3)], "Ceres: library 2 sound 45h");

    for _ in 0..3 {
        assert!(!handle_text(&mut world, TILES));
    }
    assert!(
        handle_text(&mut world, TILES),
        "the end word finishes the text"
    );
    assert_eq!(
        world.state().text,
        TEXT_START + 11,
        "the end leaves the pointer on it"
    );
}

#[test]
fn zebes_strokes_use_library_three() {
    let mut world = World::new(&[b'A', b'B', 0, 0]);
    assert!(!handle_text(&mut world, 0));
    assert!(!handle_text(&mut world, 0));
    assert_eq!(world.state().tilemap_entry, 1, "'B' is one past the base");
    assert_eq!(world.sounds, [(2, 0x0d, 3)]);
}

#[test]
fn a_vram_address_with_bit_15_keeps_it_in_the_record() {
    let mut world = World::new(&[13, 0, 0x00, 0x80, b'A', 0, 0]);
    assert!(!handle_text(&mut world, 0));
    assert_eq!(record(&world, 0).3, 0x8000);
    assert_eq!(world.state().vram_address, 0x8001);
}

impl sm_game::sequence_state::Access for World {
    fn set_cinematic_function(&mut self, value: u16) {
        for (part, byte) in value.to_le_bytes().into_iter().enumerate() {
            self.ram[0x1f51 + part] = byte;
        }
    }
    fn set_sequence_word(&mut self, word: sm_game::session_words::Word, value: u16) {
        for (part, byte) in value.to_le_bytes().into_iter().enumerate() {
            self.ram[(word as usize) + part] = byte;
        }
    }
    fn sequence_flow(&self) -> sm_game::flow::Flow {
        panic!("sequence controls were not supplied")
    }
    fn sequence_words(&self) -> sm_game::session_words::SessionWords {
        panic!("sequence controls were not supplied")
    }
    fn commit_flow(&mut self, before: &sm_game::flow::Flow, after: sm_game::flow::Flow) {
        let _ = (before, after);
        panic!("sequence controls were not supplied");
    }
    fn commit_sequence_words(
        &mut self,
        before: &sm_game::session_words::SessionWords,
        after: sm_game::session_words::SessionWords,
    ) {
        let _ = (before, after);
        panic!("sequence controls were not supplied");
    }
}
