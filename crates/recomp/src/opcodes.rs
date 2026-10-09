//! Full 65816 opcode map: mnemonic + addressing mode for all 256 opcodes.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    Imp,
    Acc,
    /// Immediate sized by M.
    ImmM,
    /// Immediate sized by X.
    ImmX,
    /// Always-8-bit immediate (REP, SEP, BRK/COP signature, WDM).
    Imm8,
    /// Always-16-bit immediate (PEA).
    Imm16,
    Dp,
    DpX,
    DpY,
    DpInd,
    DpIndX,
    DpIndY,
    DpIndLong,
    DpIndLongY,
    Abs,
    AbsX,
    AbsY,
    Long,
    LongX,
    /// JMP (a)
    AbsInd,
    /// JMP/JSR (a,x)
    AbsIndX,
    /// JML [a]
    AbsIndLong,
    Sr,
    SrIndY,
    Rel8,
    Rel16,
    /// MVN/MVP: dst bank, src bank
    Block,
}

impl Mode {
    pub fn len(self, m8: bool, x8: bool) -> u32 {
        use Mode::*;
        1 + match self {
            Imp | Acc => 0,
            ImmM => {
                if m8 {
                    1
                } else {
                    2
                }
            }
            ImmX => {
                if x8 {
                    1
                } else {
                    2
                }
            }
            Imm8 | Dp | DpX | DpY | DpInd | DpIndX | DpIndY | DpIndLong | DpIndLongY | Sr | SrIndY
            | Rel8 => 1,
            Imm16 | Abs | AbsX | AbsY | AbsInd | AbsIndX | AbsIndLong | Rel16 | Block => 2,
            Long | LongX => 3,
        }
    }
}

use Mode::*;

macro_rules! row {
    ($($m:literal $md:ident),* $(,)?) => { [$(($m, $md)),*] };
}

pub static OPCODES: [(&str, Mode); 256] = {
    let r0 = row![
        "BRK" Imm8, "ORA" DpIndX, "COP" Imm8, "ORA" Sr, "TSB" Dp, "ORA" Dp, "ASL" Dp, "ORA" DpIndLong,
        "PHP" Imp, "ORA" ImmM, "ASL" Acc, "PHD" Imp, "TSB" Abs, "ORA" Abs, "ASL" Abs, "ORA" Long,
        "BPL" Rel8, "ORA" DpIndY, "ORA" DpInd, "ORA" SrIndY, "TRB" Dp, "ORA" DpX, "ASL" DpX, "ORA" DpIndLongY,
        "CLC" Imp, "ORA" AbsY, "INC" Acc, "TCS" Imp, "TRB" Abs, "ORA" AbsX, "ASL" AbsX, "ORA" LongX,
        "JSR" Abs, "AND" DpIndX, "JSL" Long, "AND" Sr, "BIT" Dp, "AND" Dp, "ROL" Dp, "AND" DpIndLong,
        "PLP" Imp, "AND" ImmM, "ROL" Acc, "PLD" Imp, "BIT" Abs, "AND" Abs, "ROL" Abs, "AND" Long,
        "BMI" Rel8, "AND" DpIndY, "AND" DpInd, "AND" SrIndY, "BIT" DpX, "AND" DpX, "ROL" DpX, "AND" DpIndLongY,
        "SEC" Imp, "AND" AbsY, "DEC" Acc, "TSC" Imp, "BIT" AbsX, "AND" AbsX, "ROL" AbsX, "AND" LongX,
        "RTI" Imp, "EOR" DpIndX, "WDM" Imm8, "EOR" Sr, "MVP" Block, "EOR" Dp, "LSR" Dp, "EOR" DpIndLong,
        "PHA" Imp, "EOR" ImmM, "LSR" Acc, "PHK" Imp, "JMP" Abs, "EOR" Abs, "LSR" Abs, "EOR" Long,
        "BVC" Rel8, "EOR" DpIndY, "EOR" DpInd, "EOR" SrIndY, "MVN" Block, "EOR" DpX, "LSR" DpX, "EOR" DpIndLongY,
        "CLI" Imp, "EOR" AbsY, "PHY" Imp, "TCD" Imp, "JML" Long, "EOR" AbsX, "LSR" AbsX, "EOR" LongX,
        "RTS" Imp, "ADC" DpIndX, "PER" Rel16, "ADC" Sr, "STZ" Dp, "ADC" Dp, "ROR" Dp, "ADC" DpIndLong,
        "PLA" Imp, "ADC" ImmM, "ROR" Acc, "RTL" Imp, "JMP" AbsInd, "ADC" Abs, "ROR" Abs, "ADC" Long,
        "BVS" Rel8, "ADC" DpIndY, "ADC" DpInd, "ADC" SrIndY, "STZ" DpX, "ADC" DpX, "ROR" DpX, "ADC" DpIndLongY,
        "SEI" Imp, "ADC" AbsY, "PLY" Imp, "TDC" Imp, "JMP" AbsIndX, "ADC" AbsX, "ROR" AbsX, "ADC" LongX,
    ];
    let r1 = row![
        "BRA" Rel8, "STA" DpIndX, "BRL" Rel16, "STA" Sr, "STY" Dp, "STA" Dp, "STX" Dp, "STA" DpIndLong,
        "DEY" Imp, "BIT" ImmM, "TXA" Imp, "PHB" Imp, "STY" Abs, "STA" Abs, "STX" Abs, "STA" Long,
        "BCC" Rel8, "STA" DpIndY, "STA" DpInd, "STA" SrIndY, "STY" DpX, "STA" DpX, "STX" DpY, "STA" DpIndLongY,
        "TYA" Imp, "STA" AbsY, "TXS" Imp, "TXY" Imp, "STZ" Abs, "STA" AbsX, "STZ" AbsX, "STA" LongX,
        "LDY" ImmX, "LDA" DpIndX, "LDX" ImmX, "LDA" Sr, "LDY" Dp, "LDA" Dp, "LDX" Dp, "LDA" DpIndLong,
        "TAY" Imp, "LDA" ImmM, "TAX" Imp, "PLB" Imp, "LDY" Abs, "LDA" Abs, "LDX" Abs, "LDA" Long,
        "BCS" Rel8, "LDA" DpIndY, "LDA" DpInd, "LDA" SrIndY, "LDY" DpX, "LDA" DpX, "LDX" DpY, "LDA" DpIndLongY,
        "CLV" Imp, "LDA" AbsY, "TSX" Imp, "TYX" Imp, "LDY" AbsX, "LDA" AbsX, "LDX" AbsY, "LDA" LongX,
        "CPY" ImmX, "CMP" DpIndX, "REP" Imm8, "CMP" Sr, "CPY" Dp, "CMP" Dp, "DEC" Dp, "CMP" DpIndLong,
        "INY" Imp, "CMP" ImmM, "DEX" Imp, "WAI" Imp, "CPY" Abs, "CMP" Abs, "DEC" Abs, "CMP" Long,
        "BNE" Rel8, "CMP" DpIndY, "CMP" DpInd, "CMP" SrIndY, "PEI" DpInd, "CMP" DpX, "DEC" DpX, "CMP" DpIndLongY,
        "CLD" Imp, "CMP" AbsY, "PHX" Imp, "STP" Imp, "JML" AbsIndLong, "CMP" AbsX, "DEC" AbsX, "CMP" LongX,
        "CPX" ImmX, "SBC" DpIndX, "SEP" Imm8, "SBC" Sr, "CPX" Dp, "SBC" Dp, "INC" Dp, "SBC" DpIndLong,
        "INX" Imp, "SBC" ImmM, "NOP" Imp, "XBA" Imp, "CPX" Abs, "SBC" Abs, "INC" Abs, "SBC" Long,
        "BEQ" Rel8, "SBC" DpIndY, "SBC" DpInd, "SBC" SrIndY, "PEA" Imm16, "SBC" DpX, "INC" DpX, "SBC" DpIndLongY,
        "SED" Imp, "SBC" AbsY, "PLX" Imp, "XCE" Imp, "JSR" AbsIndX, "SBC" AbsX, "INC" AbsX, "SBC" LongX,
    ];
    let mut out = [("", Imp); 256];
    let mut i = 0;
    while i < 128 {
        out[i] = r0[i];
        out[i + 128] = r1[i];
        i += 1;
    }
    out
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spot_checks() {
        assert_eq!(OPCODES[0x20], ("JSR", Abs));
        assert_eq!(OPCODES[0x22], ("JSL", Long));
        assert_eq!(OPCODES[0x7c], ("JMP", AbsIndX));
        assert_eq!(OPCODES[0xa9], ("LDA", ImmM));
        assert_eq!(OPCODES[0xc2], ("REP", Imm8));
        assert_eq!(OPCODES[0xdc], ("JML", AbsIndLong));
        assert_eq!(OPCODES[0xfc], ("JSR", AbsIndX));
        assert_eq!(OPCODES[0x54], ("MVN", Block));
        assert!(OPCODES.iter().all(|(m, _)| !m.is_empty()));
    }
}
