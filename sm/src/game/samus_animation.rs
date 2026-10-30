//! Samus animation, sprite selection, echoes, atmospheric graphics and tile uploads.
#![forbid(unsafe_code)]
use crate::game::scene::samus_position_tiles;
use crate::game::session_runtime::Runtime;
use sm_game::scene::samus_body::{self, Body};
use sm_game::scene::samus_content::{PreparedBytes, Registration, Role, RoleLease};
use sm_game::scene::samus_native_content::CapturedRom;
use sm_game::visual_host::{Content, Effects, Record, Source};

pub use crate::game::prelude::*;
use crate::game::visual_runtime::project_cinematic_point;

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
pub(super) struct NativeTable {
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
struct NativeBody<'a, 'g>(&'a mut Runtime<'g>);
impl samus_body::Host for NativeBody<'_, '_> {
    type Error = std::convert::Infallible;
    type Table = NativeTable;
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
        Ok(self.0.inputs().frame)
    }
    fn pose(&mut self) -> Result<u16, Self::Error> {
        Ok(self.0.session().player.pose)
    }
    fn animation_frame(&mut self) -> Result<u16, Self::Error> {
        Ok(self.0.session().player.animation_clock.frame)
    }
    fn table(&mut self, half: samus_body::Half) -> Result<NativeTable, Self::Error> {
        let (role, address) = match half {
            samus_body::Half::Top => (Role::BodyTop, 0x929263),
            samus_body::Half::Bottom => (Role::BodyBottom, 0x92945d),
        };
        if self.0.has_owned_display() {
            return Ok(NativeTable::prepared(
                self.0.samus_game(),
                role,
                Registration::Loaded,
                0,
            ));
        }
        let record = self.0.table(Source::Loaded, address);
        Ok(NativeTable::capture(
            self.0.samus_game(),
            role,
            Registration::Loaded,
            record,
        ))
    }
    fn set_sprite(&mut self, half: samus_body::Half, index: u16) -> Result<(), Self::Error> {
        match half {
            samus_body::Half::Top => self.0.session_mut().player.top_sprite = index,
            samus_body::Half::Bottom => self.0.session_mut().player.bottom_sprite = index,
        }
        Ok(())
    }
    fn sprite_position(&mut self, doubled: u16) -> Result<(u16, u16), Self::Error> {
        let position = self.0.foreign(|game| sprite_position(doubled, game));
        Ok((position.k, position.j))
    }
    fn body_x(&mut self) -> Result<u16, Self::Error> {
        Ok(self.0.word(0xb04))
    }
    fn body_y(&mut self) -> Result<u16, Self::Error> {
        Ok(self.0.word(0xb06))
    }
    fn camera_x(&mut self) -> Result<u16, Self::Error> {
        Ok(self.0.inputs().camera_x)
    }
    fn camera_y(&mut self) -> Result<u16, Self::Error> {
        Ok(self.0.inputs().camera_y)
    }
    fn draw_sprite(&mut self, index: u16, x: u16, y: u16) -> Result<(), Self::Error> {
        self.0.draw_samus_sprite(index, x, y);
        Ok(())
    }
    fn bottom_visible(&mut self) -> Result<bool, Self::Error> {
        Ok(self.0.foreign(bottom_is_visible) & 1 != 0)
    }
    fn queue_tiles(&mut self) -> Result<(), Self::Error> {
        queue_body_tiles(self.0);
        Ok(())
    }
}

impl sm_game::samus_death::Host for Game {
    fn draw_samus(&mut self, body: Body) {
        draw_body(self, body);
    }
}

fn draw_body(game: &mut Game, mode: Body) {
    let mut runtime = Runtime::new(game);
    if runtime.has_owned_display() {
        runtime.draw_owned_body(mode);
    } else {
        samus_body::draw(&mut NativeBody(&mut runtime), mode).unwrap();
    }
}

fn sprite_index(h: &mut Runtime<'_>, table: u32, pose: u16, little_endian: bool) -> u16 {
    let half = match table {
        0x929263 => samus_body::Half::Top,
        0x92945d => samus_body::Half::Bottom,
        _ => unreachable!("Samus body table"),
    };
    samus_body::sprite_index(&mut NativeBody(h), half, pose, little_endian).unwrap()
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
    let h = &mut Runtime::new(game);
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
            h.foreign(|game| {
                crate::game::projectile_presentation_ram::echo(
                    (index * 2) as u16,
                    game,
                    &mut crate::game::projectile_presentation_ram::Native,
                )
            });
        }
    }
    if converging && h.session().projectiles.echoes.x[..2] == [0, 0] {
        h.session_mut().projectiles.echoes.index = 0;
    }
}

pub fn Samus_DrawShinesparkCrashEchoes(k: u16, game: &mut Game) {
    let h = &mut Runtime::new(game);
    if h.inputs().frame & 1 == 0 {
        return;
    }
    let pose = h.session().player.pose;
    let top = sprite_index(h, 0x929263, pose, false);
    let table = if h.has_owned_display() {
        NativeTable::prepared(
            h.samus_game(),
            Role::PositionFallback,
            Registration::Native,
            0,
        )
    } else {
        let record = h.table(Source::Native, 0x91b629);
        NativeTable::capture(
            h.samus_game(),
            Role::PositionFallback,
            Registration::Native,
            record,
        )
    };
    let height = table.byte(usize::from(h.session().player.pose) * 8 + 4);
    let y = h
        .word(0xab8 + usize::from(k & !1))
        .wrapping_sub(u16::from(height))
        .wrapping_sub(h.inputs().camera_y);
    if !(0..248).contains(&y) {
        return;
    }
    let x = h
        .word(0xab0 + usize::from(k & !1))
        .wrapping_sub(h.inputs().camera_x);
    h.draw_echo_samus_sprite(k, top, x, y);
    if if h.has_owned_display() {
        h.owned_bottom_visible()
    } else {
        h.foreign(bottom_is_visible) & 1 != 0
    } {
        let pose = h.session().player.pose;
        let bottom = sprite_index(h, 0x92945d, pose, false);
        let x = h
            .word(0xab0 + usize::from(k & !1))
            .wrapping_sub(h.inputs().camera_x);
        h.draw_echo_samus_sprite(k, bottom, x, y);
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
        let point = project_cinematic_point(Point16U { x, y }, self.0);
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
fn queue_body_tiles(h: &mut Runtime<'_>) {
    samus_position_tiles::queue_body_tiles(&mut NativeTiles(h)).unwrap();
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
struct NativeTiles<'a, 'g>(&'a mut Runtime<'g>);
impl samus_position_tiles::TileHost for NativeTiles<'_, '_> {
    type Error = std::convert::Infallible;
    type Table = NativeTable;
    fn table(
        &mut self,
        which: samus_position_tiles::TileTable,
    ) -> Result<NativeTable, Self::Error> {
        let (role, address) = match which {
            samus_position_tiles::TileTable::Pose => (Role::TilePose, 0x92d94e),
            samus_position_tiles::TileTable::Top => (Role::TileTop, 0x92d91e),
            samus_position_tiles::TileTable::Bottom => (Role::TileBottom, 0x92d938),
        };
        if self.0.has_owned_display() {
            return Ok(NativeTable::prepared(
                self.0.samus_game(),
                role,
                Registration::Loaded,
                0,
            ));
        }
        let record = self.0.table(Source::Loaded, address);
        Ok(NativeTable::capture(
            self.0.samus_game(),
            role,
            Registration::Loaded,
            record,
        ))
    }
    fn pose(&mut self) -> Result<u16, Self::Error> {
        Ok(self.0.session().player.pose)
    }
    fn frame(&mut self) -> Result<u16, Self::Error> {
        Ok(self.0.session().player.animation_clock.frame)
    }
    fn definition(&mut self, cursor: u16) -> Result<NativeTable, Self::Error> {
        if self.0.has_owned_display() {
            if cursor & 0x8000 == 0 {
                self.0.samus_source_diagnostic(0x920000 | u32::from(cursor));
            }
            let mapped = crate::rtl::lorom_offset(0x920000 | u32::from(cursor)) as usize;
            let base = self
                .0
                .samus_game()
                .samus_role_base(Role::TileDefinition, Registration::Native)
                .unwrap()
                .unwrap();
            return Ok(NativeTable::prepared(
                self.0.samus_game(),
                Role::TileDefinition,
                Registration::Native,
                mapped
                    .checked_sub(base)
                    .expect("Samus definition mapped base"),
            ));
        }
        let record = self.0.legacy_native_record(0x92, cursor);
        Ok(NativeTable::capture(
            self.0.samus_game(),
            Role::TileDefinition,
            Registration::Native,
            record,
        ))
    }
    fn request_word(&mut self, half: samus_body::Half, pointer: u16) -> Result<(), Self::Error> {
        self.0.write_word_le(
            pointer,
            match half {
                samus_body::Half::Top => 0x71f,
                samus_body::Half::Bottom => 0x721,
            },
        );
        Ok(())
    }
    fn request_flag(&mut self, half: samus_body::Half) -> Result<(), Self::Error> {
        self.0.set_byte(
            match half {
                samus_body::Half::Top => 0x71d,
                samus_body::Half::Bottom => 0x71e,
            },
            1,
        );
        Ok(())
    }
    fn request_written(&mut self, half: samus_body::Half, pointer: u16) -> Result<(), Self::Error> {
        self.0.samus_request_written(half, pointer);
        Ok(())
    }
}
