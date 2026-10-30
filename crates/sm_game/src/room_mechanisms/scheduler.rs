//! Descending live-slot scheduler with caller-owned message suspension.
#![forbid(unsafe_code)]
use super::{
    access::mechanism,
    commands,
    content::Script,
    creation,
    host::{ContentAddress, Host, Pass, RunResult, StoreOrder},
    pre, presentation,
};
use core::num::NonZeroU8;

fn capture<H: Host>(h: &mut H) -> Script<H::Record> {
    let record = h.loaded_record(ContentAddress {
        bank: 0x84,
        offset: 0x8000,
    });
    let slot = (h.session().mechanisms.selected >> 1) as isize;
    let position = mechanism!(h, slot, program);
    Script::new(record, 0x8000).seek(position)
}

pub fn advance<H: Host>(h: &mut H, pass: &mut Pass) -> RunResult {
    h.probe("coroutine.plm");
    match pass {
        Pass::Invalid(_) => {
            *pass = Pass::Instructions;
            return RunResult::Complete;
        }
        Pass::Instructions => {
            if h.session().mechanisms.flags & 0x8000 == 0 {
                return RunResult::Complete;
            }
            h.session_mut().mechanisms.draw_cursor = 0;
            h.session_mut().mechanisms.selected = 78;
        }
        Pass::DisplayMessage => {}
    }
    let mut script = None;
    loop {
        if matches!(pass, Pass::Instructions) {
            let selected = h.session().mechanisms.selected;
            if selected & 0x8000 != 0 {
                h.session_mut().mechanisms.selected = 0;
                break;
            }
            if mechanism!(h, (selected >> 1) as isize, header) == 0 {
                h.session_mut().mechanisms.selected =
                    h.session().mechanisms.selected.wrapping_sub(2);
                continue;
            }
            let selected = h.session().mechanisms.selected;
            let identity =
                u32::from(mechanism!(h, (selected >> 1) as isize, pre_action)) | 0x840000;
            let selected = h.session().mechanisms.selected;
            pre::run(h, identity, selected);
            let slot = (h.session().mechanisms.selected >> 1) as isize;
            let remaining = mechanism!(h, slot, instruction_delay).wrapping_sub(1);
            mechanism!(h, slot, instruction_delay = remaining);
            if remaining != 0 {
                h.session_mut().mechanisms.selected =
                    h.session().mechanisms.selected.wrapping_sub(2);
                continue;
            }
            script = Some(capture(h));
        }
        loop {
            if matches!(pass, Pass::DisplayMessage) {
                h.probe("coroutine.plm.message");
                let queued = h.session().messages.queued;
                if let Some(yielded) = NonZeroU8::new(h.display_message(queued)) {
                    return RunResult::Yield(yielded);
                }
                *pass = Pass::Instructions;
                h.probe("coroutine.plm.message.completed");
                h.session_mut().messages.queued = 0;
                script = Some(capture(h));
            }
            let current = script.take().expect("active mechanism instruction");
            if current.word(0) & 0x8000 == 0 {
                let slot = (h.session().mechanisms.selected >> 1) as isize;
                mechanism!(h, slot, instruction_delay = current.word(0));
                mechanism!(h, slot, draw_program = current.word(2));
                mechanism!(h, slot, program = current.advance(4).position());
                let selected = h.session().mechanisms.selected;
                presentation::process_draw(h, selected);
                let selected = h.session().mechanisms.selected;
                creation::coordinates(h, usize::from(selected >> 1));
                let selected = h.session().mechanisms.selected;
                presentation::draw(h, selected);
                break;
            }
            let identity = u32::from(current.word(0)) | 0x840000;
            let selected = h.session().mechanisms.selected;
            let Some(next) = commands::run(h, identity, current.advance(2), selected) else {
                break;
            };
            if h.session().messages.queued != 0 {
                *pass = Pass::DisplayMessage;
                let slot = (h.session().mechanisms.selected >> 1) as isize;
                mechanism!(h, slot, program = next.position(), StoreOrder::SliceNative);
            }
            script = Some(next);
        }
        h.session_mut().mechanisms.selected = h.session().mechanisms.selected.wrapping_sub(2);
    }
    *pass = Pass::Instructions;
    RunResult::Complete
}
