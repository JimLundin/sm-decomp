//! Actual device initialization and field-wise observation, synthetic assets only.
use crate::{game_state::Game, recording::RecordingBytes, rom::RomImage};
pub(crate) fn offset(a: u32) -> usize {
    crate::rtl::lorom_offset(a) as usize
}
pub(crate) fn bytes(r: &RomImage, a: u32, v: &[u8]) {
    for (i, b) in v.iter().enumerate() {
        r.write(offset(a) + i, *b);
    }
}
pub(crate) fn rw(r: &RomImage, a: u32, v: u16) {
    bytes(r, a, &v.to_ne_bytes());
}
pub(crate) fn put(g: &mut Game, a: usize, v: u16) {
    g.g_ram[a..a + 2].copy_from_slice(&v.to_le_bytes());
}
pub(crate) fn exp(e: &mut [u8], a: usize, v: u16) {
    e[a..a + 2].copy_from_slice(&v.to_le_bytes());
}
pub(crate) fn literal(r: &RomImage, a: u32, v: &[u8]) {
    assert!(!v.is_empty() && v.len() <= 32);
    bytes(r, a, &[(v.len() - 1) as u8]);
    bytes(r, a + 1, v);
    bytes(r, a + 1 + v.len() as u32, &[0xff]);
}
pub(crate) unsafe fn initialized() -> Box<Game> {
    unsafe {
        let mut g = Game::new();
        g.initialize_backend();
        let image = RomImage::zeroed(0x400000);
        for (a, v) in [(0x80a2f7, 0x4e), (0x80a2f8, 0x18), (0x80a29a, 0x38)] {
            bytes(&image, a, &[v]);
        }
        for i in 0..0x1000 {
            image.write(offset(0x9ab200) + i, (i.wrapping_mul(7) ^ 0xd3) as u8);
        }
        crate::snes::cart::cart_load((*g.g_snes).cart, 1, image.as_mut_ptr(), 0x400000, 8192);
        g.register_native_rom((*(*g.g_snes).cart).rom.clone());
        g.g_sram = (*(*g.g_snes).cart).ram.clone();
        crate::snes::snes::snes_reset(g.g_snes, true, &mut g);
        for (i, b) in g.g_ram.iter_mut().enumerate() {
            *b = (i.wrapping_mul(13) ^ (i >> 8) ^ 0x5a) as u8;
        }
        (&mut (*(*g.g_snes).ppu).vram).fill(0x4567);
        g.VERIFY_ACTIVE = true;
        g.g_fail = false;
        g.decompress_src = 0xabcdef;
        g
    }
}
/// Existing typed serializer omits pointers/routing caches. It does not call
/// snes_saveload, whose runningWhichVersion write is an observable side effect.
#[forbid(unsafe_code)]
pub(crate) fn snapshot(g: &mut Game) -> Vec<u8> {
    let mut out = g.g_ram.to_vec();
    out.push(u8::from(g.g_fail));
    out.extend_from_slice(&g.decompress_src.to_le_bytes());
    let mut machine = RecordingBytes::default();
    g.save_backend_observation(&mut machine);
    out.extend_from_slice(&(machine.size as u32).to_le_bytes());
    out.extend_from_slice(machine.as_slice());
    out
}

pub(crate) fn record(out: &mut Vec<u8>, tag: u8, payload: &[u8]) {
    out.push(tag);
    out.extend_from_slice(&(payload.len() as u32).to_le_bytes());
    out.extend_from_slice(payload);
}
pub(crate) fn word(out: &mut Vec<(u16, u8)>, a: u16, v: u16) {
    out.push((a, v as u8));
    out.push((a + 1, (v >> 8) as u8));
}
pub(crate) fn copy(out: &mut Vec<(u16, u8)>, dst: u16, src: u32, n: u16) {
    out.push((0x2100, 0x80));
    word(out, 0x2116, dst);
    word(out, 0x4310, 0x1801);
    word(out, 0x4312, src as u16);
    out.push((0x4314, (src >> 16) as u8));
    word(out, 0x4315, n);
    out.extend([(0x2115, 0x80), (0x420b, 2), (0x2100, 15)]);
}
pub(crate) fn direct(out: &mut Vec<(u16, u8)>, dst: u16, src: u16, bankword: u16, n: Option<u16>) {
    word(out, 0x2116, dst);
    word(out, 0x4310, 0x1801);
    word(out, 0x4312, src);
    word(out, 0x4314, bankword);
    if let Some(n) = n {
        word(out, 0x4315, n);
        out.extend([(0x2115, 0x80), (0x420b, 2)]);
    }
}
pub(crate) fn clear_initial(out: &mut Vec<(u16, u8)>, dst: u16, low: u16, high: u16, n: u16) {
    word(out, 0x2116, dst);
    word(out, 0x4310, 0x1808);
    word(out, 0x4312, low);
    word(out, 0x4314, 0x80);
    word(out, 0x4315, n);
    out.extend([(0x2115, 0), (0x420b, 2)]);
    word(out, 0x2116, dst);
    word(out, 0x4310, 0x1908);
    word(out, 0x4312, high);
    word(out, 0x4314, 0x80);
    word(out, 0x4315, n);
    out.extend([(0x2115, 0x80), (0x420b, 2)]);
}
pub(crate) fn vram_bytes(v: &mut [u16], dst: u16, data: &[u8]) {
    assert_eq!(data.len() % 2, 0);
    for (i, b) in data.chunks_exact(2).enumerate() {
        v[(usize::from(dst) + i) & 0x7fff] = u16::from_le_bytes([b[0], b[1]]);
    }
}
