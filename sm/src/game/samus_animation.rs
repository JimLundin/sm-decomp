//! Samus animation, sprite selection, echoes, atmospheric graphics and tile uploads.
#![forbid(unsafe_code)]
use crate::game::scene::samus_position_tiles;
use crate::game::session_runtime::Runtime;
use sm_game::samus_drawing;
use sm_game::scene::samus_body::{self, Body};
use sm_game::scene::samus_content::{PreparedBytes, Registration, Role, RoleLease};
use sm_game::scene::samus_native_content::CapturedRom;
use sm_game::visual_host::Record;

pub use crate::game::prelude::*;
use sm_game::cinematic::project_cinematic_point;

fn bottom_is_visible(game: &mut Game) -> u8 {
    u8::from(samus_body::bottom_is_visible(&mut NativeBottom(game)).unwrap())
}

struct NativeBottom<'a>(&'a mut Game);
impl samus_body::BottomHost for NativeBottom<'_> {
    type Error = std::convert::Infallible;
    fn pose(&mut self) -> Result<u16, Self::Error> {
        Ok(self.0.samus_pose())
    }
    fn animation_frame(&mut self) -> Result<u16, Self::Error> {
        Ok(self.0.samus_anim_frame())
    }
    fn movement(&mut self) -> Result<u8, Self::Error> {
        Ok(self.0.samus_movement_type() as u8)
    }
    fn clear_bottom(&mut self) -> Result<(), Self::Error> {
        let game = &mut *self.0;
        word!(game.samus_bottom_half_spritemap_index = 0);
        Ok(())
    }
    fn cursor(&mut self) -> Result<u16, Self::Error> {
        let game = &mut *self.0;
        Ok(word!(game.oam_next_ptr))
    }
    fn player_x(&mut self) -> Result<u16, Self::Error> {
        let game = &mut *self.0;
        Ok(word!(game.samus_x_pos))
    }
    fn player_y(&mut self) -> Result<u16, Self::Error> {
        let game = &mut *self.0;
        Ok(word!(game.samus_y_pos))
    }
    fn camera_x(&mut self) -> Result<u16, Self::Error> {
        let game = &mut *self.0;
        Ok(word!(game.layer1_x_pos))
    }
    fn camera_y(&mut self) -> Result<u16, Self::Error> {
        let game = &mut *self.0;
        Ok(word!(game.layer1_y_pos))
    }
    fn marker_word(&mut self, entry: usize, value: u16) -> Result<(), Self::Error> {
        let game = &mut *self.0;
        word_transfers!(game, ascending, 1; native(0x370 + entry) <- value(value));
        Ok(())
    }
    fn publish_cursor(&mut self, value: u16) -> Result<(), Self::Error> {
        let game = &mut *self.0;
        word!(game.oam_next_ptr = value);
        Ok(())
    }
    fn marker_written(&mut self, _: usize, _: u16, _: u16) -> Result<(), Self::Error> {
        Ok(())
    }
}

enum BoundTable {
    Native(RoleLease<CapturedRom>),
    Prepared(RoleLease<PreparedBytes>),
}
pub struct NativeTable {
    record: Option<crate::rom::RomRecord>,
    bound: Option<BoundTable>,
}
impl NativeTable {
    pub(super) fn capture(
        game: &Game,
        role: Role,
        source: Registration,
        record: crate::rom::RomRecord,
    ) -> Self {
        let bound = game.samus_execution().unwrap().map(|execution| {
            let base = game.samus_role_base(role, source).unwrap().unwrap();
            let displacement = record
                .base
                .checked_sub(base)
                .expect("Samus role mapped base");
            match execution {
                crate::game_state::SamusExecution::Prepared => BoundTable::Prepared(
                    game.prepared_samus_role(role, source, displacement)
                        .unwrap()
                        .unwrap(),
                ),
                crate::game_state::SamusExecution::RetainedNative => BoundTable::Native(
                    game.capture_samus_role(role, source, displacement, &record.image)
                        .unwrap()
                        .unwrap(),
                ),
            }
        });
        Self {
            record: Some(record),
            bound,
        }
    }
    fn prepared(game: &Game, role: Role, source: Registration, displacement: usize) -> Self {
        let bound = game
            .prepared_samus_role(role, source, displacement)
            .expect("Samus prepared role")
            .expect("supplied Samus role");
        Self {
            record: None,
            bound: Some(BoundTable::Prepared(bound)),
        }
    }
    pub(super) fn byte(&self, offset: usize) -> u8 {
        match &self.bound {
            None => self
                .record
                .as_ref()
                .expect("native Samus record")
                .byte(offset),
            Some(BoundTable::Native(table)) => table.byte(offset).unwrap(),
            Some(BoundTable::Prepared(table)) => table.byte(offset).unwrap(),
        }
    }
    pub(super) fn word(&self, offset: usize, little: bool) -> u16 {
        if little && self.bound.is_some() && self.record.is_some() {
            let record = self.record.as_ref().expect("native Samus record");
            let start = record
                .base
                .checked_add(offset)
                .expect("ROM copy offset overflow");
            let end = start.checked_add(2).expect("ROM copy extent overflow");
            assert!(end <= record.image.len(), "ROM copy source extent");
        }
        match &self.bound {
            None if little => u16::from_le_bytes(
                self.record
                    .as_ref()
                    .expect("native Samus record")
                    .bytes(offset),
            ),
            None => self
                .record
                .as_ref()
                .expect("native Samus record")
                .native_word(offset),
            Some(BoundTable::Native(table)) => if little {
                table.little_word(offset)
            } else {
                table.native_word(offset)
            }
            .unwrap(),
            Some(BoundTable::Prepared(table)) => if little {
                table.little_word(offset)
            } else {
                table.native_word(offset)
            }
            .unwrap(),
        }
    }
}
impl samus_body::Table for NativeTable {
    type Error = std::convert::Infallible;
    fn little_word(&self, offset: usize) -> Result<u16, Self::Error> {
        Ok(self.word(offset, true))
    }
    fn native_word(&self, offset: usize) -> Result<u16, Self::Error> {
        Ok(self.word(offset, false))
    }
}
impl sm_game::samus_death::Host for Game {
    fn draw_samus(&mut self, body: Body) {
        draw_body(self, body);
    }
}

fn draw_body(game: &mut Game, mode: Body) {
    samus_drawing::draw_body(&mut Runtime::new(game), mode);
}

pub fn Samus_Draw(game: &mut Game) {
    draw_body(game, Body::Normal);
}
pub fn Samus_DrawDuringDeathAnim(game: &mut Game) {
    draw_body(game, Body::Death);
}
pub fn Samus_DrawWhenNotAnimatingOrDying(game: &mut Game) {
    draw_body(game, Body::Frozen);
}
pub fn Samus_DrawEchoes(game: &mut Game) {
    samus_drawing::draw_echoes(&mut Runtime::new(game));
}
pub fn Samus_DrawShinesparkCrashEchoes(k: u16, game: &mut Game) {
    samus_drawing::draw_shinespark_crash_echoes(k, &mut Runtime::new(game));
}

impl samus_drawing::RoleTable for NativeTable {
    fn byte(&self, offset: usize) -> u8 {
        NativeTable::byte(self, offset)
    }
}

impl samus_drawing::Authority for Runtime<'_> {
    type Table = NativeTable;
    fn frame_counter(&self) -> u16 {
        self.inputs().frame
    }
    fn camera(&self) -> (u16, u16) {
        (self.inputs().camera_x, self.inputs().camera_y)
    }
    fn has_owned_display(&self) -> bool {
        Runtime::has_owned_display(self)
    }
    fn draw_owned_body(&mut self, mode: Body) {
        Runtime::draw_owned_body(self, mode);
    }
    fn draw_owned_echo(&mut self, slot: u16) {
        Runtime::draw_owned_echo(self, slot);
    }
    fn owned_bottom_visible(&mut self) -> bool {
        Runtime::owned_bottom_visible(self)
    }
    fn draw_echo_samus_sprite(&mut self, slot: u16, index: u16, x: u16, y: u16) {
        Runtime::draw_echo_samus_sprite(self, slot, index, x, y);
    }
    fn draw_native_echo(&mut self, slot: u16) {
        self.foreign(|game| {
            crate::game::projectile_presentation_ram::echo(
                slot,
                game,
                &mut crate::game::projectile_presentation_ram::Native,
            )
        });
    }
    fn samus_source_diagnostic(&mut self, address: u32) {
        Runtime::samus_source_diagnostic(self, address);
    }
    fn samus_request_written(&mut self, half: samus_body::Half, pointer: u16) {
        Runtime::samus_request_written(self, half, pointer);
    }
    fn prepared_role(
        &self,
        role: Role,
        registration: Registration,
        displacement: usize,
    ) -> NativeTable {
        NativeTable::prepared(self.samus_game(), role, registration, displacement)
    }
    fn captured_role(
        &self,
        role: Role,
        registration: Registration,
        record: crate::rom::RomRecord,
    ) -> NativeTable {
        NativeTable::capture(self.samus_game(), role, registration, record)
    }
    fn role_base(&self, role: Role, registration: Registration) -> usize {
        self.samus_game()
            .samus_role_base(role, registration)
            .unwrap()
            .unwrap()
    }
    fn sprite_position(&mut self, doubled: u16) -> (u16, u16) {
        let position = self.foreign(|game| sprite_position(doubled, game));
        (position.k, position.j)
    }
    fn bottom_visible(&mut self) -> bool {
        self.foreign(bottom_is_visible) & 1 != 0
    }
}

fn sprite_position(k: u16, game: &mut Game) -> PairU16 {
    let (x, y) = samus_position_tiles::sprite_position(&mut NativePosition(game), k).unwrap();
    MakePairU16(x, y)
}
struct NativePosition<'a>(&'a mut Game);
impl samus_position_tiles::PositionHost for NativePosition<'_> {
    type Error = std::convert::Infallible;
    fn ceres(&mut self) -> Result<u16, Self::Error> {
        let game = &mut *self.0;
        Ok(word!(game.ceres_status))
    }
    fn player_x(&mut self) -> Result<u16, Self::Error> {
        let game = &mut *self.0;
        Ok(word!(game.samus_x_pos))
    }
    fn player_y(&mut self) -> Result<u16, Self::Error> {
        let game = &mut *self.0;
        Ok(word!(game.samus_y_pos))
    }
    fn set_player_x(&mut self, value: u16) -> Result<(), Self::Error> {
        let game = &mut *self.0;
        word!(game.samus_x_pos = value);
        Ok(())
    }
    fn set_player_y(&mut self, value: u16) -> Result<(), Self::Error> {
        let game = &mut *self.0;
        word!(game.samus_y_pos = value);
        Ok(())
    }
    fn project(&mut self, x: u16, y: u16) -> Result<(u16, u16), Self::Error> {
        let point = project_cinematic_point(sm_game::geometry::Point { x, y }, self.0);
        Ok((point.x, point.y))
    }
    fn frame(&mut self) -> Result<u16, Self::Error> {
        Ok(self.0.samus_anim_frame())
    }
    fn movement(&mut self) -> Result<u8, Self::Error> {
        Ok(self.0.samus_movement_type() as u8)
    }
    fn landing_word(&self, bytes: [u8; 2]) -> u16 {
        u16::from_ne_bytes(bytes)
    }
    fn landing_offset(&mut self, offset: usize) -> Result<u16, Self::Error> {
        Ok(
            match self
                .0
                .prepared_samus_role(Role::Landing, Registration::Native, 0)
                .unwrap()
            {
                Some(table) => table.native_word(offset).unwrap(),
                None => self.landing_word(samus_position_tiles::default_landing_bytes(offset)),
            },
        )
    }
    fn crouching_offset(&mut self, offset: usize) -> Result<i8, Self::Error> {
        Ok(
            match self
                .0
                .prepared_samus_role(Role::Crouching, Registration::Native, 0)
                .unwrap()
            {
                Some(table) => table.byte(offset).unwrap() as i8,
                None => samus_position_tiles::default_crouching_byte(offset),
            },
        )
    }
    fn drained_offset(&mut self, offset: usize) -> Result<i8, Self::Error> {
        Ok(
            match self
                .0
                .prepared_samus_role(Role::Drained, Registration::Native, 0)
                .unwrap()
            {
                Some(table) => table.byte(offset).unwrap() as i8,
                None => samus_position_tiles::default_drained_byte(offset),
            },
        )
    }
    fn fallback_byte(&mut self, offset: usize) -> Result<u8, Self::Error> {
        let image = self.0.capture_native_rom();
        let record = crate::rom::RomRecord {
            image,
            base: crate::rtl::lorom_offset(0x91b629) as usize,
        };
        Ok(
            NativeTable::capture(self.0, Role::PositionFallback, Registration::Native, record)
                .byte(offset),
        )
    }
    fn camera_x(&mut self) -> Result<u16, Self::Error> {
        let game = &mut *self.0;
        Ok(word!(game.layer1_x_pos))
    }
    fn camera_y(&mut self) -> Result<u16, Self::Error> {
        let game = &mut *self.0;
        Ok(word!(game.layer1_y_pos))
    }
    fn set_body_x(&mut self, value: u16) -> Result<(), Self::Error> {
        let game = &mut *self.0;
        word!(game.samus_spritemap_x_pos = value);
        Ok(())
    }
    fn set_body_y(&mut self, value: u16) -> Result<(), Self::Error> {
        let game = &mut *self.0;
        word!(game.samus_spritemap_y_pos = value);
        Ok(())
    }
    fn body_x(&mut self) -> Result<u16, Self::Error> {
        let game = &mut *self.0;
        Ok(word!(game.samus_spritemap_x_pos))
    }
    fn body_y(&mut self) -> Result<u16, Self::Error> {
        let game = &mut *self.0;
        Ok(word!(game.samus_spritemap_y_pos))
    }
}
impl samus_position_tiles::Table for NativeTable {
    type Error = std::convert::Infallible;
    fn validate_source(&self, _: crate::game::scene::records::Revision) -> Result<(), Self::Error> {
        Ok(())
    }
    fn native_word(&self, offset: usize) -> Result<u16, Self::Error> {
        Ok(self.word(offset, false))
    }
    fn byte(&self, offset: usize) -> Result<u8, Self::Error> {
        Ok(self.byte(offset))
    }
}
