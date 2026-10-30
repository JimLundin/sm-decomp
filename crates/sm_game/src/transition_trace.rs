//! Verifier trace of player-transition publications, shared by the gameplay
//! modules that publish into it and the compat observer that seeds and reads it.
use std::cell::RefCell;

struct Trace {
    bytes: Vec<u8>,
    seed: Vec<u8>,
    excluded: Vec<(usize, usize)>,
}
thread_local! { static TRACE: RefCell<Option<Trace>> = const { RefCell::new(None) }; }
pub fn active() -> bool {
    TRACE.with(|s| s.borrow().is_some())
}
pub fn append(tag: u8, payload: &[u8]) {
    TRACE.with(|s| {
        if let Some(t) = s.borrow_mut().as_mut() {
            t.bytes.push(tag);
            t.bytes
                .extend_from_slice(&(payload.len() as u32).to_le_bytes());
            t.bytes.extend_from_slice(payload);
        }
    });
}
/// Start a trace with its initial bytes, the RAM seed and the CPU-only ranges.
pub fn start(bytes: Vec<u8>, seed: impl FnOnce() -> Vec<u8>, excluded: Vec<(usize, usize)>) {
    TRACE.with(|s| {
        assert!(s.borrow().is_none(), "nested transition observer");
        *s.borrow_mut() = Some(Trace {
            bytes,
            seed: seed(),
            excluded,
        });
    });
}
pub fn finish() -> Option<Vec<u8>> {
    TRACE.with(|s| s.borrow_mut().take().map(|t| t.bytes))
}
/// Assert the CPU-only excluded ranges of `ram` still hold their seed bytes.
pub fn check_excluded(ram: &[u8]) {
    TRACE.with(|s| {
        let t = s.borrow();
        let t = t.as_ref().unwrap();
        for &(start, end) in &t.excluded {
            assert_eq!(
                &ram[start..end],
                &t.seed[start..end],
                "Rust changed CPU-only excluded bytes"
            );
        }
    });
}
pub fn publication(kind: u16, at: usize, before: u16, after: u16) {
    if !active() {
        return;
    }
    let mut p = kind.to_le_bytes().to_vec();
    p.extend_from_slice(&(at as u32).to_le_bytes());
    p.extend_from_slice(&before.to_le_bytes());
    p.extend_from_slice(&after.to_le_bytes());
    append(3, &p);
}
pub fn scalar(kind: u16, value: u16) {
    if !active() {
        return;
    }
    let mut p = kind.to_le_bytes().to_vec();
    p.extend_from_slice(&value.to_le_bytes());
    append(4, &p);
}
pub fn palette_event(kind: &str, argument: u16) {
    if !active() {
        return;
    }
    let code = match kind {
        "copy" => 1,
        "load" => 2,
        "command" => 3,
        "misc" => 4,
        _ => panic!("unexpected palette effect"),
    };
    let mut p = vec![code];
    p.extend_from_slice(&argument.to_le_bytes());
    append(5, &p);
}
pub fn sound(channel: usize, sound: u8, limit: u8) {
    if !active() {
        return;
    }
    append(6, &[u8::try_from(channel).unwrap(), limit, sound]);
}
