//! Independently admitted current HUD display capability, separate from frame assembly.

use super::{Member, Model, Outcome, Sha256, buffer_address, data_failure, frame, profiles};

pub(super) fn origin(
    bytes: &[u8],
    member: &Member,
    hud: &super::super::hud::Content,
    templates: &profiles::Templates,
    buffers: &str,
) -> Result<
    (
        frame::hud_map_caller::Content,
        Model<frame::hud_stage::DisplayReceiptPin>,
    ),
    Outcome,
> {
    let accepted =
        Sha256::try_from("424f58aff2ea37c662cb2e0b44091f0b6ef36d5c2505ea2d1664b80c338827d3")
            .map_err(|_| data_failure(None, "current Display source identity"))?;
    if member.pin()? != accepted {
        return Err(data_failure(None, "current Display source identity"));
    }
    let area = super::super::frame_hud_inputs::current_area(bytes)
        .map_err(|_| data_failure(None, "current Display area"))?;
    let display =
        super::super::frame_hud_inputs::qualified(bytes, &member.text()?, area, hud.map(bytes)?)
            .map_err(|_| data_failure(None, "current Display owners and checked map"))?;
    let key = data::room::RoomKey::from_room_pointer(
        crate::source::ROOM_PTR
            .read(bytes)
            .map_err(|_| data_failure(None, "current Display room"))?,
    );
    let room = import::world::id(key)
        .ok_or_else(|| data_failure(None, "checked Display room identity"))?;
    Ok((
        frame::hud_map_caller::Content {
            physical: super::super::frame_hud_camera_owner::refresh_physical(templates)
                .map_err(|_| data_failure(None, "whole HUD producer and return"))?,
            source: display.clone(),
            room,
            address: buffer_address(buffers, "HudTilemap", 192)?,
        },
        Model(Box::new(display)),
    ))
}
