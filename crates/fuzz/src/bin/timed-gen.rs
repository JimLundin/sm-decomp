//! Compile complete production step bodies from typed binary timing vectors.
use anyhow::{Context, Result, ensure};
use pipeline_model::timed_corpus::{decode_cases, encode_cases};
use std::{collections::BTreeMap, fmt::Write, path::PathBuf};

fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    ensure!(
        args.len() == 3 || args.len() == 4,
        "usage: timed-gen input.bin bodies.rs prepared.bin [prospective-opcode]"
    );
    let mut cases = decode_cases(&std::fs::read(&args[0])?)?;
    ensure!(!cases.is_empty(), "empty timed corpus batch");
    let mut bodies = BTreeMap::new();
    let mut output = String::from("use runtime::{Machine, ops, flag, ea};\n");
    for case in &mut cases {
        let initial = &case.initial;
        ensure!(initial.e == 0 && case.fin.e == 0, "unsupported emulation case {}", case.name);
        let pc = u32::from(initial.pc) | u32::from(initial.pbr) << 16;
        let p = initial.p;
        let ram = &initial.ram;
        let read = |address| ram.iter().find(|item| item.0 == address).map(|item| item.1);
        let instruction =
            recomp::decode::decode_with(read, pc, p & 0x20 != 0, p & 0x10 != 0).context("decode failed")?;
        let body = if let Some(opcode) = args.get(3) {
            recomp::step::emit_prospective_body(&instruction, opcode)?
        } else {
            recomp::step::emit_body(&instruction)?
        };
        let next_id = bodies.len();
        let id = *bodies.entry(body.clone()).or_insert_with(|| {
            writeln!(output, "fn body_{next_id}(m: &mut Machine) {{ {body} }}").unwrap();
            next_id
        });
        let bytes: Vec<_> =
            (0..instruction.len).map(|index| read((pc & 0xff0000) | ((pc + index) & 0xffff)).unwrap()).collect();
        case.body = id;
        case.bytes = bytes;
    }
    write!(output, "pub static BODIES: &[runtime::step::StepFn] = &[")?;
    for id in 0..bodies.len() {
        write!(output, "body_{id},")?;
    }
    writeln!(output, "];\n")?;
    std::fs::write(PathBuf::from(&args[1]), output)?;
    std::fs::write(&args[2], encode_cases(&cases)?)?;
    Ok(())
}
