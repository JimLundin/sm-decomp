use crate::{memory::*, *};
#[derive(Debug, PartialEq, Eq)]
enum Call {
    Horizontal(u16, i32),
    Sine(u16, u16),
    Vertical(u16, i32),
}
#[derive(Default)]
struct Native {
    unavailable: bool,
    horizontal_hit: u8,
    vertical_hit: u8,
    sine_value: u16,
    fail: bool,
    replace_velocity: bool,
    replace_phase: bool,
    calls: Vec<Call>,
}
impl Services for Native {
    fn available(&self) -> bool {
        !self.unavailable
    }
    fn move_horizontal(&mut self, ram: &mut [u8], k: u16, amount: i32) -> Result<u8, String> {
        self.calls.push(Call::Horizontal(k, amount));
        ram[0x123] = 0x61; // Unrelated native scratch must survive the round trip.
        if self.fail {
            return Err("horizontal failure".into());
        }
        if self.replace_velocity {
            let b = ENEMIES + usize::from(k);
            put(ram, b + A, 0x3400);
            put(ram, b + B, 2);
        }
        Ok(self.horizontal_hit)
    }
    fn sine(&mut self, ram: &mut [u8], angle: u16, amplitude: u16) -> Result<u16, String> {
        self.calls.push(Call::Sine(angle, amplitude));
        if self.replace_phase {
            let b = current(ram)?.1;
            put(ram, b + D, 0xfffe);
        }
        Ok(self.sine_value)
    }
    fn move_vertical(&mut self, ram: &mut [u8], k: u16, amount: i32) -> Result<u8, String> {
        self.calls.push(Call::Vertical(k, amount));
        ram[0x124] = 0x62;
        Ok(self.vertical_hit)
    }
}
fn rom() -> Vec<u8> {
    let mut rom = vec![0; TABLE + 8];
    for (i, value) in [0x86a7, 0x86ad, 0x86b3, 0x86c7].into_iter().enumerate() {
        put(&mut rom, TABLE + 2 * i, value);
    }
    rom
}
fn fixture(name: &str) -> BodyFixture {
    body_fixtures().into_iter().find(|f| f.name == name).unwrap()
}
fn run(f: &mut BodyFixture, services: &mut Native) -> Result<Option<Output>, String> {
    execute(f.entry, f.k, f.jp, &mut f.wram, &rom(), services)
}
#[test]
fn left_init_keeps_nonstandard_sign_extension_and_untouched_timers() {
    let mut f = fixture("init-left");
    let mut expected = f.wram.clone();
    let b = current(&f.wram).unwrap().1;
    for (at, value) in [(A, 0x8000), (B, 0xfffe), (C, 0), (E, 0), (F, 0), (INSTRUCTION, 0x86a7)] {
        put(&mut expected, b + at, value);
    }
    let mut service = Native::default();
    assert_eq!(run(&mut f, &mut service).unwrap(), Some(Output::Done));
    assert_eq!(f.wram, expected);
    assert!(service.calls.is_empty());
}
#[test]
fn right_init_uses_runtime_animation_and_resets_only_changed_timers() {
    let mut f = fixture("init-right-high-parameter");
    let b = current(&f.wram).unwrap().1;
    let before = f.wram.clone();
    let mut runtime = rom();
    put(&mut runtime, TABLE + 2, 0x9011);
    execute(f.entry, f.k, f.jp, &mut f.wram, &runtime, &mut Native::default()).unwrap();
    assert_eq!(word(&f.wram, b + A), 0x8000);
    assert_eq!(word(&f.wram, b + B), 1);
    assert_eq!(word(&f.wram, b + C), 1);
    assert_eq!(word(&f.wram, b + F), 1);
    assert_eq!(word(&f.wram, b + INSTRUCTION), 0x9011);
    assert_eq!(word(&f.wram, b + TIMER), 0);
    assert_eq!(word(&f.wram, b + INSTRUCTION_TIMER), 1);
    assert_eq!(word(&f.wram, b + D), word(&before, b + D));
    assert_eq!(&f.wram[..b], &before[..b]);
    assert_eq!(&f.wram[b + 64..], &before[b + 64..]);
}
#[test]
fn unchanged_helper_precedes_rom_lookup_even_for_ffff() {
    let mut f = fixture("helper-unchanged-ffff");
    let before = f.wram.clone();
    assert_eq!(execute(f.entry, 0, 0, &mut f.wram, &[], &mut Native::default()).unwrap(), Some(Output::Done));
    assert_eq!(f.wram, before);
}
#[test]
fn changed_helper_selects_all_four_runtime_entries() {
    for selector in 0..4 {
        let mut f = fixture(&format!("helper-select-{selector}"));
        let b = current(&f.wram).unwrap().1;
        let mut expected = f.wram.clone();
        for (at, value) in [
            (C, selector),
            (TIMER, 0),
            (INSTRUCTION_TIMER, 1),
            (INSTRUCTION, [0x86a7, 0x86ad, 0x86b3, 0x86c7][usize::from(selector)]),
        ] {
            put(&mut expected, b + at, value);
        }
        run(&mut f, &mut Native::default()).unwrap();
        assert_eq!(f.wram, expected);
    }
}
#[test]
fn instruction_writes_current_slot_and_returns_odd_jp_unchanged() {
    for name in ["instruction-current", "instruction-distinct-k"] {
        let mut f = fixture(name);
        let mut expected = f.wram.clone();
        let b = current(&f.wram).unwrap().1;
        put(&mut expected, b + E, 1);
        let mut service = Native::default();
        assert_eq!(run(&mut f, &mut service).unwrap(), Some(Output::Continue(f.jp)));
        assert_eq!(f.wram, expected);
        assert!(service.calls.is_empty());
    }
}
#[test]
fn fractional_horizontal_hit_reverses_and_skips_other_services() {
    let mut f = fixture("main-right-fractional-wall");
    let b = current(&f.wram).unwrap().1;
    let mut service = Native { horizontal_hit: 1, ..Native::default() };
    run(&mut f, &mut service).unwrap();
    assert_eq!(service.calls, vec![Call::Horizontal(0x40, 0x18123)]);
    assert_eq!(word(&f.wram, b + B), 0xfffe);
    assert_eq!(word(&f.wram, b + A), 0x7f00);
    assert_eq!(word(&f.wram, b + F), 0);
    assert_eq!(word(&f.wram, b + INSTRUCTION), 0x86a7);
    assert_eq!(word(&f.wram, b + D), 0);
    assert_eq!(f.wram[0x123], 0x61);
}
#[test]
fn reversed_fraction_80_does_not_overflow_host_arithmetic() {
    let mut f = fixture("main-left-wall");
    let b = current(&f.wram).unwrap().1;
    run(&mut f, &mut Native { horizontal_hit: 1, ..Native::default() }).unwrap();
    assert_eq!(word(&f.wram, b + A), 0x8000);
    assert_eq!(word(&f.wram, b + B), 1);
}
#[test]
fn signed_sine_bits_and_native_call_order_are_preserved() {
    for (name, angle, sine, amount) in
        [("main-negative-sine", 0x40, 0xfffd, -196608), ("main-positive-sine", 0xc0, 3, 196608)]
    {
        let mut f = fixture(name);
        let mut service = Native { sine_value: sine, ..Native::default() };
        run(&mut f, &mut service).unwrap();
        assert_eq!(
            service.calls,
            vec![Call::Horizontal(0x40, 0x18000), Call::Sine(angle, 4), Call::Vertical(0x40, amount)]
        );
        assert_eq!(&f.wram[0x123..0x125], &[0x61, 0x62]);
    }
}
#[test]
fn vertical_hit_truncates_phase_and_clear_path_wraps_word() {
    let mut hit = fixture("main-negative-sine");
    let b = current(&hit.wram).unwrap().1;
    put(&mut hit.wram, b + D, 0xab40);
    run(&mut hit, &mut Native { vertical_hit: 1, sine_value: 0xfffd, ..Native::default() }).unwrap();
    assert_eq!(word(&hit.wram, b + D), 0xc0);
    let mut clear = fixture("main-phase-wrap");
    run(&mut clear, &mut Native::default()).unwrap();
    assert_eq!(word(&clear.wram, b + D), 0);
}
#[test]
fn special_phase_then_event_resets_timer_even_when_final_selector_is_original() {
    let mut f = fixture("main-special-then-event");
    let b = current(&f.wram).unwrap().1;
    run(&mut f, &mut Native::default()).unwrap();
    assert_eq!(word(&f.wram, b + D), 56);
    assert_eq!(word(&f.wram, b + E), 0);
    assert_eq!(word(&f.wram, b + F), 1);
    assert_eq!(word(&f.wram, b + C), 1);
    assert_eq!(word(&f.wram, b + INSTRUCTION), 0x86ad);
    assert_eq!(word(&f.wram, b + TIMER), 0);
    assert_eq!(word(&f.wram, b + INSTRUCTION_TIMER), 1);
}
#[test]
fn instruction_main_chain_consumes_event_and_returns_normal_animation() {
    let mut f = fixture("instruction-current");
    let b = current(&f.wram).unwrap().1;
    put(&mut f.wram, b + F, 3);
    put(&mut f.wram, b + C, 3);
    run(&mut f, &mut Native::default()).unwrap();
    f.entry = Entry::Main;
    run(&mut f, &mut Native::default()).unwrap();
    assert_eq!(word(&f.wram, b + E), 0);
    assert_eq!(word(&f.wram, b + C), 1);
    assert_eq!(word(&f.wram, b + INSTRUCTION), 0x86ad);
}
#[test]
fn fields_are_reloaded_after_native_round_trips() {
    let mut f = fixture("main-right-fractional-wall");
    let b = current(&f.wram).unwrap().1;
    run(&mut f, &mut Native { horizontal_hit: 1, replace_velocity: true, ..Native::default() }).unwrap();
    assert_eq!(word(&f.wram, b + A), 0xcc00);
    assert_eq!(word(&f.wram, b + B), 0xfffd);
    let mut f = fixture("main-zero-sine");
    run(&mut f, &mut Native { replace_phase: true, ..Native::default() }).unwrap();
    assert_eq!(word(&f.wram, b + D), 0);
}
#[test]
fn refusals_are_immutable_and_before_any_service() {
    for entry in [Entry::Main, Entry::SelectAnimation] {
        let mut f = fixture("main-zero-sine");
        f.entry = entry;
        let b = current(&f.wram).unwrap().1;
        put(&mut f.wram, b + F, 4);
        let before = f.wram.clone();
        let mut service = Native::default();
        assert_eq!(run(&mut f, &mut service).unwrap(), None);
        assert_eq!(f.wram, before);
        assert!(service.calls.is_empty());
    }
    let mut f = fixture("main-zero-sine");
    let before = f.wram.clone();
    let mut service = Native { unavailable: true, ..Native::default() };
    assert_eq!(run(&mut f, &mut service).unwrap(), None);
    assert_eq!(f.wram, before);
    assert!(service.calls.is_empty());
}
#[test]
fn malformed_inputs_fail_before_any_write() {
    for mode in 0..5 {
        let mut f = fixture("instruction-current");
        let mut runtime = rom();
        match mode {
            0 => {
                f.wram.pop();
            }
            1 => put(&mut f.wram, CURRENT, 1),
            2 => f.k = 0x800,
            3 => f.jp = 0xffff,
            _ => runtime.clear(),
        }
        let before = f.wram.clone();
        let mut service = Native::default();
        assert!(execute(f.entry, f.k, f.jp, &mut f.wram, &runtime, &mut service).is_err());
        assert_eq!(f.wram, before);
        assert!(service.calls.is_empty());
    }
    let mut f = fixture("init-right");
    let before = f.wram.clone();
    assert!(execute(f.entry, 0, 0, &mut f.wram, &[], &mut Native::default()).is_err());
    assert_eq!(f.wram, before);
}
#[test]
fn owned_service_error_and_invalid_collision_cannot_be_refusals() {
    for service in [Native { fail: true, ..Native::default() }, Native { horizontal_hit: 2, ..Native::default() }] {
        let mut service = service;
        let mut f = fixture("main-zero-sine");
        assert!(run(&mut f, &mut service).is_err());
        assert_eq!(service.calls.len(), 1);
        assert_eq!(f.wram[0x123], 0x61);
    }
}
