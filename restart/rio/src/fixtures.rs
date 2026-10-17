//! Shared deterministic actor corpus, with scripted synchronous native services.
//! Scripts model copyback and call ordering; they do not implement real terrain.
use crate::{Entry, Services, memory::*};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum ServiceKind {
    Horizontal = 0,
    Vertical = 1,
    Angle = 2,
}
#[derive(Clone, Debug)]
pub struct ServiceStep {
    pub kind: ServiceKind,
    pub result: u16,
    pub patches: Vec<(usize, u8)>,
}
#[derive(Clone, Debug)]
pub struct BodyFixture {
    pub name: String,
    pub entry: Entry,
    pub k: u16,
    pub arg: u32,
    pub wram: Vec<u8>,
    pub steps: Vec<ServiceStep>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Call {
    pub kind: ServiceKind,
    pub k: u16,
    /// Signed 16.16 movement; zero for angle requests.
    pub amount: i32,
}
#[derive(Clone, Debug)]
pub struct ScriptedServices {
    pub admitted: bool,
    pub steps: Vec<ServiceStep>,
    pub calls: Vec<Call>,
    /// Optional terminal error at the zero-based request ordinal, after patches.
    pub fail_at: Option<usize>,
}
impl ScriptedServices {
    pub fn new(steps: Vec<ServiceStep>) -> Self {
        Self { admitted: true, steps, calls: Vec::new(), fail_at: None }
    }
    pub fn complete(&self) -> bool {
        self.calls.len() == self.steps.len()
    }
    fn request(&mut self, ram: &mut [u8], kind: ServiceKind, k: u16, amount: i32) -> Result<u16, String> {
        let ordinal = self.calls.len();
        self.calls.push(Call { kind, k, amount });
        let step = self.steps.get(ordinal).ok_or("unexpected Rio fixture service call")?;
        if kind != step.kind {
            return Err("Rio fixture service order mismatch".into());
        }
        for &(offset, value) in &step.patches {
            *ram.get_mut(offset).ok_or("Rio fixture patch outside WRAM")? = value;
        }
        if self.fail_at == Some(ordinal) {
            return Err("injected terminal Rio service failure".into());
        }
        Ok(step.result)
    }
}
impl Services for ScriptedServices {
    fn available(&self) -> bool {
        self.admitted
    }
    fn move_horizontal(&mut self, ram: &mut [u8], k: u16, amount: i32) -> Result<u8, String> {
        let result = self.request(ram, ServiceKind::Horizontal, k, amount)?;
        u8::try_from(result).map_err(|_| "invalid scripted horizontal result".into())
    }
    fn move_vertical(&mut self, ram: &mut [u8], k: u16, amount: i32) -> Result<u8, String> {
        let result = self.request(ram, ServiceKind::Vertical, k, amount)?;
        u8::try_from(result).map_err(|_| "invalid scripted vertical result".into())
    }
    fn angle(&mut self, ram: &mut [u8], k: u16) -> Result<u16, String> {
        self.request(ram, ServiceKind::Angle, k, 0)
    }
}
fn base(name: &str, entry: Entry, k: u16) -> BodyFixture {
    let mut r: Vec<u8> = (0..SIZE).map(|i| (i * 73 + i / 251 + 19) as u8).collect();
    for (at, value) in [
        (CURRENT, k),
        (0x5e5, 0x1234),
        (0xaf6, 0x90),
        (0xafa, 0x90),
        (0x911, 0),
        (0x915, 0),
        (0x5f5, 0),
        (0x998, 8),
        (0x592, 0),
    ] {
        put(&mut r, at, value);
    }
    r[0x644] = 0;
    r[0x647] = 0;
    for index in [k, 0x40, 0x80, 0x7c0] {
        let b = ENEMIES + usize::from(index);
        for (at, value) in [
            (2, 0x80),
            (4, 0x1234),
            (6, 0x80),
            (8, 0x5678),
            (10, 8),
            (12, 8),
            (0x18, 0x9876),
            (0x1a, 0xbb4b),
            (0x1c, 0x7654),
            (A, 0x3456),
            (B, STATES[0]),
            (C, 0x180),
            (D, 0x200),
            (E, 0),
            (F, 0xbb4b),
        ] {
            put(&mut r, b + at, value);
        }
    }
    BodyFixture { name: name.into(), entry, k, arg: 0, wram: r, steps: Vec::new() }
}
fn field(f: &mut BodyFixture, field: usize, value: u16) {
    put(&mut f.wram, ENEMIES + usize::from(f.k) + field, value);
}
fn step(kind: ServiceKind, result: u16) -> ServiceStep {
    ServiceStep { kind, result, patches: Vec::new() }
}
fn patch_word(step: &mut ServiceStep, at: usize, value: u16) {
    step.patches.extend(value.to_le_bytes().into_iter().enumerate().map(|(i, b)| (at + i, b)));
}
/// All ten owned entries; state roots also exercise explicit-vs-current indices.
/// Runtime-ROM speed words are supplied externally by the independently pinned ROM.
pub fn body_fixtures() -> Vec<BodyFixture> {
    use ServiceKind::*;
    let mut fs = Vec::new();
    for slot in [0, 0x40, 0x7c0] {
        fs.push(base(&format!("init-{slot:03x}"), Entry::Init, slot));
    }
    for (name, current_k, explicit, jp) in
        [("instruction-current", 0x40, 0x40, 0xbb83), ("instruction-distinct-odd", 0x7c0, 0x80, 0xfffd)]
    {
        let mut f = base(name, Entry::Instruction, current_k);
        f.k = explicit;
        f.arg = jp;
        fs.push(f);
    }
    for (name, a, old) in [
        ("animation-changed", 0xbb97, 0xbb4b),
        ("animation-unchanged", 0xbb97, 0xbb97),
        ("animation-zero", 0, 0),
        ("animation-arbitrary", 0xffff, 0xbb4b),
    ] {
        let mut f = base(name, Entry::SelectAnimation, 0x40);
        field(&mut f, F, old);
        f.arg = a;
        fs.push(f);
    }
    for delta in [0u16, 159, 160, 161, 0xff61, 0xff60, 0x8000, 0x7fff] {
        let mut f = base(&format!("state1-distance-{delta:04x}"), Entry::State1, 0x40);
        put(&mut f.wram, 0xaf6, 0x80u16.wrapping_add(delta));
        fs.push(f);
    }
    for (name, x, y, left, top) in [
        ("screen-right-edge", 256, 128, 0, 0),
        ("screen-right-out", 257, 128, 0, 0),
        ("screen-left-out", 0xffff, 128, 0, 0),
        ("screen-bottom-edge", 128, 256, 0, 0),
        ("screen-bottom-out", 128, 257, 0, 0),
        ("screen-top-out", 128, 0xffff, 0, 0),
        ("screen-wrap", 0x20, 0x30, 0xff80, 0xff80),
    ] {
        let mut f = base(name, Entry::State1, 0x40);
        field(&mut f, 2, x);
        field(&mut f, 6, y);
        put(&mut f.wram, 0xaf6, x);
        put(&mut f.wram, 0x911, left);
        put(&mut f.wram, 0x915, top);
        fs.push(f);
    }
    for (name, read, write, disabled, game, bomb) in [
        ("sound-wrap", 14, 15, 0, 8, 0),
        ("sound-full-six", 0, 6, 0, 8, 0),
        ("sound-disable", 0, 0, 1, 8, 0),
        ("sound-game-limit", 0, 0, 0, 0x28, 0),
        ("sound-bomb", 0, 0, 0, 8, 0x8000),
    ] {
        let mut f = base(name, Entry::State1, 0x40);
        f.wram[0x644] = read;
        f.wram[0x647] = write;
        put(&mut f.wram, 0x5f5, disabled);
        put(&mut f.wram, 0x998, game);
        put(&mut f.wram, 0x592, bomb);
        fs.push(f);
    }
    for event in [0, 1, 0xffff] {
        let mut f = base(&format!("state2-event-{event:04x}"), Entry::State2, 0x40);
        field(&mut f, E, event);
        fs.push(f);
    }
    for (name, speed, event, h, v) in [
        ("state3-decelerate", 48, 0, 0, 0),
        ("state3-zero-boundary", 24, 0, 0, 0),
        ("state3-negative-boundary", 23, 1, 0, 0),
        ("state3-signed-wrap", 0x8000, 0, 0, 0),
        ("state3-event", 48, 1, 0, 0),
        ("state3-horizontal-hit", 48, 1, 1, 0),
        ("state3-vertical-hit", 48, 1, 0, 1),
    ] {
        let mut f = base(name, Entry::State3, 0x40);
        field(&mut f, C, speed);
        field(&mut f, E, event);
        f.steps.push(step(Horizontal, h));
        if h == 0 {
            f.steps.push(step(Vertical, v));
        }
        fs.push(f);
    }
    for (name, h, v) in
        [("state4-clear", 0, 0), ("state4-h-hit", 1, 0), ("state4-v-hit", 0, 1), ("state4-both-hit", 1, 1)]
    {
        let mut f = base(name, Entry::State4, 0x40);
        field(&mut f, C, 0xff00);
        field(&mut f, D, 0x8000);
        f.steps = vec![step(Horizontal, h), step(Vertical, v)];
        fs.push(f);
    }
    for delta in [0u16, 1, 0x7fff, 0x8000, 0xffff] {
        let mut f = base(&format!("state5-y-{delta:04x}"), Entry::State5, 0x40);
        field(&mut f, 6, 0x90u16.wrapping_add(delta));
        if delta & 0x8000 != 0 {
            f.steps = vec![step(Angle, 0), step(Horizontal, 1), step(Vertical, 1)];
        }
        fs.push(f);
    }
    for angle in [0, 1, 63, 64, 65, 127, 128, 192, 255] {
        let mut f = base(&format!("state5-angle-{angle:03}"), Entry::State5, 0x40);
        f.steps = vec![step(Angle, angle), step(Horizontal, 0), step(Vertical, 0)];
        fs.push(f);
    }
    // Full copyback mutates C, D and E. C caches E as a pointer but reloads fields.
    let mut f = base("state3-copyback", Entry::State3, 0x40);
    let b = ENEMIES + 0x40;
    f.steps = vec![step(Horizontal, 0), step(Vertical, 0)];
    patch_word(&mut f.steps[0], b + C, 0x123);
    patch_word(&mut f.steps[0], b + D, 0xffab);
    patch_word(&mut f.steps[1], b + C, 23);
    f.steps[1].patches.push((0x1ffff, 0xc5));
    fs.push(f);
    let mut f = base("state4-cached-base-current-copyback", Entry::State4, 0x40);
    f.steps = vec![step(Horizontal, 1), step(Vertical, 1)];
    patch_word(&mut f.steps[0], CURRENT, 0x80);
    patch_word(&mut f.steps[0], b + D, 0x23);
    patch_word(&mut f.steps[0], b + C, 0xff45);
    fs.push(f);
    let mut f = base("state5-copyback-angle-and-horizontal", Entry::State5, 0x40);
    f.steps = vec![step(Angle, 64), step(Horizontal, 0), step(Vertical, 0)];
    patch_word(&mut f.steps[0], CURRENT, 0x80);
    patch_word(&mut f.steps[1], b + C, 0xff80);
    fs.push(f);
    for (entry, state) in [(Entry::State1, STATES[0]), (Entry::State2, STATES[1])] {
        let mut f = base(&format!("explicit-current-{state:04x}"), entry, 0x40);
        put(&mut f.wram, CURRENT, 0x80);
        field(&mut f, E, 1);
        fs.push(f);
    }
    // Main covers RNG boundaries plus every dispatched state. Dynamic wrapper
    // covers all supported exact addresses independently, with no invented EA.
    for (state, address) in STATES.into_iter().enumerate() {
        for entry in [Entry::Main, Entry::CallRioFunc] {
            let mut f = base(&format!("dispatch-{entry:?}-{state}"), entry, 0x40);
            field(&mut f, B, address);
            field(&mut f, E, 1);
            f.arg = 0xa20000 | u32::from(address);
            match state {
                2 | 3 => f.steps = vec![step(Horizontal, 0), step(Vertical, 0)],
                4 => f.steps = vec![step(Angle, 255), step(Horizontal, 0), step(Vertical, 0)],
                _ => (),
            }
            fs.push(f);
        }
    }
    for seed in [0, 0x0033, 0x00ff, 0x3300, 0x33ff, 0xffff] {
        let mut f = base(&format!("main-random-{seed:04x}"), Entry::Main, 0x40);
        field(&mut f, B, STATES[1]);
        put(&mut f.wram, 0x5e5, seed);
        fs.push(f);
    }
    fs
}
