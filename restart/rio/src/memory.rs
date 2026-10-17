pub(crate) const SIZE: usize = 0x20000;
pub(crate) const CURRENT: usize = 0xe54;
pub(crate) const ENEMIES: usize = 0xf78;
pub(crate) const A: usize = 0x30;
pub(crate) const B: usize = 0x32;
pub(crate) const C: usize = 0x34;
pub(crate) const D: usize = 0x36;
pub(crate) const E: usize = 0x38;
pub(crate) const F: usize = 0x3a;
pub(crate) const STATES: [u16; 5] = [0xbbed, 0xbc32, 0xbc48, 0xbcb7, 0xbcff];
pub(crate) fn word(r: &[u8], at: usize) -> u16 {
    u16::from_le_bytes([r[at], r[at + 1]])
}
pub(crate) fn put(r: &mut [u8], at: usize, value: u16) {
    r[at..at + 2].copy_from_slice(&value.to_le_bytes());
}
pub(crate) fn slot(k: u16) -> Result<usize, String> {
    if k >= 0x800 || k & 63 != 0 {
        return Err("invalid Rio enemy byte index".into());
    }
    Ok(ENEMIES + usize::from(k))
}
pub(crate) fn current(r: &[u8]) -> Result<usize, String> {
    slot(word(r, CURRENT))
}
pub(crate) fn rom_word(rom: &[u8], a: usize) -> Result<u16, String> {
    let b = rom.get(a..a + 2).ok_or("missing Rio runtime ROM speed table")?;
    Ok(u16::from_le_bytes([b[0], b[1]]))
}
pub(crate) fn near(r: &[u8], base: usize) -> bool {
    let delta = word(r, 0xaf6).wrapping_sub(word(r, base + 2));
    let absolute = if delta & 0x8000 != 0 { delta.wrapping_neg() } else { delta };
    absolute < 0xa0
}
pub(crate) fn on_screen(r: &[u8], base: usize) -> bool {
    let x = word(r, base + 2);
    let y = word(r, base + 6);
    let left = word(r, 0x911);
    let top = word(r, 0x915);
    [
        x.wrapping_sub(left),
        left.wrapping_add(256).wrapping_sub(x),
        y.wrapping_sub(top),
        top.wrapping_add(256).wrapping_sub(y),
    ]
    .iter()
    .all(|v| (*v as i16) >= 0)
}
pub(crate) fn animation(r: &mut [u8], a: u16) -> Result<(), String> {
    let base = current(r)?;
    if word(r, base + F) != a {
        put(r, base + F, a);
        put(r, base + 0x1a, a);
        put(r, base + 0x1c, 1);
        put(r, base + 0x18, 0);
    }
    Ok(())
}
pub(crate) fn random(r: &mut [u8]) {
    let old = word(r, 0x5e5);
    let low = (old & 255) * 5;
    let high = ((old >> 8) * 5) & 255;
    let carry = (low >> 8) + high + 1;
    let result = ((carry & 255) << 8 | (low & 255)).wrapping_add(carry >> 8).wrapping_add(17);
    put(r, 0x5e5, result);
}
pub(crate) fn sound_preflight(r: &[u8]) -> Result<(), String> {
    if r[0x644] >= 16 || r[0x647] >= 16 {
        return Err("invalid Rio SFX2 queue indices".into());
    }
    Ok(())
}
pub(crate) fn sound(r: &mut [u8]) {
    // sfx_max_queued is an array of bytes, despite the 16-bit queue argument.
    r[0x654] = 6;
    let read = r[0x644];
    let write = r[0x647];
    if write.wrapping_sub(read) & 15 >= 6
        || word(r, 0x5f5) != 0
        || word(r, 0x998) >= 0x28
        || word(r, 0x592) & 0x8000 != 0
    {
        return;
    }
    let next = (write + 1) & 15;
    if next == read {
        let target = &mut r[0x666 + usize::from(write)];
        if 0x65 < *target {
            *target = 0x65;
        }
    } else {
        r[0x666 + usize::from(write)] = 0x65;
        r[0x647] = next;
        r[0x666 + usize::from(next)] = 0;
    }
}
