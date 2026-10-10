use snes::{Reference, Subject};
mod emitted {
    #![allow(unused_imports)]
    include!(concat!(env!("OUT_DIR"), "/test_steps.rs"));
}
fn cartridge() -> Vec<u8> {
    let mut rom = vec![0; 0x10000];
    let program = [
        0x18, 0xfb, 0xc2, 0x30, 0xa9, 0x34, 0x12, 0x8d, 0x10, 0, 0xe2, 0x20, 0xa9, 0x80, 0x8d, 0, 0x42, 0xa2, 3, 0,
        0xca, 0xd0, 0xfd, 0xa9, 0xab, 0x8d, 0x12, 0, 0x80, 0xf3,
    ];
    rom[..program.len()].copy_from_slice(&program);
    rom[0x7fc0..0x7fd5].copy_from_slice(b"SM DIAGNOSTIC TEST   ");
    rom[0x7fd5] = 0x20;
    rom[0x7fd9] = 1;
    rom[0x7fdc..0x7fe0].copy_from_slice(&[0xcb, 0xed, 0x34, 0x12]);
    rom[0x7ffc..0x7ffe].copy_from_slice(&0x8000u16.to_le_bytes());
    rom
}
#[test]
fn single_steps_compare_full_pending_state_and_passive_bus_events() {
    let rom = cartridge();
    let mut r = Reference::new(&rom, 0).unwrap();
    let mut s = Subject::with_steps(&rom, 0, emitted::STEP_TABLE).unwrap();
    r.set_input(0x1000);
    s.set_input(0x1000);
    for _ in 0..100 {
        assert_eq!(r.instruction_snapshot(), s.instruction_snapshot());
        let reference = r.step_instruction_observed().unwrap();
        let subject = s.step_instruction_observed().unwrap();
        assert_eq!(reference.events, subject.events);
        assert_eq!(reference.after, subject.after);
    }
    let mut plain = Subject::with_steps(&rom, 0, emitted::STEP_TABLE).unwrap();
    plain.set_input(0x1000);
    for _ in 0..100 {
        plain.step_instruction().unwrap();
    }
    assert_eq!(plain.instruction_snapshot(), s.instruction_snapshot());
    assert!(s.generated_steps > 0);
}

#[test]
fn transient_wrong_generated_store_is_caught_before_next_store_restores_frame_bytes() {
    fn wrong(m: &mut runtime::Machine) {
        m.write8(0x10, 0x99);
    }
    let mut rom = cartridge();
    rom[..13].copy_from_slice(&[0x18, 0xfb, 0xa9, 0x12, 0x8d, 0x10, 0, 0xa9, 0, 0x8d, 0x10, 0, 0xdb]);
    let table = &[(0x8004, 3, &[0x8d, 0x10, 0][..], wrong as runtime::StepFn)];
    let output = std::env::temp_dir().join(format!("sm-focused-transient-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&output);
    let mut cold_r = Reference::new(&rom, 0).unwrap();
    let mut cold_s = Subject::with_steps(&rom, 0, table).unwrap();
    let report = snes::focused::compare(&mut cold_r, &mut cold_s, &[0, 0], 0, 10, &output).unwrap();
    assert_eq!(report["status"], "divergence");
    assert_eq!(report["first_mismatch"]["subject"]["generated_key"], serde_json::json!([0x8004, 3]));
    assert!(report["preceding_matching_boundary"].is_object());
    assert!(output.join("subject-after.hardware.bin").is_file());
    std::fs::remove_dir_all(output).unwrap();
    let mut r = Reference::new(&rom, 0).unwrap();
    let mut s = Subject::with_steps(&rom, 0, table).unwrap();
    for _ in 0..3 {
        r.step_instruction().unwrap();
        s.step_instruction().unwrap();
    }
    let a = r.step_instruction_observed().unwrap();
    let b = s.step_instruction_observed().unwrap();
    assert_ne!(a.after, b.after);
    assert_ne!(a.events, b.events);
    assert_eq!(b.generated_key, Some((0x8004, 3)));
    r.run_frame(0).unwrap();
    s.run_frame(0).unwrap();
    assert_eq!(r.wram()[0x10], s.wram()[0x10]);
}

#[test]
fn cold_frame_focus_finds_later_failure_after_first_key_window_passes() {
    fn late_wrong(m: &mut runtime::Machine) {
        let value = m.cpu.a as u8;
        m.write8(0x10, if value == 0 { 0x99 } else { value });
    }
    let mut rom = cartridge();
    let mut program = vec![0x18, 0xfb, 0xa9, 0x12, 0x8d, 0x10, 0];
    program.extend(std::iter::repeat_n(0xea, 80));
    program.extend([0xa9, 0, 0x82, 0xa8, 0xff]); // same store key, later A=0
    rom[..program.len()].copy_from_slice(&program);
    let table = [(0x8004, 3, &[0x8d, 0x10, 0][..], late_wrong as runtime::StepFn)];
    let output = std::env::temp_dir().join(format!("sm-focused-later-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&output);
    std::fs::create_dir(&output).unwrap();
    let early = snes::focused::compare(
        &mut Reference::new(&rom, 0).unwrap(),
        &mut Subject::with_steps(&rom, 0, &table).unwrap(),
        &[0],
        0,
        65,
        &output.join("early"),
    )
    .unwrap();
    assert_eq!(early["status"], "pass");
    let later = snes::focused::compare_frame_from_boundary(
        &mut Reference::new(&rom, 0).unwrap(),
        &mut Subject::with_steps(&rom, 0, &table).unwrap(),
        &[0],
        0,
        &output.join("later"),
    )
    .unwrap();
    assert_eq!(later["status"], "divergence");
    assert!(later["first_mismatch"]["instruction"].as_u64().unwrap() > 65);
    assert_eq!(later["first_mismatch"]["subject"]["generated_key"], serde_json::json!([0x8004, 3]));
    assert!(output.join("later/subject-after.hardware.bin").is_file());
    // A failing next-frame cache cannot prevent cold mismatch evidence.
    assert!(
        snes::paired::prepare_cache(&rom, 0, &table, &[0], (0, 0), serde_json::json!({}), &output.join("bad-cache"))
            .is_err()
    );
    for (name, steps) in [("predecessor", &[][..]), ("disabled", &[][..])] {
        let report = snes::focused::compare_from_boundary(
            &mut Reference::new(&rom, 0).unwrap(),
            &mut Subject::with_steps(&rom, 0, steps).unwrap(),
            &[0],
            0,
            0,
            later["completed_instructions"].as_u64().unwrap() as usize,
            &output.join(name),
        )
        .unwrap();
        assert_eq!(report["status"], "pass");
    }
    std::fs::remove_dir_all(output).unwrap();
}

#[test]
fn internal_dma_accesses_are_ordered_and_observation_preserves_hardware() {
    use rustysnes_core::{Bus, cart::Region, cpu::Bus as CpuBus};
    fn ready() -> Bus {
        let mut b = Bus::new(Region::Ntsc);
        b.dma.write_reg(0, 0, 0);
        b.dma.write_reg(0, 1, 0x40);
        b.dma.write_reg(0, 2, 0x10);
        b.dma.write_reg(0, 3, 0);
        b.dma.write_reg(0, 4, 0x7e);
        b.dma.write_reg(0, 5, 2);
        b.dma.write_reg(0, 6, 0);
        b.write24(0x7e0010, 0x12);
        b.write24(0x7e0011, 0x34);
        b.write24(0x420b, 1);
        b
    }
    let mut observed = ready();
    let mut plain = ready();
    observed.readiness_observe();
    for b in [&mut observed, &mut plain] {
        b.begin_cpu_cycle(8);
        b.advance(8);
        b.begin_cpu_cycle(6);
        b.advance(6);
    }
    let events = observed.readiness_events();
    let accesses: Vec<_> = events
        .iter()
        .filter(|e| e.kind.starts_with("dma-") && e.value.is_some())
        .map(|e| (e.kind, e.address, e.value))
        .collect();
    assert_eq!(
        accesses,
        vec![
            ("dma-read-a", 0x7e0010, Some(0x12)),
            ("dma-write-b", 0x2140, Some(0x12)),
            ("dma-read-a", 0x7e0011, Some(0x34)),
            ("dma-write-b", 0x2140, Some(0x34))
        ]
    );
    assert_eq!(observed.readiness_state(), plain.readiness_state());
}
