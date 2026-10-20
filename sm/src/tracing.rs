use crate::c_library::sprintf;
use crate::game_state::Game;
use crate::snes::apu::apu_cpuRead;
use crate::snes::snes::snes_read;
pub use crate::types::{
    Apu, ApuHistory as C2RustUnnamed, BgLayer, Cart, Cpu, Dma, DmaChannel, Dsp, DspChannel,
    DspRegWriteHistory, Input, Layer, Ppu, PpuPixelPrioBufs, PpuZbufType, Snes, Spc, Timer,
    WindowLayer,
};
use ::core::ffi::{c_char, c_void};
use ::core::mem::transmute;
use ::core::ptr::{null, null_mut};
pub const NULL: *mut c_void = null_mut::<c_void>();
static opcodeNames: crate::types::ImmutableCStringPointers<256> = crate::types::immutable_c_strings! {
    b"brk          \0", b"ora ($%02x,x)  \0", b"cop #$%02x     \0", b"ora $%02x,s    \0",
    b"tsb $%02x      \0", b"ora $%02x      \0", b"asl $%02x      \0", b"ora [$%02x]    \0",
    b"php          \0", b"ora #$%04x   \0", b"asl          \0", b"phd          \0",
    b"tsb $%04x    \0", b"ora $%04x    \0", b"asl $%04x    \0", b"ora $%06x  \0",
    b"bpl $%04x    \0", b"ora ($%02x),y  \0", b"ora ($%02x)    \0", b"ora ($%02x,s),y\0",
    b"trb $%02x      \0", b"ora $%02x,x    \0", b"asl $%02x,x    \0", b"ora [$%02x],y  \0",
    b"clc          \0", b"ora $%04x,y  \0", b"inc          \0", b"tcs          \0",
    b"trb $%04x    \0", b"ora $%04x,x  \0", b"asl $%04x,x  \0", b"ora $%06x,x\0",
    b"jsr $%04x    \0", b"and ($%02x,x)  \0", b"jsl $%06x  \0", b"and $%02x,s    \0",
    b"bit $%02x      \0", b"and $%02x      \0", b"rol $%02x      \0", b"and [$%02x]    \0",
    b"plp          \0", b"and #$%04x   \0", b"rol          \0", b"pld          \0",
    b"bit $%04x    \0", b"and $%04x    \0", b"rol $%04x    \0", b"and $%06x  \0",
    b"bmi $%04x    \0", b"and ($%02x),y  \0", b"and ($%02x)    \0", b"and ($%02x,s),y\0",
    b"bit $%02x,x    \0", b"and $%02x,x    \0", b"rol $%02x,x    \0", b"and [$%02x],y  \0",
    b"sec          \0", b"and $%04x,y  \0", b"dec          \0", b"tsc          \0",
    b"bit $%04x,x  \0", b"and $%04x,x  \0", b"rol $%04x,x  \0", b"and $%06x,x\0",
    b"rti          \0", b"eor ($%02x,x)  \0", b"wdm #$%02x     \0", b"eor $%02x,s    \0",
    b"mvp $%02x, $%02x \0", b"eor $%02x      \0", b"lsr $%02x      \0", b"eor [$%02x]    \0",
    b"pha          \0", b"eor #$%04x   \0", b"lsr          \0", b"phk          \0",
    b"jmp $%04x    \0", b"eor $%04x    \0", b"lsr $%04x    \0", b"eor $%06x  \0",
    b"bvc $%04x    \0", b"eor ($%02x),y  \0", b"eor ($%02x)    \0", b"eor ($%02x,s),y\0",
    b"mvn $%02x, $%02x \0", b"eor $%02x,x    \0", b"lsr $%02x,x    \0", b"eor [$%02x],y  \0",
    b"cli          \0", b"eor $%04x,y  \0", b"phy          \0", b"tcd          \0",
    b"jml $%06x  \0", b"eor $%04x,x  \0", b"lsr $%04x,x  \0", b"eor $%06x,x\0",
    b"rts          \0", b"adc ($%02x,x)  \0", b"per $%04x    \0", b"adc $%02x,s    \0",
    b"stz $%02x      \0", b"adc $%02x      \0", b"ror $%02x      \0", b"adc [$%02x]    \0",
    b"pla          \0", b"adc #$%04x   \0", b"ror          \0", b"rtl          \0",
    b"jmp ($%04x)  \0", b"adc $%04x    \0", b"ror $%04x    \0", b"adc $%06x  \0",
    b"bvs $%04x    \0", b"adc ($%02x),y  \0", b"adc ($%02x)    \0", b"adc ($%02x,s),y\0",
    b"stz $%02x,x    \0", b"adc $%02x,x    \0", b"ror $%02x,x    \0", b"adc [$%02x],y  \0",
    b"sei          \0", b"adc $%04x,y  \0", b"ply          \0", b"tdc          \0",
    b"jmp ($%04x,x)\0", b"adc $%04x,x  \0", b"ror $%04x,x  \0", b"adc $%06x,x\0",
    b"bra $%04x    \0", b"sta ($%02x,x)  \0", b"brl $%04x    \0", b"sta $%02x,s    \0",
    b"sty $%02x      \0", b"sta $%02x      \0", b"stx $%02x      \0", b"sta [$%02x]    \0",
    b"dey          \0", b"bit #$%04x   \0", b"txa          \0", b"phb          \0",
    b"sty $%04x    \0", b"sta $%04x    \0", b"stx $%04x    \0", b"sta $%06x  \0",
    b"bcc $%04x    \0", b"sta ($%02x),y  \0", b"sta ($%02x)    \0", b"sta ($%02x,s),y\0",
    b"sty $%02x,x    \0", b"sta $%02x,x    \0", b"stx $%02x,y    \0", b"sta [$%02x],y  \0",
    b"tya          \0", b"sta $%04x,y  \0", b"txs          \0", b"txy          \0",
    b"stz $%04x    \0", b"sta $%04x,x  \0", b"stz $%04x,x  \0", b"sta $%06x,x\0",
    b"ldy #$%04x   \0", b"lda ($%02x,x)  \0", b"ldx #$%04x   \0", b"lda $%02x,s    \0",
    b"ldy $%02x      \0", b"lda $%02x      \0", b"ldx $%02x      \0", b"lda [$%02x]    \0",
    b"tay          \0", b"lda #$%04x   \0", b"tax          \0", b"plb          \0",
    b"ldy $%04x    \0", b"lda $%04x    \0", b"ldx $%04x    \0", b"lda $%06x  \0",
    b"bcs $%04x    \0", b"lda ($%02x),y  \0", b"lda ($%02x)    \0", b"lda ($%02x,s),y\0",
    b"ldy $%02x,x    \0", b"lda $%02x,x    \0", b"ldx $%02x,y    \0", b"lda [$%02x],y  \0",
    b"clv          \0", b"lda $%04x,y  \0", b"tsx          \0", b"tyx          \0",
    b"ldy $%04x,x  \0", b"lda $%04x,x  \0", b"ldx $%04x,y  \0", b"lda $%06x,x\0",
    b"cpy #$%04x   \0", b"cmp ($%02x,x)  \0", b"rep #$%02x     \0", b"cmp $%02x,s    \0",
    b"cpy $%02x      \0", b"cmp $%02x      \0", b"dec $%02x      \0", b"cmp [$%02x]    \0",
    b"iny          \0", b"cmp #$%04x   \0", b"dex          \0", b"wai          \0",
    b"cpy $%04x    \0", b"cmp $%04x    \0", b"dec $%04x    \0", b"cmp $%06x  \0",
    b"bne $%04x    \0", b"cmp ($%02x),y  \0", b"cmp ($%02x)    \0", b"cmp ($%02x,s),y\0",
    b"pei $%02x      \0", b"cmp $%02x,x    \0", b"dec $%02x,x    \0", b"cmp [$%02x],y  \0",
    b"cld          \0", b"cmp $%04x,y  \0", b"phx          \0", b"stp          \0",
    b"jml [$%04x]  \0", b"cmp $%04x,x  \0", b"dec $%04x,x  \0", b"cmp $%06x,x\0",
    b"cpx #$%04x   \0", b"sbc ($%02x,x)  \0", b"sep #$%02x     \0", b"sbc $%02x,s    \0",
    b"cpx $%02x      \0", b"sbc $%02x      \0", b"inc $%02x      \0", b"sbc [$%02x]    \0",
    b"inx          \0", b"sbc #$%04x   \0", b"nop          \0", b"xba          \0",
    b"cpx $%04x    \0", b"sbc $%04x    \0", b"inc $%04x    \0", b"sbc $%06x  \0",
    b"beq $%04x    \0", b"sbc ($%02x),y  \0", b"sbc ($%02x)    \0", b"sbc ($%02x,s),y\0",
    b"pea #$%04x   \0", b"sbc $%02x,x    \0", b"inc $%02x,x    \0", b"sbc [$%02x],y  \0",
    b"sed          \0", b"sbc $%04x,y  \0", b"plx          \0", b"xce          \0",
    b"jsr ($%04x,x)\0", b"sbc $%04x,x  \0", b"inc $%04x,x  \0", b"sbc $%06x,x\0",
};
static opcodeNamesSp: crate::types::ImmutableCStringPointers<256> = crate::types::immutable_c_strings! {
    None, None, None, None,
    None, None, None, None,
    None, b"ora #$%02x     \0", None, None,
    None, None, None, None,
    None, None, None, None,
    None, None, None, None,
    None, None, None, None,
    None, None, None, None,
    None, None, None, None,
    None, None, None, None,
    None, b"and #$%02x     \0", None, None,
    None, None, None, None,
    None, None, None, None,
    None, None, None, None,
    None, None, None, None,
    None, None, None, None,
    None, None, None, None,
    None, None, None, None,
    None, b"eor #$%02x     \0", None, None,
    None, None, None, None,
    None, None, None, None,
    None, None, None, None,
    None, None, None, None,
    None, None, None, None,
    None, None, None, None,
    None, None, None, None,
    None, b"adc #$%02x     \0", None, None,
    None, None, None, None,
    None, None, None, None,
    None, None, None, None,
    None, None, None, None,
    None, None, None, None,
    None, None, None, None,
    None, None, None, None,
    None, b"bit #$%02x     \0", None, None,
    None, None, None, None,
    None, None, None, None,
    None, None, None, None,
    None, None, None, None,
    None, None, None, None,
    b"ldy #$%02x     \0", None, b"ldx #$%02x     \0", None,
    None, None, None, None,
    None, b"lda #$%02x     \0", None, None,
    None, None, None, None,
    None, None, None, None,
    None, None, None, None,
    None, None, None, None,
    None, None, None, None,
    b"cpy #$%02x     \0", None, None, None,
    None, None, None, None,
    None, b"cmp #$%02x     \0", None, None,
    None, None, None, None,
    None, None, None, None,
    None, None, None, None,
    None, None, None, None,
    None, None, None, None,
    b"cpx #$%02x     \0", None, None, None,
    None, None, None, None,
    None, b"sbc #$%02x     \0", None, None,
    None, None, None, None,
    None, None, None, None,
    None, None, None, None,
    None, None, None, None,
    None, None, None, None,
};
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
static opcodeNamesSpc: crate::types::ImmutableCStringPointers<256> = crate::types::immutable_c_strings! {
    b"nop              \0", b"tcall 0          \0", b"set1 $%02x.0       \0", b"bbs $%02x.0, $%04x \0",
    b"or a, $%02x        \0", b"or a, $%04x      \0", b"or a, [X]        \0", b"or a, [$%02x+x]    \0",
    b"or a, #$%02x       \0", b"or $%02x, $%02x      \0", b"or1 c, $%04x.%01x   \0", b"asl $%02x          \0",
    b"asl $%04x        \0", b"push p           \0", b"tset $%04x       \0", b"brk              \0",
    b"bpl $%04x        \0", b"tcall 1          \0", b"clr1 $%02x.0       \0", b"bbc $%02x.0, $%04x \0",
    b"or a, $%02x+x      \0", b"or a, $%04x+x    \0", b"or a, $%04x+y    \0", b"or a, [$%02x]+y    \0",
    b"or $%02x, #$%02x     \0", b"or [X], [Y]      \0", b"decw $%02x         \0", b"asl $%02x+x        \0",
    b"asl a            \0", b"dec x            \0", b"cmp x, $%04x     \0", b"jmp [$%04x+x]    \0",
    b"clrp             \0", b"tcall 2          \0", b"set1 $%02x.1       \0", b"bbs $%02x.1, $%04x \0",
    b"and a, $%02x       \0", b"and a, $%04x     \0", b"and a, [X]       \0", b"and a, [$%02x+x]   \0",
    b"and a, #$%02x      \0", b"and $%02x, $%02x     \0", b"or1 c, /$%04x.%01x  \0", b"rol $%02x          \0",
    b"rol $%04x        \0", b"push a           \0", b"cbne $%02x, $%04x  \0", b"bra $%04x        \0",
    b"bmi $%04x        \0", b"tcall 3          \0", b"clr1 $%02x.1       \0", b"bbc $%02x.1, $%04x \0",
    b"and a, $%02x+x     \0", b"and a, $%04x+x   \0", b"and a, $%04x+y   \0", b"and a, [$%02x]+y   \0",
    b"and $%02x, #$%02x    \0", b"and [X], [Y]     \0", b"incw $%02x         \0", b"rol $%02x+x        \0",
    b"rol a            \0", b"inc x            \0", b"cmp x, $%02x       \0", b"call $%04x       \0",
    b"setp             \0", b"tcall 4          \0", b"set1 $%02x.2       \0", b"bbs $%02x.2, $%04x \0",
    b"eor a, $%02x       \0", b"eor a, $%04x     \0", b"eor a, [X]       \0", b"eor a, [$%02x+x]   \0",
    b"eor a, #$%02x      \0", b"eor $%02x, $%02x     \0", b"and1 c, $%04x.%01x  \0", b"lsr $%02x          \0",
    b"lsr $%04x        \0", b"push x           \0", b"tclr $%04x       \0", b"pcall $%02x        \0",
    b"bvc $%04x        \0", b"tcall 5          \0", b"clr1 $%02x.2       \0", b"bbc $%02x.2, $%04x \0",
    b"eor a, $%02x+x     \0", b"eor a, $%04x+x   \0", b"eor a, $%04x+y   \0", b"eor a, [$%02x]+y   \0",
    b"eor $%02x, #$%02x    \0", b"eor [X], [Y]     \0", b"cmpw ya, $%02x     \0", b"lsr $%02x+x        \0",
    b"lsr a            \0", b"mov x, a         \0", b"cmp y, $%04x     \0", b"jmp $%04x        \0",
    b"clrc             \0", b"tcall 6          \0", b"set1 $%02x.3       \0", b"bbs $%02x.3, $%04x \0",
    b"cmp a, $%02x       \0", b"cmp a, $%04x     \0", b"cmp a, [X]       \0", b"cmp a, [$%02x+x]   \0",
    b"cmp a, #$%02x      \0", b"cmp $%02x, $%02x     \0", b"and1 c, /$%04x.%01x \0", b"ror $%02x          \0",
    b"ror $%04x        \0", b"push y           \0", b"dbnz $%02x, $%04x  \0", b"ret              \0",
    b"bvs $%04x        \0", b"tcall 7          \0", b"clr1 $%02x.3       \0", b"bbc $%02x.3, $%04x \0",
    b"cmp a, $%02x+x     \0", b"cmp a, $%04x+x   \0", b"cmp a, $%04x+y   \0", b"cmp a, [$%02x]+y   \0",
    b"cmp $%02x, #$%02x    \0", b"cmp [X], [Y]     \0", b"addw ya, $%02x     \0", b"ror $%02x+x        \0",
    b"ror a            \0", b"mov a, x         \0", b"cmp y, $%02x       \0", b"reti             \0",
    b"setc             \0", b"tcall 8          \0", b"set1 $%02x.4       \0", b"bbs $%02x.4, $%04x \0",
    b"adc a, $%02x       \0", b"adc a, $%04x     \0", b"adc a, [X]       \0", b"adc a, [$%02x+x]   \0",
    b"adc a, #$%02x      \0", b"adc $%02x, $%02x     \0", b"eor1 c, $%04x.%01x  \0", b"dec $%02x          \0",
    b"dec $%04x        \0", b"mov y, #$%02x      \0", b"pop p            \0", b"mov $%02x, #$%02x    \0",
    b"bcc $%04x        \0", b"tcall 9          \0", b"clr1 $%02x.4       \0", b"bbc $%02x.4, $%04x \0",
    b"adc a, $%02x+x     \0", b"adc a, $%04x+x   \0", b"adc a, $%04x+y   \0", b"adc a, [$%02x]+y   \0",
    b"adc $%02x, #$%02x    \0", b"adc [X], [Y]     \0", b"subw ya, $%02x     \0", b"dec $%02x+x        \0",
    b"dec a            \0", b"mov x, sp        \0", b"div ya, x        \0", b"xcn a            \0",
    b"ei               \0", b"tcall 10         \0", b"set1 $%02x.5       \0", b"bbs $%02x.5, $%04x \0",
    b"sbc a, $%02x       \0", b"sbc a, $%04x     \0", b"sbc a, [X]       \0", b"sbc a, [$%02x+x]   \0",
    b"sbc a, #$%02x      \0", b"sbc $%02x, $%02x     \0", b"mov1 c, $%04x.%01x  \0", b"inc $%02x          \0",
    b"inc $%04x        \0", b"cmp y, #$%02x      \0", b"pop a            \0", b"mov [x+], a      \0",
    b"bcs $%04x        \0", b"tcall 11         \0", b"clr1 $%02x.5       \0", b"bbc $%02x.5, $%04x \0",
    b"sbc a, $%02x+x     \0", b"sbc a, $%04x+x   \0", b"sbc a, $%04x+y   \0", b"sbc a, [$%02x]+y   \0",
    b"sbc $%02x, #$%02x    \0", b"sbc [X], [Y]     \0", b"movw ya, $%02x     \0", b"inc $%02x+x        \0",
    b"inc a            \0", b"mov sp, x        \0", b"das a            \0", b"mov a, [x+]      \0",
    b"di               \0", b"tcall 12         \0", b"set1 $%02x.6       \0", b"bbs $%02x.6, $%04x \0",
    b"mov $%02x, a       \0", b"mov $%04x, a     \0", b"mov [X], a       \0", b"mov [$%02x+x], a   \0",
    b"cmp x, #$%02x      \0", b"mov $%04x, x     \0", b"mov1 $%04x.%01x, c  \0", b"mov $%02x, y       \0",
    b"mov $%04x, y     \0", b"mov x, #$%02x      \0", b"pop x            \0", b"mul ya           \0",
    b"bne $%04x        \0", b"tcall 13         \0", b"clr1 $%02x.6       \0", b"bbc $%02x.6, $%04x \0",
    b"mov $%02x+x, a     \0", b"mov $%04x+x, a   \0", b"mov $%04x+y, a   \0", b"mov [$%02x]+y, a   \0",
    b"mov $%02x, x       \0", b"mov $%02x+y, x     \0", b"movw $%02x, ya     \0", b"mov $%02x+x, y     \0",
    b"dec y            \0", b"mov a, y         \0", b"cbne $%02x+x, $%04x\0", b"daa a            \0",
    b"clrv             \0", b"tcall 14         \0", b"set1 $%02x.7       \0", b"bbs $%02x.7, $%04x \0",
    b"mov a, $%02x       \0", b"mov a, $%04x     \0", b"mov a, [X]       \0", b"mov a, [$%02x+x]   \0",
    b"mov a, #$%02x      \0", b"mov x, $%04x     \0", b"not1 $%04x.%01x     \0", b"mov y, $%02x       \0",
    b"mov y, $%04x     \0", b"notc             \0", b"pop y            \0", b"sleep            \0",
    b"beq $%04x        \0", b"tcall 15         \0", b"clr1 $%02x.7       \0", b"bbc $%02x.7, $%04x \0",
    b"mov a, $%02x+x     \0", b"mov a, $%04x+x   \0", b"mov a, $%04x+y   \0", b"mov a, [$%02x]+y   \0",
    b"mov x, $%02x       \0", b"mov x, $%02x+y     \0", b"mov $%02x, $%02x     \0", b"mov y, $%02x+x     \0",
    b"inc y            \0", b"mov y, a         \0", b"dbnz y, $%04x    \0", b"stop             \0",
};
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
pub unsafe fn getProcessorStateCpu(snes: *mut Snes, line: *mut c_char, game: &mut Game) {
    let mut disLine: [c_char; 14] = transmute::<[u8; 14], [c_char; 14]>(*b"             \0");
    getDisassemblyCpu(snes, &raw mut disLine as *mut c_char, game);
    sprintf(
        line,
        b"CPU %02x:%04x %s A:%04x X:%04x Y:%04x SP:%04x DP:%04x DB:%02x %c %c%c%c%c%c%c%c%c\0"
            as *const u8 as *const c_char,
        (*(*snes).cpu).k as i32,
        (*(*snes).cpu).pc as i32,
        &raw mut disLine as *mut c_char,
        (*(*snes).cpu).a as i32,
        (*(*snes).cpu).x as i32,
        (*(*snes).cpu).y as i32,
        (*(*snes).cpu).sp as i32,
        (*(*snes).cpu).dp as i32,
        (*(*snes).cpu).db as i32,
        if (*(*snes).cpu).e as i32 != 0 {
            'E' as i32
        } else {
            'e' as i32
        },
        if (*(*snes).cpu).n as i32 != 0 {
            'N' as i32
        } else {
            'n' as i32
        },
        if (*(*snes).cpu).v as i32 != 0 {
            'V' as i32
        } else {
            'v' as i32
        },
        if (*(*snes).cpu).mf as i32 != 0 {
            'M' as i32
        } else {
            'm' as i32
        },
        if (*(*snes).cpu).xf as i32 != 0 {
            'X' as i32
        } else {
            'x' as i32
        },
        if (*(*snes).cpu).d as i32 != 0 {
            'D' as i32
        } else {
            'd' as i32
        },
        if (*(*snes).cpu).i as i32 != 0 {
            'I' as i32
        } else {
            'i' as i32
        },
        if (*(*snes).cpu).z as i32 != 0 {
            'Z' as i32
        } else {
            'z' as i32
        },
        if (*(*snes).cpu).c as i32 != 0 {
            'C' as i32
        } else {
            'c' as i32
        },
    );
}
pub unsafe fn getProcessorStateSpc(apu: *mut Apu, line: *mut c_char) {
    let mut disLine: [c_char; 18] = transmute::<[u8; 18], [c_char; 18]>(*b"                 \0");
    getDisassemblySpc(apu, &raw mut disLine as *mut c_char);
    sprintf(
        line,
        b"SPC %04x %s A:%02x X:%02x Y:%02x SP:%02x %c%c%c%c%c%c%c%c\0" as *const u8
            as *const c_char,
        (*(*apu).spc).pc as i32,
        &raw mut disLine as *mut c_char,
        (*(*apu).spc).a as i32,
        (*(*apu).spc).x as i32,
        (*(*apu).spc).y as i32,
        (*(*apu).spc).sp as i32,
        if (*(*apu).spc).n as i32 != 0 {
            'N' as i32
        } else {
            'n' as i32
        },
        if (*(*apu).spc).v as i32 != 0 {
            'V' as i32
        } else {
            'v' as i32
        },
        if (*(*apu).spc).p as i32 != 0 {
            'P' as i32
        } else {
            'p' as i32
        },
        if (*(*apu).spc).b as i32 != 0 {
            'B' as i32
        } else {
            'b' as i32
        },
        if (*(*apu).spc).h as i32 != 0 {
            'H' as i32
        } else {
            'h' as i32
        },
        if (*(*apu).spc).i as i32 != 0 {
            'I' as i32
        } else {
            'i' as i32
        },
        if (*(*apu).spc).z as i32 != 0 {
            'Z' as i32
        } else {
            'z' as i32
        },
        if (*(*apu).spc).c as i32 != 0 {
            'C' as i32
        } else {
            'c' as i32
        },
    );
}
unsafe fn getDisassemblyCpu(snes: *mut Snes, line: *mut c_char, game: &mut Game) {
    let adr: u32 = ((*(*snes).cpu).pc as i32 | ((*(*snes).cpu).k as i32) << 16i32) as u32;
    let opcode: u8 = snes_read(snes, adr, game);
    let byte: u8 = snes_read(snes, adr.wrapping_add(1u32) & 0xffffffu32, game);
    let byte2: u8 = snes_read(snes, adr.wrapping_add(2u32) & 0xffffffu32, game);
    let word: u16 = ((byte2 as i32) << 8i32 | byte as i32) as u16;
    let longv: u32 = ((snes_read(snes, adr.wrapping_add(3u32) & 0xffffffu32, game) as i32) << 16i32
        | word as i32) as u32;
    let rel: u16 = ((*(*snes).cpu).pc as i32 + 2i32 + byte as i8 as i32) as u16;
    let rell: u16 = ((*(*snes).cpu).pc as i32 + 3i32 + word as i16 as i32) as u16;
    match opcodeType[opcode as usize] {
        0 => {
            sprintf(
                line,
                b"%s\0" as *const u8 as *const c_char,
                opcodeNames[opcode as usize],
            );
        }
        1 => {
            sprintf(line, opcodeNames[opcode as usize], byte as i32);
        }
        2 => {
            sprintf(line, opcodeNames[opcode as usize], word as i32);
        }
        3 => {
            sprintf(line, opcodeNames[opcode as usize], longv);
        }
        4 => {
            if (*(*snes).cpu).mf {
                sprintf(line, opcodeNamesSp[opcode as usize], byte as i32);
            } else {
                sprintf(line, opcodeNames[opcode as usize], word as i32);
            }
        }
        5 => {
            if (*(*snes).cpu).xf {
                sprintf(line, opcodeNamesSp[opcode as usize], byte as i32);
            } else {
                sprintf(line, opcodeNames[opcode as usize], word as i32);
            }
        }
        6 => {
            sprintf(line, opcodeNames[opcode as usize], rel as i32);
        }
        7 => {
            sprintf(line, opcodeNames[opcode as usize], rell as i32);
        }
        8 => {
            sprintf(
                line,
                opcodeNames[opcode as usize],
                byte2 as i32,
                byte as i32,
            );
        }
        _ => {}
    };
}
unsafe fn getDisassemblySpc(apu: *mut Apu, line: *mut c_char) {
    let adr: u16 = (*(*apu).spc).pc;
    let opcode: u8 = apu_cpuRead(apu, adr);
    let byte: u8 = apu_cpuRead(apu, (adr as i32 + 1i32 & 0xffffi32) as u16);
    let byte2: u8 = apu_cpuRead(apu, (adr as i32 + 2i32 & 0xffffi32) as u16);
    let word: u16 = ((byte2 as i32) << 8i32 | byte as i32) as u16;
    let rel: u16 = ((*(*apu).spc).pc as i32 + 2i32 + byte as i8 as i32) as u16;
    let rel2: u16 = ((*(*apu).spc).pc as i32 + 2i32 + byte2 as i8 as i32) as u16;
    let wordb: u16 = (word as i32 & 0x1fffi32) as u16;
    let bit: u8 = (word as i32 >> 13i32) as u8;
    match opcodeTypeSpc[opcode as usize] {
        0 => {
            sprintf(
                line,
                b"%s\0" as *const u8 as *const c_char,
                opcodeNamesSpc[opcode as usize],
            );
        }
        1 => {
            sprintf(line, opcodeNamesSpc[opcode as usize], byte as i32);
        }
        2 => {
            sprintf(line, opcodeNamesSpc[opcode as usize], word as i32);
        }
        3 => {
            sprintf(line, opcodeNamesSpc[opcode as usize], rel as i32);
        }
        4 => {
            sprintf(
                line,
                opcodeNamesSpc[opcode as usize],
                byte2 as i32,
                byte as i32,
            );
        }
        5 => {
            sprintf(
                line,
                opcodeNamesSpc[opcode as usize],
                byte as i32,
                rel2 as i32,
            );
        }
        6 => {
            sprintf(
                line,
                opcodeNamesSpc[opcode as usize],
                wordb as i32,
                bit as i32,
            );
        }
        _ => {}
    };
}
