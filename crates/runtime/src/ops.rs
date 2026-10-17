//! Instruction semantics shared by all generated code.
//! `w` = operation is 16-bit. The emitter resolves widths at codegen time,
//! so these are always called with literal `true`/`false`.
//!
//! Correctness of this file is what the `fuzz` crate checks against the
//! SingleStepTests 65816 corpus. Decimal-mode V flag in particular is marked
//! TODO until those tests pass.

use crate::Machine;
use crate::cpu::flag;

#[inline]
fn mask(w: bool) -> u32 {
    if w { 0xffff } else { 0xff }
}

#[inline]
fn sign(w: bool) -> u32 {
    if w { 0x8000 } else { 0x80 }
}

// ---- register access ---------------------------------------------------------

#[inline]
pub fn get_a(m: &Machine, w: bool) -> u16 {
    if w { m.cpu.a } else { m.cpu.a & 0xff }
}

#[inline]
pub fn set_a(m: &mut Machine, v: u16, w: bool) {
    m.cpu.a = if w { v } else { (m.cpu.a & 0xff00) | (v & 0xff) };
}

#[inline]
fn set_index(v: u16, w: bool) -> u16 {
    if w { v } else { v & 0xff }
}

// ---- loads -------------------------------------------------------------------

pub fn lda(m: &mut Machine, v: u16, w: bool) {
    set_a(m, v, w);
    m.cpu.set_nz(v, w);
}

pub fn ldx(m: &mut Machine, v: u16, w: bool) {
    m.cpu.x = set_index(v, w);
    m.cpu.set_nz(v, w);
}

pub fn ldy(m: &mut Machine, v: u16, w: bool) {
    m.cpu.y = set_index(v, w);
    m.cpu.set_nz(v, w);
}

// ---- logic -------------------------------------------------------------------

pub fn ora(m: &mut Machine, v: u16, w: bool) {
    let r = get_a(m, w) | v;
    lda(m, r, w)
}

pub fn and(m: &mut Machine, v: u16, w: bool) {
    let r = get_a(m, w) & v;
    lda(m, r, w)
}

pub fn eor(m: &mut Machine, v: u16, w: bool) {
    let r = get_a(m, w) ^ v;
    lda(m, r, w)
}

/// BIT. Immediate form only sets Z.
pub fn bit(m: &mut Machine, v: u16, w: bool, imm: bool) {
    let a = get_a(m, w);
    m.cpu.set_flag(flag::Z, (a & v) as u32 & mask(w) == 0);
    if !imm {
        m.cpu.set_flag(flag::N, v as u32 & sign(w) != 0);
        m.cpu.set_flag(flag::V, v as u32 & (sign(w) >> 1) != 0);
    }
}

// ---- arithmetic --------------------------------------------------------------

pub fn adc(m: &mut Machine, v: u16, w: bool) {
    let a = get_a(m, w) as u32;
    let v = v as u32 & mask(w);
    let c = m.cpu.flag(flag::C) as u32;
    let r = if !m.cpu.flag(flag::D) {
        let r = a + v + c;
        m.cpu.set_flag(flag::C, r > mask(w));
        r
    } else {
        // Nibble-wise BCD add. TODO(fuzz): verify V against SingleStepTests.
        let nibbles = if w { 4 } else { 2 };
        let mut carry = c;
        let mut r = 0u32;
        for i in 0..nibbles {
            let sh = i * 4;
            let mut d = ((a >> sh) & 0xf) + ((v >> sh) & 0xf) + carry;
            carry = 0;
            if d > 9 {
                d += 6;
                carry = 1;
            }
            r |= (d & 0xf) << sh;
        }
        m.cpu.set_flag(flag::C, carry != 0);
        r
    };
    let r16 = (r & mask(w)) as u16;
    m.cpu.set_flag(flag::V, (!(a ^ v) & (a ^ r) & sign(w)) != 0);
    lda(m, r16, w);
}

pub fn sbc(m: &mut Machine, v: u16, w: bool) {
    if !m.cpu.flag(flag::D) {
        adc(m, !v & mask(w) as u16, w);
        return;
    }
    // BCD subtract. TODO(fuzz): verify V against SingleStepTests.
    let a = get_a(m, w) as u32;
    let v = v as u32 & mask(w);
    let nibbles = if w { 4 } else { 2 };
    let c_in = m.cpu.flag(flag::C) as u32;
    let mut borrow = 1 - c_in as i32;
    let mut r = 0u32;
    for i in 0..nibbles {
        let sh = i * 4;
        let mut d = ((a >> sh) & 0xf) as i32 - ((v >> sh) & 0xf) as i32 - borrow;
        borrow = 0;
        if d < 0 {
            d += 10;
            borrow = 1;
        }
        r |= ((d as u32) & 0xf) << sh;
    }
    m.cpu.set_flag(flag::C, borrow == 0);
    let bin = a.wrapping_sub(v).wrapping_sub(1 - c_in);
    m.cpu.set_flag(flag::V, ((a ^ v) & (a ^ bin) & sign(w)) != 0);
    lda(m, r as u16, w);
}

/// CMP/CPX/CPY: compare `reg` with `v`.
pub fn cmp(m: &mut Machine, reg: u16, v: u16, w: bool) {
    let r = reg as u32 & mask(w);
    let v = v as u32 & mask(w);
    m.cpu.set_flag(flag::C, r >= v);
    m.cpu.set_nz(r.wrapping_sub(v) as u16, w);
}

// ---- read-modify-write (return the new value; caller stores it) -------------

pub fn asl(m: &mut Machine, v: u16, w: bool) -> u16 {
    let v = v as u32 & mask(w);
    m.cpu.set_flag(flag::C, v & sign(w) != 0);
    let r = ((v << 1) & mask(w)) as u16;
    m.cpu.set_nz(r, w);
    r
}

pub fn lsr(m: &mut Machine, v: u16, w: bool) -> u16 {
    let v = v as u32 & mask(w);
    m.cpu.set_flag(flag::C, v & 1 != 0);
    let r = (v >> 1) as u16;
    m.cpu.set_nz(r, w);
    r
}

pub fn rol(m: &mut Machine, v: u16, w: bool) -> u16 {
    let v = v as u32 & mask(w);
    let c = m.cpu.flag(flag::C) as u32;
    m.cpu.set_flag(flag::C, v & sign(w) != 0);
    let r = (((v << 1) | c) & mask(w)) as u16;
    m.cpu.set_nz(r, w);
    r
}

pub fn ror(m: &mut Machine, v: u16, w: bool) -> u16 {
    let v = v as u32 & mask(w);
    let c = m.cpu.flag(flag::C) as u32;
    m.cpu.set_flag(flag::C, v & 1 != 0);
    let r = ((v >> 1) | if c != 0 { sign(w) } else { 0 }) as u16;
    m.cpu.set_nz(r, w);
    r
}

pub fn inc(m: &mut Machine, v: u16, w: bool) -> u16 {
    let r = (v as u32).wrapping_add(1) & mask(w);
    m.cpu.set_nz(r as u16, w);
    r as u16
}

pub fn dec(m: &mut Machine, v: u16, w: bool) -> u16 {
    let r = (v as u32).wrapping_sub(1) & mask(w);
    m.cpu.set_nz(r as u16, w);
    r as u16
}

pub fn tsb(m: &mut Machine, v: u16, w: bool) -> u16 {
    let a = get_a(m, w);
    m.cpu.set_flag(flag::Z, (a & v) as u32 & mask(w) == 0);
    v | a
}

pub fn trb(m: &mut Machine, v: u16, w: bool) -> u16 {
    let a = get_a(m, w);
    m.cpu.set_flag(flag::Z, (a & v) as u32 & mask(w) == 0);
    v & !a
}

// ---- index registers ---------------------------------------------------------

pub fn inx(m: &mut Machine, w: bool) {
    let v = m.cpu.x;
    m.cpu.x = inc(m, v, w);
}
pub fn iny(m: &mut Machine, w: bool) {
    let v = m.cpu.y;
    m.cpu.y = inc(m, v, w);
}
pub fn dex(m: &mut Machine, w: bool) {
    let v = m.cpu.x;
    m.cpu.x = dec(m, v, w);
}
pub fn dey(m: &mut Machine, w: bool) {
    let v = m.cpu.y;
    m.cpu.y = dec(m, v, w);
}

// ---- transfers ---------------------------------------------------------------
// `m_w` = accumulator width, `x_w` = index width (true = 16-bit).

pub fn tax(m: &mut Machine, x_w: bool) {
    let v = m.cpu.a;
    ldx(m, v, x_w)
}
pub fn tay(m: &mut Machine, x_w: bool) {
    let v = m.cpu.a;
    ldy(m, v, x_w)
}
pub fn txa(m: &mut Machine, m_w: bool) {
    let v = m.cpu.x;
    lda(m, v, m_w)
}
pub fn tya(m: &mut Machine, m_w: bool) {
    let v = m.cpu.y;
    lda(m, v, m_w)
}
pub fn txy(m: &mut Machine, x_w: bool) {
    let v = m.cpu.x;
    ldy(m, v, x_w)
}
pub fn tyx(m: &mut Machine, x_w: bool) {
    let v = m.cpu.y;
    ldx(m, v, x_w)
}
pub fn tsx(m: &mut Machine, x_w: bool) {
    let v = m.cpu.s;
    ldx(m, v, x_w)
}
pub fn txs(m: &mut Machine) {
    m.cpu.s = m.cpu.x;
}
pub fn tcd(m: &mut Machine) {
    m.cpu.d = m.cpu.a;
    let v = m.cpu.d;
    m.cpu.set_nz(v, true);
}
pub fn tdc(m: &mut Machine) {
    m.cpu.a = m.cpu.d;
    let v = m.cpu.a;
    m.cpu.set_nz(v, true);
}
pub fn tcs(m: &mut Machine) {
    m.cpu.s = m.cpu.a;
}
pub fn tsc(m: &mut Machine) {
    m.cpu.a = m.cpu.s;
    let v = m.cpu.a;
    m.cpu.set_nz(v, true);
}
pub fn xba(m: &mut Machine) {
    m.cpu.a = m.cpu.a.rotate_left(8);
    let v = m.cpu.a;
    m.cpu.set_nz(v, false);
}

// ---- flags -------------------------------------------------------------------

pub fn rep(m: &mut Machine, v: u8) {
    let p = m.cpu.p & !v;
    m.cpu.set_p(p)
}
pub fn sep(m: &mut Machine, v: u8) {
    let p = m.cpu.p | v;
    m.cpu.set_p(p)
}
pub fn xce(m: &mut Machine) {
    let c = m.cpu.flag(flag::C);
    m.cpu.set_flag(flag::C, m.cpu.e);
    m.cpu.e = c;
    let p = m.cpu.p;
    m.cpu.set_p(p);
}

// ---- stack -------------------------------------------------------------------

pub fn pha(m: &mut Machine, w: bool) {
    let a = m.cpu.a;
    if w { m.push16(a) } else { m.push8(a as u8) }
}
pub fn pla(m: &mut Machine, w: bool) {
    let v = if w { m.pull16() } else { m.pull8() as u16 };
    lda(m, v, w)
}
pub fn phx(m: &mut Machine, w: bool) {
    let v = m.cpu.x;
    if w { m.push16(v) } else { m.push8(v as u8) }
}
pub fn plx(m: &mut Machine, w: bool) {
    let v = if w { m.pull16() } else { m.pull8() as u16 };
    ldx(m, v, w)
}
pub fn phy(m: &mut Machine, w: bool) {
    let v = m.cpu.y;
    if w { m.push16(v) } else { m.push8(v as u8) }
}
pub fn ply(m: &mut Machine, w: bool) {
    let v = if w { m.pull16() } else { m.pull8() as u16 };
    ldy(m, v, w)
}
pub fn php(m: &mut Machine) {
    let p = m.cpu.p;
    m.push8(p)
}
pub fn plp(m: &mut Machine) {
    let p = m.pull8();
    m.cpu.set_p(p)
}
pub fn phb(m: &mut Machine) {
    let v = m.cpu.db;
    m.push8(v)
}
pub fn plb(m: &mut Machine) {
    let v = m.pull8();
    m.cpu.db = v;
    m.cpu.set_nz(v as u16, false);
}
pub fn phd(m: &mut Machine) {
    let v = m.cpu.d;
    m.push16(v)
}
pub fn pld(m: &mut Machine) {
    let v = m.pull16();
    m.cpu.d = v;
    m.cpu.set_nz(v, true);
}
pub fn phk(m: &mut Machine) {
    let v = m.cpu.pb;
    m.push8(v)
}

// ---- block move ----------------------------------------------------------------

/// MVN/MVP, run to completion. `inc` = true for MVN.
pub fn block_move(m: &mut Machine, dst_bank: u8, src_bank: u8, inc: bool) {
    m.cpu.db = dst_bank;
    let wide_idx = !m.cpu.x8();
    loop {
        let v = m.read8((src_bank as u32) << 16 | m.cpu.x as u32);
        m.write8((dst_bank as u32) << 16 | m.cpu.y as u32, v);
        let (x, y) = if inc {
            (m.cpu.x.wrapping_add(1), m.cpu.y.wrapping_add(1))
        } else {
            (m.cpu.x.wrapping_sub(1), m.cpu.y.wrapping_sub(1))
        };
        m.cpu.x = set_index(x, wide_idx);
        m.cpu.y = set_index(y, wide_idx);
        m.cpu.a = m.cpu.a.wrapping_sub(1);
        if m.cpu.a == 0xffff {
            break;
        }
    }
}
