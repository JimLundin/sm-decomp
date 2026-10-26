//! Candidate verifier events; same assertions and binary traces as predecessor40.
use std::cell::RefCell;

struct Trace {
    id: u32,
    vector: usize,
    spikes: usize,
    touches: usize,
    bytes: Vec<u8>,
}
thread_local! { static TRACE: RefCell<Option<Trace>> = const { RefCell::new(None) }; }
pub(crate) fn begin(id: u32) {
    TRACE.with(|t| {
        assert!(t.borrow().is_none(), "movement observer already armed");
        *t.borrow_mut() = Some(Trace {
            id,
            vector: 0,
            spikes: 0,
            touches: 0,
            bytes: Vec::new(),
        });
    });
}
pub(crate) fn cancel() {
    TRACE.with(|t| {
        t.borrow_mut().take();
    });
}
pub(crate) fn finish() -> Vec<u8> {
    TRACE.with(|t| {
        let t = t.borrow_mut().take().expect("movement observer armed");
        assert_eq!(
            t.vector,
            if t.id < 3 { 4 } else { 0 },
            "exact vector write observations"
        );
        assert_eq!(
            t.spikes,
            usize::from((26..=28).contains(&t.id)),
            "exact spike callbacks"
        );
        assert_eq!(
            t.touches,
            usize::from((37..=38).contains(&t.id)),
            "exact body-stage observations"
        );
        t.bytes
    })
}
/// After each existing vector half-store, borrows only that already borrowed
/// four-byte coordinate window. No second Game/RAM reference is manufactured.
pub(crate) fn vector_store(coordinate: &[u8]) {
    TRACE.with(|t| {
        let mut active = t.borrow_mut();
        let Some(t) = active.as_mut() else {
            return;
        };
        assert!(t.id < 3, "unexpected vector write");
        const EXPECTED: [[[u16; 2]; 4]; 3] = [
            [[0x1234, 0], [0x1235, 0], [0x00ff, 1], [0x0100, 1]],
            [[1, 0xffff], [0, 0xffff], [0, 0xffff], [0xfffe, 0xffff]],
            [[0xfffe, 1], [0, 1], [0x8000, 0], [0x8001, 0]],
        ];
        assert!(t.vector < 4, "too many vector writes");
        let expected = EXPECTED[t.id as usize][t.vector];
        assert_eq!(
            coordinate,
            [expected[0].to_le_bytes(), expected[1].to_le_bytes()].concat(),
            "fraction-before-whole and X-before-Y"
        );
        t.bytes.extend_from_slice(&[1, t.vector as u8]);
        t.bytes.extend_from_slice(coordinate);
        t.vector += 1;
    });
}

pub(crate) fn active_case() -> Option<u32> {
    TRACE.with(|t| t.borrow().as_ref().map(|t| t.id))
}
pub(crate) fn record_snapshot(tag: u8, bytes: Vec<u8>, failed: bool) {
    assert_eq!(bytes.len(), 131072, "exact full RAM observation");
    TRACE.with(|t| {
        if let Some(t) = t.borrow_mut().as_mut() {
            t.bytes.push(tag);
            t.bytes.extend(bytes);
            t.bytes.push(u8::from(failed));
        }
    });
}
pub(crate) fn start_spike(header: u16, block: u16, selected: u16) -> Option<u32> {
    let id = TRACE.with(|t| {
        t.borrow_mut().as_mut().map(|t| {
            assert!((26..=28).contains(&t.id));
            assert_eq!(t.spikes, 0);
            t.spikes += 1;
            t.id
        })
    });
    if id.is_some() {
        assert_eq!(header, 0x9000);
        assert_eq!(block, 264, "collision cursor published before PLM");
        assert_eq!(selected, 0, "source selector at callback entry");
    }
    id
}
pub(crate) fn check_body_stage(stage: u16, map: u16) -> bool {
    let armed = TRACE.with(|t| {
        let mut active = t.borrow_mut();
        let Some(t) = active.as_mut() else {
            return false;
        };
        assert!((37..=38).contains(&t.id), "unexpected body observer");
        t.touches += 1;
        true
    });
    if armed {
        assert_eq!(stage, 9);
        assert_eq!(map, 9, "stage write visible through aliased Map");
    }
    armed
}
