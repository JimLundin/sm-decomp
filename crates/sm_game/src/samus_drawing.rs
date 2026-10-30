//! Samus body, echo and shinespark-echo drawing and body tile requests over
//! the Samus authority: the owned display when it draws, otherwise the
//! native body tables captured through the compatibility runtime.
#![forbid(unsafe_code)]
use crate::runtime_access::SessionRam;
use crate::scene::samus_body::{self, Body, Half};
use crate::scene::samus_content::{Registration, Role};
use crate::scene::samus_position_tiles::{self, TileTable};
use crate::session_access::SessionAccess;
use crate::visual_host::{Content, Effects, Source};
use core::convert::Infallible;
use sm_data::cartridge::RomRecord;

/// A Samus role table: prepared bytes or a native capture.
pub trait RoleTable:
    samus_body::Table<Error = Infallible> + samus_position_tiles::Table<Error = Infallible>
{
    fn byte(&self, offset: usize) -> u8;
}

/// Who owns Samus's presentation and role tables, as the owned-Session view
/// sees it.
pub trait Authority: SessionAccess + SessionRam + Content<Record = RomRecord> + Effects {
    type Table: RoleTable;
    fn frame_counter(&self) -> u16;
    fn camera(&self) -> (u16, u16);
    fn has_owned_display(&self) -> bool;
    fn draw_owned_body(&mut self, mode: Body);
    fn draw_owned_echo(&mut self, slot: u16);
    fn owned_bottom_visible(&mut self) -> bool;
    fn draw_echo_samus_sprite(&mut self, slot: u16, index: u16, x: u16, y: u16);
    fn draw_native_echo(&mut self, slot: u16);
    /// The original diagnostic for a Samus source below $8000 in its bank.
    fn samus_source_diagnostic(&mut self, address: u32);
    fn samus_request_written(&mut self, half: Half, pointer: u16);
    /// The prepared role table at `displacement` from the role's base.
    fn prepared_role(
        &self,
        role: Role,
        registration: Registration,
        displacement: usize,
    ) -> Self::Table;
    /// The role table bound over the native `record`.
    fn captured_role(
        &self,
        role: Role,
        registration: Registration,
        record: RomRecord,
    ) -> Self::Table;
    /// The role's mapped base in the prepared Samus content.
    fn role_base(&self, role: Role, registration: Registration) -> usize;
    /// Samus's sprite position for the doubled pose offset.
    fn sprite_position(&mut self, doubled: u16) -> (u16, u16);
    fn bottom_visible(&mut self) -> bool;
}

fn loaded_role<A: Authority>(h: &mut A, role: Role, address: u32) -> A::Table {
    if h.has_owned_display() {
        return h.prepared_role(role, Registration::Loaded, 0);
    }
    let record = h.table(Source::Loaded, address);
    h.captured_role(role, Registration::Loaded, record)
}

struct BodyHost<'a, A>(&'a mut A);
impl<A: Authority> samus_body::Host for BodyHost<'_, A> {
    type Error = Infallible;
    type Table = A::Table;
    fn knockback(&mut self) -> Result<u16, Self::Error> {
        Ok(self.0.session().player.knockback_timer)
    }
    fn invincibility(&mut self) -> Result<u16, Self::Error> {
        Ok(self.0.session().player.invincibility)
    }
    fn shine(&mut self) -> Result<u16, Self::Error> {
        Ok(self.0.session().player.shine_timer)
    }
    fn frame_counter(&mut self) -> Result<u16, Self::Error> {
        Ok(self.0.frame_counter())
    }
    fn pose(&mut self) -> Result<u16, Self::Error> {
        Ok(self.0.session().player.pose)
    }
    fn animation_frame(&mut self) -> Result<u16, Self::Error> {
        Ok(self.0.session().player.animation_clock.frame)
    }
    fn table(&mut self, half: Half) -> Result<A::Table, Self::Error> {
        let (role, address) = match half {
            Half::Top => (Role::BodyTop, 0x929263),
            Half::Bottom => (Role::BodyBottom, 0x92945d),
        };
        Ok(loaded_role(self.0, role, address))
    }
    fn set_sprite(&mut self, half: Half, index: u16) -> Result<(), Self::Error> {
        match half {
            Half::Top => self.0.session_mut().player.top_sprite = index,
            Half::Bottom => self.0.session_mut().player.bottom_sprite = index,
        }
        Ok(())
    }
    fn sprite_position(&mut self, doubled: u16) -> Result<(u16, u16), Self::Error> {
        Ok(self.0.sprite_position(doubled))
    }
    fn body_x(&mut self) -> Result<u16, Self::Error> {
        Ok(self.0.word(0xb04))
    }
    fn body_y(&mut self) -> Result<u16, Self::Error> {
        Ok(self.0.word(0xb06))
    }
    fn camera_x(&mut self) -> Result<u16, Self::Error> {
        Ok(self.0.camera().0)
    }
    fn camera_y(&mut self) -> Result<u16, Self::Error> {
        Ok(self.0.camera().1)
    }
    fn draw_sprite(&mut self, index: u16, x: u16, y: u16) -> Result<(), Self::Error> {
        self.0.draw_samus_sprite(index, x, y);
        Ok(())
    }
    fn bottom_visible(&mut self) -> Result<bool, Self::Error> {
        Ok(self.0.bottom_visible())
    }
    fn queue_tiles(&mut self) -> Result<(), Self::Error> {
        queue_body_tiles(self.0);
        Ok(())
    }
}

/// Draw Samus's body in `mode`.
pub fn draw_body<A: Authority>(h: &mut A, mode: Body) {
    if h.has_owned_display() {
        h.draw_owned_body(mode);
    } else {
        let Ok(()) = samus_body::draw(&mut BodyHost(h), mode);
    }
}

fn sprite_index<A: Authority>(h: &mut A, half: Half, pose: u16, little_endian: bool) -> u16 {
    let Ok(index) = samus_body::sprite_index(&mut BodyHost(h), half, pose, little_endian);
    index
}

/// Draw the speed-boost echoes, converging them first while they return.
pub fn draw_echoes<A: Authority>(h: &mut A) {
    let converging = h.session().projectiles.echoes.index & 0x8000 != 0;
    if !converging && h.session().player.contact.speed_boost & 0xff00 != 0x400 {
        return;
    }
    for index in [1, 0] {
        if h.session().projectiles.echoes.x[index] == 0 {
            continue;
        }
        if converging {
            let s = h.session_mut();
            let e = &mut s.projectiles.echoes;
            if e.y[index] != s.player.y {
                e.y[index] = if (e.y[index].wrapping_sub(s.player.y) as i16) < 0 {
                    e.y[index].wrapping_add(2)
                } else {
                    e.y[index].wrapping_sub(2)
                };
            }
            e.x[index] = e.x[index].wrapping_add(e.x_speed[index]);
            let left = (e.x[index].wrapping_sub(s.player.x) as i16) < 0;
            if left == (e.x_speed[index] & 0x8000 != 0) {
                e.x[index] = 0;
                continue;
            }
        }
        if h.has_owned_display() {
            h.draw_owned_echo((index * 2) as u16);
        } else {
            h.draw_native_echo((index * 2) as u16);
        }
    }
    if converging && h.session().projectiles.echoes.x[..2] == [0, 0] {
        h.session_mut().projectiles.echoes.index = 0;
    }
}

/// Draw the shinespark crash echo in slot `k` on odd frames.
pub fn draw_shinespark_crash_echoes<A: Authority>(k: u16, h: &mut A) {
    if h.frame_counter() & 1 == 0 {
        return;
    }
    let pose = h.session().player.pose;
    let top = sprite_index(h, Half::Top, pose, false);
    let table = if h.has_owned_display() {
        h.prepared_role(Role::PositionFallback, Registration::Native, 0)
    } else {
        let record = h.table(Source::Native, 0x91b629);
        h.captured_role(Role::PositionFallback, Registration::Native, record)
    };
    let height = table.byte(usize::from(h.session().player.pose) * 8 + 4);
    let y = h
        .word(0xab8 + usize::from(k & !1))
        .wrapping_sub(u16::from(height))
        .wrapping_sub(h.camera().1);
    if !(0..248).contains(&y) {
        return;
    }
    let x = h
        .word(0xab0 + usize::from(k & !1))
        .wrapping_sub(h.camera().0);
    h.draw_echo_samus_sprite(k, top, x, y);
    if if h.has_owned_display() {
        h.owned_bottom_visible()
    } else {
        h.bottom_visible()
    } {
        let pose = h.session().player.pose;
        let bottom = sprite_index(h, Half::Bottom, pose, false);
        let x = h
            .word(0xab0 + usize::from(k & !1))
            .wrapping_sub(h.camera().0);
        h.draw_echo_samus_sprite(k, bottom, x, y);
    }
}

fn queue_body_tiles<A: Authority>(h: &mut A) {
    let Ok(()) = samus_position_tiles::queue_body_tiles(&mut TileHost(h));
}

struct TileHost<'a, A>(&'a mut A);
impl<A: Authority> samus_position_tiles::TileHost for TileHost<'_, A> {
    type Error = Infallible;
    type Table = A::Table;
    fn table(&mut self, which: TileTable) -> Result<A::Table, Self::Error> {
        let (role, address) = match which {
            TileTable::Pose => (Role::TilePose, 0x92d94e),
            TileTable::Top => (Role::TileTop, 0x92d91e),
            TileTable::Bottom => (Role::TileBottom, 0x92d938),
        };
        Ok(loaded_role(self.0, role, address))
    }
    fn pose(&mut self) -> Result<u16, Self::Error> {
        Ok(self.0.session().player.pose)
    }
    fn frame(&mut self) -> Result<u16, Self::Error> {
        Ok(self.0.session().player.animation_clock.frame)
    }
    fn definition(&mut self, cursor: u16) -> Result<A::Table, Self::Error> {
        if self.0.has_owned_display() {
            if cursor & 0x8000 == 0 {
                self.0.samus_source_diagnostic(0x920000 | u32::from(cursor));
            }
            let mapped = sm_data::cartridge::lorom_offset(0x920000 | u32::from(cursor)) as usize;
            let base = self.0.role_base(Role::TileDefinition, Registration::Native);
            return Ok(self.0.prepared_role(
                Role::TileDefinition,
                Registration::Native,
                mapped
                    .checked_sub(base)
                    .expect("Samus definition mapped base"),
            ));
        }
        let record = self.0.legacy_native_record(0x92, cursor);
        Ok(self
            .0
            .captured_role(Role::TileDefinition, Registration::Native, record))
    }
    fn request_word(&mut self, half: Half, pointer: u16) -> Result<(), Self::Error> {
        self.0.write_word_le(
            pointer,
            match half {
                Half::Top => 0x71f,
                Half::Bottom => 0x721,
            },
        );
        Ok(())
    }
    fn request_flag(&mut self, half: Half) -> Result<(), Self::Error> {
        self.0.set_byte(
            match half {
                Half::Top => 0x71d,
                Half::Bottom => 0x71e,
            },
            1,
        );
        Ok(())
    }
    fn request_written(&mut self, half: Half, pointer: u16) -> Result<(), Self::Error> {
        self.0.samus_request_written(half, pointer);
        Ok(())
    }
}
