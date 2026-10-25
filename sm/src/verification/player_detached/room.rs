//! Includes the actual candidate room command and plant modules.
#[path = "../../game/room_mechanisms/access.rs"]
pub(crate) mod access;
#[path = "../../game/room_mechanisms/commands.rs"]
pub(crate) mod commands;
#[path = "../../game/room_mechanisms/content.rs"]
pub(crate) mod content;
#[path = "../../game/room_mechanisms/creation.rs"]
pub(crate) mod creation;
#[path = "../../game/room_mechanisms/graphics.rs"]
pub(crate) mod graphics;
#[path = "../../game/room_mechanisms/host.rs"]
pub(crate) mod host;
#[path = "../../game/room_mechanisms/pre.rs"]
pub(crate) mod pre;
#[path = "../../game/room_mechanisms/presentation.rs"]
pub(crate) mod presentation;
#[path = "room_rules.rs"]
pub(crate) mod rules;
pub(crate) use rules::treadmill::{Direction, carry};
#[path = "../../game/room_mechanisms/scheduler.rs"]
pub(crate) mod scheduler;
#[path = "../../game/room_mechanisms/setup.rs"]
pub(crate) mod setup;
pub(crate) fn command(h: &mut impl host::Host, id: u32, cursor: u16, handle: u16) {
    use host::Content;
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
