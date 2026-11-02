//! Message canvas operations shared by physical and prepared display consumers.
#![forbid(unsafe_code)]
use crate::message_composition as composition;
use crate::presentation_workspace::{Point, Role, SliceWriter};
use crate::scene::state::DisplayError;

pub fn tile(
    workspace: &mut (impl SliceWriter + ?Sized),
    word: u16,
    value: u16,
) -> Result<(), DisplayError> {
    let offset = composition::destination_byte_offset(word) - 0x200;
    workspace.write_slice(Point::new(Role::MessageText, offset), &value.to_le_bytes())
}
pub fn row(
    workspace: &mut (impl SliceWriter + ?Sized),
    mut word: u16,
    tiles: &[u16],
) -> Result<(), DisplayError> {
    for value in tiles {
        tile(workspace, word, *value)?;
        word = word.wrapping_add(1);
    }
    Ok(())
}
pub fn background(workspace: &mut (impl SliceWriter + ?Sized)) -> Result<(), DisplayError> {
    for offset in (0..0x700).step_by(2) {
        workspace.write_slice(Point::new(Role::Equipment, offset), &14u16.to_le_bytes())?;
    }
    Ok(())
}
pub fn radius(
    workspace: &mut (impl SliceWriter + ?Sized),
    radius: u16,
) -> Result<composition::RadiusValues, DisplayError> {
    let values = composition::radius_values(radius);
    for offset in (0xbc..0xf8).step_by(2) {
        workspace.write_slice(
            Point::new(Role::Pause, offset),
            &values.top_scroll.to_le_bytes(),
        )?;
    }
    for offset in (0xf8..0x134).step_by(2) {
        workspace.write_slice(
            Point::new(Role::Pause, offset),
            &values.bottom_scroll.to_le_bytes(),
        )?;
    }
    workspace.write_slice(Point::new(Role::Pause, 0x134), &[0; 0xac])?;
    Ok(values)
}
