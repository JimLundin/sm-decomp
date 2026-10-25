//! Legacy entry and backend boundary for the shared owned sprite-effect core.
use crate::game::projectile_presentation_ram as sprite_output;
use crate::game::sprite_drawing::Clip;
use crate::game::{
    session::Point,
    session_runtime::Runtime,
    sprite_effect_controllers as controllers,
    sprite_effect_host::{self, SpriteMember},
    sprite_effect_state::{
        LegacySpriteDefinition, LegacySpriteHandle, LegacySpriteProgram, LegacySpriteWordIndex,
        SpriteDraw, SpriteSpawn,
    },
    sprite_objects,
};
use crate::game_state::Game;

fn address(member: SpriteMember, index: LegacySpriteWordIndex) -> usize {
    let base = match member {
        SpriteMember::Instruction => 0xef78,
        SpriteMember::Timer => 0xeff8,
        SpriteMember::Appearance => 0xf078,
        SpriteMember::X => 0xf0f8,
        SpriteMember::XFraction => 0xf178,
        SpriteMember::Y => 0xf1f8,
        SpriteMember::YFraction => 0xf278,
        SpriteMember::DisableFlags => 0xf2f8,
    };
    base + usize::from(index.0) * 2
}
#[forbid(unsafe_code)]
impl sprite_effect_host::Aliases for Runtime<'_> {
    fn sprite_word(&self, member: SpriteMember, index: LegacySpriteWordIndex) -> u16 {
        self.word(address(member, index))
    }
    fn set_sprite_word_le_slice(
        &mut self,
        member: SpriteMember,
        index: LegacySpriteWordIndex,
        value: u16,
    ) {
        self.write_slice_le(address(member, index), value);
    }
    fn sprite_word_native(&self, member: SpriteMember, index: LegacySpriteWordIndex) -> u16 {
        self.native_word(address(member, index))
    }
    fn set_sprite_word_native_slice(
        &mut self,
        member: SpriteMember,
        index: LegacySpriteWordIndex,
        value: u16,
    ) {
        self.write_slice_native(address(member, index), value);
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
        self.foreign(|game| unsafe { crate::rtl::Unreachable(game) });
    }
}
#[forbid(unsafe_code)]
impl sprite_effect_host::ControllerInputs for Runtime<'_> {
    fn attachment_x(&self, enemy: u16) -> u16 {
        self.native_word(
            0xf78 + usize::from(enemy) + core::mem::offset_of!(crate::types::EnemyBase, x_pos),
        )
    }
    fn attachment_y(&self, enemy: u16) -> u16 {
        self.native_word(
            0xf78 + usize::from(enemy) + core::mem::offset_of!(crate::types::EnemyBase, y_pos),
        )
    }
    fn attachment_frozen_timer(&self, enemy: u16) -> u16 {
        self.native_word(
            0xf78
                + usize::from(enemy)
                + core::mem::offset_of!(crate::types::EnemyBase, frozen_timer),
        )
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
pub(crate) fn retire_native(handle: u16, game: &mut Game) {
    controllers::retire_native(&mut Runtime::new(game), LegacySpriteHandle(handle));
}
pub(crate) fn freeze_attachment(handle: u16, appearance: u16, program: u16, game: &mut Game) {
    controllers::freeze_attachment(
        &mut Runtime::new(game),
        LegacySpriteHandle(handle),
        appearance,
        LegacySpriteProgram(program),
    );
}
pub(crate) fn attach_actor(handle: u16, enemy: u16, appearance: u16, game: &mut Game) {
    controllers::attach_actor(
        &mut Runtime::new(game),
        LegacySpriteHandle(handle),
        enemy,
        appearance,
    );
}
pub(crate) fn flash_attachment(handle: u16, appearance: u16, game: &mut Game) {
    controllers::flash_attachment(
        &mut Runtime::new(game),
        LegacySpriteHandle(handle),
        appearance,
    );
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
pub(crate) fn draygon_trajectory(cursor: u16, game: &mut Game) -> u16 {
    controllers::draygon_trajectory(&mut Runtime::new(game), cursor)
}
pub(crate) fn draygon_spawn_ring(game: &mut Game) {
    controllers::draygon_spawn_ring(&mut Runtime::new(game));
}
pub(crate) fn draygon_spawn_trails(game: &mut Game) {
    controllers::draygon_spawn_trails(&mut Runtime::new(game));
}
