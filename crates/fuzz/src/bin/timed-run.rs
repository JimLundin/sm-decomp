//! Execute compiled production bodies and compare independent CPU/bus vectors.
use anyhow::{Result, ensure};
use pipeline_model::timed_corpus::{State, TimedResult, decode_cases, encode_ledger};
use runtime::{Bus, Cpu, Machine, TimedBus};
use std::{cell::RefCell, collections::BTreeMap, rc::Rc};

mod generated {
    #![allow(unused_imports)]
    include!(concat!(env!("OUT_DIR"), "/timed_steps.rs"));
}

fn cpu(state: &State) -> Cpu {
    let self_state = state;
    Cpu {
        a: self_state.a,
        x: self_state.x,
        y: self_state.y,
        s: self_state.s,
        p: self_state.p,
        db: self_state.dbr,
        d: self_state.d,
        pb: self_state.pbr,
        e: self_state.e != 0,
    }
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
    ensure!(args.len() == 2, "usage: timed-run prepared.bin ledger.bin");
    let cases = decode_cases(&std::fs::read(&args[0])?)?;
    ensure!(!cases.is_empty() && !generated::BODIES.is_empty(), "inactive timed runner");
    let mut ledger = Vec::new();
    let mut failures = 0;
    for case in cases {
        let recording =
            Rc::new(RefCell::new(Recording { ram: case.initial.ram.iter().copied().collect(), events: Vec::new() }));
        let mut machine = Machine::new(Box::new(TimedBus::new(Sparse(recording.clone()))), &[], None);
        machine.cpu = cpu(&case.initial);
        let pc = (u32::from(case.initial.pbr) << 16) | u32::from(case.initial.pc);
        let outcome = runtime::step::execute_step(
            &mut machine,
            pc,
            &case.bytes,
            *generated::BODIES.get(case.body).ok_or_else(|| anyhow::anyhow!("invalid prepared timing body"))?,
        );
        let expected_pc = (u32::from(case.fin.pbr) << 16) | u32::from(case.fin.pc);
        let actual = recording.borrow();
        let wanted_ram: BTreeMap<_, _> = case.fin.ram.iter().copied().collect();
        let active: Vec<_> = case
            .cycles
            .iter()
            .filter_map(|cycle| cycle.value.map(|byte| (cycle.address, Some(byte), cycle.pins.contains('w'))))
            .collect();
        let mut differences = Vec::new();
        if outcome != Ok(expected_pc) {
            differences.push(format!("PC: {outcome:?} != {expected_pc:06X}"));
        }
        if machine.cpu != cpu(&case.fin) {
            differences.push(format!("CPU: {:?} != {:?}", machine.cpu, cpu(&case.fin)));
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
        ledger.push(TimedResult {
            name: case.name,
            case_sha256: case.case_sha256,
            status: status.into(),
            differences,
            opcode: format!("{:02x}", case.bytes[0]),
            mx: cpu(&case.initial).mx(),
            decimal: case.initial.p & 8 != 0,
            expected_cycles: case.cycles.len(),
        });
    }
    std::fs::write(&args[1], encode_ledger(&ledger)?)?;
    if failures > 0 {
        std::process::exit(1);
    }
    Ok(())
}
