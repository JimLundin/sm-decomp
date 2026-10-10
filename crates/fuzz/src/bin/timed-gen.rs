//! Compile complete production step bodies; JSON stays runtime data.
use anyhow::{Context, Result, ensure};
use serde_json::Value;
use std::{collections::BTreeMap, fmt::Write, path::PathBuf};

fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    ensure!(args.len() == 3, "usage: timed-gen input.json bodies.rs prepared.json");
    let mut cases: Vec<Value> = serde_json::from_slice(&std::fs::read(&args[0])?)?;
    ensure!(!cases.is_empty(), "empty timed corpus batch");
    let mut bodies = BTreeMap::new();
    let mut output = String::from("use runtime::{Machine, ops, flag, ea};\n");
    for case in &mut cases {
        let initial = &case["initial"];
        ensure!(initial["e"] == 0 && case["final"]["e"] == 0, "unsupported emulation case {}", case["name"]);
        let pc = initial["pc"].as_u64().context("pc")? as u32 | (initial["pbr"].as_u64().context("pbr")? as u32) << 16;
        let p = initial["p"].as_u64().context("p")?;
        let ram = initial["ram"].as_array().context("ram")?;
        let read = |address| {
            ram.iter()
                .find(|item| item[0].as_u64() == Some(u64::from(address)))
                .and_then(|item| item[1].as_u64())
                .map(|value| value as u8)
        };
        let instruction =
            recomp::decode::decode_with(read, pc, p & 0x20 != 0, p & 0x10 != 0).context("decode failed")?;
        let body = recomp::step::emit_body(&instruction)?;
        let next_id = bodies.len();
        let id = *bodies.entry(body.clone()).or_insert_with(|| {
            writeln!(output, "fn body_{next_id}(m: &mut Machine) {{ {body} }}").unwrap();
            next_id
        });
        let bytes: Vec<_> =
            (0..instruction.len).map(|index| read((pc & 0xff0000) | ((pc + index) & 0xffff)).unwrap()).collect();
        case["body"] = id.into();
        case["bytes"] = serde_json::to_value(bytes)?;
    }
    write!(output, "pub static BODIES: &[runtime::step::StepFn] = &[")?;
    for id in 0..bodies.len() {
        write!(output, "body_{id},")?;
    }
    writeln!(output, "];\n")?;
    std::fs::write(PathBuf::from(&args[1]), output)?;
    std::fs::write(&args[2], serde_json::to_vec(&cases)?)?;
    Ok(())
}
