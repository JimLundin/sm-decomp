//! Original source locations for Drawing and post-background room owners.

use super::{data_failure, result::Outcome};
use crate::lock::Sha256;
use game::production::frame::{frame_history::History, frame_tail::Current, room_tail::Admission};
pub(super) struct TailInputs {
    pub current: Current,
    pub inactive_debug: Option<bool>,
    pub time: [u16; 4],
    pub shake: [u16; 2],
}

#[derive(serde::Deserialize)]
struct DebugOperand {
    bank: u8,
    operand: u16,
    width: usize,
}

pub(super) fn debug_mask(source: &str, rom: &data::rom::Rom) -> Result<u16, Outcome> {
    let fault = || data_failure(None, "checked final debug shortcut operand");
    let expected =
        Sha256::try_from("764514d6557a91087f11b31fdf6e7362a3bfe06d51516e2d7cf5b935cfa70b3f")
            .map_err(|_| fault())?;
    if Sha256::of(source.as_bytes()) != expected {
        return Err(fault());
    }
    let operand: DebugOperand = toml::from_str(source).map_err(|_| fault())?;
    let site = operand.operand.checked_sub(1).ok_or_else(fault)?;
    let bytes = rom
        .lorom()
        .read(
            data::rom::RomAddress::new(operand.bank, site),
            operand.width.checked_add(1).ok_or_else(fault)?,
        )
        .map_err(|_| fault())?;
    let [0x29, lo, hi] = bytes else {
        return Err(fault());
    };
    Ok(u16::from_le_bytes([*lo, *hi]))
}

pub(super) fn origin(
    wram: &[u8],
    definition: &str,
    rom: &data::rom::Rom,
    debug_mask: Option<u16>,
) -> Result<(History, Admission, TailInputs), Outcome> {
    let fault = || data_failure(None, "qualified original Drawing and room history");
    let pin =
        |bytes: &[u8], hash: &str| Sha256::try_from(hash).is_ok_and(|pin| Sha256::of(bytes) == pin);
    if !pin(
        definition.as_bytes(),
        "bf3a0bd2a27c81b18a573da9812c59f8827163e4faccd5058394277e33b86c0f",
    ) || !pin(
        wram,
        "bfee6ac13da413a36ad42194a36aa3224af6a84e818092b1e7040ed4782f03db",
    ) {
        return Err(fault());
    }
    let fields: toml::Value = toml::from_str(definition).map_err(|_| fault())?;
    let value = |name| {
        let row = fields.get(name)?.as_array()?;
        let [offset, width] = row.as_slice() else {
            return None;
        };
        let offset = usize::try_from(offset.as_integer()?).ok()?;
        let width = usize::try_from(width.as_integer()?).ok()?;
        match wram.get(offset..offset.checked_add(width)?)? {
            [lo] => Some(u16::from(*lo)),
            [lo, hi] => Some(u16::from_le_bytes([*lo, *hi])),
            _ => None,
        }
    };
    let need = |name| value(name).ok_or_else(fault);
    let word = |name| {
        if fields
            .get(name)
            .and_then(toml::Value::as_array)
            .and_then(|member| member.get(1))
            .and_then(toml::Value::as_integer)
            != Some(2)
        {
            return Err(fault());
        }
        need(name)
    };
    let hook = data::rom::RomAddress::new(0x8f, need("room_main")?);
    if rom.lorom().read(hook, 1).map_err(|_| fault())? != [0x60] {
        return Err(fault());
    }
    let id = import::world::id(data::room::RoomKey::from_room_pointer(
        crate::source::ROOM_PTR.read(wram).map_err(|_| fault())?,
    ))
    .ok_or_else(fault)?;
    let history = History::new(
        [need("held")?, need("pressed")?],
        [need("previous_held")?, need("previous_pressed")?],
        need("draw_previous_health")?,
        need("auto_jump_timer")?,
        need("jump_binding")?,
        need("controller_latch")?,
    );
    let room_entry = Admission {
        room: id,
        previous_layer_one: [
            need("previous_layer1_block_x")?,
            need("previous_layer1_block_y")?,
        ],
        external_layer_two: [
            need("layer2_ownership_x")? & 1 != 0,
            need("layer2_ownership_y")? & 1 != 0,
        ],
        departure_inactive: None,
    };
    Ok((
        history,
        room_entry,
        TailInputs {
            current: Current {
                timers: [
                    need("hurt_invincibility")?,
                    need("hurt_knockback")?,
                    need("hurt_projectile")?,
                ],
                cursors: [need("enemy_cursor")?, need("enemy_interaction_cursor")?],
            },
            inactive_debug: match debug_mask {
                None => None,
                Some(mask) => {
                    Some(need("debug_shortcut_edges")? & mask == 0 && need("spare_display")? == 0)
                }
            },
            time: [
                word("play_hours")?,
                word("play_minutes")?,
                word("play_seconds")?,
                word("play_frames")?,
            ],
            shake: [word("shake_timer")?, word("shake_phase")?],
        },
    ))
}

pub(super) fn play_time(words: [u16; 4]) -> Result<types::PlayTime, Outcome> {
    use types::{PlayFrames, PlayHours, PlayMinutes, PlaySeconds, PlayTime};

    let fault = || data_failure(None, "complete legal play time");
    let narrow = |value| u8::try_from(value).map_err(|_| fault());
    let [hours, minutes, seconds, frames] = words;
    Ok(PlayTime {
        hours: PlayHours::new(narrow(hours)?).ok_or_else(fault)?,
        minutes: PlayMinutes::new(narrow(minutes)?).ok_or_else(fault)?,
        seconds: PlaySeconds::new(narrow(seconds)?).ok_or_else(fault)?,
        frames: PlayFrames::new(narrow(frames)?).ok_or_else(fault)?,
    })
}
