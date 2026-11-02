//! Synchronous retained effects for the owned Draygon actor/player cohort.
#![forbid(unsafe_code)]
use crate::game::session_runtime::Runtime;
use sm_game::actors::host::{ActorRuntime, BossEffects, Death, Shot};
use sm_game::actors::{draygon_death, draygon_flight, draygon_grab, draygon_sweep};

impl draygon_grab::owned::Host for Runtime<'_> {
    fn actor_record(&mut self, bank: u8, address: u16) -> sm_data::cartridge::RomRecord {
        sm_game::cartridge::native_record(bank, address, self)
    }
    fn fire_wall_turret(&mut self) -> Option<u16> {
        draygon_flight::owned::fire_wall_turret(self)
    }
    fn foam_at_mouth(&mut self, key: u16, mask: u16) {
        draygon_flight::owned::foam_at_mouth(key, mask, self);
    }
    fn set_step_scratch(&mut self, value: u16) {
        self.session_mut().compatibility_enemy_scratch = value;
    }
    fn direction_step(&mut self, phase: u16, amplitude: u16) -> sm_game::geometry::Vector32 {
        self.foreign(|game| {
            crate::game::session_runtime::actor_direction_vector(phase, amplitude, game)
        })
    }
    fn step_along(&mut self, phase: u16, vector: sm_game::geometry::Vector32) {
        super::api::apply_vector(self, phase, vector);
    }
    fn scaled_cosine(&mut self, phase: u16, amplitude: u16) -> u16 {
        sm_game::actors::boss_lease::BossState::scaled_cosine(self, phase, amplitude)
    }
    fn scaled_sine(&mut self, phase: u16, amplitude: u16) -> u16 {
        sm_game::actors::boss_lease::BossState::scaled_sine(self, phase, amplitude)
    }
    fn command_player(&mut self, command: u16) -> u16 {
        self.foreign(|game| crate::game::session_runtime::player_command(command, game))
    }
    fn grab_samus(&mut self, direction: u16) {
        crate::game::player_movement::forced::grab_by_draygon(direction, self);
    }
    fn release_samus(&mut self) {
        crate::game::player_movement::forced::release_from_draygon(self);
    }
    fn correct_previous_position(&mut self) {
        self.foreign(crate::game::session_runtime::correct_previous_position);
    }
    fn suit_damage(&mut self, amount: u16) -> u16 {
        super::combat::suit_damage(self, amount)
    }
    fn damage_player(&mut self, amount: u16) {
        self.foreign(|game| crate::game::session_runtime::damage_player(amount, game));
    }
    fn create_sprite(&mut self, x: u16, y: u16, kind: u16, appearance: u16) -> u16 {
        self.foreign(|game| {
            crate::game::sprite_effect_runtime::create_sprite(x, y, kind, appearance, game)
        })
    }
}
impl draygon_flight::owned::Host for Runtime<'_> {
    fn frame(&self) -> u16 {
        sm_game::actors::boss_lease::BossState::frame(self)
    }
    fn roll_random(&mut self) -> u16 {
        sm_game::actors::boss_lease::BossState::roll_random(self)
    }
    fn turret_position(&mut self, index: isize) -> u16 {
        sm_game::actor_tables::word(self, |tables| &tables.draygon_turret_positions, index)
    }
    fn place_projectile_spawn(&mut self, x: u16, y: u16) {
        self.session_mut().enemy_projectiles.spawn.point = sm_game::geometry::Point { x, y };
    }
    fn spawn_actor_eproj(&mut self, parameter: u16, key: u16, definition: u16) -> u16 {
        sm_game::enemy_projectiles::lifecycle::spawn_from_enemy(self, parameter, key, definition)
    }
}
impl draygon_sweep::owned::Host for Runtime<'_> {
    fn body_bubble(&mut self, turret: Option<u16>) {
        draygon_flight::owned::body_bubble(turret, self);
    }
    fn move_body_horizontally(&mut self, right: bool) {
        draygon_flight::owned::move_horizontally(right, self);
    }
    fn swoop_table_overflow(&mut self) -> ! {
        self.foreign(crate::game::animated_tiles::native::unreachable);
        unreachable!()
    }
    fn imported_swoop_byte(&self, relative: usize) -> u8 {
        self.byte(0x9000 + relative)
    }
    fn set_imported_swoop_byte(&mut self, relative: usize, value: u8) {
        self.set_byte(0x9000 + relative, value);
    }
}
impl draygon_death::owned::Host for Runtime<'_> {
    fn apply_hurt_palettes(&mut self) {
        self.foreign(draygon_death::apply_hurt_palettes);
    }
    fn update_health_palette(&mut self) {
        self.foreign(draygon_death::update_health_palette);
    }
    fn shot_reaction(&mut self, shot: Shot) {
        ActorRuntime::shot_reaction(self, shot);
    }
    fn contact_reaction(&mut self, death: Death) {
        ActorRuntime::contact_reaction(self, death);
    }
    fn power_bomb_reaction(&mut self, death: Death) {
        ActorRuntime::power_bomb_reaction(self, death);
    }
    fn draygon_spawn_ring(&mut self) {
        self.foreign(crate::game::sprite_effect_runtime::draygon_spawn_ring);
    }
    fn draygon_motion(&mut self, bias: u16) {
        self.foreign(|game| crate::game::sprite_effect_runtime::draygon_motion(bias, game));
    }
    fn queue_music_delayed(&mut self, track: u16) {
        self.foreign(|game| crate::game::music::queue_music_delayed_8(track, game));
    }
    fn boss_word(&mut self, area: u16) -> u16 {
        self.foreign(|game| BossEffects::boss_word(game, area))
    }
    fn set_boss_word(&mut self, area: u16, value: u16) {
        self.foreign(|game| BossEffects::set_boss_word(game, area, value));
    }
    fn scatter_draygon_drop(&mut self, area: u16) {
        self.foreign(|game| {
            crate::game::enemy_projectiles::scatter_drop(
                sm_game::enemy_projectiles::drops::DropSource::Draygon,
                area,
                game,
            )
        });
    }
    fn retire_all_sprites(&mut self) {
        self.foreign(crate::game::sprite_effect_runtime::retire_all);
    }
}
// Ridley finite ports.
use ridley::owned::ProjectileField;
use sm_game::actors::ridley;
fn projectile_array(field: ProjectileField) -> sm_game::actors::boss_interaction::Array {
    use sm_game::actors::boss_interaction::Array;
    match field {
        ProjectileField::X => Array::ProjectileXPos,
        ProjectileField::Y => Array::ProjectileYPos,
        ProjectileField::XRadius => Array::ProjectileXRadius,
        ProjectileField::YRadius => Array::ProjectileYRadius,
        ProjectileField::Kind => Array::ProjectileType,
        ProjectileField::Direction => Array::ProjectileDir,
    }
}
impl ridley::owned::Host for Runtime<'_> {
    fn actor_record(&mut self, bank: u8, address: u16) -> sm_data::cartridge::RomRecord {
        sm_game::cartridge::native_record(bank, address, self)
    }
    fn run_ceres_ridley_function(&mut self, function: u16) {
        sm_game::actors::ceres_ridley::owned::run_function(self, function);
    }
    fn frame(&self) -> u16 {
        sm_game::actors::boss_lease::BossState::frame(self)
    }
    fn roll_random(&mut self) -> u16 {
        sm_game::actors::boss_lease::BossState::roll_random(self)
    }
    fn place_projectile_spawn(&mut self, x: u16, y: u16) {
        self.session_mut().enemy_projectiles.spawn.point = sm_game::geometry::Point { x, y };
    }
    fn spawn_room_projectile(&mut self, definition: u16, parameter: u16) -> u16 {
        sm_game::enemy_projectiles::lifecycle::spawn_room(self, definition, parameter)
    }
    fn spawn_population(&mut self, bank: u8, population: u16) -> u16 {
        ridley::Host::spawn_population(self, bank, population)
    }
    fn set_boss_bit(&mut self, mask: u16) {
        ridley::Host::set_boss_bit(self, mask);
    }
    fn hardware_sine_cosine(&mut self, amplitude: u16, angle: u16) -> (u16, u16) {
        ridley::Host::hardware_sine_cosine(self, amplitude, angle)
    }
    fn move_liquid(&mut self, target: u16, velocity: u16, delay: u16) {
        ridley::Host::move_liquid(self, target, velocity, delay);
    }
    fn command_player(&mut self, command: u16) -> u16 {
        self.foreign(|game| crate::game::session_runtime::player_command(command, game))
    }
    fn normal_death(&mut self, animation: u16) {
        self.foreign(|game| sm_game::actors::deaths::Deaths::normal_death(game, animation));
    }
    fn scatter_enemy_drop(
        &mut self,
        source: sm_game::enemy_projectiles::drops::DropSource,
        key: u16,
    ) {
        self.foreign(|game| sm_game::actors::deaths::Deaths::scatter_enemy_drop(game, source, key));
    }
    fn queue_music_delayed(&mut self, track: u16) {
        self.foreign(|game| crate::game::music::queue_music_delayed_8(track, game));
    }
    fn suit_damage(&mut self, amount: u16) -> u16 {
        super::combat::suit_damage(self, amount)
    }
    fn damage_player(&mut self, amount: u16) {
        self.foreign(|game| crate::game::session_runtime::damage_player(amount, game));
    }
    fn contact_reaction(&mut self, death: Death) {
        ActorRuntime::contact_reaction(self, death);
    }
    fn shot_reaction(&mut self, shot: Shot) {
        ActorRuntime::shot_reaction(self, shot);
    }
    fn power_bomb_reaction(&mut self, death: Death) {
        ActorRuntime::power_bomb_reaction(self, death);
    }
    fn unreachable(&mut self) -> ! {
        self.foreign(crate::game::animated_tiles::native::unreachable);
        unreachable!()
    }
    fn imported_tail_byte(&self, relative: usize) -> u8 {
        self.byte(0x2000 + relative)
    }
    fn set_imported_tail_byte(&mut self, relative: usize, value: u8) {
        self.set_byte(0x2000 + relative, value);
    }
    fn imported_projectile_field(&self, index: usize, field: ProjectileField) -> u16 {
        sm_game::actors::boss_interaction::get_at(self, projectile_array(field), index)
    }
    fn set_imported_projectile_field(&mut self, index: usize, field: ProjectileField, value: u16) {
        sm_game::actors::boss_interaction::set_at(self, projectile_array(field), index, value);
    }
    fn tail_tip_explosion_list(&mut self, index: isize) -> u16 {
        sm_game::cartridge::NativeImage::native_image(self)
            .table_word(sm_data::cartridge::lorom_offset(0xa6_c7ba) as usize, index)
    }
    fn pogo_row(&mut self, horizontal: bool, index: isize) -> u16 {
        if horizontal {
            sm_game::actor_tables::word(self, |t| &t.ridley_x_velocities, index)
        } else {
            sm_game::actor_tables::word(self, |t| &t.ridley_y_velocities, index)
        }
    }
    fn pogo_acceleration(&mut self, upwards: bool, index: isize) -> u16 {
        if upwards {
            sm_game::actor_tables::word(self, |t| &t.ridley_upwards, index)
        } else {
            sm_game::actor_tables::word(self, |t| &t.ridley_downwards, index)
        }
    }
    fn set_imported_tail_destination_byte(&mut self, destination: u16, offset: usize, value: u8) {
        self.set_byte(usize::from(destination) + offset, value);
    }
    fn install_baby_graphics_hook(&mut self) {
        self.set_byte(0x178c, 0xf2);
        self.set_byte(0x178d, 0xa2);
        self.set_byte(0x178e, 0xa6);
    }
    fn baby_metroid_load_palette(&mut self, arguments: u16) -> u16 {
        ridley::baby_metroid_load_palette(arguments, self)
    }
    fn queue_tile_pair(&mut self, sources: [u16; 2], destinations: [u16; 2], size: u16) {
        ridley::queue_tile_pair(self, sources, destinations, size);
    }
    fn load_health_palette(&mut self, index: u16) {
        ridley::load_health_palette(index, self);
    }
    fn ridley_hover_behind(&mut self, index: isize) -> u16 {
        sm_game::ridley_data::hover_behind(self, index)
    }
    fn ridley_hover_in_front(&mut self, index: isize) -> u16 {
        sm_game::ridley_data::hover_in_front(self, index)
    }
    fn ridley_pogo_target(&mut self, index: isize) -> u16 {
        sm_game::ridley_data::pogo_target(self, index)
    }
    fn ridley_feet_x(&mut self, index: isize) -> u16 {
        sm_game::ridley_data::feet_x(self, index)
    }
    fn ridley_feet_y(&mut self, index: isize) -> u16 {
        sm_game::ridley_data::feet_y(self, index)
    }
    fn ridley_lunge_side(&mut self, index: isize) -> u16 {
        sm_game::ridley_data::lunge_side(self, index)
    }
    fn ridley_lunge_acceleration(&mut self, index: isize) -> u16 {
        sm_game::ridley_data::lunge_acceleration(self, index)
    }
    fn ridley_tail_angles(&mut self, index: isize) -> sm_data::ridley::TailAngleLimits {
        sm_game::ridley_data::tail_angles(self, index)
    }
    fn explosion_lifetime(&mut self, index: isize) -> u16 {
        sm_game::ridley_explosion_data::lifetime(self, index)
    }
    fn explosion_placement(
        &mut self,
        part: u32,
        side: isize,
    ) -> sm_data::ridley_explosion::PartPlacement {
        sm_game::ridley_explosion_data::placement(self, part, side)
    }
    fn draw_baby_metroid_and_door(&mut self) {
        ridley::draw_baby_metroid_and_door(self);
    }
    fn draw_baby_metroid(&mut self) {
        ridley::draw_baby_metroid(self);
    }
    fn draw_tail(&mut self) {
        ridley::draw_tail(self);
    }
    fn draw_wings(&mut self) {
        ridley::draw_wings(self);
    }
    fn hurt_flash(&mut self) {
        ridley::hurt_flash(self);
    }
}
