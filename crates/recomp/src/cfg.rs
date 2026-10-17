//! Control-flow recovery for one (entry, M, X) function variant.
//!
//! Blocks are keyed by (address, M, X), so the same bytes reached with different
//! widths become different blocks. A block ends at a control-flow instruction or
//! when it runs into an already-known block start. Overlap is allowed (a branch
//! into the middle of a block just creates a second block there); the emitted
//! code stays correct because each block is self-contained.

use std::collections::{HashMap, HashSet, VecDeque};

use anyhow::{Result, bail};

use crate::decode::{Insn, decode_with};
use crate::opcodes::Mode;

pub const MAX_BLOCKS: usize = 4096;

#[derive(Debug, Clone)]
pub enum Exit {
    /// Continue at another block (fallthrough, BRA/BRL, JMP within the function).
    Goto(usize),
    /// Last instruction is a conditional branch.
    Cond { taken: usize, fall: usize },
    /// RTS/RTL/RTI.
    Return,
    /// JMP/JML to another function's entry.
    Tail(u32),
    /// JMP (a) / JMP (a,x) / JML [a]: target computed at runtime.
    IndirectTail,
    /// Unsupported instruction or undecodable byte; emitted as `unimplemented`.
    Stop(String),
}

#[derive(Debug, Clone)]
pub struct Block {
    pub start: u32,
    pub m8: bool,
    pub x8: bool,
    pub insns: Vec<Insn>,
    pub exit: Exit,
}

#[derive(Debug)]
pub struct FunctionCfg {
    pub entry: u32,
    pub m8: bool,
    pub x8: bool,
    pub blocks: Vec<Block>,
}

fn is_cond_branch(m: &str) -> bool {
    matches!(m, "BPL" | "BMI" | "BVC" | "BVS" | "BCC" | "BCS" | "BNE" | "BEQ")
}

/// `other_entries`: entries of other configured functions. A JMP to one of
/// them becomes a tail call instead of being inlined.
pub fn build(
    read: &dyn Fn(u32) -> Option<u8>,
    entry: u32,
    m8: bool,
    x8: bool,
    other_entries: &HashSet<u32>,
) -> Result<FunctionCfg> {
    let mut ids: HashMap<(u32, bool, bool), usize> = HashMap::new();
    let mut blocks: Vec<Option<Block>> = Vec::new();
    let mut work: VecDeque<(u32, bool, bool)> = VecDeque::new();

    type Key = (u32, bool, bool);
    fn intern(
        key: Key,
        ids: &mut HashMap<Key, usize>,
        blocks: &mut Vec<Option<Block>>,
        work: &mut VecDeque<Key>,
    ) -> usize {
        *ids.entry(key).or_insert_with(|| {
            blocks.push(None);
            work.push_back(key);
            blocks.len() - 1
        })
    }
    macro_rules! get_id {
        ($key:expr) => {
            intern($key, &mut ids, &mut blocks, &mut work)
        };
    }

    get_id!((entry, m8, x8));

    while let Some((start, bm8, bx8)) = work.pop_front() {
        if blocks.len() > MAX_BLOCKS {
            bail!("${entry:06X}: more than {MAX_BLOCKS} blocks — probably decoding data as code");
        }
        let my_id = blocks.len() - work.len() - 1; // ids are assigned in queue order
        let (mut cm8, mut cx8) = (bm8, bx8);
        let mut pc = start;
        let mut insns = Vec::new();
        let exit = loop {
            if !insns.is_empty() {
                // Ran into a known block with the same widths: chain to it.
                let key = (pc, cm8, cx8);
                if let Some(&id) = ids.get(&key) {
                    break Exit::Goto(id);
                }
            }
            let Some(insn) = decode_with(read, pc, cm8, cx8) else {
                break Exit::Stop(format!("undecodable byte at ${pc:06X}"));
            };
            let next = insn.next();
            let mnem = insn.mnem;
            let mode = insn.mode;
            let operand = insn.operand;
            let target = insn.rel_target();
            insns.push(insn);

            match mnem {
                "REP" => {
                    if operand & 0x20 != 0 {
                        cm8 = false
                    }
                    if operand & 0x10 != 0 {
                        cx8 = false
                    }
                }
                "SEP" => {
                    if operand & 0x20 != 0 {
                        cm8 = true
                    }
                    if operand & 0x10 != 0 {
                        cx8 = true
                    }
                }
                // Assume PLP restores the widths the function was entered with;
                // the emitter inserts a runtime check right after it.
                "PLP" => {
                    cm8 = m8;
                    cx8 = x8;
                }
                "RTS" | "RTL" | "RTI" => break Exit::Return,
                "BRA" | "BRL" => break Exit::Goto(get_id!((target, cm8, cx8))),
                m if is_cond_branch(m) => {
                    let taken = get_id!((target, cm8, cx8));
                    let fall = get_id!((next, cm8, cx8));
                    break Exit::Cond { taken, fall };
                }
                "JMP" | "JML" => match mode {
                    Mode::Abs | Mode::Long => {
                        let t = if mode == Mode::Abs { (pc & 0xff_0000) | (operand & 0xffff) } else { operand };
                        if (t != entry && other_entries.contains(&t)) || (t >> 16) != (pc >> 16) {
                            break Exit::Tail(t);
                        }
                        break Exit::Goto(get_id!((t, cm8, cx8)));
                    }
                    _ => break Exit::IndirectTail,
                },
                "BRK" | "COP" | "STP" | "WAI" | "WDM" => break Exit::Stop(format!("{mnem} at ${pc:06X}")),
                _ => {}
            }
            pc = next;
        };
        blocks[my_id] = Some(Block { start, m8: bm8, x8: bx8, insns, exit });
    }

    Ok(FunctionCfg {
        entry,
        m8,
        x8,
        blocks: blocks.into_iter().map(|b| b.expect("every queued block is built")).collect(),
    })
}
