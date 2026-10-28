//! Samus animation, sprite selection, echoes, atmospheric graphics and tile uploads.
#![forbid(unsafe_code)]
use crate::game::player_transitions as pt;
use crate::game::projectile_presentation_ram as sprite_output;
use crate::game::scene::samus_body::{self, Body};
use crate::game::scene::samus_position_tiles;
use crate::game::session_runtime::Runtime;
use crate::game::visual_host::{Content, Effects, Record, Source};

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

struct NativeTable(crate::rom::RomRecord);
impl samus_body::Table for NativeTable {
    type Error = std::convert::Infallible;
    fn little_word(&self, offset: usize) -> Result<u16, Self::Error> {
        Ok(u16::from_le_bytes(self.0.bytes(offset)))
    }
    fn native_word(&self, offset: usize) -> Result<u16, Self::Error> {
        Ok(self.0.native_word(offset))
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
        Ok(NativeTable(self.0.table(
            Source::Loaded,
            match half {
                samus_body::Half::Top => 0x929263,
                samus_body::Half::Bottom => 0x92945d,
            },
        )))
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

fn draw_body(game: &mut Game, mode: Body) {
    samus_body::draw(&mut NativeBody(&mut Runtime::new(game)), mode).unwrap();
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
pub(crate) fn advance_death_pose(game: &mut Game) -> CoroutineRet {
    pt::advance_animation(&mut Runtime::new(game), pt::state::AnimationStep::Death);
    Samus_DrawDuringDeathAnim(game);
    let expired = word!(game.g_word_7E0DE8 -= 1, reread) & 0x8000 != 0;
    if word!(game.g_word_7E0DE8) == 0 || expired {
        word!(game.game_state += 1);
    }
    0
}

// Death phases share palette selection and the native upload queue with live drawing.
pub(crate) fn begin_death(game: &mut Game) -> CoroutineRet {
    Samus_DrawWhenNotAnimatingOrDying(game);
    if crate::game::music::HasQueuedMusic(game) == 0 {
        const FRAMES: [u8; 28] = [
            5, 5, 5, 5, 1, 5, 5, 0, 1, 0, 5, 5, 5, 5, 5, 5, 5, 1, 1, 1, 5, 5, 5, 5, 5, 5, 5, 5,
        ];
        let movement = game.ram().read_byte(0xa1f);
        if game.ram().read_byte(0xa1f) == 3 {
            crate::game::music::QueueSfx1_Max6(0x32, game);
        }
        let frame = u16::from(FRAMES[usize::from(movement)]);
        word!(
            game.samus_pose = if game.ram().read_byte(0xa1e) == 4 {
                0xd8
            } else {
                0xd7
            }
        );
        pt::finish_pose_change(&mut Runtime::new(game));
        word! { game; samus_anim_frame_skip = 0; samus_anim_frame = frame; }
        word!(game.samus_x_pos -= word!(game.layer1_x_pos));
        word!(game.samus_y_pos -= word!(game.layer1_y_pos));
        word!(game.game_state += 1);
    }
    0
}

pub(crate) fn flash_death(game: &mut Game) -> CoroutineRet {
    if word!(game.g_word_7E0DE6) as i32 - 4i32 & 0x8000 != 0 {
        queue_death_tiles((2u16).wrapping_mul(word!(game.g_word_7E0DE6)), game);
    }
    if word!(game.g_word_7E0DE6 += 1, reread) as i32 - 60i32 & 0x8000 != 0 {
        let expired = word!(game.game_options_screen_index -= 1, reread) & 0x8000 != 0;
        if word!(game.game_options_screen_index) == 0 || expired {
            if word!(game.g_word_7E0DE4) != 0 {
                word! { game; g_word_7E0DE4 = 0; game_options_screen_index = 3; }
            } else {
                word! { game, 1; g_word_7E0DE4, game_options_screen_index }
            }
            death_palette(Some(word!(game.g_word_7E0DE4).wrapping_mul(2)), game);
        }
        Samus_DrawDuringDeathAnim(game);
    } else {
        death_palette(None, game);
        queue_death_tiles(8, game);
        let delay = Runtime::new(game).table(Source::Native, 0x9bb823).byte(0);
        word!(game.game_options_screen_index = u16::from(delay));
        word! { game, 0; g_word_7E0DE4, g_word_7E0DE6 }
        advance_suit_explosion(game);
        word!(
            game.substate = (word!(game.joypad2_last) as i32 & 0xb0
                == kButton_A as i32 | kButton_L as i32 | kButton_R as i32)
                as u16
        );
        word!(game.game_state += 1);
    }
    0
}

fn death_palette(frame: Option<u16>, game: &mut Game) {
    let table = if frame.is_some() { 0x9bb6d2 } else { 0x9bb5c8 };
    let pointer = crate::rtl::rom_table_word!(
        game,
        table,
        (word!(game.samus_suit_palette_index) >> 1) as isize
    )
    .wrapping_add(frame.unwrap_or(0));
    let address = Runtime::new(game)
        .legacy_native_record(0x9b, pointer)
        .native_word(0);
    let colors = Runtime::new(game)
        .legacy_native_record(0x9b, address)
        .bytes::<32>(0);
    game.ram().copy_in(0xc180, &colors);
    // The second source is resolved only after the first complete palette copy.
    let address = match frame {
        Some(frame) => crate::rtl::rom_table_word!(game, 0x9bb80f, (frame >> 1) as isize),
        None => 0xa120,
    };
    let colors = Runtime::new(game)
        .legacy_native_record(0x9b, address)
        .bytes::<32>(0);
    game.ram().copy_in(0xc1e0, &colors);
}

fn queue_death_tiles(frame: u16, game: &mut Game) {
    const SOURCES: [u16; 5] = [0x8400, 0x8800, 0x8c00, 0x9000, 0x8000];
    const DESTINATIONS: [u16; 5] = [0x6200, 0x6400, 0x6600, 0x6800, 0x6000];
    crate::game::display_refresh::queue_native_transfer(
        word!(game.vram_write_queue_tail),
        1024,
        |_| SOURCES[usize::from(frame >> 1)],
        0x9b,
        |_| DESTINATIONS[usize::from(frame >> 1)],
        game,
    );
}

fn whiten_death_palette(game: &mut Game) {
    if word!(game.substate) == 0 && word!(game.g_word_7E0DE4) != 0 {
        const SHADES: [u16; 22] = [
            0x421, 0xc63, 0x14a5, 0x1ce7, 0x2529, 0x2d6b, 0x35ad, 0x4210, 0x4a52, 0x4e73, 0x5294,
            0x56b5, 0x5ad6, 0x5ef7, 0x6318, 0x6739, 0x6b5a, 0x6f7b, 0x739c, 0x77bd, 0x7bde, 0x7fff,
        ];
        let shade = SHADES[usize::from(word!(game.g_word_7E0DE6))];
        word_transfers!(game, ascending, 192; native(0xc000) <- value(shade));
        word_transfers!(game, ascending, 32; native(0xc1a0) <- value(shade));
        if word!(game.g_word_7E0DE6) as i32 - 20i32 & 0x8000 != 0 {
            word!(game.g_word_7E0DE6 += 1);
        }
    }
}

pub(crate) fn explode_suit(game: &mut Game) -> CoroutineRet {
    whiten_death_palette(game);
    if advance_suit_explosion(game) {
        word! { game, 1; screen_fade_delay, screen_fade_counter }
        word!(game.game_state += 1);
    }
    0
}

fn advance_suit_explosion(game: &mut Game) -> bool {
    let expired = word!(game.game_options_screen_index -= 1, reread) & 0x8000 != 0;
    if word!(game.game_options_screen_index) == 0 || expired {
        if word!(game.g_word_7E0DE4 += 1, reread) as i32 - 9i32 & 0x8000 == 0 {
            word!(game.g_word_7E0DE6 = 21);
            whiten_death_palette(game);
            word!(game.substate = 0);
            return true;
        }
        let recolor =
            word!(game.substate) == 0 || word!(game.g_word_7E0DE4) as i32 - 2i32 & 0x8000 != 0;
        // Keep native source capture ahead of each live, linear frame displacement.
        let delay = Runtime::new(game).table(Source::Native, 0x9bb823);
        word!(
            game.game_options_screen_index =
                u16::from(delay.byte(2 * usize::from(word!(game.g_word_7E0DE4))))
        );
        if recolor {
            let frames = Runtime::new(game).table(Source::Native, 0x9bb823);
            let frame = frames.byte(2 * usize::from(word!(game.g_word_7E0DE4)) + 1);
            death_palette(Some(2u16.wrapping_mul(u16::from(frame))), game);
        }
    }
    let sprite = (if game.ram().read_byte(0xa1e) == 4 {
        word!(game.g_word_7E0DE4) as i32 + 2085
    } else {
        word!(game.g_word_7E0DE4) as i32 + 2076
    }) as u16;
    sprite_output::samus(
        sprite,
        (word!(game.samus_x_pos), word!(game.samus_y_pos)),
        game,
        &mut sprite_output::Native,
    );
    false
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
        h.foreign(|game| {
            crate::game::projectile_presentation_ram::echo(
                (index * 2) as u16,
                game,
                &mut crate::game::projectile_presentation_ram::Native,
            )
        });
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
    let height = h
        .table(Source::Native, 0x91b629)
        .byte(usize::from(h.session().player.pose) * 8 + 4);
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
    h.draw_samus_sprite(top, x, y);
    if h.foreign(bottom_is_visible) & 1 != 0 {
        let pose = h.session().player.pose;
        let bottom = sprite_index(h, 0x92945d, pose, false);
        let x = h
            .word(0xab0 + usize::from(k & !1))
            .wrapping_sub(h.inputs().camera_x);
        h.draw_samus_sprite(bottom, x, y);
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
    fn fallback_byte(&mut self, offset: usize) -> Result<u8, Self::Error> {
        let image = self.0.capture_native_rom();
        Ok(image.read(crate::rtl::lorom_offset(0x91b629) as usize + offset))
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
    fn native_word(&self, offset: usize) -> Result<u16, Self::Error> {
        Ok(self.0.native_word(offset))
    }
    fn byte(&self, offset: usize) -> Result<u8, Self::Error> {
        Ok(self.0.byte(offset))
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
        Ok(NativeTable(self.0.table(
            Source::Loaded,
            match which {
                samus_position_tiles::TileTable::Pose => 0x92d94e,
                samus_position_tiles::TileTable::Top => 0x92d91e,
                samus_position_tiles::TileTable::Bottom => 0x92d938,
            },
        )))
    }
    fn pose(&mut self) -> Result<u16, Self::Error> {
        Ok(self.0.session().player.pose)
    }
    fn frame(&mut self) -> Result<u16, Self::Error> {
        Ok(self.0.session().player.animation_clock.frame)
    }
    fn definition(&mut self, cursor: u16) -> Result<NativeTable, Self::Error> {
        Ok(NativeTable(self.0.legacy_native_record(0x92, cursor)))
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
    fn request_written(&mut self, _: samus_body::Half, _: u16) -> Result<(), Self::Error> {
        Ok(())
    }
}
