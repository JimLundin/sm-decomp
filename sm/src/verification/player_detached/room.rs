//! Includes the actual candidate room command and plant modules.
pub(crate) use sm_game::room_mechanisms::commands;
pub(crate) use sm_game::room_mechanisms::content;
pub(crate) use sm_game::room_mechanisms::creation;
pub(crate) use sm_game::room_mechanisms::host;
#[path = "room_rules.rs"]
pub(crate) mod rules;
pub(crate) use rules::treadmill::{Direction, carry};
pub(crate) use sm_game::room_mechanisms::scheduler;
pub(crate) fn command(h: &mut impl host::Host, id: u32, cursor: u16, handle: u16) {
    let record = h.loaded_record(host::ContentAddress {
        bank: 0x84,
        offset: 0x8000,
    });
    let script = content::Script::new(record, 0x8000).seek(cursor);
    let _ = commands::run(h, id, script, handle);
}
pub(crate) fn hold(h: &mut impl host::Host, slot: usize) {
    rules::plants::hold(h, slot);
}

pub(crate) fn advance(h: &mut impl host::Host, pass: &mut host::Pass) -> host::RunResult {
    scheduler::advance(h, pass)
}

pub(crate) fn flag_command(
    h: &mut impl host::Host,
    id: u32,
    cursor: u16,
    handle: u16,
) -> Option<u16> {
    let record = h.loaded_record(host::ContentAddress {
        bank: 0x84,
        offset: 0x8000,
    });
    let script = content::Script::new(record, 0x8000).seek(cursor);
    commands::run(h, id, script, handle).map(|next| next.position())
}
