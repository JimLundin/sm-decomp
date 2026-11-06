use super::{WIDTH, compose};
use data::video::{DisplayFrame, Line, LineWrite, PpuRegisters, VideoMemory};

/// A Mode 1 screen: BG1 maps every cell to tile 1 at word $0400, tiles at $2000, tile 1's
/// rows all colour 1; colour 0 is blue and colour 1 red; every sprite is off screen.
fn screen() -> (VideoMemory, PpuRegisters) {
    let mut memory = VideoMemory::default();
    for word in memory.vram.iter_mut().skip(0x400).take(0x400) {
        *word = 1;
    }
    for word in memory.vram.iter_mut().skip(0x2010).take(8) {
        *word = 0x00ff;
    }
    let [backdrop, red, ..] = &mut *memory.cgram;
    (*backdrop, *red) = (0x7c00, 0x001f);
    for (index, byte) in memory.oam.iter_mut().enumerate().take(512) {
        *byte = if index % 4 == 1 { 0xf0 } else { 0 };
    }
    let registers = PpuRegisters {
        inidisp: 0x0f,
        bgmode: 0x09,
        bg_sc: [0x04, 0, 0, 0],
        bg12nba: 0x02,
        screens: [0x01, 0, 0, 0],
        ..PpuRegisters::default()
    };
    (memory, registers)
}

#[test]
fn a_shown_background_and_forced_blank_lines() {
    let (memory, start) = screen();
    let frame = DisplayFrame {
        start,
        writes: vec![LineWrite {
            line: Line(100),
            beam_line: 100,
            port: 0x00,
            value: 0x8f,
        }],
        ..DisplayFrame::default()
    };
    let picture = compose(&memory, &frame);
    let row = |line: usize| picture.dots.iter().skip(line * WIDTH).take(WIDTH);
    assert!(row(0).all(|&dot| dot == 0x001f));
    assert!(row(99).all(|&dot| dot == 0));
    assert_eq!(picture.forced_blank.get(98..100), Some(&[false, true][..]));
    assert_eq!(picture.brightness.first(), Some(&15));
}

#[test]
fn a_window_masks_its_span() {
    let (memory, mut start) = screen();
    // BG1 masked on the main screen inside window 1 (dots 10-19).
    start.window_select = [0x02, 0, 0];
    start.window_positions = [10, 19, 0, 0];
    start.screens = [0x01, 0, 0x01, 0];
    let picture = compose(
        &memory,
        &DisplayFrame {
            start,
            ..DisplayFrame::default()
        },
    );
    let dots: Vec<u16> = picture.dots.iter().skip(9).take(12).copied().collect();
    let mut expected = vec![0x7c00; 12];
    expected[0] = 0x001f;
    expected[11] = 0x001f;
    assert_eq!(dots, expected);
}
