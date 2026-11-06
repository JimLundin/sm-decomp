//! Milliseconds per composed frame on a synthetic busy screen, against ARCHITECTURE §12's
//! compositor target of at most 0.3.
//! Run with `cargo run --release -p runtime --example compose_bench`.
//!
//! The screen needs no ROM: pseudo-random tiles, maps and colours under the settings a
//! Super Metroid room uses (Mode 1 with BG3 in front, BG1 and BG2 4bpp, BG3 2bpp, 64 sprites
//! on screen, a window, colour math against the sub screen), plus the line program a room
//! frame carries: the HUD split and a scroll write every eighth line.

use std::hint::black_box;
use std::io::{self, Write};
use std::process::ExitCode;
use std::time::Instant;

use data::video::{DisplayFrame, Line, LineWrite, PpuRegisters, VideoMemory};
use runtime::compose::{Picture, compose_into};

const FRAMES: u32 = 2_000;

/// A xorshift generator, so the screen is the same on every run.
struct Noise(u32);

impl Noise {
    const fn next(&mut self) -> u32 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 17;
        self.0 ^= self.0 << 5;
        self.0
    }

    fn word(&mut self) -> u16 {
        self.next()
            .to_le_bytes()
            .into_iter()
            .fold(0, |word, byte| (word << 8) ^ u16::from(byte))
    }

    fn byte(&mut self) -> u8 {
        self.next()
            .to_le_bytes()
            .into_iter()
            .fold(0, |sum, byte| sum ^ byte)
    }
}

/// Video memory with noisy tiles and maps, and 64 sprites spread over the screen.
fn memory(noise: &mut Noise) -> VideoMemory {
    let mut memory = VideoMemory::default();
    for word in &mut *memory.vram {
        *word = noise.word();
    }
    for colour in &mut *memory.cgram {
        *colour = noise.word() & 0x7fff;
    }
    let (low, high) = memory.oam.split_at_mut(512);
    for (sprite, bytes) in low.as_chunks_mut::<4>().0.iter_mut().enumerate() {
        let on_screen = sprite < 64;
        *bytes = [
            noise.byte(),
            if on_screen { noise.byte() % 224 } else { 0xf0 },
            noise.byte(),
            noise.byte(),
        ];
    }
    // Every fourth sprite large, none past X 255.
    high.fill(0b1000_1000);
    memory
}

/// A room's registers: Mode 1, BG3 in front, a colour window, subtractive colour math.
fn start() -> PpuRegisters {
    PpuRegisters {
        inidisp: 0x0f,
        obsel: 0x03,
        bgmode: 0x09,
        bg_sc: [0x51, 0x49, 0x5a, 0],
        bg12nba: 0x00,
        bg34nba: 0x04,
        bg_scroll: [[0x0123, 0x0045], [0x0091, 0x0022], [0, 0], [0, 0]],
        window_select: [0, 0, 0x20],
        window_positions: [0x30, 0xc0, 0, 0xff],
        screens: [0x17, 0x04, 0, 0],
        cgwsel: 0x12,
        cgadsub: 0xb3,
        fixed_color: [4, 2, 8],
        ..PpuRegisters::default()
    }
}

/// The HUD split at line 32 and a BG1 scroll write every eighth line after it.
fn writes() -> Vec<LineWrite> {
    let write = |line: u16, port: u8, value: u8| LineWrite {
        line: Line(line),
        beam_line: line,
        port,
        value,
    };
    let hud = [(0x05, 0x09), (0x2c, 0x17), (0x0d, 0x23), (0x0d, 0x01)]
        .map(|(port, value)| write(32, port, value));
    let scroll = (40..224_u16).step_by(8).flat_map(|line| {
        let low = u8::try_from(line & 0xff).unwrap_or_default();
        [write(line, 0x0d, low), write(line, 0x0d, 0x01)]
    });
    hud.into_iter().chain(scroll).collect()
}

fn main() -> ExitCode {
    let mut noise = Noise(0x2468_ace1);
    let memory = memory(&mut noise);
    let frame = DisplayFrame {
        start: start(),
        writes: writes(),
        overscan: false,
        even_frame: false,
    };
    let mut picture = Picture::default();
    let started = Instant::now();
    for _ in 0..FRAMES {
        compose_into(black_box(&memory), black_box(&frame), &mut picture);
        black_box(&picture);
    }
    let millis = started.elapsed().as_secs_f64() * 1000.0 / f64::from(FRAMES);
    let digest = picture
        .dots
        .iter()
        .fold(0_u32, |sum, &dot| sum.rotate_left(5) ^ u32::from(dot));
    let mut out = io::stdout().lock();
    if writeln!(
        out,
        "compose_bench: {millis:.3} ms/frame over {FRAMES} frames (target 0.3); picture digest {digest:08x}"
    )
    .is_err()
    {
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}
