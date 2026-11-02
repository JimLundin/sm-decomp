//! Nonvisual setup executes over canonical room/progress state and synchronous effects.
use crate::session_access::SessionAccess;
pub trait Host: SessionAccess {
    fn setup_mechanism(&mut self, x: u8, y: u8, definition: u16);
    fn setup_room_projectile(&mut self, definition: u16, parameter: u16);
    fn setup_event(&mut self, event: u16);
}
fn shake<H: Host + ?Sized>(h: &mut H, kind: u16) {
    h.session_mut().feedback.earthquake_kind = kind;
    h.session_mut().feedback.earthquake_timer = u16::MAX;
}
/// False means the same captured action belongs to the existing FX/map executor.
/// Callers execute that remaining action once; it is never an Unsupported result.
pub fn execute<H: Host + ?Sized>(script: u16, h: &mut H) -> bool {
    match script {
        0x9194 => {
            h.setup_mechanism(0x3d, 0xb, 0xbb30);
            shake(h, 24);
        }
        0x91a9 => h.setup_mechanism(0x10, 0x87, 0xb964),
        0x91b2 => h.setup_mechanism(0xf, 0xa, 0xb9ed),
        0xc8c8 => h.setup_room_projectile(0xa3b0, 0xc8c8),
        0xc8d3 => h.setup_mechanism(0, 0, 0xb8eb),
        0xc91f => {
            h.setup_event(0xe);
            shake(h, 18);
        }
        0xc933 | 0xc953 => {
            let kind = if script == 0xc953 {
                h.setup_mechanism(0x10, 0x10, 0xb968);
                21
            } else {
                18
            };
            h.session_mut().feedback.earthquake_kind = kind;
            h.session_mut().room.set_main_native(1, kind);
            h.session_mut().room.set_main_native(0, 0);
            h.session_mut().feedback.earthquake_timer = u16::MAX;
        }
        0xc946 => shake(h, 21),
        0x91bb | 0x91bc | 0x91d3 | 0x91d4 | 0x91d5 | 0x91d6 | 0x91f4 | 0x91f5 | 0x91f6 | 0x91f7
        | 0xc8c7 | 0xc8d0 | 0xc8d1 | 0xc8d2 | 0xc8dc | 0xc91e => {}
        _ => return false,
    }
    true
}
