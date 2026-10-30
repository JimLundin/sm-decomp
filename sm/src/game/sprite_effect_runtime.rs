//! Legacy entry and backend boundary for the shared owned sprite-effect core.
use crate::game::enemy_projectiles::spawn_with_enemy_gfx;
use crate::game_state::Game;
use crate::ram_map::{slot, word};
use crate::types::Point16U;
use sm_game::actors::layouts::EnemyLayout104 as Draygon;
use sm_game::geometry::Point;
use sm_game::sprite_drawing::Clip;
use sm_game::sprite_output;
use {
    crate::game::session_runtime::Runtime,
    sm_game::{
        sprite_effect_controllers as controllers,
        sprite_effect_host::{self, SpriteMember},
        sprite_effect_state::{
            LegacySpriteDefinition, LegacySpriteHandle, LegacySpriteWordIndex, SpriteDraw,
            SpriteSpawn,
        },
        sprite_objects,
    },
};

#[forbid(unsafe_code)]
impl sprite_effect_host::Aliases for Runtime<'_> {
    fn sprite_word(&self, member: SpriteMember, index: LegacySpriteWordIndex) -> u16 {
        self.word(member.address(index))
    }
    fn set_sprite_word_le_slice(
        &mut self,
        member: SpriteMember,
        index: LegacySpriteWordIndex,
        value: u16,
    ) {
        self.write_slice_le(member.address(index), value);
    }
    fn sprite_word_native(&self, member: SpriteMember, index: LegacySpriteWordIndex) -> u16 {
        self.native_word(member.address(index))
    }
    fn set_sprite_word_native_slice(
        &mut self,
        member: SpriteMember,
        index: LegacySpriteWordIndex,
        value: u16,
    ) {
        self.write_slice_native(member.address(index), value);
    }
}
impl sprite_effect_host::Effects for Runtime<'_> {
    fn submit_sprite(&mut self, request: SpriteDraw) {
        self.foreign(|game| {
            #[cfg(sm_verify_dll)]
            crate::sprite_effect_observer::draw(
                request.map.0,
                request.position.x,
                request.position.y,
                request.palette,
                request.base_tile,
                game,
            );
            sprite_output::with_base_tile(
                0xb4,
                request.map.0,
                (request.position.x, request.position.y),
                request.palette,
                request.base_tile,
                Clip::None,
                game,
            );
        });
    }
    fn invalid_sprite_instruction(&mut self) {
        self.foreign(|game| crate::rtl::Unreachable(game));
    }
}
#[forbid(unsafe_code)]
impl sprite_effect_host::ControllerInputs for Runtime<'_> {
    fn attachment_word(&self, at: usize) -> u16 {
        self.native_word(at)
    }
}
impl sprite_effect_host::Host for Runtime<'_> {
    #[cfg(sm_verify_dll)]
    fn observe_draygon(&self, kind: u8, handle: u16, value: u16) {
        if crate::draygon_sprite_candidate::active() {
            let ram = if kind == 6 {
                (0..0x20000).map(|at| self.byte(at)).collect::<Vec<_>>()
            } else {
                Vec::new()
            };
            crate::draygon_sprite_candidate::event(kind, handle, value, &ram);
        }
    }
}

pub(crate) fn create_sprite(
    x: u16,
    y: u16,
    selector: u16,
    appearance: u16,
    game: &mut Game,
) -> u16 {
    let result = sprite_objects::spawn(
        &mut Runtime::new(game),
        SpriteSpawn {
            position: Point { x, y },
            definition: LegacySpriteDefinition(selector),
            appearance,
        },
    );
    sprite_objects::legacy_result(result).0
}
pub(crate) fn update_sprites(game: &mut Game) {
    sprite_objects::update(&mut Runtime::new(game));
}
pub(crate) fn draw_sprites(game: &mut Game) {
    sprite_objects::draw(&mut Runtime::new(game));
}
pub(crate) fn clear_sprites(game: &mut Game) {
    sprite_objects::clear_for_population_load(Runtime::new(game).session_mut());
}
pub(crate) fn retire_all(game: &mut Game) {
    sprite_objects::retire_all(Runtime::new(game).session_mut());
}
pub(crate) fn retire(handle: u16, game: &mut Game) {
    controllers::retire(&mut Runtime::new(game), LegacySpriteHandle(handle));
}
pub(crate) fn maw_appearance(handle: u16, appearance: u16, enemy: u16, game: &mut Game) {
    controllers::maw_appearance(
        &mut Runtime::new(game),
        LegacySpriteHandle(handle),
        appearance,
        enemy,
    );
}
pub(crate) fn draygon_motion(horizontal_bias: u16, game: &mut Game) {
    controllers::draygon_motion(&mut Runtime::new(game), horizontal_bias);
}
#[forbid(unsafe_code)]
pub(crate) fn draygon_trajectory(game: &mut Game) {
    let cursor = slot!(game, Draygon[0].var_46);
    let next = controllers::draygon_trajectory(&mut Runtime::new(game), cursor);
    slot!(game, Draygon[0].var_46 = next);
}
pub(crate) fn draygon_spawn_ring(game: &mut Game) {
    controllers::draygon_spawn_ring(&mut Runtime::new(game));
}
pub(crate) fn draygon_spawn_trails(game: &mut Game) {
    controllers::draygon_spawn_trails(&mut Runtime::new(game));
}

// These compatibility entrypoints retain native actor observations until the
// remaining boss controller joins the actor owner. Sprite lifetime stays owned.
#[forbid(unsafe_code)]
pub(crate) fn draygon_random_sprite(definition: u16, game: &mut Game) {
    game.next_random();
    let dx = (word!(game.random_number) & 0x7f).wrapping_sub(64);
    let dy = ((word!(game.random_number) >> 8) & 0x7f).wrapping_sub(64);
    let x = slot!(game, Draygon[0].base.x_pos).wrapping_add(dx);
    let y = slot!(game, Draygon[0].base.y_pos).wrapping_add(dy);
    create_sprite(x, y, definition, 0, game);
}
#[forbid(unsafe_code)]
pub(crate) fn draygon_death_bubble(game: &mut Game) {
    if word!(game.nmi_frame_counter_word) & 7 == 0 {
        let x = (game.next_random() & 0x7f) + 192;
        let y = ((word!(game.random_number) >> 8) & 0x3f) + 400;
        create_sprite(x, y, 21, 0, game);
    }
}
#[forbid(unsafe_code)]
pub(crate) fn draygon_spit(right: bool, game: &mut Game) {
    let dx = if right { 24 } else { 0u16.wrapping_sub(28) };
    game.eproj_spawn_pt = Point16U {
        x: slot!(game, Draygon[0].base.x_pos).wrapping_add(dx),
        y: slot!(game, Draygon[0].base.y_pos).wrapping_sub(16),
    };
    let direction = if right { 192 } else { 128 };
    word!(game.cinematic_y_sub = (game.next_random() & 0x3f) + direction);
    spawn_with_enemy_gfx(2, word!(game.cur_enemy_index), 0x8e50, game);
}
