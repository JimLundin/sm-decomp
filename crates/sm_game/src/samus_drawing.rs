//! Samus body, echo and shinespark-echo drawing and body tile requests over
//! the Samus authority: the owned display when it draws, otherwise the
//! native body tables captured through the compatibility runtime.
use crate::runtime_access::SessionRam;
use crate::scene::objects::Host as _;
use crate::scene::samus_body::{self, Body, Half};
use crate::scene::samus_content::{CapturedBytes, PreparedBytes, Registration, Role, RoleLease};
use crate::scene::samus_native_content::CapturedRom;
use crate::scene::samus_position_tiles::{self, TileTable};
use crate::session_access::SessionAccess;
use crate::sprite_drawing::SessionObjects;
use crate::visual_host::{Content, Effects, Record as _, Source};
use core::convert::Infallible;
use sm_data::cartridge::RomRecord;

/// The role lease a Samus table reads through once Samus content is bound.
pub enum BoundTable {
    Native(RoleLease<CapturedRom>),
    Prepared(RoleLease<PreparedBytes>),
}
/// A Samus role table: a native ROM record, read through its bound lease when
/// Samus content is bound, or prepared bytes alone.
pub struct RoleTable {
    record: Option<RomRecord>,
    bound: Option<BoundTable>,
}
impl RoleTable {
    pub fn captured(record: RomRecord, bound: Option<BoundTable>) -> Self {
        Self {
            record: Some(record),
            bound,
        }
    }
    pub fn prepared(lease: RoleLease<PreparedBytes>) -> Self {
        Self {
            record: None,
            bound: Some(BoundTable::Prepared(lease)),
        }
    }
    fn record(&self) -> &RomRecord {
        self.record.as_ref().expect("native Samus record")
    }
    pub fn byte(&self, offset: usize) -> u8 {
        match &self.bound {
            None => self.record().byte(offset),
            Some(BoundTable::Native(table)) => table.byte(offset).unwrap(),
            Some(BoundTable::Prepared(table)) => table.byte(offset).unwrap(),
        }
    }
    pub fn word(&self, offset: usize, little: bool) -> u16 {
        match &self.bound {
            None if little => u16::from_le_bytes(self.record().bytes(offset)),
            None => self.record().native_word(offset),
            Some(BoundTable::Native(table)) => lease_word(table, offset, little),
            Some(BoundTable::Prepared(table)) => lease_word(table, offset, little),
        }
    }
}
fn lease_word<B: CapturedBytes>(table: &RoleLease<B>, offset: usize, little: bool) -> u16 {
    if little {
        table.little_word(offset)
    } else {
        table.native_word(offset)
    }
    .unwrap()
}
impl samus_body::Table for RoleTable {
    type Error = Infallible;
    fn little_word(&self, offset: usize) -> Result<u16, Infallible> {
        Ok(self.word(offset, true))
    }
    fn native_word(&self, offset: usize) -> Result<u16, Infallible> {
        Ok(self.word(offset, false))
    }
}
impl samus_position_tiles::Table for RoleTable {
    type Error = Infallible;
    fn native_word(&self, offset: usize) -> Result<u16, Infallible> {
        Ok(self.word(offset, false))
    }
    fn byte(&self, offset: usize) -> Result<u8, Infallible> {
        Ok(RoleTable::byte(self, offset))
    }
}

/// Who owns Samus's presentation and role tables, as the owned-Session view
/// sees it.
pub trait Authority: SessionAccess + SessionRam + Content<Record = RomRecord> + Effects {
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
    ) -> RoleTable;
    /// The role table bound over the native `record`.
    fn captured_role(&self, role: Role, registration: Registration, record: RomRecord)
    -> RoleTable;
    /// The role's mapped base in the prepared Samus content.
    fn role_base(&self, role: Role, registration: Registration) -> usize;
    /// The prepared native position-offset table for `role`, when the
    /// prepared content supplies it.
    fn native_offsets(&self, role: Role) -> Option<RoleTable>;
}

/// The bottom-half and sprite-position protocols on the lease's Session,
/// with the OAM staging as the session view projects it.
struct LeaseBody<'a, A>(&'a mut A);
impl<A: Authority> samus_body::BottomHost for LeaseBody<'_, A> {
    type Error = Infallible;
    fn pose(&mut self) -> Result<u16, Infallible> {
        Ok(self.0.session().player.pose)
    }
    fn animation_frame(&mut self) -> Result<u16, Infallible> {
        Ok(self.0.session().player.animation_clock.frame)
    }
    fn movement(&mut self) -> Result<u8, Infallible> {
        Ok(self.0.session().player.movement_and_facing.to_le_bytes()[1])
    }
    fn clear_bottom(&mut self) -> Result<(), Infallible> {
        self.0.session_mut().player.bottom_sprite = 0;
        Ok(())
    }
    fn cursor(&mut self) -> Result<u16, Infallible> {
        SessionObjects(&mut *self.0).read_little_word(544)
    }
    fn player_x(&mut self) -> Result<u16, Infallible> {
        Ok(self.0.session().player.x)
    }
    fn player_y(&mut self) -> Result<u16, Infallible> {
        Ok(self.0.session().player.y)
    }
    fn camera_x(&mut self) -> Result<u16, Infallible> {
        Ok(self.0.session().room.camera.layer1[0])
    }
    fn camera_y(&mut self) -> Result<u16, Infallible> {
        Ok(self.0.session().room.camera.layer1[1])
    }
    fn marker_word(&mut self, entry: usize, value: u16) -> Result<(), Infallible> {
        SessionObjects(&mut *self.0).write_native_bytes(entry, value)
    }
    fn publish_cursor(&mut self, value: u16) -> Result<(), Infallible> {
        let [low, high] = value.to_le_bytes();
        SessionObjects(&mut *self.0).write_byte(544, low)?;
        SessionObjects(&mut *self.0).write_byte(545, high)
    }
    fn marker_written(&mut self, _: usize, _: u16, _: u16) -> Result<(), Infallible> {
        Ok(())
    }
}
impl<A: Authority> samus_position_tiles::PositionHost for LeaseBody<'_, A> {
    type Error = Infallible;
    fn ceres(&mut self) -> Result<u16, Infallible> {
        Ok(self.0.session().room.ceres_status)
    }
    fn player_x(&mut self) -> Result<u16, Infallible> {
        Ok(self.0.session().player.x)
    }
    fn player_y(&mut self) -> Result<u16, Infallible> {
        Ok(self.0.session().player.y)
    }
    fn set_player_x(&mut self, value: u16) -> Result<(), Infallible> {
        self.0.session_mut().player.x = value;
        Ok(())
    }
    fn set_player_y(&mut self, value: u16) -> Result<(), Infallible> {
        self.0.session_mut().player.y = value;
        Ok(())
    }
    fn project(&mut self, x: u16, y: u16) -> Result<(u16, u16), Infallible> {
        let point = crate::cinematic::project_cinematic_point_reads(
            crate::geometry::Point { x, y },
            |at| self.0.word(at),
        );
        Ok((point.x, point.y))
    }
    fn frame(&mut self) -> Result<u16, Infallible> {
        Ok(self.0.session().player.animation_clock.frame)
    }
    fn movement(&mut self) -> Result<u8, Infallible> {
        samus_body::BottomHost::movement(self)
    }
    fn landing_word(&self, bytes: [u8; 2]) -> u16 {
        u16::from_ne_bytes(bytes)
    }
    fn landing_offset(&mut self, offset: usize) -> Result<u16, Infallible> {
        Ok(match self.0.native_offsets(Role::Landing) {
            Some(table) => table.word(offset, false),
            None => self.landing_word(samus_position_tiles::default_landing_bytes(offset)),
        })
    }
    fn crouching_offset(&mut self, offset: usize) -> Result<i8, Infallible> {
        Ok(match self.0.native_offsets(Role::Crouching) {
            Some(table) => table.byte(offset) as i8,
            None => samus_position_tiles::default_crouching_byte(offset),
        })
    }
    fn drained_offset(&mut self, offset: usize) -> Result<i8, Infallible> {
        Ok(match self.0.native_offsets(Role::Drained) {
            Some(table) => table.byte(offset) as i8,
            None => samus_position_tiles::default_drained_byte(offset),
        })
    }
    fn fallback_byte(&mut self, offset: usize) -> Result<u8, Infallible> {
        let record = self.0.table(Source::Native, 0x91b629);
        let table = self
            .0
            .captured_role(Role::PositionFallback, Registration::Native, record);
        Ok(table.byte(offset))
    }
    fn camera_x(&mut self) -> Result<u16, Infallible> {
        samus_body::BottomHost::camera_x(self)
    }
    fn camera_y(&mut self) -> Result<u16, Infallible> {
        samus_body::BottomHost::camera_y(self)
    }
    fn set_body_x(&mut self, value: u16) -> Result<(), Infallible> {
        self.0.session_mut().player.body_position.x = value;
        Ok(())
    }
    fn set_body_y(&mut self, value: u16) -> Result<(), Infallible> {
        self.0.session_mut().player.body_position.y = value;
        Ok(())
    }
}
fn sprite_position<A: Authority>(h: &mut A, doubled: u16) -> (u16, u16) {
    samus_position_tiles::sprite_position(&mut LeaseBody(h), doubled).unwrap()
}
fn bottom_visible<A: Authority>(h: &mut A) -> bool {
    samus_body::bottom_is_visible(&mut LeaseBody(h)).unwrap()
}

fn loaded_role<A: Authority>(h: &mut A, role: Role, address: u32) -> RoleTable {
    if h.has_owned_display() {
        return h.prepared_role(role, Registration::Loaded, 0);
    }
    let record = h.table(Source::Loaded, address);
    h.captured_role(role, Registration::Loaded, record)
}

struct BodyHost<'a, A>(&'a mut A);
impl<A: Authority> samus_body::Host for BodyHost<'_, A> {
    type Error = Infallible;
    type Table = RoleTable;
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
    fn table(&mut self, half: Half) -> Result<RoleTable, Self::Error> {
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
        Ok(sprite_position(self.0, doubled))
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
        Ok(bottom_visible(self.0))
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
    // Setting the tiles definitions ($92:8023) leaves the top half's table index in $16, the
    // scratch word the next door-closing PLM takes as its room argument.
    let Ok(definition) = samus_position_tiles::definition(&mut TileHost(h));
    h.session_mut().spawn_scratch.argument = u16::from(definition.byte(0));
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
        bottom_visible(h)
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
    type Table = RoleTable;
    fn table(&mut self, which: TileTable) -> Result<RoleTable, Self::Error> {
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
    fn definition(&mut self, cursor: u16) -> Result<RoleTable, Self::Error> {
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
