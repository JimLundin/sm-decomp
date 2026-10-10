use snes::{Reference, Subject};
fn rom() -> Vec<u8> {
    let mut r = vec![0; 0x10000];
    r[..5].copy_from_slice(&[0x18, 0xfb, 0xea, 0x80, 0xfd]);
    r[0x7fc0..0x7fd5].copy_from_slice(b"SM RECOMP TEST       ");
    r[0x7fd5] = 0x20;
    r[0x7fd9] = 1;
    r[0x7fdc..0x7fe0].copy_from_slice(&[0xcb, 0xed, 0x34, 0x12]);
    r[0x7ffc..0x7ffe].copy_from_slice(&0x8000u16.to_le_bytes());
    r
}
#[test]
fn discovery_finds_actual_compiled_execution_and_names_missing_key() {
    fn nop(m: &mut runtime::Machine) {
        m.bus.implied_cycle(0x8003, m.cpu.flag(runtime::flag::I));
    }
    let rom = rom();
    let mut reference = Reference::new(&rom, 0).unwrap();
    let mut subject = Subject::with_steps(&rom, 0, &[(0x8002, 3, &[0xea], nop)]).unwrap();
    let result =
        snes::controls::covered_intervals(&mut reference, &mut subject, &[0, 0], 1, &[(0x8002, 3), (0x8020, 3)])
            .unwrap();
    assert_eq!(result["status"], "uncovered");
    assert_eq!(result["intervals"][0]["first_execution_instruction"], 2);
    assert_eq!(result["unexecuted"][0]["address"], 0x8020);
}

#[test]
fn wrong_generated_candidate_diverges_and_disabled_and_predecessor_controls_pass() {
    fn wrong(m: &mut runtime::Machine) {
        m.cpu.a = 0xdead;
        m.bus.implied_cycle(0x8003, m.cpu.flag(runtime::flag::I));
    }
    fn correct(m: &mut runtime::Machine) {
        m.bus.implied_cycle(0x8003, m.cpu.flag(runtime::flag::I));
    }
    let rom = rom();
    let pads = [0, 0, 0];
    let folder = std::env::temp_dir().join(format!("sm-control-fault-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&folder);
    std::fs::create_dir(&folder).unwrap();
    let mut reference = Reference::new(&rom, 0).unwrap();
    let mut faulty = Subject::with_steps(&rom, 0, &[(0x8002, 3, &[0xea], wrong)]).unwrap();
    let failed =
        snes::controls::compare_interval(&mut reference, &mut faulty, &pads, 0, 0, 6, &folder.join("candidate"))
            .unwrap();
    assert_eq!(failed["status"], "divergence");
    assert_eq!(failed["first_mismatch"]["subject"]["generated_key"][0], 0x8002);
    for (name, table) in
        [("disabled", &[][..]), ("predecessor", &[(0x8002, 3, &[0xea][..], correct as runtime::StepFn)][..])]
    {
        let mut reference = Reference::new(&rom, 0).unwrap();
        let mut subject = Subject::with_steps(&rom, 0, table).unwrap();
        let result =
            snes::controls::compare_interval(&mut reference, &mut subject, &pads, 0, 0, 6, &folder.join(name)).unwrap();
        assert_eq!(result["status"], "pass");
        assert_eq!(result["completed_instructions"], 6);
    }
    std::fs::remove_dir_all(folder).unwrap();
}
