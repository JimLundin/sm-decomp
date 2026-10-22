use crate::game_state::Game;
use crate::snes::apu::apu_cpuRead;
use crate::snes::snes::snes_read;
use crate::types::{Apu, Snes};
use std::fmt::Write;

#[rustfmt::skip]
static opcodeNames: [&str; 256] = [
    "brk          ", "ora ($%02x,x)  ", "cop #$%02x     ", "ora $%02x,s    ",
    "tsb $%02x      ", "ora $%02x      ", "asl $%02x      ", "ora [$%02x]    ",
    "php          ", "ora #$%04x   ", "asl          ", "phd          ",
    "tsb $%04x    ", "ora $%04x    ", "asl $%04x    ", "ora $%06x  ",
    "bpl $%04x    ", "ora ($%02x),y  ", "ora ($%02x)    ", "ora ($%02x,s),y",
    "trb $%02x      ", "ora $%02x,x    ", "asl $%02x,x    ", "ora [$%02x],y  ",
    "clc          ", "ora $%04x,y  ", "inc          ", "tcs          ",
    "trb $%04x    ", "ora $%04x,x  ", "asl $%04x,x  ", "ora $%06x,x",
    "jsr $%04x    ", "and ($%02x,x)  ", "jsl $%06x  ", "and $%02x,s    ",
    "bit $%02x      ", "and $%02x      ", "rol $%02x      ", "and [$%02x]    ",
    "plp          ", "and #$%04x   ", "rol          ", "pld          ",
    "bit $%04x    ", "and $%04x    ", "rol $%04x    ", "and $%06x  ",
    "bmi $%04x    ", "and ($%02x),y  ", "and ($%02x)    ", "and ($%02x,s),y",
    "bit $%02x,x    ", "and $%02x,x    ", "rol $%02x,x    ", "and [$%02x],y  ",
    "sec          ", "and $%04x,y  ", "dec          ", "tsc          ",
    "bit $%04x,x  ", "and $%04x,x  ", "rol $%04x,x  ", "and $%06x,x",
    "rti          ", "eor ($%02x,x)  ", "wdm #$%02x     ", "eor $%02x,s    ",
    "mvp $%02x, $%02x ", "eor $%02x      ", "lsr $%02x      ", "eor [$%02x]    ",
    "pha          ", "eor #$%04x   ", "lsr          ", "phk          ",
    "jmp $%04x    ", "eor $%04x    ", "lsr $%04x    ", "eor $%06x  ",
    "bvc $%04x    ", "eor ($%02x),y  ", "eor ($%02x)    ", "eor ($%02x,s),y",
    "mvn $%02x, $%02x ", "eor $%02x,x    ", "lsr $%02x,x    ", "eor [$%02x],y  ",
    "cli          ", "eor $%04x,y  ", "phy          ", "tcd          ",
    "jml $%06x  ", "eor $%04x,x  ", "lsr $%04x,x  ", "eor $%06x,x",
    "rts          ", "adc ($%02x,x)  ", "per $%04x    ", "adc $%02x,s    ",
    "stz $%02x      ", "adc $%02x      ", "ror $%02x      ", "adc [$%02x]    ",
    "pla          ", "adc #$%04x   ", "ror          ", "rtl          ",
    "jmp ($%04x)  ", "adc $%04x    ", "ror $%04x    ", "adc $%06x  ",
    "bvs $%04x    ", "adc ($%02x),y  ", "adc ($%02x)    ", "adc ($%02x,s),y",
    "stz $%02x,x    ", "adc $%02x,x    ", "ror $%02x,x    ", "adc [$%02x],y  ",
    "sei          ", "adc $%04x,y  ", "ply          ", "tdc          ",
    "jmp ($%04x,x)", "adc $%04x,x  ", "ror $%04x,x  ", "adc $%06x,x",
    "bra $%04x    ", "sta ($%02x,x)  ", "brl $%04x    ", "sta $%02x,s    ",
    "sty $%02x      ", "sta $%02x      ", "stx $%02x      ", "sta [$%02x]    ",
    "dey          ", "bit #$%04x   ", "txa          ", "phb          ",
    "sty $%04x    ", "sta $%04x    ", "stx $%04x    ", "sta $%06x  ",
    "bcc $%04x    ", "sta ($%02x),y  ", "sta ($%02x)    ", "sta ($%02x,s),y",
    "sty $%02x,x    ", "sta $%02x,x    ", "stx $%02x,y    ", "sta [$%02x],y  ",
    "tya          ", "sta $%04x,y  ", "txs          ", "txy          ",
    "stz $%04x    ", "sta $%04x,x  ", "stz $%04x,x  ", "sta $%06x,x",
    "ldy #$%04x   ", "lda ($%02x,x)  ", "ldx #$%04x   ", "lda $%02x,s    ",
    "ldy $%02x      ", "lda $%02x      ", "ldx $%02x      ", "lda [$%02x]    ",
    "tay          ", "lda #$%04x   ", "tax          ", "plb          ",
    "ldy $%04x    ", "lda $%04x    ", "ldx $%04x    ", "lda $%06x  ",
    "bcs $%04x    ", "lda ($%02x),y  ", "lda ($%02x)    ", "lda ($%02x,s),y",
    "ldy $%02x,x    ", "lda $%02x,x    ", "ldx $%02x,y    ", "lda [$%02x],y  ",
    "clv          ", "lda $%04x,y  ", "tsx          ", "tyx          ",
    "ldy $%04x,x  ", "lda $%04x,x  ", "ldx $%04x,y  ", "lda $%06x,x",
    "cpy #$%04x   ", "cmp ($%02x,x)  ", "rep #$%02x     ", "cmp $%02x,s    ",
    "cpy $%02x      ", "cmp $%02x      ", "dec $%02x      ", "cmp [$%02x]    ",
    "iny          ", "cmp #$%04x   ", "dex          ", "wai          ",
    "cpy $%04x    ", "cmp $%04x    ", "dec $%04x    ", "cmp $%06x  ",
    "bne $%04x    ", "cmp ($%02x),y  ", "cmp ($%02x)    ", "cmp ($%02x,s),y",
    "pei $%02x      ", "cmp $%02x,x    ", "dec $%02x,x    ", "cmp [$%02x],y  ",
    "cld          ", "cmp $%04x,y  ", "phx          ", "stp          ",
    "jml [$%04x]  ", "cmp $%04x,x  ", "dec $%04x,x  ", "cmp $%06x,x",
    "cpx #$%04x   ", "sbc ($%02x,x)  ", "sep #$%02x     ", "sbc $%02x,s    ",
    "cpx $%02x      ", "sbc $%02x      ", "inc $%02x      ", "sbc [$%02x]    ",
    "inx          ", "sbc #$%04x   ", "nop          ", "xba          ",
    "cpx $%04x    ", "sbc $%04x    ", "inc $%04x    ", "sbc $%06x  ",
    "beq $%04x    ", "sbc ($%02x),y  ", "sbc ($%02x)    ", "sbc ($%02x,s),y",
    "pea #$%04x   ", "sbc $%02x,x    ", "inc $%02x,x    ", "sbc [$%02x],y  ",
    "sed          ", "sbc $%04x,y  ", "plx          ", "xce          ",
    "jsr ($%04x,x)", "sbc $%04x,x  ", "inc $%04x,x  ", "sbc $%06x,x",
];
static opcodeType: [i32; 256] = [
    0, 1, 1, 1, 1, 1, 1, 1, 0, 4, 0, 0, 2, 2, 2, 3, 6, 1, 1, 1, 1, 1, 1, 1, 0, 2, 0, 0, 2, 2, 2, 3,
    2, 1, 3, 1, 1, 1, 1, 1, 0, 4, 0, 0, 2, 2, 2, 3, 6, 1, 1, 1, 1, 1, 1, 1, 0, 2, 0, 0, 2, 2, 2, 3,
    0, 1, 1, 1, 8, 1, 1, 1, 0, 4, 0, 0, 2, 2, 2, 3, 6, 1, 1, 1, 8, 1, 1, 1, 0, 2, 0, 0, 3, 2, 2, 3,
    0, 1, 7, 1, 1, 1, 1, 1, 0, 4, 0, 0, 2, 2, 2, 3, 6, 1, 1, 1, 1, 1, 1, 1, 0, 2, 0, 0, 2, 2, 2, 3,
    6, 1, 7, 1, 1, 1, 1, 1, 0, 4, 0, 0, 2, 2, 2, 3, 6, 1, 1, 1, 1, 1, 1, 1, 0, 2, 0, 0, 2, 2, 2, 3,
    5, 1, 5, 1, 1, 1, 1, 1, 0, 4, 0, 0, 2, 2, 2, 3, 6, 1, 1, 1, 1, 1, 1, 1, 0, 2, 0, 0, 2, 2, 2, 3,
    5, 1, 1, 1, 1, 1, 1, 1, 0, 4, 0, 0, 2, 2, 2, 3, 6, 1, 1, 1, 1, 1, 1, 1, 0, 2, 0, 0, 2, 2, 2, 3,
    5, 1, 1, 1, 1, 1, 1, 1, 0, 4, 0, 0, 2, 2, 2, 3, 6, 1, 1, 1, 2, 1, 1, 1, 0, 2, 0, 0, 2, 2, 2, 3,
];
#[rustfmt::skip]
static opcodeNamesSpc: [&str; 256] = [
    "nop              ", "tcall 0          ", "set1 $%02x.0       ", "bbs $%02x.0, $%04x ",
    "or a, $%02x        ", "or a, $%04x      ", "or a, [X]        ", "or a, [$%02x+x]    ",
    "or a, #$%02x       ", "or $%02x, $%02x      ", "or1 c, $%04x.%01x   ", "asl $%02x          ",
    "asl $%04x        ", "push p           ", "tset $%04x       ", "brk              ",
    "bpl $%04x        ", "tcall 1          ", "clr1 $%02x.0       ", "bbc $%02x.0, $%04x ",
    "or a, $%02x+x      ", "or a, $%04x+x    ", "or a, $%04x+y    ", "or a, [$%02x]+y    ",
    "or $%02x, #$%02x     ", "or [X], [Y]      ", "decw $%02x         ", "asl $%02x+x        ",
    "asl a            ", "dec x            ", "cmp x, $%04x     ", "jmp [$%04x+x]    ",
    "clrp             ", "tcall 2          ", "set1 $%02x.1       ", "bbs $%02x.1, $%04x ",
    "and a, $%02x       ", "and a, $%04x     ", "and a, [X]       ", "and a, [$%02x+x]   ",
    "and a, #$%02x      ", "and $%02x, $%02x     ", "or1 c, /$%04x.%01x  ", "rol $%02x          ",
    "rol $%04x        ", "push a           ", "cbne $%02x, $%04x  ", "bra $%04x        ",
    "bmi $%04x        ", "tcall 3          ", "clr1 $%02x.1       ", "bbc $%02x.1, $%04x ",
    "and a, $%02x+x     ", "and a, $%04x+x   ", "and a, $%04x+y   ", "and a, [$%02x]+y   ",
    "and $%02x, #$%02x    ", "and [X], [Y]     ", "incw $%02x         ", "rol $%02x+x        ",
    "rol a            ", "inc x            ", "cmp x, $%02x       ", "call $%04x       ",
    "setp             ", "tcall 4          ", "set1 $%02x.2       ", "bbs $%02x.2, $%04x ",
    "eor a, $%02x       ", "eor a, $%04x     ", "eor a, [X]       ", "eor a, [$%02x+x]   ",
    "eor a, #$%02x      ", "eor $%02x, $%02x     ", "and1 c, $%04x.%01x  ", "lsr $%02x          ",
    "lsr $%04x        ", "push x           ", "tclr $%04x       ", "pcall $%02x        ",
    "bvc $%04x        ", "tcall 5          ", "clr1 $%02x.2       ", "bbc $%02x.2, $%04x ",
    "eor a, $%02x+x     ", "eor a, $%04x+x   ", "eor a, $%04x+y   ", "eor a, [$%02x]+y   ",
    "eor $%02x, #$%02x    ", "eor [X], [Y]     ", "cmpw ya, $%02x     ", "lsr $%02x+x        ",
    "lsr a            ", "mov x, a         ", "cmp y, $%04x     ", "jmp $%04x        ",
    "clrc             ", "tcall 6          ", "set1 $%02x.3       ", "bbs $%02x.3, $%04x ",
    "cmp a, $%02x       ", "cmp a, $%04x     ", "cmp a, [X]       ", "cmp a, [$%02x+x]   ",
    "cmp a, #$%02x      ", "cmp $%02x, $%02x     ", "and1 c, /$%04x.%01x ", "ror $%02x          ",
    "ror $%04x        ", "push y           ", "dbnz $%02x, $%04x  ", "ret              ",
    "bvs $%04x        ", "tcall 7          ", "clr1 $%02x.3       ", "bbc $%02x.3, $%04x ",
    "cmp a, $%02x+x     ", "cmp a, $%04x+x   ", "cmp a, $%04x+y   ", "cmp a, [$%02x]+y   ",
    "cmp $%02x, #$%02x    ", "cmp [X], [Y]     ", "addw ya, $%02x     ", "ror $%02x+x        ",
    "ror a            ", "mov a, x         ", "cmp y, $%02x       ", "reti             ",
    "setc             ", "tcall 8          ", "set1 $%02x.4       ", "bbs $%02x.4, $%04x ",
    "adc a, $%02x       ", "adc a, $%04x     ", "adc a, [X]       ", "adc a, [$%02x+x]   ",
    "adc a, #$%02x      ", "adc $%02x, $%02x     ", "eor1 c, $%04x.%01x  ", "dec $%02x          ",
    "dec $%04x        ", "mov y, #$%02x      ", "pop p            ", "mov $%02x, #$%02x    ",
    "bcc $%04x        ", "tcall 9          ", "clr1 $%02x.4       ", "bbc $%02x.4, $%04x ",
    "adc a, $%02x+x     ", "adc a, $%04x+x   ", "adc a, $%04x+y   ", "adc a, [$%02x]+y   ",
    "adc $%02x, #$%02x    ", "adc [X], [Y]     ", "subw ya, $%02x     ", "dec $%02x+x        ",
    "dec a            ", "mov x, sp        ", "div ya, x        ", "xcn a            ",
    "ei               ", "tcall 10         ", "set1 $%02x.5       ", "bbs $%02x.5, $%04x ",
    "sbc a, $%02x       ", "sbc a, $%04x     ", "sbc a, [X]       ", "sbc a, [$%02x+x]   ",
    "sbc a, #$%02x      ", "sbc $%02x, $%02x     ", "mov1 c, $%04x.%01x  ", "inc $%02x          ",
    "inc $%04x        ", "cmp y, #$%02x      ", "pop a            ", "mov [x+], a      ",
    "bcs $%04x        ", "tcall 11         ", "clr1 $%02x.5       ", "bbc $%02x.5, $%04x ",
    "sbc a, $%02x+x     ", "sbc a, $%04x+x   ", "sbc a, $%04x+y   ", "sbc a, [$%02x]+y   ",
    "sbc $%02x, #$%02x    ", "sbc [X], [Y]     ", "movw ya, $%02x     ", "inc $%02x+x        ",
    "inc a            ", "mov sp, x        ", "das a            ", "mov a, [x+]      ",
    "di               ", "tcall 12         ", "set1 $%02x.6       ", "bbs $%02x.6, $%04x ",
    "mov $%02x, a       ", "mov $%04x, a     ", "mov [X], a       ", "mov [$%02x+x], a   ",
    "cmp x, #$%02x      ", "mov $%04x, x     ", "mov1 $%04x.%01x, c  ", "mov $%02x, y       ",
    "mov $%04x, y     ", "mov x, #$%02x      ", "pop x            ", "mul ya           ",
    "bne $%04x        ", "tcall 13         ", "clr1 $%02x.6       ", "bbc $%02x.6, $%04x ",
    "mov $%02x+x, a     ", "mov $%04x+x, a   ", "mov $%04x+y, a   ", "mov [$%02x]+y, a   ",
    "mov $%02x, x       ", "mov $%02x+y, x     ", "movw $%02x, ya     ", "mov $%02x+x, y     ",
    "dec y            ", "mov a, y         ", "cbne $%02x+x, $%04x", "daa a            ",
    "clrv             ", "tcall 14         ", "set1 $%02x.7       ", "bbs $%02x.7, $%04x ",
    "mov a, $%02x       ", "mov a, $%04x     ", "mov a, [X]       ", "mov a, [$%02x+x]   ",
    "mov a, #$%02x      ", "mov x, $%04x     ", "not1 $%04x.%01x     ", "mov y, $%02x       ",
    "mov y, $%04x     ", "notc             ", "pop y            ", "sleep            ",
    "beq $%04x        ", "tcall 15         ", "clr1 $%02x.7       ", "bbc $%02x.7, $%04x ",
    "mov a, $%02x+x     ", "mov a, $%04x+x   ", "mov a, $%04x+y   ", "mov a, [$%02x]+y   ",
    "mov x, $%02x       ", "mov x, $%02x+y     ", "mov $%02x, $%02x     ", "mov y, $%02x+x     ",
    "inc y            ", "mov y, a         ", "dbnz y, $%04x    ", "stop             ",
];
static opcodeTypeSpc: [i32; 256] = [
    0, 0, 1, 5, 1, 2, 0, 1, 1, 4, 6, 1, 2, 0, 2, 0, 3, 0, 1, 5, 1, 2, 2, 1, 4, 0, 1, 1, 0, 0, 2, 2,
    0, 0, 1, 5, 1, 2, 0, 1, 1, 4, 6, 1, 2, 0, 5, 3, 3, 0, 1, 5, 1, 2, 2, 1, 4, 0, 1, 1, 0, 0, 1, 2,
    0, 0, 1, 5, 1, 2, 0, 1, 1, 4, 6, 1, 2, 0, 2, 1, 3, 0, 1, 5, 1, 2, 2, 1, 4, 0, 1, 1, 0, 0, 2, 2,
    0, 0, 1, 5, 1, 2, 0, 1, 1, 4, 6, 1, 2, 0, 5, 0, 3, 0, 1, 5, 1, 2, 2, 1, 4, 0, 1, 1, 0, 0, 1, 0,
    0, 0, 1, 5, 1, 2, 0, 1, 1, 4, 6, 1, 2, 1, 0, 4, 3, 0, 1, 5, 1, 2, 2, 1, 4, 0, 1, 1, 0, 0, 0, 0,
    0, 0, 1, 5, 1, 2, 0, 1, 1, 4, 6, 1, 2, 1, 0, 0, 3, 0, 1, 5, 1, 2, 2, 1, 4, 0, 1, 1, 0, 0, 0, 0,
    0, 0, 1, 5, 1, 2, 0, 1, 1, 2, 6, 1, 2, 1, 0, 0, 3, 0, 1, 5, 1, 2, 2, 1, 1, 1, 1, 1, 0, 0, 5, 0,
    0, 0, 1, 5, 1, 2, 0, 1, 1, 2, 6, 1, 2, 0, 0, 0, 3, 0, 1, 5, 1, 2, 2, 1, 1, 1, 4, 1, 0, 0, 3, 0,
];
// Checked writes retain the caller's NUL-terminated output and untouched tail.
struct TraceOutput<'a>(&'a mut [u8], usize);
#[forbid(unsafe_code)]
impl TraceOutput<'_> {
    fn bytes(&mut self, bytes: &[u8]) {
        let end = self.1 + bytes.len();
        self.0[self.1..end].copy_from_slice(bytes);
        self.1 = end;
    }
    fn finish(self) {
        self.0[self.1] = 0;
    }
}
#[forbid(unsafe_code)]
impl std::fmt::Write for TraceOutput<'_> {
    fn write_str(&mut self, value: &str) -> std::fmt::Result {
        self.bytes(value.as_bytes());
        Ok(())
    }
}
fn flag_letters<const N: usize>(uppercase: [u8; N], values: [bool; N]) -> [u8; N] {
    std::array::from_fn(|i| {
        if values[i] {
            uppercase[i]
        } else {
            uppercase[i].to_ascii_lowercase()
        }
    })
}
#[forbid(unsafe_code)]
fn instruction_text(line: &mut [u8], template: &str, operands: [u32; 2], short: bool) {
    let mut out = TraceOutput(line, 0);
    let mut remaining = template.as_bytes();
    let mut operand = 0;
    while let Some(offset) = remaining.iter().position(|&byte| byte == b'%') {
        out.bytes(&remaining[..offset]);
        let width = if short {
            2
        } else {
            usize::from(remaining[offset + 2] - b'0')
        };
        write!(out, "{:0width$x}", operands[operand]).unwrap();
        operand += 1;
        remaining = &remaining[offset + 4..];
    }
    out.bytes(remaining);
    // The twelve byte-immediate forms trade two hex digits for two spaces.
    if short {
        out.bytes(b"  ");
    }
    out.finish();
}
pub unsafe fn getProcessorStateCpu(snes: *mut Snes, line: &mut [u8; 80], game: &mut Game) {
    let mut disassembly = [0; 14];
    getDisassemblyCpu(snes, &mut disassembly, game);
    // Bus reads can change CPU fields or its owning pointer. Capture afterwards.
    let cpu = &*(*snes).cpu;
    let (bank, pc, a, x, y, sp, dp, db) =
        (cpu.k, cpu.pc, cpu.a, cpu.x, cpu.y, cpu.sp, cpu.dp, cpu.db);
    let flags = flag_letters(
        *b"ENVMXDIZC",
        [
            cpu.e, cpu.n, cpu.v, cpu.mf, cpu.xf, cpu.d, cpu.i, cpu.z, cpu.c,
        ],
    );
    let text = std::ffi::CStr::from_bytes_until_nul(&disassembly)
        .unwrap()
        .to_str()
        .unwrap();
    let mut out = TraceOutput(line, 0);
    write!(out, "CPU {bank:02x}:{pc:04x} {text} A:{a:04x} X:{x:04x} Y:{y:04x} SP:{sp:04x} DP:{dp:04x} DB:{db:02x} {} {}",
        flags[0] as char, std::str::from_utf8(&flags[1..]).unwrap()).unwrap();
    out.finish();
}
pub unsafe fn getProcessorStateSpc(apu: *mut Apu, line: &mut [u8; 80]) {
    let mut disassembly = [0; 18];
    getDisassemblySpc(apu, &mut disassembly);
    let spc = &*(*apu).spc;
    let (pc, a, x, y, sp) = (spc.pc, spc.a, spc.x, spc.y, spc.sp);
    let flags = flag_letters(
        *b"NVPBHIZC",
        [spc.n, spc.v, spc.p, spc.b, spc.h, spc.i, spc.z, spc.c],
    );
    let text = std::ffi::CStr::from_bytes_until_nul(&disassembly)
        .unwrap()
        .to_str()
        .unwrap();
    let mut out = TraceOutput(line, 0);
    write!(
        out,
        "SPC {pc:04x} {text} A:{a:02x} X:{x:02x} Y:{y:02x} SP:{sp:02x} {}",
        std::str::from_utf8(&flags).unwrap()
    )
    .unwrap();
    out.finish();
}
unsafe fn getDisassemblyCpu(snes: *mut Snes, line: &mut [u8; 14], game: &mut Game) {
    let address = u32::from((*(*snes).cpu).pc) | (u32::from((*(*snes).cpu).k) << 16);
    let opcode = snes_read(snes, address, game);
    let byte = snes_read(snes, address.wrapping_add(1) & 0xffffff, game);
    let byte2 = snes_read(snes, address.wrapping_add(2) & 0xffffff, game);
    let word = u16::from_le_bytes([byte, byte2]);
    let long = (u32::from(snes_read(snes, address.wrapping_add(3) & 0xffffff, game)) << 16)
        | u32::from(word);
    let relative = (*(*snes).cpu)
        .pc
        .wrapping_add(2)
        .wrapping_add_signed(i16::from(byte as i8));
    let relative_long = (*(*snes).cpu)
        .pc
        .wrapping_add(3)
        .wrapping_add_signed(word as i16);
    let kind = opcodeType[usize::from(opcode)];
    let short = (kind == 4 && (*(*snes).cpu).mf) || (kind == 5 && (*(*snes).cpu).xf);
    let operands = match kind {
        0 => [0, 0],
        1 => [u32::from(byte), 0],
        2 => [u32::from(word), 0],
        3 => [long, 0],
        4 | 5 => [
            if short {
                u32::from(byte)
            } else {
                u32::from(word)
            },
            0,
        ],
        6 => [u32::from(relative), 0],
        7 => [u32::from(relative_long), 0],
        8 => [u32::from(byte2), u32::from(byte)],
        _ => unreachable!(),
    };
    instruction_text(line, opcodeNames[usize::from(opcode)], operands, short);
}
unsafe fn getDisassemblySpc(apu: *mut Apu, line: &mut [u8; 18]) {
    let address = (*(*apu).spc).pc;
    let opcode = apu_cpuRead(apu, address);
    let byte = apu_cpuRead(apu, address.wrapping_add(1));
    let byte2 = apu_cpuRead(apu, address.wrapping_add(2));
    let word = u16::from_le_bytes([byte, byte2]);
    let relative = (*(*apu).spc)
        .pc
        .wrapping_add(2)
        .wrapping_add_signed(i16::from(byte as i8));
    let relative2 = (*(*apu).spc)
        .pc
        .wrapping_add(2)
        .wrapping_add_signed(i16::from(byte2 as i8));
    let operands = match opcodeTypeSpc[usize::from(opcode)] {
        0 => [0, 0],
        1 => [u32::from(byte), 0],
        2 => [u32::from(word), 0],
        3 => [u32::from(relative), 0],
        4 => [u32::from(byte2), u32::from(byte)],
        5 => [u32::from(byte), u32::from(relative2)],
        6 => [u32::from(word & 0x1fff), u32::from(word >> 13)],
        _ => unreachable!(),
    };
    instruction_text(line, opcodeNamesSpc[usize::from(opcode)], operands, false);
}
