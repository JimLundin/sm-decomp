//! Candidate observations, with no synthesized CPU registers.
use crate::game_state::Game;
use std::cell::RefCell;
thread_local! { static ACTIVE: RefCell<Option<Vec<u8>>> = const { RefCell::new(None) }; }
pub(crate) fn active() -> bool {
    ACTIVE.with(|a| a.borrow().is_some())
}
fn record(out: &mut Vec<u8>, tag: u8, bytes: &[u8]) {
    out.push(tag);
    out.extend_from_slice(&(bytes.len() as u32).to_le_bytes());
    out.extend_from_slice(bytes);
}
pub(crate) fn event(kind: u8, handle: u16, value: u16, ram: &[u8]) {
    ACTIVE.with(|a| {
        if let Some(out) = a.borrow_mut().as_mut() {
            let mut bytes = handle.to_le_bytes().to_vec();
            bytes.extend_from_slice(&value.to_le_bytes());
            bytes.extend_from_slice(ram);
            record(out, kind, &bytes);
        }
    });
}
fn entry(id: u32, oracle: &[u8]) -> Result<&[u8], String> {
    if id > 4 || !oracle.starts_with(b"DSOR1") {
        return Err("invalid oracle header/id".into());
    }
    let mut p = 5;
    let mut found_id = false;
    let mut ram = None;
    while p < oracle.len() {
        let header = oracle.get(p..p + 5).ok_or("truncated oracle record")?;
        let tag = header[0];
        let n = u32::from_le_bytes(header[1..5].try_into().unwrap()) as usize;
        p += 5;
        let payload = oracle
            .get(p..p.checked_add(n).ok_or("oracle length overflow")?)
            .ok_or("truncated oracle payload")?;
        p += n;
        if tag == 0 {
            if found_id || payload != id.to_le_bytes() {
                return Err("oracle id mismatch".into());
            }
            found_id = true;
        }
        if tag == 1 {
            if ram.is_some()
                || payload.len() != 16 + 0x20000
                || payload[..4] != 0xa5a13eu32.to_le_bytes()
            {
                return Err("invalid oracle entry".into());
            }
            ram = Some(&payload[16..]);
        }
    }
    if !found_id {
        return Err("missing oracle id".into());
    }
    ram.ok_or("missing oracle entry".into())
}
/// Fresh initialized DLL process required; consumes existing actual CPU entry RAM.
pub fn capture(
    id: u32,
    oracle: &[u8],
    original: &[u8],
    game: &mut Game,
) -> Result<Vec<u8>, String> {
    let initial = entry(id, oracle)?;
    let loaded = game.loaded_rom.as_ref().ok_or("missing loaded ROM")?;
    let native = game.capture_native_rom();
    for (address, len) in [(0xa5a19fu32, 8usize), (0xa5ce07, 0x31f9)] {
        let start = crate::rtl::lorom_offset(address) as usize;
        let expected = original
            .get(start..start + len)
            .ok_or("short original ROM")?;
        for (i, byte) in expected.iter().enumerate() {
            if loaded.read(start + i) != *byte || native.read(start + i) != *byte {
                return Err("candidate ROM provenance mismatch".into());
            }
        }
    }
    if active() {
        return Err("nested Draygon observer".into());
    }
    let previous = game.g_ram.to_vec();
    let previous_fail = game.g_fail;
    game.g_ram.copy_from_slice(initial);
    game.g_fail = false;
    let mut out = b"DSCR1".to_vec();
    let mut payload = id.to_le_bytes().to_vec();
    payload.extend_from_slice(initial);
    record(&mut out, 0, &payload);
    ACTIVE.with(|a| *a.borrow_mut() = Some(out));
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        crate::game::sprite_effect_runtime::draygon_trajectory(game);
        event(7, 0, 0, &game.g_ram);
        if game.g_fail {
            return Err("candidate raised g_fail".to_owned());
        }
        Ok(())
    }));
    let bytes = ACTIVE.with(|a| a.borrow_mut().take().unwrap());
    game.g_ram.copy_from_slice(&previous);
    game.g_fail = previous_fail;
    result.map_err(|_| "candidate panicked".to_owned())??;
    Ok(bytes)
}
