//! Flight, numeric motion identities, reflection and release on one owner.
use super::{Context, Transaction, animation};
use crate::game::projectile_flight_control::{self as control, Flight};
use crate::game::projectile_initialization::Initialization;
use crate::game::projectile_room::Axis;
use crate::ram_map::word_arrays as arrays;
pub(super) fn clear(k: u16, mut game: Transaction<'_>) -> Transaction<'_> {
    use crate::ram_map::{word_arrays::*, words};
    let slot = usize::from(k >> 1);
    for field in [
        projectile_x_pos,
        projectile_y_pos,
        projectile_bomb_x_subpos,
        projectile_bomb_y_subpos,
        projectile_dir,
        projectile_bomb_x_speed,
        projectile_bomb_y_speed,
        projectile_x_radius,
        projectile_y_radius,
        projectile_type,
        projectile_damage,
        projectile_bomb_instruction_ptr,
        projectile_bomb_instruction_timers,
        projectile_variables,
        projectile_spritemap_pointers,
    ] {
        word!(game, field + 2 * slot; = 0);
    }
    word!(game.projectile_bomb_pre_instructions[slot] = 0xb169);
    let counter = if (k as i16).wrapping_sub(10) >= 0 {
        words::bomb_counter
    } else {
        words::projectile_counter
    };
    word!(game, counter; = word!(game, counter;).wrapping_sub(1));
    if word!(game, counter;) & 0x8000 != 0 {
        word!(game, counter; = 0);
    }
    game
}

impl<'g> control::Context<Transaction<'g>> for Context {
    type Direction = usize;
    fn selected(&self, o: &Transaction<'g>) -> u16 {
        word!(o.projectile_index)
    }
    fn retiring(&self, o: &Transaction<'g>, slot: u16) -> bool {
        word!(o.projectile_dir[slot >> 1]) & 0xf0 != 0
    }
    fn extension(&self, o: &Transaction<'g>, slot: u16) -> bool {
        word!(o.projectile_type[slot >> 1]) & 0xfff == 0x200
    }
    fn release(&mut self, o: Transaction<'g>, slot: u16) -> Transaction<'g> {
        clear(slot, o)
    }
    fn trail_ready(&mut self, o: &mut Transaction<'g>, slot: u16, interval: u16) -> bool {
        let at = arrays::projectile_timers + usize::from(slot >> 1) * 2;
        let previous = o.read_native(at);
        o.write_slice_native(at, previous.wrapping_sub(1));
        if previous == 1 {
            word!(o.projectile_timers[slot >> 1] = interval);
        }
        previous == 1
    }
    fn trail(&mut self, o: Transaction<'g>, slot: u16) -> Transaction<'g> {
        super::presentation::emit(slot, o)
    }
    fn advance(&mut self, o: Transaction<'g>, slot: u16, kind: Flight) -> Transaction<'g> {
        control::advance(o, slot, kind, self)
    }
    fn direction(&self, o: &Transaction<'g>, slot: u16) -> usize {
        usize::from(word!(o.projectile_dir[slot >> 1]) & 0xf)
    }
    fn accelerate(
        &mut self,
        mut o: Transaction<'g>,
        slot: u16,
        direction: usize,
    ) -> Transaction<'g> {
        word!(o.projectile_bomb_x_speed[slot >> 1] += control::X_ACCELERATION[direction] as u16);
        word!(o.projectile_bomb_y_speed[slot >> 1] += control::Y_ACCELERATION[direction] as u16);
        o
    }
    fn axes(&mut self, o: Transaction<'g>, direction: usize) -> (Transaction<'g>, u8) {
        (o, control::AXES[direction])
    }
    fn missile(&mut self, o: Transaction<'g>, slot: u16) -> Transaction<'g> {
        let variable = word!(o.projectile_variables[slot >> 1]);
        control::accelerate_missile(o, slot, variable, self)
    }
    fn collide(
        &mut self,
        o: Transaction<'g>,
        slot: u16,
        kind: Flight,
        axis: Axis,
    ) -> (Transaction<'g>, u8) {
        super::collision::scan(slot, kind, axis, o)
    }
    fn extension_collision(&mut self, o: Transaction<'g>, axis: Axis) -> Transaction<'g> {
        let parent = word!(o.projectile_index);
        if !matches!(word!(o.projectile_type[parent >> 1]) & 0xf00, 0x200 | 0x800) {
            return o;
        }
        let child = u16::from(word!(o.projectile_variables[parent >> 1]) as u8);
        // The active byte deliberately uses the unrounded parent selector.
        if o.byte(arrays::projectile_variables + usize::from(parent) + 1) != 0 {
            control::contact_extension(o, parent, child, axis, self)
        } else {
            o
        }
    }
    fn cull(&mut self, o: Transaction<'g>) -> (Transaction<'g>, bool) {
        let position = |base, o: &Transaction<'g>| {
            o.read_native(base + usize::from(word!(o.projectile_index) >> 1) * 2)
        };
        if control::visible(position(0xb64, &o), word!(o.layer1_x_pos))
            && control::visible(position(0xb78, &o), word!(o.layer1_y_pos))
        {
            return (o, false);
        }
        let selected = word!(o.projectile_index);
        (clear(selected, o), true)
    }
}
pub(super) fn update(
    slot: u16,
    kind: Flight,
    interval: u16,
    o: Transaction<'_>,
) -> Transaction<'_> {
    crate::verify_probe::hit("player_projectiles.flight");
    control::update(o, slot, kind, interval, &mut Context::default())
}
pub(super) fn advance(slot: u16, kind: Flight, o: Transaction<'_>) -> Transaction<'_> {
    control::advance(o, slot, kind, &mut Context::default())
}
pub(super) fn collide(axes: u8, kind: Flight, o: Transaction<'_>) -> Transaction<'_> {
    control::collide(o, axes, kind, &mut Context::default())
}
pub(super) fn update_extension(slot: u16, o: Transaction<'_>) -> Transaction<'_> {
    control::update_extension(o, slot, &mut Context::default())
}
pub(super) fn initialize_speed(slot: u16, speed: u16, mut o: Transaction<'_>) -> Transaction<'_> {
    use crate::game::launch_motion::Aim;
    let offset = usize::from(slot >> 1) * 2;
    o.write_slice_le(0xb8c + offset, 0);
    o.write_slice_le(0xba0 + offset, 0);
    let Some(aim) = Aim::from_direction(o.read_le(0xc04 + offset)) else {
        return o.foreign(|g| unsafe { crate::rtl::Unreachable(g) }).0;
    };
    let velocity = o.state.launch.launch(aim, speed, o.read_le(0xda8));
    o.write_slice_le(0xbf0 + offset, velocity.y as u16);
    // A broad vertical write can alter this same owned history at DAC.
    let velocity = o.state.launch.launch(aim, speed, o.read_le(0xda8));
    o.write_slice_le(0xbdc + offset, velocity.x as u16);
    o
}
pub(super) fn initialize_beam_speed(slot: u16, mut o: Transaction<'_>) -> Transaction<'_> {
    use crate::game::launch_motion::{BEAM_SPEEDS, beam_selection};
    let (index, invalid) = beam_selection(
        word!(o.projectile_type[slot >> 1]),
        word!(o.projectile_dir[slot >> 1]),
    );
    if invalid {
        o = o.foreign(|g| unsafe { crate::rtl::Unreachable(g) }).0;
    }
    initialize_speed(slot, BEAM_SPEEDS[index], o)
}
impl<'g> control::MissileContext<Transaction<'g>> for Context {
    type Record = crate::rom::RomRecord;
    fn activate(&mut self, o: &mut Transaction<'g>, slot: u16, variable: u16) {
        word!(o.projectile_variables[slot >> 1] = variable);
    }
    fn acceleration(&mut self, o: Transaction<'g>, slot: u16) -> (Transaction<'g>, Self::Record) {
        let address = control::missile_acceleration_address(
            word!(o.projectile_type[slot >> 1]),
            word!(o.projectile_dir[slot >> 1]),
        );
        Context::record(o, 0x90, address)
    }
    fn add_acceleration(
        &mut self,
        mut o: Transaction<'g>,
        slot: u16,
        record: &Self::Record,
        axis: Axis,
    ) -> Transaction<'g> {
        match axis {
            Axis::X => word!(o.projectile_bomb_x_speed[slot >> 1] += record.word(0)),
            Axis::Y => word!(o.projectile_bomb_y_speed[slot >> 1] += record.word(2)),
        }
        o
    }
    fn launch(&mut self, o: Transaction<'g>, slot: u16, speed: u16) -> Transaction<'g> {
        initialize_speed(slot, speed, o)
    }
    fn heavy(&self, o: &Transaction<'g>, slot: u16) -> bool {
        word!(o.projectile_type[slot >> 1]) & 0x200 != 0
    }
    fn spawn(&mut self, o: Transaction<'g>) -> Transaction<'g> {
        control::spawn_extension(o, self)
    }
}
impl<'g> control::ExtensionContext<Transaction<'g>> for Context {
    fn speed(&self, o: &Transaction<'g>, slot: u16, axis: Axis) -> i16 {
        (match axis {
            Axis::X => word!(o.projectile_bomb_x_speed[slot >> 1]),
            Axis::Y => word!(o.projectile_bomb_y_speed[slot >> 1]),
        }) as i16
    }
    fn impacted(&self, o: &Transaction<'g>, slot: u16) -> bool {
        word!(o.projectile_type[slot >> 1]) & 0xf00 == 0x800
    }
    fn shift_extension(
        &mut self,
        o: &mut Transaction<'g>,
        parent: u16,
        child: u16,
        axis: Axis,
        delta: i16,
    ) {
        let base = match axis {
            Axis::X => arrays::projectile_x_pos,
            Axis::Y => arrays::projectile_y_pos,
        };
        let at = base + usize::from(parent & !1);
        let position = if delta > 0 {
            o.read_native(at)
        } else {
            o.read_le(at)
        };
        o.write_slice_le(
            base + usize::from(child & !1),
            position.wrapping_add_signed(delta),
        );
    }
    fn select(&mut self, o: &mut Transaction<'g>, slot: u16) {
        word!(o.projectile_index = slot);
    }
}
pub(super) fn link_extension(slot: u16, mut o: Transaction<'_>) -> Transaction<'_> {
    let address = arrays::projectile_variables + usize::from(word!(o.projectile_index) & !1);
    let value = o.read_native(address);
    o.write_slice_native(address, (value & 0xff00).wrapping_add(slot));
    o
}
impl<'g> control::CreationContext<Transaction<'g>> for Context {
    fn occupied(&self, o: &Transaction<'g>, slot: u16) -> bool {
        word!(o.projectile_damage[slot >> 1]) != 0
    }
    fn prepare(&mut self, mut o: Transaction<'g>, slot: u16) -> (Transaction<'g>, bool) {
        let child = slot >> 1;
        word!(o.projectile_type[child] = word!(o.projectile_type[child]) | 0x8200);
        let parent = word!(o.projectile_index) >> 1;
        word!(o.projectile_x_pos[child] = word!(o.projectile_x_pos[parent]));
        word!(o.projectile_y_pos[child] = word!(o.projectile_y_pos[parent]));
        word!(o.projectile_dir[child] = word!(o.projectile_dir[parent]));
        (o, true)
    }
    fn place(&mut self, o: Transaction<'g>, slot: u16) -> Transaction<'g> {
        super::placement::place(slot, o).0
    }
    fn initialize(&mut self, o: Transaction<'g>, slot: u16) -> Transaction<'g> {
        animation::initialize(slot, Initialization::SuperMissile, o)
    }
    fn arm(&mut self, o: &mut Transaction<'g>, slot: u16) {
        word!(o.projectile_bomb_pre_instructions[slot >> 1] = 0xb075);
    }
    fn link(&mut self, o: Transaction<'g>, slot: u16) -> Transaction<'g> {
        link_extension(slot, o)
    }
    fn count(&mut self, o: &mut Transaction<'g>) {
        word!(o.projectile_counter = word!(o.projectile_counter).wrapping_add(1));
    }
}
pub(super) fn point(angle: u16, radius: u16) -> (u16, u16) {
    let table = &crate::game::sm_94::kSinCosTable8bit_Sext;
    crate::game::special_geometry::point(angle, radius, table.len(), |index| {
        table.get(index).map(|&v| u16::from_le(v as u16))
    })
    .unwrap_or_else(|error| {
        let _ = table[error.index];
        unreachable!()
    })
}
fn speed_echo(slot: u16, mut o: Transaction<'_>) -> Transaction<'_> {
    let index = usize::from(slot >> 1);
    word!(o.projectile_bomb_x_speed[index] += 8);
    let (x, y) = point(
        word!(o.projectile_variables[index]),
        u16::from(o.byte(0xbdc + 2 * index)),
    );
    let position = x.wrapping_add(word!(o.samus_x_pos));
    // This original u16 pointer store used native byte order and raw byte k.
    o.write_slice_native(
        0xaae + usize::from(slot),
        x.wrapping_add(word!(o.samus_x_pos)),
    );
    word!(o.projectile_x_pos[index] = position);
    let dx = (position as i16).wrapping_sub(word!(o.layer1_x_pos) as i16);
    let outside = if dx < 0 || (i32::from(dx) - 256) & 0x8000 == 0 {
        true
    } else {
        let position = y.wrapping_add(word!(o.samus_y_pos));
        word!(o.speed_echo_xpos[index + 3] = y.wrapping_add(word!(o.samus_y_pos)));
        word!(o.projectile_y_pos[index] = position);
        let dy = (position as i16).wrapping_sub(word!(o.layer1_y_pos) as i16);
        dy < 0 || (i32::from(dy) - 256) & 0x8000 == 0
    };
    if outside {
        word!(o.speed_echo_ypos[index + 3] = 0);
        o.write_slice_native(0xaae + usize::from(slot), 0);
        word!(o.speed_echo_xpos[index + 3] = 0);
        o = clear(slot, o);
    }
    o
}
fn suit_pickup_shot(slot: u16, mut o: Transaction<'_>) -> Transaction<'_> {
    const X: [i16; 4] = [-4, -4, 4, 4];
    const Y: [i16; 4] = [4, -4, -4, 4];
    const RESTART_X: [i16; 4] = [0x80, 0x80, -0x80, -0x80];
    const RESTART_Y: [i16; 4] = [-0x80, 0x80, 0x80, -0x80];
    let index = usize::from(slot >> 1);
    word!(o.projectile_x_pos[index] += X[index] as u16);
    let y = (Y[index] as u16).wrapping_add(word!(o.projectile_y_pos[index]));
    word!(o.projectile_y_pos[index] = y);
    if y == word!(o.samus_y_pos) {
        if word!(o.projectile_variables[index]) == 1 {
            if slot == 6 {
                word!(o.samus_movement_handler = 0xf04b);
            }
            o = clear(slot, o);
        } else {
            word!(o.projectile_variables[index] += 1);
            word!(o.samus_shine_timer = 180);
            word!(o.timer_for_shine_timer = 1);
            word!(o.special_samus_palette_frame = 0);
            word!(
                o.projectile_x_pos[index] =
                    (RESTART_X[index] as u16).wrapping_add(word!(o.samus_x_pos))
            );
            word!(
                o.projectile_y_pos[index] =
                    (RESTART_Y[index] as u16).wrapping_add(word!(o.samus_y_pos))
            );
        }
    }
    o
}
pub(super) fn reflect(slot: u16, mut o: Transaction<'_>) -> Transaction<'_> {
    let index = usize::from(slot >> 1);
    let kind = word!(o.projectile_type[index]);
    if kind & 0x100 != 0 {
        o = animation::initialize(slot, Initialization::Projectile, o);
        word!(o.projectile_bomb_pre_instructions[index] = 0xaf68);
        word!(o.projectile_variables[index] = 240);
    } else if kind & 0x200 != 0 {
        let child = u16::from(o.byte(0xc7c + 2 * index));
        o = clear(child, o);
        o = animation::initialize(slot, Initialization::Projectile, o);
        word!(o.projectile_bomb_pre_instructions[index] = 0xafe5);
        word!(o.projectile_variables[index] = 240);
    } else {
        o = initialize_beam_speed(slot, o);
        o = animation::initialize(slot, Initialization::Projectile, o);
        word!(
            o.projectile_bomb_pre_instructions[index] = super::weapons::charged_instruction(
                usize::from(word!(o.projectile_type[index]) & 15)
            )
        );
    }
    o
}
pub(super) fn advance_all(mut o: Transaction<'_>) -> Transaction<'_> {
    word!(o.projectile_index = 18);
    let mut cursor: i32 = 18;
    while cursor >= 0 {
        let index = (cursor >> 1) as usize;
        if word!(o.projectile_bomb_instruction_ptr[index]) != 0 {
            let motion = word!(o.projectile_bomb_pre_instructions[index]);
            o = run(motion, cursor as u16, o);
            o = animation::advance(o);
            cursor = i32::from(word!(o.projectile_index));
        }
        cursor -= 2;
        word!(o.projectile_index = cursor as u16);
    }
    o
}
pub(super) fn run(motion: u16, slot: u16, o: Transaction<'_>) -> Transaction<'_> {
    use crate::game::{bomb_lifecycle::Kind as Bomb, special_attacks::Kind as Special};
    match motion {
        0xd4d2 => speed_echo(slot, o),
        0xefd3 => suit_pickup_shot(slot, o),
        0xd793 => super::specials::update(slot, Special::Plasma, o),
        0xcf09 => super::specials::update(slot, Special::Ice, o),
        0xcf7a => super::specials::update(slot, Special::IceReleased, o),
        0xda08 => super::specials::update(slot, Special::Wave, o),
        0xdb06 => super::specials::update(slot, Special::Spazer, o),
        0xdc9c => super::specials::update(slot, Special::SpazerReleased, o),
        0xd7e1 => super::specials::plasma_radius(slot, false, o),
        0xd7fa => super::specials::plasma_radius(slot, true, o),
        0xd813 => super::specials::depart_plasma(slot, o),
        0xb099 => super::bombs::bomb_update(slot, Bomb::Bomb, o),
        0xb0ae => super::bombs::bomb_update(slot, Bomb::PowerBomb, o),
        0xd8f7 => super::bombs::bomb_update(slot, Bomb::Spread, o),
        0xb169 => o,
        0xaef3 => update(slot, Flight::NonWave, 4, o),
        0xaf4a => collide(2, Flight::NonWave, o),
        0xaf52 => collide(3, Flight::NonWave, o),
        0xaf60 => collide(1, Flight::NonWave, o),
        0xaf68 => update(slot, Flight::Missile, 4, o),
        0xafc7 => collide(2, Flight::Missile, o),
        0xafcf => collide(3, Flight::Missile, o),
        0xafdd => collide(1, Flight::Missile, o),
        0xafe5 => update(slot, Flight::SuperMissile, 2, o),
        0xb047 => collide(2, Flight::SuperMissile, o),
        0xb052 => collide(3, Flight::SuperMissile, o),
        0xb06a => collide(1, Flight::SuperMissile, o),
        0xb075 => update_extension(slot, o),
        0xb0c3 => update(slot, Flight::Wave, 4, o),
        0xb0e4 => update(slot, Flight::Wave, 3, o),
        0xb103 => advance(slot, Flight::Wave, o),
        0xb13b => collide(2, Flight::Wave, o),
        0xb143 => collide(3, Flight::Wave, o),
        0xb151 => collide(1, Flight::Wave, o),
        0xb159 => update(slot, Flight::Wave, 0, o),
        _ => o.foreign(|g| unsafe { crate::rtl::Unreachable(g) }).0,
    }
}
pub(super) fn reset(mut o: Transaction<'_>) -> Transaction<'_> {
    use crate::ram_map::word_arrays::*;
    for index in 0..10 {
        for field in [
            projectile_timers,
            projectile_x_pos,
            projectile_y_pos,
            projectile_dir,
            projectile_bomb_x_speed,
            projectile_bomb_y_speed,
            projectile_x_radius,
            projectile_y_radius,
            projectile_type,
            projectile_damage,
            projectile_bomb_instruction_ptr,
            projectile_bomb_instruction_timers,
            projectile_variables,
            projectile_spritemap_pointers,
        ] {
            o.write_word_le(0, field + 2 * index);
        }
        word!(o.projectile_bomb_pre_instructions[index] = 0xb169);
    }
    word!(o.bomb_counter = 0);
    word!(o.cooldown_timer = 0);
    word!(o.projectile_counter = 0);
    word!(o.power_bomb_flag = 0);
    if word!(o.hud_auto_cancel_flag) != 0 {
        word!(o.hud_item_index = 0);
        word!(o.samus_auto_cancel_hud_item_index = 0);
    }
    for field in [speed_echo_xpos, speed_echo_ypos, speed_echo_xspeed] {
        for index in 0..4 {
            o.write_word_le(0, field + 2 * index);
        }
    }
    word!(o.speed_echoes_index = 0);
    if word!(o.samus_special_super_palette_flags) != 0 {
        word!(o.samus_special_super_palette_flags = 0);
        o = o
            .foreign(|g| unsafe { crate::game::sm_91::Samus_LoadSuitTargetPalette(g) })
            .0;
    }
    if word!(o.hyper_beam_flag) != 0 {
        o = o
            .foreign(|g| crate::game::palette_effects::spawn(0xe1f0, g))
            .0;
    }
    o
}
