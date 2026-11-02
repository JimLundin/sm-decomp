//! Speed-boost and crash-echo orchestration shared by both output bindings.
#![forbid(unsafe_code)]
use crate::scene::samus_body::Half;
use crate::session::Session;
pub trait Host {
    type Error;
    fn session(&self) -> &Session;
    fn session_mut(&mut self) -> &mut Session;
    fn draw_echo_slot(&mut self, slot: u16) -> Result<(), Self::Error>;
    fn frame_counter(&self) -> u16;
    fn sprite_index(&mut self, half: Half, pose: u16) -> Result<u16, Self::Error>;
    fn height(&mut self) -> Result<u8, Self::Error>;
    fn echo_x(&mut self, slot: u16) -> Result<u16, Self::Error>;
    fn echo_y(&mut self, slot: u16) -> Result<u16, Self::Error>;
    fn camera(&self) -> (u16, u16);
    fn draw_sprite(&mut self, slot: u16, index: u16, x: u16, y: u16) -> Result<(), Self::Error>;
    fn bottom_visible(&mut self) -> Result<bool, Self::Error>;
}
pub fn draw<H: Host>(h: &mut H) -> Result<(), H::Error> {
    let converging = h.session().projectiles.echoes.index & 0x8000 != 0;
    if !converging && h.session().player.contact.speed_boost & 0xff00 != 0x400 {
        return Ok(());
    }
    for index in [1, 0] {
        if h.session().projectiles.echoes.x[index] == 0 {
            continue;
        }
        if converging {
            let s = h.session_mut();
            let e = &mut s.projectiles.echoes;
            if e.y[index] != s.player.y {
                e.y[index] =
                    if i16::from_le_bytes(e.y[index].wrapping_sub(s.player.y).to_le_bytes()) < 0 {
                        e.y[index].wrapping_add(2)
                    } else {
                        e.y[index].wrapping_sub(2)
                    };
            }
            e.x[index] = e.x[index].wrapping_add(e.x_speed[index]);
            let left = i16::from_le_bytes(e.x[index].wrapping_sub(s.player.x).to_le_bytes()) < 0;
            if left == (e.x_speed[index] & 0x8000 != 0) {
                e.x[index] = 0;
                continue;
            }
        }
        h.draw_echo_slot(u16::try_from(index * 2).expect("echo slot"))?;
    }
    if converging && h.session().projectiles.echoes.x[..2] == [0, 0] {
        h.session_mut().projectiles.echoes.index = 0;
    }
    Ok(())
}
pub fn crash<H: Host>(slot: u16, h: &mut H) -> Result<(), H::Error> {
    if h.frame_counter() & 1 == 0 {
        return Ok(());
    }
    let pose = h.session().player.pose;
    let top = h.sprite_index(Half::Top, pose)?;
    let height = h.height()?;
    let y = h
        .echo_y(slot)?
        .wrapping_sub(u16::from(height))
        .wrapping_sub(h.camera().1);
    if !(0..248).contains(&y) {
        return Ok(());
    }
    let x = h.echo_x(slot)?.wrapping_sub(h.camera().0);
    h.draw_sprite(slot, top, x, y)?;
    if h.bottom_visible()? {
        let pose = h.session().player.pose;
        let bottom = h.sprite_index(Half::Bottom, pose)?;
        let x = h.echo_x(slot)?.wrapping_sub(h.camera().0);
        h.draw_sprite(slot, bottom, x, y)?;
    }
    Ok(())
}
