//! Detached verification of the actual palette core. No Game, CPU, PPU or audio backend.
#![forbid(unsafe_code)]
#![allow(dead_code)]
#[path = "../game/palette_effects/content.rs"]
mod content;
#[path = "../game/palette_effects/execution.rs"]
mod execution;
#[path = "../rom.rs"]
mod rom;
#[path = "../game/palette_effects/state.rs"]
mod state;
// The actual rom module's unused long-record accessor names this plain data record.
mod types {
    pub(crate) struct LongPtr {
        pub addr: u16,
        pub bank: u8,
    }
}
use execution::{Host, SoundChannel};
use rom::{RomImage, RomRecord};
use state::{Field, Session};

#[derive(Clone, Debug, PartialEq, Eq)]
enum Event {
    Color(u16, u16),
    Sound(SoundChannel, u16),
    Music(u16),
}
struct Detached {
    session: Session,
    image: RomImage,
    events: Vec<Event>,
}
impl Detached {
    fn new() -> Self {
        Self {
            session: Session::default(),
            image: RomImage::zeroed(0x70000),
            events: Vec::new(),
        }
    }
    fn byte(&self, address: u16, value: u8) {
        self.image
            .write(0x68000 + usize::from(address & 0x7fff), value);
    }
    fn word(&self, address: u16, value: u16) {
        for (index, byte) in value.to_ne_bytes().into_iter().enumerate() {
            self.image
                .write(0x68000 + usize::from(address & 0x7fff) + index, byte);
        }
    }
}
impl Host for Detached {
    type Record = RomRecord;
    fn session(&self) -> &Session {
        &self.session
    }
    fn session_mut(&mut self) -> &mut Session {
        &mut self.session
    }
    fn record(&mut self, address: u16) -> RomRecord {
        content::record(self.image.clone(), address)
    }
    fn sound(&mut self, channel: SoundChannel, sound: u16) {
        self.events.push(Event::Sound(channel, sound));
    }
    fn music(&mut self, track: u16) {
        self.events.push(Event::Music(track));
    }
    fn color_written(&mut self, index: u16, value: u16) {
        self.events.push(Event::Color(index, value));
    }
    fn unreachable(&mut self) {
        panic!("invalid detached palette command")
    }
    fn outside_field(&mut self, _: Field, _: u16) -> u16 {
        panic!("compatibility slot read")
    }
    fn put_outside_field(&mut self, _: Field, _: u16, _: u16) {
        panic!("compatibility slot write")
    }
    fn put_outside_counter_byte(&mut self, _: u16, _: u8) {
        panic!("compatibility counter byte")
    }
    fn outside_flag(&mut self, _: u16) -> u16 {
        panic!("compatibility flag read")
    }
    fn outside_color(&mut self, _: u16, _: u16) {
        panic!("compatibility palette displacement")
    }
}
fn independent_ticks() {
    let mut host = Detached::new();
    host.word(0x8000, 9291397u32 as u16); // ordinary setup
    host.word(0x8002, 0x9000);
    host.word(0x9000, 2); // first duration
    host.word(0x9002, 0x1234);
    host.word(0x9004, 9291157u32 as u16); // finish color pass; next program = 9006
    host.word(0x9006, 9291358u32 as u16); // queue music
    host.byte(0x9008, 7);
    host.word(0x9009, 9291370u32 as u16); // queue sound, operand reads a native word
    host.word(0x900b, 9); // high byte also begins the next duration
    host.word(0x900c, 1);
    host.word(0x900e, 0x2345);
    host.word(0x9010, 9291157u32 as u16);
    execution::spawn(&mut host, 0x8000);
    assert_eq!(host.session.effects.slots[7].id, 0x8000);
    host.session.set_enabled(true);
    execution::advance(&mut host);
    assert_eq!(host.events, [Event::Color(0, 0x1234)]);
    assert_eq!(host.session.effects.slots[7].program, 0x9006);
    let snapshot = host.session.snapshot();
    host.events.clear();
    execution::advance(&mut host); // timer 2->1; no script work
    assert!(host.events.is_empty());
    execution::advance(&mut host);
    let final_state = host.session.snapshot();
    let final_events = host.events.clone();
    assert_eq!(
        final_events,
        [
            Event::Music(7),
            // Deliberately overlapping byte operands: sound reads 0x0109.
            Event::Sound(SoundChannel::One, 0x0109),
            Event::Color(0, 0x2345),
        ]
    );
    host.session.reset();
    assert_eq!(host.session, Session::default());
    host.session.restore(snapshot);
    host.events.clear();
    execution::advance(&mut host);
    execution::advance(&mut host);
    assert_eq!(host.session, final_state);
    assert_eq!(host.events, final_events);
    // Clear only identities, preserving stale fields and other session state.
    let slot = host.session.effects.slots[7];
    host.session.clear();
    assert_eq!(host.session.effects.slots[7].id, 0);
    assert_eq!(host.session.effects.slots[7].program, slot.program);
}
#[path = "palette_detached_checks.rs"]
mod review_checks;
fn main() {
    independent_ticks();
    review_checks::run();
    println!("palette Session detached checks passed");
}
