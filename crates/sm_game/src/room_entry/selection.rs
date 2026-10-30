//! Arrival decisions and ordered selectors over the shared room owner.
use super::{
    api::{ArrivalRequest, ContactSite, EntryPoll, StartSite},
    content::{HeaderScalar, SelectorCursor, SelectorKind, SelectorLayout, StationScalar},
    effects::{CameraField, EntryHost, ExternalField},
    world::{Identity, RoomId},
};
use crate::progress_flags;

pub fn begin(h: &mut impl EntryHost, arrival: ArrivalRequest) -> EntryPoll {
    match arrival {
        ArrivalRequest::Contact(site) => contact(h, site),
        ArrivalRequest::Start(site) => start(h, site),
    }
}

fn contact(h: &mut impl EntryHost, site: ContactSite) -> EntryPoll {
    h.session_mut().room.entry.transition_selector = 0xe17d;
    let exit = match site {
        ContactSite::CurrentCollision => {
            let bts = h.contact_bts();
            let doors = h.session().room.entry.doors.clone();
            h.contact_exit(&doors, bts)
        }
        ContactSite::Authored(exit) => Identity::Known(exit),
    };
    if !h.contact_has_destination(&exit) {
        if h.session().player.pose < 9 {
            h.session_mut().room.entry.elevator_flags = 1;
        }
        EntryPoll::Rejected
    } else {
        h.session_mut().room.entry.exit = exit;
        h.write_external(ExternalField::GameMode, 9);
        EntryPoll::Returned(0)
    }
}

fn start(h: &mut impl EntryHost, site: StartSite) -> EntryPoll {
    h.session_mut().mechanisms.save_locked = 1;
    let station = h.capture_station(site);
    h.session_mut().room.entry.room = station.room();
    h.session_mut().room.entry.exit = station.exit();
    h.write_camera(CameraField::Layer1X, station.scalar(StationScalar::CameraX));
    let x = h.read_camera(CameraField::Layer1X);
    h.write_camera(CameraField::Bg1XOffset, x);
    h.write_camera(CameraField::Layer1Y, station.scalar(StationScalar::CameraY));
    let y = h.read_camera(CameraField::Layer1Y);
    h.write_camera(CameraField::Bg1YOffset, y);
    let y = h.read_camera(CameraField::Layer1Y);
    h.session_mut().player.y = y.wrapping_add(station.scalar(StationScalar::SamusYOffset));
    h.write_external(ExternalField::PreviousPlayerY, h.session().player.y);
    let x = h.read_camera(CameraField::Layer1X);
    h.session_mut().player.x = x
        .wrapping_add(128)
        .wrapping_add(station.scalar(StationScalar::SamusXOffset));
    h.write_external(ExternalField::PreviousPlayerX, h.session().player.x);
    h.write_camera(CameraField::Bg1Hofs, 0);
    h.write_camera(CameraField::Bg1Vofs, 0);
    let room = h.session().room.entry.room;
    let header = h.capture_start_header(&room);
    let area = header.scalar(HeaderScalar::Area) as u8;
    h.session_mut().room.area = (h.session().room.area & 0xff00) | u16::from(area);
    h.write_external(ExternalField::LoadingLowByte, 0);
    EntryPoll::Returned(0)
}

pub fn select(h: &mut impl EntryHost, room: Identity<RoomId>) {
    let mut cursor = SelectorCursor::for_room(&room, &h.session().world);
    loop {
        let (kind, operand) = h.selector(&cursor);
        match select_entry(h, kind, operand) {
            Some(next) => cursor = next,
            None => return,
        }
    }
}

/// None means the selector stopped, independently of the selected state's key.
pub fn select_entry(
    h: &mut impl EntryHost,
    kind: SelectorKind,
    operand: SelectorCursor,
) -> Option<SelectorCursor> {
    let state = match kind {
        SelectorKind::Default => operand.inline_state(&h.session().world),
        SelectorKind::Event | SelectorKind::Boss => {
            let record = h.capture_operand(&operand);
            let condition = u16::from(record.condition_id());
            let selected = if kind == SelectorKind::Event {
                progress_flags::event_happened(h, condition)
            } else {
                progress_flags::boss_defeated(h, condition)
            };
            if !selected {
                return operand.next(SelectorLayout::ConditionAndTarget, &h.session().world);
            }
            record.target()
        }
        SelectorKind::BossOne | SelectorKind::MorphAndMissiles | SelectorKind::PowerBombs => {
            let selected = match kind {
                SelectorKind::BossOne => progress_flags::boss_defeated(h, 1),
                SelectorKind::MorphAndMissiles => {
                    h.session().inventory.equipment.owned & 4 != 0
                        && h.session().inventory.ammunition[0].maximum != 0
                }
                _ => h.session().inventory.ammunition[2].maximum != 0,
            };
            if !selected {
                return operand.next(SelectorLayout::TargetOnly, &h.session().world);
            }
            h.direct_target(&operand)
        }
        SelectorKind::Unknown(address) => return h.invalid_selector(address),
    };
    h.session_mut().room.entry.state = state;
    None
}
