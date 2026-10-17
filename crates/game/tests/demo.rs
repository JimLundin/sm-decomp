//! End-to-end: bytes → CFG → Rust → compiled → run → check memory.
//! Only meaningful for the built-in demo (no ROM present).

#[cfg(not(sm_generated))]
#[test]
fn demo_program_runs() {
    use game::hw::RamBus;
    let mut m = game::machine(Box::new(RamBus::new()), None);
    m.cpu.set_p(0x30);
    m.cpu.s = 0x1ff;
    m.push16(0xfffe);
    m.call(0x80_8000);
    assert_eq!(m.read(0x10, true), 0x1235, "16-bit store + INC in callee");
    assert_eq!(m.read8(0x12), 0xAB, "8-bit store after SEP");
    assert_eq!(m.cpu.x, 0, "DEX/BNE loop ran to zero");
    assert_eq!(m.cpu.s, 0x1ff, "stack balanced");
    assert_eq!(m.generated_calls, 2);
    assert!(m.misses.is_empty());
}

#[cfg(not(sm_generated))]
#[test]
#[should_panic(expected = "no generated function")]
fn miss_without_fallback_names_the_function() {
    use game::hw::RamBus;
    let mut m = game::machine(Box::new(RamBus::new()), None);
    m.cpu.s = 0x1ff;
    m.call(0x80_9000);
}
