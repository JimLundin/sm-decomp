//! Passive canonical instruction state shared by reference and mixed execution.
use rustysnes_core::{Bus, SaveWriter, cpu::Cpu};
use serde::Serialize;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct CpuBoundary {
    pub pc: u32,
    pub a: u16,
    pub x: u16,
    pub y: u16,
    pub s: u16,
    pub d: u16,
    pub db: u8,
    pub p: u8,
    pub emulation: bool,
    pub waiting: bool,
    pub stopped: bool,
    pub irq_masked: bool,
    pub interrupts_taken: u64,
    pub last_interrupt_was_nmi: bool,
    pub cpu_cycles: u64,
    pub master_clock: u64,
    pub scanline: u16,
    pub dot: u16,
    pub frame: u64,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InstructionSnapshot {
    pub cpu: CpuBoundary,
    pub pending_cpu: Vec<u8>,
    pub hardware: Vec<u8>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct BusEvent {
    pub kind: String,
    pub address: u32,
    pub value: Option<u8>,
    pub master: u64,
    pub clocks: u32,
}
#[derive(Clone, Debug)]
pub struct InstructionTrace {
    pub before: InstructionSnapshot,
    pub after: InstructionSnapshot,
    pub events: Vec<BusEvent>,
    pub generated_key: Option<(u32, u8)>,
}

pub(crate) fn snapshot(cpu: &Cpu, bus: &Bus) -> InstructionSnapshot {
    let r = cpu.regs;
    let mut writer = SaveWriter::new();
    cpu.save_state(&mut writer);
    let mut pending = writer.into_bytes();
    // CPU0 payload private cyc is reset before the next step and is adapter bookkeeping.
    // Preserve the entire serializer otherwise, including the old-I override presence/value.
    normalize_cpu_bookkeeping(&mut pending);
    InstructionSnapshot {
        cpu: CpuBoundary {
            pc: (u32::from(r.pbr) << 16) | u32::from(r.pc),
            a: r.a,
            x: r.x,
            y: r.y,
            s: r.s,
            d: r.d,
            db: r.dbr,
            p: r.p.bits(),
            emulation: r.emulation,
            waiting: cpu.waiting,
            stopped: cpu.stopped,
            irq_masked: cpu.irq_masked(),
            interrupts_taken: cpu.interrupts_taken,
            last_interrupt_was_nmi: cpu.last_interrupt_was_nmi,
            cpu_cycles: cpu.cycles,
            master_clock: bus.clock.master,
            scanline: bus.ppu.scanline(),
            dot: bus.ppu.dot(),
            frame: bus.ppu.frame_count(),
        },
        pending_cpu: pending,
        hardware: bus.readiness_state(),
    }
}
fn normalize_cpu_bookkeeping(bytes: &mut [u8]) {
    const CPU_SECTION_HEADER: usize = 8;
    const REGISTERS: usize = 16;
    const TOTAL_CYCLES: usize = 8;
    const WAIT_STOP_LATCHES: usize = 2;
    let offset = CPU_SECTION_HEADER + REGISTERS + TOTAL_CYCLES + WAIT_STOP_LATCHES;
    assert_eq!(&bytes[..4], b"CPU0", "unsupported diagnostic CPU serializer");
    assert!(bytes.len() >= offset + 4, "truncated diagnostic CPU serializer");
    bytes[offset..offset + 4].fill(0);
}
pub(crate) fn events(bus: &mut Bus) -> Vec<BusEvent> {
    bus.readiness_events()
        .into_iter()
        .map(|e| BusEvent {
            kind: e.kind.into(),
            address: e.address,
            value: e.value,
            master: e.master,
            clocks: e.clocks,
        })
        .collect()
}
pub(crate) fn classify(events: &mut [BusEvent], pc: u32) {
    let interrupt = events.iter().any(|event| event.kind == "interrupt");
    if let Some(event) = events.iter_mut().find(|event| event.kind == "read" && event.address == pc) {
        event.kind = if interrupt { "interrupt-discarded-fetch" } else { "fetch" }.into();
    }
    // An enabled implied body has an opcode fetch followed by its final
    // six-clock cycle; classify that already-recorded cycle without observing
    // hardware again. Other opcodes preserve the authoritative cycle kinds.
    let implied = events
        .iter()
        .find(|event| event.kind == "fetch")
        .and_then(|event| event.value)
        .is_some_and(|opcode| matches!(opcode, 0xea | 0xca));
    if implied && let Some(index) = events.iter().rposition(|event| event.kind == "cycle") {
        events[index].kind = "implied-cycle".into();
    }
}
