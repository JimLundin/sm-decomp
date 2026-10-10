//! Execute compiled production bodies and compare independent CPU/bus vectors.
use anyhow::{Result, ensure};
use runtime::{Bus, Cpu, Machine, TimedBus};
use serde::Deserialize;
use serde_json::json;
use std::{cell::RefCell, collections::BTreeMap, io::Write, rc::Rc};

mod generated {
    #![allow(unused_imports)]
    include!(concat!(env!("OUT_DIR"), "/timed_steps.rs"));
}

#[derive(Deserialize)]
struct State {
    pc: u16,
    pbr: u8,
    s: u16,
    p: u8,
    a: u16,
    x: u16,
    y: u16,
    dbr: u8,
    d: u16,
    e: u8,
    ram: Vec<(u32, u8)>,
}
impl State {
    fn cpu(&self) -> Cpu {
        Cpu {
            a: self.a,
            x: self.x,
            y: self.y,
            s: self.s,
            p: self.p,
            db: self.dbr,
            d: self.d,
            pb: self.pbr,
            e: self.e != 0,
        }
    }
}
#[derive(Deserialize)]
struct Case {
    name: String,
    initial: State,
    #[serde(rename = "final")]
    fin: State,
    cycles: Vec<(u32, Option<u8>, String)>,
    body: usize,
    bytes: Vec<u8>,
    case_sha256: String,
}
#[derive(Default)]
struct Recording {
    ram: BTreeMap<u32, u8>,
    events: Vec<(u32, Option<u8>, bool)>,
}
struct Sparse(Rc<RefCell<Recording>>);
impl Bus for Sparse {
    fn read(&mut self, address: u32) -> u8 {
        let mut r = self.0.borrow_mut();
        let value = *r.ram.get(&address).unwrap_or(&0);
        r.events.push((address, Some(value), false));
        value
    }
    fn write(&mut self, address: u32, value: u8) {
        let mut r = self.0.borrow_mut();
        r.ram.insert(address, value);
        r.events.push((address, Some(value), true));
    }
}

fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    ensure!(args.len() == 2, "usage: timed-run prepared.json ledger.jsonl");
    let cases: Vec<Case> = serde_json::from_slice(&std::fs::read(&args[0])?)?;
    ensure!(!cases.is_empty() && !generated::BODIES.is_empty(), "inactive timed runner");
    let mut ledger = std::fs::File::create(&args[1])?;
    let mut failures = 0;
    for case in cases {
        let recording =
            Rc::new(RefCell::new(Recording { ram: case.initial.ram.iter().copied().collect(), events: Vec::new() }));
        let mut machine = Machine::new(Box::new(TimedBus::new(Sparse(recording.clone()))), &[], None);
        machine.cpu = case.initial.cpu();
        let pc = (u32::from(case.initial.pbr) << 16) | u32::from(case.initial.pc);
        let outcome = runtime::step::execute_step(&mut machine, pc, &case.bytes, generated::BODIES[case.body]);
        let expected_pc = (u32::from(case.fin.pbr) << 16) | u32::from(case.fin.pc);
        let actual = recording.borrow();
        let wanted_ram: BTreeMap<_, _> = case.fin.ram.iter().copied().collect();
        let active: Vec<_> = case
            .cycles
            .iter()
            .filter_map(|(address, value, pins)| value.map(|byte| (*address, Some(byte), pins.contains('w'))))
            .collect();
        let mut differences = Vec::new();
        if outcome != Ok(expected_pc) {
            differences.push(format!("PC: {outcome:?} != {expected_pc:06X}"));
        }
        if machine.cpu != case.fin.cpu() {
            differences.push(format!("CPU: {:?} != {:?}", machine.cpu, case.fin.cpu()));
        }
        if actual.ram != wanted_ram {
            differences.push("complete sparse memory differs".into());
        }
        if actual.events != active {
            differences.push(format!("ordered active bus: {:?} != {:?}", actual.events, active));
        }
        if machine.bus.cpu_cycles() != case.cycles.len() as u64 {
            differences.push(format!("cycles: {} != {}", machine.bus.cpu_cycles(), case.cycles.len()));
        }
        let status = if differences.is_empty() {
            "pass"
        } else {
            failures += 1;
            "divergence"
        };
        writeln!(
            ledger,
            "{}",
            json!({"name":case.name,"case_sha256":case.case_sha256,"status":status,
            "differences":differences,"opcode":format!("{:02x}",case.bytes[0]),"mx":case.initial.cpu().mx(),
            "decimal":case.initial.p & 8 != 0,"expected_cycles":case.cycles.len()})
        )?;
    }
    if failures > 0 {
        std::process::exit(1);
    }
    Ok(())
}
