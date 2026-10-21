use crate::game_state::Game;
use crate::snes::apu::apu_cpuRead;
use crate::snes::snes::snes_read;
use crate::types::{Apu, Snes};
use core::ffi::c_char;
use std::fmt::Write;

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
// These writers retain the existing caller-owned, NUL-terminated output API.
struct TraceOutput(*mut u8);
impl TraceOutput {
    unsafe fn bytes(&mut self, bytes: &[u8]) {
        core::ptr::copy_nonoverlapping(bytes.as_ptr(), self.0, bytes.len());
        self.0 = self.0.add(bytes.len());
    }
    unsafe fn finish(self) {
        *self.0 = 0;
    }
}
impl std::fmt::Write for TraceOutput {
    fn write_str(&mut self, value: &str) -> std::fmt::Result {
        unsafe {
            self.bytes(value.as_bytes());
        }
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
unsafe fn instruction_text(
    line: *mut c_char,
    template: *const c_char,
    operands: [u32; 2],
    short: bool,
) {
    let mut out = TraceOutput(line.cast());
    let mut remaining = std::ffi::CStr::from_ptr(template).to_bytes();
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
pub unsafe fn getProcessorStateCpu(snes: *mut Snes, line: *mut c_char, game: &mut Game) {
    let mut disassembly = [0 as c_char; 14];
    getDisassemblyCpu(snes, disassembly.as_mut_ptr(), game);
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
    let text = std::ffi::CStr::from_ptr(disassembly.as_ptr())
        .to_str()
        .unwrap();
    let mut out = TraceOutput(line.cast());
    write!(out, "CPU {bank:02x}:{pc:04x} {text} A:{a:04x} X:{x:04x} Y:{y:04x} SP:{sp:04x} DP:{dp:04x} DB:{db:02x} {} {}",
        flags[0] as char, std::str::from_utf8(&flags[1..]).unwrap()).unwrap();
    out.finish();
}
pub unsafe fn getProcessorStateSpc(apu: *mut Apu, line: *mut c_char) {
    let mut disassembly = [0 as c_char; 18];
    getDisassemblySpc(apu, disassembly.as_mut_ptr());
    let spc = &*(*apu).spc;
    let (pc, a, x, y, sp) = (spc.pc, spc.a, spc.x, spc.y, spc.sp);
    let flags = flag_letters(
        *b"NVPBHIZC",
        [spc.n, spc.v, spc.p, spc.b, spc.h, spc.i, spc.z, spc.c],
    );
    let text = std::ffi::CStr::from_ptr(disassembly.as_ptr())
        .to_str()
        .unwrap();
    let mut out = TraceOutput(line.cast());
    write!(
        out,
        "SPC {pc:04x} {text} A:{a:02x} X:{x:02x} Y:{y:02x} SP:{sp:02x} {}",
        std::str::from_utf8(&flags).unwrap()
    )
    .unwrap();
    out.finish();
}
unsafe fn getDisassemblyCpu(snes: *mut Snes, line: *mut c_char, game: &mut Game) {
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
unsafe fn getDisassemblySpc(apu: *mut Apu, line: *mut c_char) {
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
