use crate::{Entry, Output, execute, fixtures::*, memory::*};
fn rom() -> Vec<u8> {
    let mut rom = vec![0; 0x300000];
    put(&mut rom, 0x113bbb, 0x180);
    put(&mut rom, 0x113bbf, 0x200);
    rom
}
fn fixture(name: &str) -> BodyFixture {
    body_fixtures().into_iter().find(|f| f.name == name).unwrap()
}
fn run(f: &mut BodyFixture) -> ScriptedServices {
    let mut s = ScriptedServices::new(f.steps.clone());
    assert_eq!(execute(f.entry, f.k, f.arg, &mut f.wram, &rom(), &mut s).unwrap(), Some(Output::Done));
    assert!(s.complete(), "{} unused script requests", f.name);
    s
}
#[test]
fn distance_boundary_and_signed_direction() {
    for (name, attacked, d) in [
        ("state1-distance-009f", true, 0x200),
        ("state1-distance-00a0", false, 0x200),
        ("state1-distance-ff61", true, 0xfe00),
        ("state1-distance-ff60", false, 0x200),
        ("state1-distance-8000", false, 0x200),
    ] {
        let mut f = fixture(name);
        run(&mut f);
        let b = ENEMIES + 0x40;
        assert_eq!(word(&f.wram, b + B), if attacked { STATES[2] } else { STATES[0] });
        assert_eq!(word(&f.wram, b + D), d);
    }
}
#[test]
fn horizontal_collision_short_circuits_vertical() {
    let mut f = fixture("state3-horizontal-hit");
    let calls = run(&mut f).calls;
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].amount, 0x20000);
    let b = ENEMIES + 0x40;
    assert_eq!(word(&f.wram, b + D), 0xfe00);
    assert_eq!(word(&f.wram, b + C), 0xffd0);
    assert_eq!(word(&f.wram, b + B), STATES[3]);
}
#[test]
fn deceleration_boundary_precedes_event() {
    let mut f = fixture("state3-negative-boundary");
    run(&mut f);
    let b = ENEMIES + 0x40;
    assert_eq!(word(&f.wram, b + A), 0x200);
    assert_eq!(word(&f.wram, b + E), 1);
    assert_eq!(word(&f.wram, b + B), STATES[4]);
    assert_eq!(word(&f.wram, b + C), 0);
    let mut zero = fixture("state3-zero-boundary");
    run(&mut zero);
    assert_eq!(word(&zero.wram, b + B), STATES[0]);
    assert_eq!(word(&zero.wram, b + C), 0);
}
#[test]
fn full_copyback_rereads_fields_and_retains_cached_pointer() {
    let mut f = fixture("state3-copyback");
    let calls = run(&mut f).calls;
    assert_eq!(calls[1].amount, 0x12300);
    assert_eq!(word(&f.wram, ENEMIES + 0x40 + A), 0xffab);
    assert_eq!(f.wram[0x1ffff], 0xc5);
    let mut f = fixture("state4-cached-base-current-copyback");
    run(&mut f);
    assert_eq!(word(&f.wram, ENEMIES + 0x40 + D), 0xffdd);
    assert_eq!(word(&f.wram, ENEMIES + 0x40 + B), STATES[1]);
    assert_eq!(word(&f.wram, ENEMIES + 0x80 + F), 0xbba3);
}
#[test]
fn signed_sine_pursuit_ignores_collision_outputs() {
    let mut f = fixture("state5-y-ffff");
    let calls = run(&mut f).calls;
    assert_eq!(
        calls.iter().map(|c| (c.kind, c.amount)).collect::<Vec<_>>(),
        vec![(ServiceKind::Angle, 0), (ServiceKind::Horizontal, 0), (ServiceKind::Vertical, -65536)]
    );
    let b = ENEMIES + 0x40;
    assert_eq!(word(&f.wram, b + B), STATES[0]);
    let mut f = fixture("state5-copyback-angle-and-horizontal");
    let calls = run(&mut f).calls;
    assert_eq!(calls[1].amount, 65536);
    assert_eq!(calls[2].amount, -32768);
}
#[test]
fn current_slot_helpers_differ_from_explicit_state_slot() {
    let mut f = fixture("explicit-current-bc32");
    run(&mut f);
    assert_eq!(word(&f.wram, ENEMIES + 0x40 + E), 0);
    assert_eq!(word(&f.wram, ENEMIES + 0x40 + F), 0xbb4b);
    assert_eq!(word(&f.wram, ENEMIES + 0x80 + F), 0xbb53);
    let mut f = fixture("instruction-distinct-odd");
    let mut s = ScriptedServices::new(vec![]);
    assert_eq!(execute(f.entry, f.k, f.arg, &mut f.wram, &rom(), &mut s).unwrap(), Some(Output::Continue(0xfffd)));
    assert_eq!(word(&f.wram, ENEMIES + 0x7c0 + E), 1);
    assert_eq!(word(&f.wram, ENEMIES + 0x80 + E), 0);
}
#[test]
fn random_carry_matches_recovered_byte_algorithm() {
    for (seed, expected) in [(0, 0x111), (0x3300, 0x12), (0x33ff, 0x50d), (0xffff, 0x10d)] {
        let mut f = fixture(&format!("main-random-{seed:04x}"));
        run(&mut f);
        assert_eq!(word(&f.wram, 0x5e5), expected);
    }
}
#[test]
fn screen_edges_and_sound_ring_are_exact() {
    for (name, queued) in [
        ("screen-right-edge", true),
        ("screen-right-out", false),
        ("screen-bottom-edge", true),
        ("screen-bottom-out", false),
    ] {
        let mut f = fixture(name);
        run(&mut f);
        assert_eq!(f.wram[0x647], if queued { 1 } else { 0 });
    }
    let mut f = fixture("sound-wrap");
    let high_sentinel = f.wram[0x655];
    run(&mut f);
    assert_eq!(f.wram[0x647], 0);
    assert_eq!(f.wram[0x675], 0x65);
    assert_eq!(f.wram[0x666], 0);
    assert_eq!(f.wram[0x654], 6);
    assert_eq!(f.wram[0x655], high_sentinel);
    for name in ["sound-full-six", "sound-disable", "sound-game-limit", "sound-bomb"] {
        let mut f = fixture(name);
        let before = f.wram[0x647];
        run(&mut f);
        assert_eq!(f.wram[0x647], before);
        assert_eq!(f.wram[0x654], 6);
    }
}
#[test]
fn unchanged_animation_preserves_full_wram() {
    let mut f = fixture("animation-unchanged");
    let old = f.wram.clone();
    run(&mut f);
    assert_eq!(f.wram, old);
}
#[test]
fn unsupported_state_and_native_domain_refuse_before_rng() {
    for unsupported in [true, false] {
        let mut f = fixture("dispatch-Main-2");
        if unsupported {
            put(&mut f.wram, ENEMIES + 0x40 + B, 0x1234);
        }
        let old = f.wram.clone();
        let mut s = ScriptedServices::new(f.steps);
        s.admitted = false;
        assert_eq!(execute(f.entry, f.k, f.arg, &mut f.wram, &rom(), &mut s).unwrap(), None);
        assert_eq!(f.wram, old);
        assert!(s.calls.is_empty());
    }
    let mut f = fixture("dispatch-CallRioFunc-0");
    let old = f.wram.clone();
    let mut s = ScriptedServices::new(vec![]);
    assert_eq!(execute(Entry::CallRioFunc, 0, 0xa3bbed, &mut f.wram, &rom(), &mut s).unwrap(), None);
    assert_eq!(f.wram, old);
}
#[test]
fn malformed_input_is_atomic_before_main_rng() {
    for missing_rom in [true, false] {
        let mut f = fixture("dispatch-Main-0");
        if !missing_rom {
            f.wram[0x647] = 16;
        }
        let old = f.wram.clone();
        let mut s = ScriptedServices::new(vec![]);
        let rom = if missing_rom { vec![] } else { rom() };
        assert!(execute(f.entry, f.k, f.arg, &mut f.wram, &rom, &mut s).is_err());
        assert_eq!(f.wram, old);
        assert!(s.calls.is_empty());
    }
}
#[test]
fn service_failure_is_terminal_after_owned_effects() {
    let mut f = fixture("dispatch-Main-2");
    let seed = word(&f.wram, 0x5e5);
    let mut s = ScriptedServices::new(f.steps);
    s.fail_at = Some(0);
    assert!(execute(f.entry, f.k, f.arg, &mut f.wram, &rom(), &mut s).is_err());
    assert_ne!(word(&f.wram, 0x5e5), seed);
    assert_eq!(s.calls.len(), 1);
    let mut f = fixture("state5-angle-000");
    f.steps[0].result = 256;
    let mut s = ScriptedServices::new(f.steps);
    assert!(execute(f.entry, f.k, f.arg, &mut f.wram, &rom(), &mut s).is_err());
    assert_eq!(s.calls.len(), 1);
}

#[test]
fn invalid_collision_result_is_terminal_and_stops_remaining_services() {
    for ordinal in [0, 1] {
        let mut f = fixture("dispatch-Main-2");
        f.steps[ordinal].result = 2;
        let seed = word(&f.wram, 0x5e5);
        let mut s = ScriptedServices::new(f.steps);
        assert!(execute(f.entry, f.k, f.arg, &mut f.wram, &rom(), &mut s).is_err());
        assert_eq!(s.calls.len(), ordinal + 1);
        assert_ne!(word(&f.wram, 0x5e5), seed);
    }
}
