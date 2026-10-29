//! Shared atmospheric slots and ordered protocol, independent of display content.
#![forbid(unsafe_code)]

/// The existing four mutable slots have one owner throughout gameplay and drawing.
#[derive(Clone, Default, Debug, PartialEq)]
pub struct AtmosphericSlot {
    pub remaining: u16,
    pub x: u16,
    pub y: u16,
    pub frame_and_kind: u16,
}
#[derive(Clone, Default, Debug, PartialEq)]
pub struct AtmosphericEffects {
    pub slots: [AtmosphericSlot; 4],
}

/// All original timer, native-content, retirement and drawing effects are ordered.
pub trait AdvanceHost {
    type Error;
    fn frame_and_kind(&mut self, slot: usize) -> Result<u16, Self::Error>;
    fn set_frame_and_kind(&mut self, slot: usize, value: u16) -> Result<(), Self::Error>;
    fn remaining(&mut self, slot: usize) -> Result<u16, Self::Error>;
    fn set_remaining(&mut self, slot: usize, value: u16) -> Result<(), Self::Error>;
    fn x(&mut self, slot: usize) -> Result<u16, Self::Error>;
    fn set_x(&mut self, slot: usize, value: u16) -> Result<(), Self::Error>;
    fn y(&mut self, slot: usize) -> Result<u16, Self::Error>;
    fn set_y(&mut self, slot: usize, value: u16) -> Result<(), Self::Error>;
    fn camera_x(&mut self) -> Result<u16, Self::Error>;
    fn camera_y(&mut self) -> Result<u16, Self::Error>;
    fn fx_y(&mut self) -> Result<u16, Self::Error>;
    fn duration_pointer(&mut self, kind: u16) -> Result<u16, Self::Error>;
    fn duration(&mut self, address: u16) -> Result<u16, Self::Error>;
    fn frame_count(&mut self, kind: u16) -> Result<u16, Self::Error>;
    fn draw_tile(&mut self, slot: usize, kind: u16) -> Result<(), Self::Error>;
    fn draw_sprite(&mut self, slot: usize, sprite: u16, x: u16, y: u16) -> Result<(), Self::Error>;
    fn probe(&mut self, name: &'static str);
}
pub fn advance<H: AdvanceHost>(h: &mut H) -> Result<(), H::Error> {
    h.probe("atmosphere.advance");
    for slot in (0..4).rev() {
        let initial = h.frame_and_kind(slot)?;
        if initial == 0 {
            continue;
        }
        let kind = initial >> 8;
        let frame_offset = 2 * u16::from(initial as u8);
        let remaining = h.remaining(slot)?.wrapping_sub(1);
        h.set_remaining(slot, remaining)?;
        if remaining == 0 || remaining == 0x8000 {
            h.probe("atmosphere.reload");
            let pointer = h.duration_pointer(kind)?;
            let duration = h.duration(pointer.wrapping_add(frame_offset))?;
            h.set_remaining(slot, duration)?;
            if remaining == 0 {
                let next = h.frame_and_kind(slot)?.wrapping_add(1);
                h.set_frame_and_kind(slot, next)?;
                let count = h.frame_count(kind)?;
                if i16::from(next as u8).wrapping_sub(count as i16) >= 0 {
                    h.set_frame_and_kind(slot, 0)?;
                    h.probe("atmosphere.retire");
                    continue;
                }
            }
        } else if remaining & 0x8000 != 0 {
            continue;
        }
        // The original checked/null dispatch occurs after the timer's effects.
        match [0, 1, 1, 3, 4, 5, 6, 6][usize::from(kind)] {
            0 => panic!("non-null function pointer"),
            1 => {
                h.probe("atmosphere.footstep");
                h.draw_tile(slot, kind)?;
            }
            3 | 5 => {
                let frame = u16::from(h.frame_and_kind(slot)? as u8);
                let sprite = if kind == 3 {
                    h.probe("atmosphere.dive");
                    let y = h.fx_y()?;
                    h.set_y(slot, y)?;
                    frame.wrapping_add(399)
                } else {
                    h.probe("atmosphere.bubbles");
                    frame.wrapping_add(390)
                };
                if h.y(slot)?.wrapping_sub(h.camera_y()?) & 0xff00 == 0 {
                    let x = h.x(slot)?.wrapping_sub(h.camera_x()?);
                    let y = h.y(slot)?.wrapping_sub(h.camera_y()?);
                    h.draw_sprite(slot, sprite, x, y)?;
                }
            }
            _ => {
                if kind == 4 {
                    h.probe("atmosphere.lava");
                    let x = h.x(slot)?;
                    h.set_x(
                        slot,
                        if slot & 2 != 0 {
                            x.wrapping_sub(1)
                        } else {
                            x.wrapping_add(1)
                        },
                    )?;
                } else {
                    h.probe("atmosphere.dust");
                }
                let y = h.y(slot)?.wrapping_sub(1);
                h.set_y(slot, y)?;
                h.draw_tile(slot, kind)?;
            }
        }
    }
    Ok(())
}
