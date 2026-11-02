//! Concrete acquired boss callbacks on the exclusive prepared actor owner.
#![forbid(unsafe_code)]
use super::owned_common_lifecycle::BossState;
use super::owned_execution::{MawDispatchHost, PreparedActorState};
use crate::actors::content::ActorRoutine;
use crate::actors::deaths::Deaths;
use crate::actors::host::{ActorRuntime, Death, Shot};
use crate::actors::session_slots::session_slot;
use crate::actors::{
    ceres_ridley, draygon_death, draygon_flight, draygon_grab, draygon_sweep, ridley, spore_spawn,
    statues, yapping_maw,
};
use crate::enemy_script::EnemyScript;
use crate::geometry::Point;
use crate::room_mechanisms::host::ByteOrder;
use crate::session_access::SessionAccess;
use crate::sprite_effect_state::{LegacySpriteDefinition, LegacySpriteHandle, SpriteSpawn};
use ridley::owned::ProjectileField;

impl PreparedActorState<'_> {
    /// Full X and low-byte Y/width select the native terrain alias.
    pub(crate) fn projectile_ridley_solid(&mut self, x: u16, y: u16) -> u8 {
        let row = crate::display_refresh::mult8x8(
            (y >> 4).to_le_bytes()[0],
            self.session().room.width.to_le_bytes()[0],
        );
        let offset = usize::try_from(2 * (i32::from(row) + (i32::from(x) >> 4)))
            .expect("Ridley level offset");
        u8::from(crate::room_terrain::level_word(self, offset, ByteOrder::Native) & 0xf000 != 0)
    }
    fn acquired_sprite(&mut self, x: u16, y: u16, kind: u16, appearance: u16) -> u16 {
        match crate::sprite_objects::spawn(
            self,
            SpriteSpawn {
                position: Point { x, y },
                definition: LegacySpriteDefinition(kind),
                appearance,
            },
        ) {
            crate::sprite_effect_state::SpawnResult::Allocated(slot) => u16::from(slot.0) * 2,
            crate::sprite_effect_state::SpawnResult::Full => 0xffff,
        }
    }
    fn acquired_colour(&self, index: usize) -> u16 {
        self.current_word(0xc000 + 2 * index)
    }
    fn acquired_set_colour(&mut self, index: usize, value: u16) {
        self.presentation
            .palette_word(
                &mut self.owner.gameplay,
                isize::try_from(index).expect("palette index"),
                value,
                self.aliases.presentation_diagnostics(),
            )
            .expect("admitted actor palette");
    }
    /// A byte producer retains its observable partial palette prefix.
    fn acquired_palette_byte(&mut self, at: usize, value: u8) {
        crate::scene::actor_presentation::write_byte(
            &mut self.owner.gameplay,
            self.aliases.presentation_diagnostics(),
            at,
            value,
        )
        .expect("admitted palette byte");
    }
    fn acquired_palette_copy(&mut self, destination: u16, bank: u8, address: u16, colours: u16) {
        let (mut destination, mut address, mut remaining) =
            (destination, address, i32::from(colours));
        loop {
            let value = self.native_actor_record(bank, address).word(0);
            self.acquired_set_colour(usize::from(destination >> 1), value);
            destination = destination.wrapping_add(2);
            address = address.wrapping_add(2);
            remaining -= 1;
            if remaining == 0 {
                break;
            }
        }
    }
    fn acquired_queue(&mut self, bank: u8, source: u16, size: u16, destination: u16) {
        let cursor = self.session().queued_tilemaps.tail;
        for (field, value) in [(0, size), (2, source)] {
            self.presentation
                .queue_word(
                    &mut self.owner.gameplay,
                    cursor,
                    field,
                    value,
                    self.aliases.presentation_diagnostics(),
                )
                .expect("admitted actor transfer");
        }
        self.presentation
            .queue_byte(
                &mut self.owner.gameplay,
                usize::from(cursor) + 4,
                bank,
                self.aliases.presentation_diagnostics(),
            )
            .expect("admitted actor transfer bank");
        self.presentation
            .queue_word(
                &mut self.owner.gameplay,
                cursor,
                5,
                destination,
                self.aliases.presentation_diagnostics(),
            )
            .expect("admitted actor transfer destination");
        self.presentation
            .publish_queue_tail(&mut self.owner.gameplay, cursor.wrapping_add(7))
            .expect("admitted actor transfer tail");
    }
    fn acquired_draw(
        &mut self,
        bank: u8,
        map: u16,
        position: Point,
        palette: u16,
        clip: crate::scene::objects::Clip,
    ) {
        self.presentation
            .draw_sprite(
                &mut self.owner.gameplay,
                crate::scene::actor_presentation::SpriteRequest {
                    bank,
                    map,
                    position,
                    palette,
                    base_tile: 0,
                    clip,
                },
                self.aliases.presentation_diagnostics(),
            )
            .expect("admitted boss sprite");
    }
    fn draygon_health_colours(&mut self, native: bool) {
        use crate::actors::draygon_host::{BODY, Draygon};
        let mut cursor = session_slot!(self, Draygon[BODY].health_palette).wrapping_mul(4);
        for index in 89..93 {
            let value = crate::actor_tables::word(
                self,
                |t| &t.draygon_health_palettes,
                isize::try_from(cursor >> 1).expect("signed actor index"),
            );
            if native {
                let bytes = value.to_ne_bytes();
                self.acquired_palette_byte(0xc000 + 2 * index, bytes[0]);
                self.acquired_palette_byte(0xc001 + 2 * index, bytes[1]);
            } else {
                self.acquired_set_colour(index, value);
            }
            cursor = cursor.wrapping_add(2);
        }
    }
    fn draygon_hurt_copy(&mut self, default: u16, target: usize) {
        use crate::actors::draygon_host::Draygon;
        let k = i32::from(self.session().actors.controls.selected);
        let address = if session_slot!(self, Draygon[k].base.flash_timer) & 2 != 0 {
            0xa297
        } else {
            default
        };
        let record = self.native_actor_record(0xa5, address);
        for offset in 0..32 {
            let value = record.byte(offset);
            self.acquired_palette_byte(target + offset, value);
        }
    }
}

impl draygon_grab::owned::Host for PreparedActorState<'_> {
    fn actor_record(&mut self, bank: u8, address: u16) -> sm_data::cartridge::RomRecord {
        self.native_actor_record(bank, address)
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
    fn direction_step(&mut self, phase: u16, amplitude: u16) -> crate::geometry::Vector32 {
        self.ordinary_direction_vector(phase, amplitude)
    }
    fn step_along(&mut self, phase: u16, vector: crate::geometry::Vector32) {
        crate::actors::movement::apply_vector(self, phase, vector);
    }
    fn scaled_cosine(&mut self, phase: u16, amplitude: u16) -> u16 {
        BossState::scaled_cosine(self, phase, amplitude)
    }
    fn scaled_sine(&mut self, phase: u16, amplitude: u16) -> u16 {
        BossState::scaled_sine(self, phase, amplitude)
    }
    fn command_player(&mut self, command: u16) -> u16 {
        self.projectile_player_command(command)
    }
    fn grab_samus(&mut self, direction: u16) {
        crate::player_movement::forced::grab_by_draygon(direction, self);
    }
    fn release_samus(&mut self) {
        crate::player_movement::forced::release_from_draygon(self);
    }
    fn correct_previous_position(&mut self) {
        crate::player_position::correct_previous_position(self);
    }
    fn suit_damage(&mut self, amount: u16) -> u16 {
        crate::actors::combat::suit_damage(self, amount)
    }
    fn damage_player(&mut self, amount: u16) {
        crate::enemy_projectiles::host::Effects::damage_player(self, amount);
    }
    fn create_sprite(&mut self, x: u16, y: u16, kind: u16, appearance: u16) -> u16 {
        self.acquired_sprite(x, y, kind, appearance)
    }
}

impl draygon_flight::owned::Host for PreparedActorState<'_> {
    fn frame(&self) -> u16 {
        BossState::frame(self)
    }
    fn roll_random(&mut self) -> u16 {
        BossState::roll_random(self)
    }
    fn turret_position(&mut self, index: isize) -> u16 {
        crate::actor_tables::word(self, |tables| &tables.draygon_turret_positions, index)
    }
    fn place_projectile_spawn(&mut self, x: u16, y: u16) {
        self.session_mut().enemy_projectiles.spawn.point = crate::geometry::Point { x, y };
    }
    fn spawn_actor_eproj(&mut self, parameter: u16, key: u16, definition: u16) -> u16 {
        crate::enemy_projectiles::lifecycle::spawn_from_enemy(self, parameter, key, definition)
    }
}

impl draygon_sweep::owned::Host for PreparedActorState<'_> {
    fn body_bubble(&mut self, turret: Option<u16>) {
        draygon_flight::owned::body_bubble(turret, self);
    }
    fn move_body_horizontally(&mut self, right: bool) {
        draygon_flight::owned::move_horizontally(right, self);
    }
    fn swoop_table_overflow(&mut self) -> ! {
        self.unknown_function()
    }
    fn imported_swoop_byte(&self, relative: usize) -> u8 {
        self.current_byte(0x9000 + relative)
    }
    fn set_imported_swoop_byte(&mut self, relative: usize, value: u8) {
        self.set_current_byte(0x9000 + relative, value);
    }
}

impl draygon_death::owned::Host for PreparedActorState<'_> {
    fn apply_hurt_palettes(&mut self) {
        use crate::actors::draygon_host::Draygon;
        self.draygon_hurt_copy(0xa277, 0xc0a0);
        let k = i32::from(self.session().actors.controls.selected);
        if session_slot!(self, Draygon[k].base.flash_timer) & 2 == 0 {
            self.draygon_health_colours(false);
        }
        self.draygon_hurt_copy(0xa1f7, 0xc1e0);
    }
    fn update_health_palette(&mut self) {
        use crate::actors::draygon_host::{BODY, Draygon};
        let mut cursor = 0i32;
        while i16::from_le_bytes(session_slot!(self, Draygon[BODY].base.health).to_le_bytes())
            .wrapping_sub(i16::from_le_bytes(
                crate::actor_tables::word(
                    self,
                    |t| &t.draygon_health_thresholds,
                    isize::try_from(cursor >> 1).expect("health threshold cursor"),
                )
                .to_le_bytes(),
            ))
            < 0
        {
            cursor += 2;
        }
        if cursor != i32::from(session_slot!(self, Draygon[BODY].health_palette)) {
            let low = u16::from_le_bytes(
                cursor.to_le_bytes()[..2]
                    .try_into()
                    .expect("health cursor low word"),
            );
            session_slot!(self, Draygon[BODY].health_palette = low);
            self.draygon_health_colours(true);
        }
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
        crate::sprite_effect_controllers::draygon_spawn_ring(self);
    }
    fn draygon_motion(&mut self, bias: u16) {
        crate::sprite_effect_controllers::draygon_motion(self, bias);
    }
    fn queue_music_delayed(&mut self, track: u16) {
        crate::enemy_projectiles::host::Effects::queue_music(self, track);
    }
    fn boss_word(&mut self, area: u16) -> u16 {
        crate::progress::boss_word_at(self, area, crate::progress::BossWordOrder::Native)
    }
    fn set_boss_word(&mut self, area: u16, value: u16) {
        crate::progress::set_boss_word_native(self, area, value);
    }
    fn scatter_draygon_drop(&mut self, area: u16) {
        Deaths::scatter_enemy_drop(
            self,
            crate::enemy_projectiles::drops::DropSource::Draygon,
            area,
        );
    }
    fn retire_all_sprites(&mut self) {
        crate::sprite_objects::retire_all(self.session_mut());
    }
}

impl ridley::owned::Host for PreparedActorState<'_> {
    fn actor_record(&mut self, bank: u8, address: u16) -> sm_data::cartridge::RomRecord {
        self.native_actor_record(bank, address)
    }
    fn run_ceres_ridley_function(&mut self, function: u16) {
        ceres_ridley::owned::run_function(self, function);
    }
    fn frame(&self) -> u16 {
        BossState::frame(self)
    }
    fn roll_random(&mut self) -> u16 {
        BossState::roll_random(self)
    }
    fn place_projectile_spawn(&mut self, x: u16, y: u16) {
        self.session_mut().enemy_projectiles.spawn.point = crate::geometry::Point { x, y };
    }
    fn spawn_room_projectile(&mut self, definition: u16, parameter: u16) -> u16 {
        crate::enemy_projectiles::lifecycle::spawn_room(self, definition, parameter)
    }
    fn spawn_population(&mut self, bank: u8, population: u16) -> u16 {
        self.lifecycle_spawn_template(bank, population)
    }
    fn set_boss_bit(&mut self, mask: u16) {
        crate::progress::mark_boss_defeated(self, mask);
    }
    fn hardware_sine_cosine(&mut self, amplitude: u16, angle: u16) -> (u16, u16) {
        let sine = self.acquired_mode7_multiply(amplitude, angle);
        let cosine = self.acquired_mode7_multiply(amplitude, angle.wrapping_add(64));
        (sine, cosine)
    }
    fn move_liquid(&mut self, target: u16, velocity: u16, delay: u16) {
        let room = &mut self.session_mut().room;
        room.liquid_target = target;
        room.liquid_velocity = velocity;
        room.liquid_delay = delay;
    }
    fn command_player(&mut self, command: u16) -> u16 {
        self.projectile_player_command(command)
    }
    fn normal_death(&mut self, animation: u16) {
        Deaths::normal_death(self, animation);
    }
    fn scatter_enemy_drop(
        &mut self,
        source: crate::enemy_projectiles::drops::DropSource,
        key: u16,
    ) {
        Deaths::scatter_enemy_drop(self, source, key);
    }
    fn queue_music_delayed(&mut self, track: u16) {
        crate::enemy_projectiles::host::Effects::queue_music(self, track);
    }
    fn suit_damage(&mut self, amount: u16) -> u16 {
        crate::actors::combat::suit_damage(self, amount)
    }
    fn damage_player(&mut self, amount: u16) {
        crate::enemy_projectiles::host::Effects::damage_player(self, amount);
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
        self.unknown_function()
    }
    fn imported_tail_byte(&self, relative: usize) -> u8 {
        self.current_byte(0x2000 + relative)
    }
    fn set_imported_tail_byte(&mut self, relative: usize, value: u8) {
        self.set_current_byte(0x2000 + relative, value);
    }
    fn imported_projectile_field(&self, index: usize, field: ProjectileField) -> u16 {
        self.current_word(projectile_address(field) + 2 * index)
    }
    fn set_imported_projectile_field(&mut self, index: usize, field: ProjectileField, value: u16) {
        let at = projectile_address(field) + 2 * index;
        for (lane, byte) in value.to_le_bytes().into_iter().enumerate() {
            self.set_current_byte(at + lane, byte);
        }
    }
    fn tail_tip_explosion_list(&mut self, index: isize) -> u16 {
        crate::cartridge::NativeImage::native_image(self)
            .table_word(sm_data::cartridge::lorom_offset(0xa6_c7ba) as usize, index)
    }
    fn pogo_row(&mut self, horizontal: bool, index: isize) -> u16 {
        if horizontal {
            crate::actor_tables::word(self, |t| &t.ridley_x_velocities, index)
        } else {
            crate::actor_tables::word(self, |t| &t.ridley_y_velocities, index)
        }
    }
    fn pogo_acceleration(&mut self, upwards: bool, index: isize) -> u16 {
        if upwards {
            crate::actor_tables::word(self, |t| &t.ridley_upwards, index)
        } else {
            crate::actor_tables::word(self, |t| &t.ridley_downwards, index)
        }
    }
    fn set_imported_tail_destination_byte(&mut self, destination: u16, offset: usize, value: u8) {
        self.set_current_byte(usize::from(destination) + offset, value);
    }
    fn install_baby_graphics_hook(&mut self) {
        self.acquired_graphics_hook(0xa6, 0xa2f2);
    }
    fn baby_metroid_load_palette(&mut self, arguments: u16) -> u16 {
        let colours = self.native_actor_record(0xa6, arguments).word(0);
        self.acquired_palette_copy(0x162, 0xa6, colours, 0xf);
        arguments.wrapping_add(2)
    }
    fn queue_tile_pair(&mut self, sources: [u16; 2], destinations: [u16; 2], size: u16) {
        for (source, destination) in sources.into_iter().zip(destinations) {
            self.acquired_queue(0xb0, source, size, destination);
        }
        let tail = usize::from(self.session().queued_tilemaps.tail);
        self.presentation
            .queue_byte(
                &mut self.owner.gameplay,
                tail,
                0,
                self.aliases.presentation_diagnostics(),
            )
            .expect("tile pair terminator");
        self.presentation
            .queue_byte(
                &mut self.owner.gameplay,
                tail + 1,
                0,
                self.aliases.presentation_diagnostics(),
            )
            .expect("tile pair terminator");
    }
    fn load_health_palette(&mut self, index: u16) {
        if self.session().presentation.visual().palette_fade.step == 0 {
            self.acquired_palette_copy(
                0x1e2,
                0xa6,
                0xe46au16.wrapping_add(index.wrapping_mul(28)),
                0xe,
            );
        }
    }
    fn ridley_hover_behind(&mut self, index: isize) -> u16 {
        crate::ridley_data::hover_behind(self, index)
    }
    fn ridley_hover_in_front(&mut self, index: isize) -> u16 {
        crate::ridley_data::hover_in_front(self, index)
    }
    fn ridley_pogo_target(&mut self, index: isize) -> u16 {
        crate::ridley_data::pogo_target(self, index)
    }
    fn ridley_feet_x(&mut self, index: isize) -> u16 {
        crate::ridley_data::feet_x(self, index)
    }
    fn ridley_feet_y(&mut self, index: isize) -> u16 {
        crate::ridley_data::feet_y(self, index)
    }
    fn ridley_lunge_side(&mut self, index: isize) -> u16 {
        crate::ridley_data::lunge_side(self, index)
    }
    fn ridley_lunge_acceleration(&mut self, index: isize) -> u16 {
        crate::ridley_data::lunge_acceleration(self, index)
    }
    fn ridley_tail_angles(&mut self, index: isize) -> sm_data::ridley::TailAngleLimits {
        crate::ridley_data::tail_angles(self, index)
    }
    fn explosion_lifetime(&mut self, index: isize) -> u16 {
        crate::ridley_explosion_data::lifetime(self, index)
    }
    fn explosion_placement(
        &mut self,
        part: u32,
        side: isize,
    ) -> sm_data::ridley_explosion::PartPlacement {
        crate::ridley_explosion_data::placement(self, part, side)
    }
    fn draw_baby_metroid_and_door(&mut self) {
        self.acquired_draw_baby_and_door();
    }
    fn draw_baby_metroid(&mut self) {
        self.acquired_draw_baby();
    }
    fn draw_tail(&mut self) {
        self.acquired_draw_tail();
    }
    fn draw_wings(&mut self) {
        self.acquired_draw_wings();
    }
    fn hurt_flash(&mut self) {
        use ridley::Ridley;
        let mut palette = 3584;
        if session_slot!(self, Ridley[0].base.flash_timer) >= 2
            && self.session().actors.controls.phase_counter.wrapping_add(1) & 2 != 0
        {
            palette = 0;
        }
        session_slot!(self, Ridley[0].palette = palette);
    }
}

fn projectile_address(field: ProjectileField) -> usize {
    match field {
        ProjectileField::X => 0xb64,
        ProjectileField::Y => 0xb78,
        ProjectileField::XRadius => 0xbb4,
        ProjectileField::YRadius => 0xbc8,
        ProjectileField::Kind => 0xc18,
        ProjectileField::Direction => 0xc04,
    }
}
impl MawDispatchHost for PreparedActorState<'_> {
    fn unknown_maw_function(&mut self) -> ! {
        self.unknown_function()
    }
}
impl spore_spawn::owned_return::Host for PreparedActorState<'_> {
    fn update_spore_stem(&mut self) {
        crate::enemy_projectiles::attachments_spore_maw::update_spore_stem(self);
    }
}
impl yapping_maw::owned::Host for PreparedActorState<'_> {
    fn create_sprite(&mut self, x: u16, y: u16, kind: u16, appearance: u16) -> u16 {
        self.acquired_sprite(x, y, kind, appearance)
    }
    fn spawn_projectile(&mut self, parameter: u16, key: u16, definition: u16) -> u16 {
        crate::enemy_projectiles::lifecycle::spawn_from_enemy(self, parameter, key, definition)
    }
    fn distance_and_direction(&mut self, origin: Point, target: Point) -> (u16, u16) {
        let pair = crate::actors::queries::distance_and_direction(self, origin, target);
        (pair.distance, pair.direction)
    }
    fn command_player(&mut self, command: u16) {
        self.projectile_player_command(command);
    }
    fn correct_previous_position(&mut self) {
        crate::player_position::correct_previous_position(self);
    }
    fn write_maw_operand(&mut self, operand: u8) {
        self.acquired_multiplicand(operand);
    }
    fn enemy_sine(&mut self, index: isize) -> u16 {
        crate::gameplay_tables::enemy_sine(self, index)
    }
    fn attack_list(&mut self, index: isize) -> u16 {
        crate::actor_tables::word(self, |t| &t.maw_attack_lists, index)
    }
    fn attachment_offset(&mut self, y: bool, native: bool, index: isize) -> u16 {
        let address = if y { 0xa8a0a9 } else { 0xa8a0a7 };
        if native {
            self.allocations.native().table_word(
                usize::try_from(sm_data::cartridge::lorom_offset(address))
                    .expect("Maw native table"),
                index,
            )
        } else {
            self.loaded_table_word(address, index)
        }
    }
    fn enemy_speed(&self, table: sm_data::enemy_speeds::SpeedTable, index: isize) -> u16 {
        self.speed_word(table, index)
    }
    fn update_maw_visibility(&mut self, key: i32) {
        use yapping_maw::Maw;
        let outside = u16::from(crate::actors::queries::current_outside_screen(self));
        session_slot!(
            self,
            Maw[key].off_screen = u16::from_le_bytes(outside.to_ne_bytes())
        );
    }
    fn update_maw_segment(&mut self, part: u8) {
        crate::enemy_projectiles::attachments_spore_maw::update_maw_segment(part, self);
    }
    fn maw_appearance(&mut self, handle: u16, appearance: u16, key: u16) {
        crate::sprite_effect_controllers::maw_appearance(
            self,
            LegacySpriteHandle(handle),
            appearance,
            key,
        );
    }
    fn shot_reaction(&mut self, shot: Shot) {
        ActorRuntime::shot_reaction(self, shot);
    }
    fn free_projectile(&mut self, key: u16) {
        if key & 1 == 0 && usize::from(key >> 1) < 18 {
            self.session_mut().enemy_projectiles.slots[usize::from(key >> 1)].definition = 0;
        } else {
            self.write_current_slice(0x1997 + usize::from(key), &0u16.to_ne_bytes());
        }
    }
    fn retire_sprite(&mut self, handle: u16) {
        crate::sprite_effect_controllers::retire(self, LegacySpriteHandle(handle));
    }
    fn update_frozen_enemy(&mut self) {
        self.update_current_frozen_ai();
    }
    fn update_maw_palette(&mut self, handle: u16) {
        crate::enemy_projectiles::attachments_spore_maw::update_maw_palette(handle, self);
    }
}
impl crate::escape_typewriter::owned::Host for PreparedActorState<'_> {
    fn text_record(&mut self, cursor: u16) -> sm_data::cartridge::RomRecord {
        self.native_actor_record(0xa6, cursor)
    }
    fn colour(&self, index: usize) -> u16 {
        self.acquired_colour(index)
    }
    fn set_colour(&mut self, index: usize, value: u16) {
        self.acquired_set_colour(index, value);
    }
    fn set_bg2_scroll(&mut self, horizontal: bool, value: u16) {
        self.session_mut()
            .presentation
            .owner_mut()
            .expect("owned text scroll")
            .state
            .controls
            .scroll_words[2 + usize::from(!horizontal)] = value;
    }
    fn area_index(&self) -> u16 {
        self.session().room.area
    }
    fn queue_typewriter_tile(&mut self, address: u16) {
        self.acquired_queue(0x7e, 0x8034, 2, address);
    }
}

const ACQUIRED_FACING_FORWARD: u16 = 1;

impl PreparedActorState<'_> {
    fn acquired_graphics_hook(&mut self, bank: u8, address: u16) {
        let [low, high] = address.to_le_bytes();
        self.session_mut().actors.drawing.hook_address[0] = low;
        self.session_mut().actors.drawing.hook_address[1] = high;
        self.session_mut().actors.drawing.hook_address[2] = bank;
    }
    fn acquired_native_graphics_hook(&mut self, bank: u8, address: u16) {
        let bank = u16::from(bank).to_ne_bytes();
        self.session_mut().actors.drawing.hook_address[2] = bank[0];
        self.session_mut().actors.drawing.hook_address[3] = bank[1];
        let address = address.to_ne_bytes();
        self.session_mut().actors.drawing.hook_address[0] = address[0];
        self.session_mut().actors.drawing.hook_address[1] = address[1];
    }
    fn acquired_bg_mode(&mut self, value: u8) {
        crate::display::preparation::set_display_byte(
            &mut self
                .session_mut()
                .presentation
                .owner_mut()
                .expect("owned boss display")
                .state,
            crate::display::preparation::Field::Mode,
            value,
        );
    }
    fn acquired_mode7_settings(&mut self, value: u8) {
        crate::display::preparation::set_display_byte(
            &mut self
                .session_mut()
                .presentation
                .owner_mut()
                .expect("owned boss display")
                .state,
            crate::display::preparation::Field::AffineSelection,
            value,
        );
    }
    fn acquired_affine(&mut self, index: usize, value: u16) {
        use crate::display::preparation::Field;
        let (low, high) = [
            (Field::AffineWords0Low, Field::AffineWords0High),
            (Field::AffineWords1Low, Field::AffineWords1High),
            (Field::AffineWords2Low, Field::AffineWords2High),
            (Field::AffineWords3Low, Field::AffineWords3High),
            (Field::AffineWords4Low, Field::AffineWords4High),
            (Field::AffineWords5Low, Field::AffineWords5High),
        ][index];
        let bytes = value.to_le_bytes();
        crate::display::preparation::set_display_byte(
            &mut self
                .session_mut()
                .presentation
                .owner_mut()
                .expect("owned boss affine")
                .state,
            low,
            bytes[0],
        );
        crate::display::preparation::set_display_byte(
            &mut self
                .session_mut()
                .presentation
                .owner_mut()
                .expect("owned boss affine")
                .state,
            high,
            bytes[1],
        );
    }
    fn acquired_bg1_scroll(&mut self, horizontal: bool, value: u16) {
        self.session_mut()
            .presentation
            .owner_mut()
            .expect("owned Ceres scroll")
            .state
            .controls
            .scroll_words[usize::from(!horizontal)] = value;
    }
    fn acquired_target_colour(&mut self, index: usize, value: u16) {
        // Presentation owns both endpoints. Out-of-range selectors retain
        // aggregate aliases, with the same low/high prefix as other stores.
        let at = 0xc200usize
            .checked_add(index.checked_mul(2).expect("target colour extent"))
            .expect("target colour address");
        for (lane, byte) in value.to_le_bytes().into_iter().enumerate() {
            crate::scene::actor_presentation::write_byte(
                &mut self.owner.gameplay,
                self.aliases.presentation_diagnostics(),
                at + lane,
                byte,
            )
            .expect("owned target colour");
        }
    }
    fn acquired_target_copy(
        &mut self,
        bank: u8,
        mut destination: u16,
        mut address: u16,
        colours: u16,
    ) {
        let mut remaining = i32::from(colours);
        loop {
            let value = self.native_actor_record(bank, address).word(0);
            self.acquired_target_colour(usize::from(destination >> 1), value);
            destination = destination.wrapping_add(2);
            address = address.wrapping_add(2);
            remaining -= 1;
            if remaining == 0 {
                break;
            }
        }
    }
    fn acquired_target_descending(
        &mut self,
        bank: u8,
        address: u16,
        destination: usize,
        colours: usize,
    ) {
        // Capture the loaded ROM endpoint before the descending transfer.
        let image = self.allocations.loaded().clone();
        let source = usize::try_from(sm_data::cartridge::lorom_offset(
            (u32::from(bank) << 16) | u32::from(address),
        ))
        .expect("palette endpoint");
        for offset in (0..colours).rev() {
            let at = source + 2 * offset;
            let value = u16::from_ne_bytes([image.read(at), image.read(at + 1)]);
            self.acquired_target_colour(destination - colours + 1 + offset, value);
        }
    }
    fn acquired_draw_baby(&mut self) {
        use ridley::Ridley;
        let Some(map) = ridley::owned::baby_metroid_spritemap(self) else {
            return;
        };
        let x = session_slot!(self, Ridley[0].baby_metroid_x);
        let y = session_slot!(self, Ridley[0].baby_metroid_y);
        self.acquired_draw_room_sprite(0xa6, map, Point { x, y }, 0);
    }
    fn acquired_draw_baby_and_door(&mut self) {
        use ridley::CeresDoor;
        if self.session().room.ceres_status == 0 {
            self.acquired_draw_baby();
        }
        if session_slot!(self, CeresDoor[0x40].drawn_by_ridley) != 0 {
            let offset = [0u16, 0xfc00, 0xfffc, 0xffff]
                [usize::from(self.session().feedback.earthquake_timer & 3)];
            let x = session_slot!(self, CeresDoor[0x40].base.x_pos).wrapping_add(offset);
            let y = session_slot!(self, CeresDoor[0x40].base.y_pos);
            self.acquired_draw(
                0xa6,
                0xa329,
                Point { x, y },
                1024,
                crate::scene::objects::Clip::Bounded,
            );
        }
    }
    fn acquired_draw_wings(&mut self) {
        use ridley::Ridley;
        let side = match session_slot!(self, Ridley[0].facing) {
            ridley::FACING_LEFT => 0,
            ACQUIRED_FACING_FORWARD => return,
            _ => 10,
        };
        let frame = isize::try_from(session_slot!(self, Ridley[0].wings_frame))
            .expect("signed actor index")
            + side;
        let map = crate::ridley_data::wing_spritemap(self, frame);
        let x = session_slot!(self, Ridley[0].base.x_pos);
        let y = session_slot!(self, Ridley[0].base.y_pos);
        let palette = session_slot!(self, Ridley[0].palette);
        self.acquired_draw_room_sprite(0xa6, map, Point { x, y }, palette);
    }
    fn acquired_tail_word(&self, index: usize) -> u16 {
        let low = ridley::owned::Host::tail_byte(self, 2 * index);
        let high = ridley::owned::Host::tail_byte(self, 2 * index + 1);
        u16::from_le_bytes([low, high])
    }
    fn acquired_draw_tail_part(&mut self, map: u16, segment: usize) {
        use ridley::Ridley;
        let x = self.acquired_tail_word(ridley::tail::x(segment));
        let y = self.acquired_tail_word(ridley::tail::y(segment));
        let palette = session_slot!(self, Ridley[0].palette);
        self.acquired_draw_room_sprite(0xa6, map, Point { x, y }, palette);
    }
    fn acquired_draw_tail(&mut self) {
        use ridley::Ridley;
        if session_slot!(self, Ridley[0].base.properties)
            & u16::try_from(crate::constants::enemy_properties::K_ENEMY_PROPS_INVISIBLE)
                .expect("invisible property")
            != 0
        {
            return;
        }
        let sector = (u16::from(ridley::owned::Host::tail_byte(
            self,
            2 * ridley::tail::angle(5),
        )) + u16::from(ridley::owned::Host::tail_byte(
            self,
            2 * ridley::tail::angle(6),
        )) + 8)
            & 0xf0;
        let tip = crate::ridley_data::tail_tip_spritemap(
            self,
            isize::try_from(sector >> 4).expect("signed actor index"),
        );
        self.acquired_draw_tail_part(tip, 6);
        for segment in (0..6).rev() {
            let map = [0xdc90, 0xdc97, 0xdc9e][segment / 2];
            self.acquired_draw_tail_part(map, segment);
        }
    }
    fn acquired_rom_queue_copy(&mut self, record: &sm_data::cartridge::RomRecord, tail: usize) {
        for field in [0, 3, 2, 5] {
            let bytes = record.word(field).to_ne_bytes();
            for (lane, byte) in bytes.into_iter().enumerate() {
                self.presentation
                    .queue_byte(
                        &mut self.owner.gameplay,
                        tail + field + lane,
                        byte,
                        self.aliases.presentation_diagnostics(),
                    )
                    .expect("admitted ROM queue entry");
            }
        }
    }
    fn acquired_rom_queue_push(&mut self, record: &sm_data::cartridge::RomRecord) {
        let tail = self.session().queued_tilemaps.tail;
        self.acquired_rom_queue_copy(record, usize::from(tail));
        self.presentation
            .publish_queue_tail(&mut self.owner.gameplay, tail.wrapping_add(7))
            .expect("ROM queue tail");
    }
    fn acquired_escape_timer_tiles(&mut self) -> bool {
        use crate::actors::layouts::EnemyLayout8 as EnemyData;
        let cursor = session_slot!(self, EnemyData[0].var_e);
        let record = self.native_actor_record(0xa6, cursor);
        if record.word(0) != 0 {
            self.acquired_rom_queue_push(&record);
            session_slot!(self, EnemyData[0].var_e = cursor.wrapping_add(7));
            if self
                .native_actor_record(0xa6, cursor.wrapping_add(7))
                .word(0)
                != 0
            {
                return false;
            }
        }
        true
    }
    fn acquired_escape_tile_table(&mut self) {
        let (mut cursor, mut tail) = (0xc3b8u16, usize::from(self.session().queued_tilemaps.tail));
        loop {
            let record = self.native_actor_record(0xa6, cursor);
            if record.word(0) == 0 {
                break;
            }
            self.acquired_rom_queue_copy(&record, tail);
            cursor = cursor.wrapping_add(7);
            tail += 7;
        }
        let tail = u16::from_le_bytes(
            usize::to_le_bytes(tail)[..2]
                .try_into()
                .expect("tail low word"),
        );
        self.presentation
            .publish_queue_tail(&mut self.owner.gameplay, tail)
            .expect("table queue tail");
    }
    fn acquired_emergency_glow(&mut self) {
        use ridley::Ridley;
        if self.session().room.entry.enemy_interlock
            | self.session().presentation.visual().palette_fade.step
            == 0
            && BossState::frame(self) & 3 == 0
        {
            let mut step = session_slot!(self, Ridley[0].emergency_glow_index).wrapping_add(1);
            if step >= 16 {
                step = 0;
            }
            session_slot!(self, Ridley[0].emergency_glow_index = step);
            let colour = isize::try_from(session_slot!(self, Ridley[0].emergency_glow_index))
                .expect("signed actor index")
                + 2 * isize::try_from(step).expect("signed actor index");
            for offset in 0..3 {
                let value = crate::ridley_data::emergency_glow(
                    self,
                    colour + isize::try_from(offset).expect("glow colour"),
                );
                self.acquired_set_colour(97 + offset, value);
            }
        }
    }
    fn acquired_ceres_health_palette(&mut self) {
        use ridley::Ridley;
        if session_slot!(self, Ridley[0].fight_mode) != 0
            && session_slot!(self, Ridley[0].hit_counter) >= 0x32
        {
            let index = if session_slot!(self, Ridley[0].hit_counter) >= 0x46 {
                2
            } else {
                0
            };
            ridley::owned::Host::load_health_palette(self, index);
        }
    }
}

impl statues::owned::Host for PreparedActorState<'_> {
    fn tourian_statue(&self, index: isize) -> sm_data::statues::Statue {
        crate::statue_tables::tourian_statue(self, index)
    }
    fn chozo_statue(&self, index: isize) -> sm_data::statues::Statue {
        crate::statue_tables::chozo_statue(self, index)
    }
    fn statue_movement(&self, index: isize) -> sm_data::statues::Movement {
        crate::statue_tables::movement(self, index)
    }
    fn install_tourian_palettes(&mut self) {
        statues::install_tourian_palettes(self);
    }
    fn install_tube_palette(&mut self) {
        statues::install_tube_palette(self);
    }
    fn install_chozo_palettes(&mut self, lower_norfair: bool) {
        statues::install_chozo_palettes(lower_norfair, self);
    }
    fn boss_word_native(&mut self, area: u16) -> u16 {
        crate::progress::boss_word_at(self, area, crate::progress::BossWordOrder::Native)
    }
    fn start_lowering_acid(&mut self) {
        let room = &mut self.session_mut().room;
        room.liquid_delay = 32;
        room.liquid_velocity = 64;
    }
    fn lower_acid(&mut self) {
        crate::room_mechanisms::rules::hands::lower_acid(self);
    }
    fn select_block_at(&mut self, x: u16, y: u16) {
        crate::actors::movement::select_block_at(self, Point { x, y });
    }
    fn spawn_collision_plm(&mut self, header: u16) {
        crate::room_mechanisms::creation::spawn(
            self,
            crate::room_mechanisms::host::SpawnSource::Collision { header },
            &mut crate::room_mechanisms::host::Contact::Absent,
        );
    }
    fn spawn_room_eproj(&mut self, definition: u16, parameter: u16) -> u16 {
        crate::enemy_projectiles::lifecycle::spawn_room(self, definition, parameter)
    }
    fn spawn_plm(&mut self, x: u16, y: u16, header: u16) {
        crate::room_mechanisms::creation::spawn(
            self,
            crate::room_mechanisms::host::SpawnSource::Hardcoded {
                x: x.to_le_bytes()[0],
                y: y.to_le_bytes()[0],
                header,
            },
            &mut crate::room_mechanisms::host::Contact::Absent,
        );
    }
    fn command_player(&mut self, command: u16) {
        self.projectile_player_command(command);
    }
    fn invalid_function(&mut self, _address: u16) -> ! {
        self.unknown_function()
    }
}

impl draygon_flight::entry::Host for PreparedActorState<'_> {
    fn queue_intro_transfer(&mut self) {
        self.acquired_queue(0xb1, 0x9400, 0x600, 0x6d00);
    }
    fn spawn_intro_trails(&mut self) {
        crate::sprite_effect_controllers::draygon_spawn_trails(self);
    }
    fn step_intro_trajectory(&mut self) {
        let cursor = *self.session().actors.extended.word(2, 6);
        let cursor = crate::sprite_effect_controllers::draygon_trajectory(self, cursor);
        *self.session_mut().actors.extended.word_mut(2, 6) = cursor;
    }
    fn install_initial_palette(&mut self) {
        self.acquired_target_descending(0xa5, 0xa217, 168, 25);
    }
    fn install_bg2_scroll_hook(&mut self) {
        self.acquired_native_graphics_hook(0xa5, 0x9342);
    }
    fn install_layers(&mut self) {
        self.acquired_install_draygon_layers();
    }
    fn disable_minimap_for_boss(&mut self) {
        crate::player_update::minimap::disable_for_boss(self);
    }
    fn dispatch_draygon_other(&mut self, address: u32, key: u16) {
        match address {
            10863757 => crate::actors::draygon_eye::owned::look_at_samus(key, false, self),
            10863891 => crate::actors::draygon_eye::owned::look_at_samus(key, true, self),
            10846283 | 10873626 => {}
            10873627 => spore_spawn::owned_return::descend(self),
            10873682 => spore_spawn::owned_return::orbit(key, self),
            10873755 => spore_spawn::owned_return::begin_return(self),
            10873838 => spore_spawn::owned_return::finish_return(self),
            _ => self.unknown_function(),
        }
    }
    fn set_bg2_scroll(&mut self, horizontal: bool, value: u16) {
        self.session_mut()
            .presentation
            .owner_mut()
            .expect("owned boss scroll")
            .state
            .controls
            .scroll_words[2 + usize::from(!horizontal)] = value;
    }
}

impl ceres_ridley::owned::Host for PreparedActorState<'_> {
    fn boss_byte(&mut self, area: u16) -> u8 {
        crate::progress::boss_byte_at(self, area)
    }
    fn disable_minimap_for_boss(&mut self) {
        crate::player_update::minimap::disable_for_boss(self);
    }
    fn copy_palette(&mut self, destination: u16, bank: u8, address: u16, colours: u16) {
        self.acquired_palette_copy(destination, bank, address, colours);
    }
    fn copy_target_palette(&mut self, bank: u8, destination: u16, address: u16, colours: u16) {
        self.acquired_target_copy(bank, destination, address, colours);
    }
    fn clear_initial_palettes(&mut self, norfair: bool) {
        for offset in 0..15 {
            if norfair {
                self.acquired_target_colour(113 + offset, 0);
            }
            self.acquired_target_colour(241 + offset, 0);
        }
    }
    fn colour(&self, index: usize) -> u16 {
        self.acquired_colour(index)
    }
    fn set_colour(&mut self, index: usize, value: u16) {
        self.acquired_set_colour(index, value);
    }
    fn eye_fade_step(&mut self, index: u16) -> u8 {
        self.allocations.native().read(
            usize::try_from(sm_data::cartridge::lorom_offset(0xa6e269)).expect("fade table")
                + usize::from(index),
        )
    }
    fn ceres_eye_fade(&mut self, index: isize) -> u16 {
        crate::ceres_ridley_tables::eye_fade(self, index)
    }
    fn ceres_body_fade(&mut self, index: isize) -> u16 {
        crate::ceres_ridley_tables::body_fade(self, index)
    }
    fn ceres_background_fade(&mut self, index: isize) -> u16 {
        crate::ceres_ridley_tables::background_fade(self, index)
    }
    fn ceres_attack(&mut self, index: isize) -> u16 {
        crate::ceres_ridley_tables::attack(self, index)
    }
    fn ceres_getaway_zoom(&mut self, index: isize) -> u16 {
        crate::ceres_ridley_tables::getaway_zoom(self, index)
    }
    fn ceres_getaway_velocity(&mut self, index: isize) -> sm_data::ceres_ridley::GetawayVelocity {
        crate::ceres_ridley_tables::getaway_velocity(self, index)
    }
    fn ceres_baby_metroid_frame(&mut self, index: isize) -> u16 {
        crate::ceres_ridley_tables::baby_metroid_frame(self, index)
    }
    fn ceres_wing_frame(&mut self, index: isize) -> u16 {
        crate::ceres_ridley_tables::wing_frame(self, index)
    }
    fn ceres_health_palette(&mut self) {
        self.acquired_ceres_health_palette();
    }
    fn emergency_text_glow(&mut self) {
        self.acquired_emergency_glow();
    }
    fn set_mode7_irq(&mut self, value: u16) {
        self.acquired_mode7_irq(value);
    }
    fn set_affine(&mut self, index: usize, value: u16) {
        self.acquired_affine(index, value);
    }
    fn set_bg1_scroll(&mut self, horizontal: bool, value: u16) {
        self.acquired_bg1_scroll(horizontal, value);
    }
    fn set_bg_mode(&mut self, value: u8) {
        self.acquired_bg_mode(value);
    }
    fn set_mode7_settings(&mut self, value: u8) {
        self.acquired_mode7_settings(value);
    }
    fn begin_ceres_push(&mut self) {
        crate::player_movement::forced::begin_ceres_push(self.session_mut());
    }
    fn queue_mode7_transfer(&mut self, bank: u8, list: u16) {
        self.acquired_mode7_queue(bank, list);
    }
    fn queue_escape_timer_tiles(&mut self) -> bool {
        self.acquired_escape_timer_tiles()
    }
    fn queue_getaway_tiles(&mut self) {
        let record = self.native_actor_record(0xa6, 0xc15d);
        self.acquired_rom_queue_push(&record);
    }
    fn queue_escape_tile_table(&mut self) {
        self.acquired_escape_tile_table();
    }
    fn type_text(&mut self, tiles: u16) -> bool {
        crate::escape_typewriter::owned::handle_text(self, tiles)
    }
}

pub(crate) fn try_ai(routine: ActorRoutine, host: &mut PreparedActorState<'_>) -> bool {
    let selected = host.session().actors.controls.selected;
    if super::owned_execution::draygon_state_ai(routine, selected, host)
        || super::owned_execution::draygon_reaction_ai(routine, host)
        || super::owned_execution::yapping_maw_ai(routine, host)
    {
        return true;
    }
    match routine.0 {
        10847879 => draygon_flight::entry::initialize(host),
        10847996 => draygon_flight::entry::main_ai(true, host),
        10863750 => draygon_flight::entry::main_ai(false, host),
        10848027 => draygon_flight::entry::fight_intro(true, host),
        10848139 => draygon_flight::entry::fight_intro(false, host),
        10863723 => crate::actors::draygon_eye::owned::initialize(host),
        10863757 => crate::actors::draygon_eye::owned::look_at_samus(selected, false, host),
        10863891 => crate::actors::draygon_eye::owned::look_at_samus(selected, true, host),
        10846283 | 10873626 => {}
        10873627 => spore_spawn::owned_return::descend(host),
        10873682 => spore_spawn::owned_return::orbit(selected, host),
        10873755 => spore_spawn::owned_return::begin_return(host),
        10873838 => spore_spawn::owned_return::finish_return(host),
        10920181 => ceres_ridley::owned::init(host),
        10920584 => ceres_ridley::owned::main(host),
        10920659 => ceres_ridley::owned::hurt(host),
        10924583 => ridley::owned::main(host),
        10924682 => ridley::owned::time_frozen(host),
        10924695 => ridley::owned::hurt(host),
        10929814 => ridley::owned::explosion_init(host),
        10930388 => ridley::owned::explosion_main(host),
        10936202 => ridley::owned::shot(host),
        10936242 => ridley::owned::power_bomb(host),
        10940337 => crate::actors::ceres_doors::initialize_steam(host),
        10940429 => crate::actors::ceres_doors::steam_main(host),
        10942149 => crate::actors::ceres_doors::initialize_door(host),
        10942309 => crate::actors::ceres_doors::door_main(host),
        11196360 => statues::owned::initialize_tourian_statue(host),
        11200278 => statues::owned::initialize_tube_cracks(host),
        11200293 => statues::owned::initialize_chozo_statue(host),
        11200423 => statues::owned::main_ai(host),
        _ => {
            if routine.0 >> 16 != 0xa6 {
                return false;
            }
            let function = u16::from_le_bytes(
                routine.0.to_le_bytes()[..2]
                    .try_into()
                    .expect("banked routine"),
            );
            return ceres_ridley::owned::run_ceres_function(host, function)
                || ridley::owned::run_ridley_function(host, function);
        }
    }
    true
}
pub(crate) fn try_instruction(
    routine: ActorRoutine,
    actor: crate::actors::access::ActorKey,
    script: EnemyScript,
    host: &mut PreparedActorState<'_>,
) -> Option<EnemyScript> {
    let actor = actor.byte_index();
    if let Ok(next) = ridley::owned::instruction(routine.0, script.clone(), host) {
        return Some(next);
    }
    if let Ok(next) = super::owned_execution::draygon_instruction(routine, script.clone(), host) {
        return Some(next);
    }
    if let Ok(next) = super::owned_execution::yapping_maw_instruction(routine, script.clone(), host)
    {
        return Some(next);
    }
    Some(match routine.0 {
        10851549 => draygon_flight::owned::set_instruction_lists(script, host),
        10852150 => crate::actors::draygon_eye::owned::set_function(actor, script, host),
        10852563 => draygon_flight::owned::paralyse_tail_and_arms(script, host),
        10852591 => draygon_flight::owned::set_intangible(script, host),
        10853898 => draygon_flight::owned::displace_graphics(script, host),
        10854231 => crate::actors::draygon_eye::owned::set_function(0, script, host),
        10863739 => crate::actors::draygon_eye::owned::set_function(0x40, script, host),
        11196831 => script,
        11199529 => {
            statues::owned::start_lowering_acid(host);
            script
        }
        11199542 => {
            statues::owned::set_lowered_acid(host);
            script
        }
        11199549 => {
            statues::owned::unlock_samus(host);
            script
        }
        11199871 => {
            statues::owned::grab_sound(host);
            script
        }
        11199879 => {
            statues::owned::footstep_sound(host);
            script
        }
        11200240 => {
            statues::owned::release_samus(host);
            script
        }
        11199887 => statues::owned::spike_clearing_footstep(host, actor, script),
        11199960 => statues::owned::movement(host, actor, script),
        _ => return None,
    })
}
pub(crate) fn try_hitbox_touch(routine: ActorRoutine, host: &mut PreparedActorState<'_>) -> bool {
    match routine.0 {
        10846284 => {}
        10851818 => draygon_death::owned::touch(host),
        10936153 => ActorRuntime::contact_reaction(host, Death::LeavePending),
        _ => return false,
    }
    true
}
pub(crate) fn try_hitbox_shot(
    routine: ActorRoutine,
    _argument: u16,
    host: &mut PreparedActorState<'_>,
) -> bool {
    match routine.0 {
        10851824 => draygon_death::owned::shot(host),
        10936202 => ridley::owned::shot(host),
        _ => return false,
    }
    true
}
pub(crate) fn try_graphics_hook(routine: ActorRoutine, host: &mut PreparedActorState<'_>) -> bool {
    match routine.0 {
        10851138 => draygon_flight::entry::set_bg2_scroll(host),
        10920690 => host.acquired_draw_baby_and_door(),
        _ => return false,
    }
    true
}
