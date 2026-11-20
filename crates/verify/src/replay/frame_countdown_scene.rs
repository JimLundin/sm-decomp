//! Strict edge decoding of the independently qualified countdown scene scheduling slot.

use game::production::frame::countdown_scene::{Admission, Choice, Current};
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(transparent)]
struct Flag(bool);

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Projection {
    format: String,
    authority: String,
    scene_choice: String,
    private_wram_offset: usize,
    quantity_width: usize,
    private_maintenance_encoding: u16,
    private_presentation_encoding: u16,
    origin_index: usize,
    selection_index: usize,
    revalidation_index: usize,
    read_indices: Vec<usize>,
    executing_read_bytes: usize,
    physical_callback_reads: usize,
    preceding_writer_count: usize,
    intervening_writer_count: usize,
    irq_writer_count: usize,
    origin_equal_at_both_uses: Flag,
    coarse_gameplay_dispatch_is_authority: Flag,
    countdown_result_is_authority: Flag,
    later_maintenance_dto_is_authority: Flag,
    scope: String,
}

/// Invalid projection, incomplete origin or unsupported native scheduling class.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fault {
    /// Descriptor or source ownership is incompatible with the reviewed projection.
    Projection,
    /// The initial complete source image or its scalar is missing.
    Length,
    /// The native value has no approved semantic decoder class.
    Choice,
}

const OFFSET: usize = 2650;
const MAINTAIN: u16 = 57574;
const PRESENT: u16 = 57620;

/// Decode only the separate actual initial scene slot through the pinned descriptor.
/// No later result, coarse scene field or expected endpoint supplies this input.
/// # Errors
/// Wrong descriptor/ownership, truncated origin or an unknown native class.
pub fn current(bytes: &[u8], text: &str) -> Result<(Admission, Current), Fault> {
    let p: Projection = toml::from_str(text).map_err(|_| Fault::Projection)?;
    if p.format != "bounded-current-countdown-scene-selection-v1"
        || p.authority != "ActualCurrentCountdownSceneControllerOwner"
        || p.scene_choice != "MaintainCountdown"
        || p.private_wram_offset != OFFSET
        || p.quantity_width != 2
        || p.private_maintenance_encoding != MAINTAIN
        || p.private_presentation_encoding != PRESENT
        || p.origin_index != 56
        || p.selection_index != 3561
        || p.revalidation_index != 4277
        || p.read_indices != [3561, 4277]
        || p.executing_read_bytes != 4
        || p.physical_callback_reads != 8
        || p.preceding_writer_count != 0
        || p.intervening_writer_count != 0
        || p.irq_writer_count != 0
        || !p.origin_equal_at_both_uses.0
        || p.coarse_gameplay_dispatch_is_authority.0
        || p.countdown_result_is_authority.0
        || p.later_maintenance_dto_is_authority.0
        || p.scope != "borrowed-unselected-scene-scheduling-only"
    {
        return Err(Fault::Projection);
    }
    if bytes.len() != 131_072 {
        return Err(Fault::Length);
    }
    let raw = bytes
        .get(OFFSET..OFFSET.saturating_add(2))
        .ok_or(Fault::Length)?;
    let raw = u16::from_le_bytes(raw.try_into().map_err(|_| Fault::Length)?);
    let choice = match raw {
        MAINTAIN => Choice::MaintainCountdown,
        PRESENT => Choice::PresentCountdownOnly,
        _ => return Err(Fault::Choice),
    };
    Ok((Admission { choice }, Current::from_origin(choice)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "requires accepted projection and complete current-origin member"]
    fn actual_scene_decoder_is_strict_and_independent_of_coarse_slot() {
        let projection =
            std::fs::read_to_string(std::env::var_os("SM_SCENE_PROJECTION").unwrap()).unwrap();
        let original = std::fs::read(std::env::var_os("SM_FRAME_CURRENT_ORIGIN").unwrap()).unwrap();
        let (admitted, _) = current(&original, &projection).unwrap();
        assert_eq!(admitted.choice, Choice::MaintainCountdown);
        assert!(current(&[], &projection).is_err());
        let mut value = original.clone();
        value
            .get_mut(OFFSET..OFFSET.saturating_add(2))
            .unwrap()
            .copy_from_slice(&PRESENT.to_le_bytes());
        let (admitted, owner) = current(&value, &projection).unwrap();
        assert_eq!(admitted.choice, Choice::PresentCountdownOnly);
        assert!(owner.offer(admitted).is_err());
        let unknown = [0_u16, 1, u16::MAX]
            .into_iter()
            .find(|x| *x != PRESENT && *x != MAINTAIN)
            .unwrap();
        value
            .get_mut(OFFSET..OFFSET.saturating_add(2))
            .unwrap()
            .copy_from_slice(&unknown.to_le_bytes());
        assert_eq!(current(&value, &projection), Err(Fault::Choice));
        let mut descriptor: toml::Value = toml::from_str(&projection).unwrap();
        descriptor.as_table_mut().unwrap().insert(
            "coarse_gameplay_dispatch_is_authority".to_owned(),
            toml::Value::Boolean(true),
        );
        assert_eq!(
            current(&original, &toml::to_string(&descriptor).unwrap()),
            Err(Fault::Projection)
        );
    }
}
