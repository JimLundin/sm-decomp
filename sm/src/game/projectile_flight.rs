//! Launch speed and flight of ordinary beams, missiles and their linked extensions.
use crate::game::projectile_animation::initialize_super_missile;
use crate::game::sm_90::{ClearProjectile, InitProjectilePositionDirection};
use crate::game::sm_94::{
    BlockCollMissileHoriz, BlockCollMissileVert, BlockCollNoWaveBeamHoriz, BlockCollNoWaveBeamVert,
    BlockCollWaveBeamHoriz, BlockCollWaveBeamVert,
};
use crate::game_state::Game;
use crate::ram_map::{checked_word, word_arrays as arrays};
use crate::rtl::Unreachable;

pub use crate::game::projectile_flight_control::Flight;
use crate::game::{projectile_flight_control as control, projectile_room::Axis};

struct FlightContext;
impl<'a> control::Context<&'a mut Game> for FlightContext {
    type Direction = usize;
    fn selected(&self, game: &&mut Game) -> u16 {
        checked_word!(game.projectile_index)
    }
    fn retiring(&self, game: &&mut Game, slot: u16) -> bool {
        checked_word!(game.projectile_dir[(slot >> 1)]) & 0xf0 != 0
    }
    fn extension(&self, game: &&mut Game, slot: u16) -> bool {
        checked_word!(game.projectile_type[(slot >> 1)]) & 0xfff == 0x200
    }
    fn release(&mut self, game: &'a mut Game, slot: u16) -> &'a mut Game {
        ClearProjectile(slot, game);
        game
    }
    #[forbid(unsafe_code)]
    fn trail_ready(&mut self, game: &mut &'a mut Game, slot: u16, interval: u16) -> bool {
        let at = crate::ram_map::word_arrays::projectile_timers + usize::from(slot >> 1) * 2;
        // The countdown uses native words; an expired timer resets in SNES order.
        let previous = u16::from_ne_bytes([game.g_ram[at], game.g_ram[at + 1]]);
        game.g_ram[at..at + 2].copy_from_slice(&previous.wrapping_sub(1).to_ne_bytes());
        if previous == 1 {
            checked_word!(game.projectile_timers[(slot >> 1)] = interval);
        }
        previous == 1
    }
    fn trail(&mut self, game: &'a mut Game, slot: u16) -> &'a mut Game {
        unsafe { crate::game::projectile_trails::emit(slot, game) };
        game
    }
    fn advance(&mut self, game: &'a mut Game, slot: u16, kind: Flight) -> &'a mut Game {
        control::advance(game, slot, kind, self)
    }
    fn direction(&self, game: &&mut Game, slot: u16) -> usize {
        usize::from(checked_word!(game.projectile_dir[(slot >> 1)]) & 0xf)
    }
    #[forbid(unsafe_code)]
    fn accelerate(&mut self, game: &'a mut Game, slot: u16, direction: usize) -> &'a mut Game {
        checked_word!(
            game.projectile_bomb_x_speed[(slot >> 1)] += control::X_ACCELERATION[direction] as u16
        );
        checked_word!(
            game.projectile_bomb_y_speed[(slot >> 1)] += control::Y_ACCELERATION[direction] as u16
        );
        game
    }
    fn axes(&mut self, game: &'a mut Game, direction: usize) -> (&'a mut Game, u8) {
        (game, control::AXES[direction])
    }
    fn missile(&mut self, game: &'a mut Game, slot: u16) -> &'a mut Game {
        accelerate_missile(slot, game);
        game
    }
    fn collide(
        &mut self,
        game: &'a mut Game,
        slot: u16,
        kind: Flight,
        axis: Axis,
    ) -> (&'a mut Game, u8) {
        let hit = unsafe {
            match (kind, axis) {
                (Flight::NonWave, Axis::X) => BlockCollNoWaveBeamHoriz(slot, game),
                (Flight::NonWave, Axis::Y) => BlockCollNoWaveBeamVert(slot, game),
                (Flight::Wave, Axis::X) => BlockCollWaveBeamHoriz(slot, game),
                (Flight::Wave, Axis::Y) => BlockCollWaveBeamVert(slot, game),
                (_, Axis::X) => BlockCollMissileHoriz(slot, game),
                (_, Axis::Y) => BlockCollMissileVert(slot, game),
            }
        };
        (game, hit)
    }
    fn extension_collision(&mut self, game: &'a mut Game, axis: Axis) -> &'a mut Game {
        unsafe { collide_extension(axis, game) };
        game
    }
    fn cull(&mut self, game: &'a mut Game) -> (&'a mut Game, bool) {
        let removed = delete_offscreen(game) != 0;
        (game, removed)
    }
}
pub(crate) unsafe fn update(k: u16, kind: Flight, trail_interval: u16, game: &mut Game) {
    control::update(game, k, kind, trail_interval, &mut FlightContext);
}
pub(crate) unsafe fn advance(k: u16, kind: Flight, game: &mut Game) {
    control::advance(game, k, kind, &mut FlightContext);
}
pub(crate) unsafe fn collide(axes: u8, kind: Flight, game: &mut Game) {
    control::collide(game, axes, kind, &mut FlightContext);
}
pub(crate) fn update_extension(k: u16, game: &mut Game) {
    control::update_extension(game, k, &mut FlightContext);
}

pub(crate) unsafe fn initialize_speed(k: u16, speed: u16, game: &mut Game) {
    unsafe {
        if !crate::game::launch_motion_ram::initialize(k, speed, game) {
            Unreachable(game);
        }
    }
}

#[forbid(unsafe_code)]
fn delete_offscreen(game: &mut Game) -> u8 {
    let position = |base, game: &Game| {
        let at = base + usize::from(checked_word!(game.projectile_index) >> 1) * 2;
        u16::from_ne_bytes([game.g_ram[at], game.g_ram[at + 1]])
    };
    if control::visible(position(0xb64, game), checked_word!(game.layer1_x_pos))
        && control::visible(position(0xb78, game), checked_word!(game.layer1_y_pos))
    {
        return 0;
    }
    ClearProjectile(checked_word!(game.projectile_index), game);
    1
}

pub(crate) unsafe fn initialize_beam_speed(slot: u16, game: &mut Game) {
    use crate::game::launch_motion::{BEAM_SPEEDS, beam_selection};
    let (index, invalid) = beam_selection(
        checked_word!(game.projectile_type[(slot >> 1)]),
        checked_word!(game.projectile_dir[(slot >> 1)]),
    );
    if invalid {
        unsafe { Unreachable(game) };
    }
    unsafe { initialize_speed(slot, BEAM_SPEEDS[index], game) };
}

#[forbid(unsafe_code)]
fn accelerate_missile(slot: u16, game: &mut Game) {
    let variable = checked_word!(game.projectile_variables[(slot >> 1)]);
    control::accelerate_missile(game, slot, variable, &mut FlightContext);
}
impl<'a> control::MissileContext<&'a mut Game> for FlightContext {
    type Record = crate::rom::RomRecord;
    fn activate(&mut self, game: &mut &'a mut Game, slot: u16, variable: u16) {
        checked_word!(game.projectile_variables[(slot >> 1)] = variable);
    }
    fn acceleration(&mut self, game: &'a mut Game, slot: u16) -> (&'a mut Game, Self::Record) {
        let address = control::missile_acceleration_address(
            checked_word!(game.projectile_type[(slot >> 1)]),
            checked_word!(game.projectile_dir[(slot >> 1)]),
        );
        let record = unsafe { crate::rtl::rom_bank_record::<0x90>(address, game) };
        (game, record)
    }
    #[forbid(unsafe_code)]
    fn add_acceleration(
        &mut self,
        game: &'a mut Game,
        slot: u16,
        record: &Self::Record,
        axis: Axis,
    ) -> &'a mut Game {
        match axis {
            Axis::X => checked_word!(game.projectile_bomb_x_speed[(slot >> 1)] += record.word(0)),
            Axis::Y => checked_word!(game.projectile_bomb_y_speed[(slot >> 1)] += record.word(2)),
        };
        game
    }
    fn launch(&mut self, game: &'a mut Game, slot: u16, speed: u16) -> &'a mut Game {
        unsafe { initialize_speed(slot, speed, game) };
        game
    }
    fn heavy(&self, game: &&mut Game, slot: u16) -> bool {
        checked_word!(game.projectile_type[(slot >> 1)]) & 0x200 != 0
    }
    fn spawn(&mut self, game: &'a mut Game) -> &'a mut Game {
        spawn_extension(game);
        game
    }
}

#[forbid(unsafe_code)]
impl control::ExtensionContext<&mut Game> for FlightContext {
    fn speed(&self, game: &&mut Game, slot: u16, axis: Axis) -> i16 {
        (match axis {
            Axis::X => checked_word!(game.projectile_bomb_x_speed[(slot >> 1)]),
            Axis::Y => checked_word!(game.projectile_bomb_y_speed[(slot >> 1)]),
        }) as i16
    }
    fn impacted(&self, game: &&mut Game, slot: u16) -> bool {
        checked_word!(game.projectile_type[(slot >> 1)]) & 0xf00 == 0x800
    }
    fn shift_extension(
        &mut self,
        game: &mut &mut Game,
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
        let bytes = [game.g_ram[at], game.g_ram[at + 1]];
        // The original negative-speed branch alone reads a native position word.
        let position = if delta > 0 {
            u16::from_ne_bytes(bytes)
        } else {
            u16::from_le_bytes(bytes)
        };
        let at = base + usize::from(child & !1);
        game.g_ram[at..at + 2].copy_from_slice(&position.wrapping_add_signed(delta).to_le_bytes());
    }
    fn select(&mut self, game: &mut &mut Game, slot: u16) {
        checked_word!(game.projectile_index = slot);
    }
}
unsafe fn collide_extension(axis: Axis, game: &mut Game) {
    let parent = checked_word!(game.projectile_index);
    if !matches!(
        checked_word!(game.projectile_type[(parent >> 1)]) & 0xf00,
        0x200 | 0x800
    ) {
        return;
    }
    let child = u16::from(checked_word!(game.projectile_variables[(parent >> 1)]) as u8);
    // This is deliberately an unrounded byte selector, including for odd parents.
    if game.g_ram[arrays::projectile_variables + usize::from(parent) + 1] != 0 {
        control::contact_extension(game, parent, child, axis, &mut FlightContext);
    }
}

#[forbid(unsafe_code)]
pub(crate) fn link_extension(slot: u16, game: &mut Game) {
    let address = crate::ram_map::word_arrays::projectile_variables
        + usize::from(checked_word!(game.projectile_index) & !1);
    // This field used native raw loads/stores; the selector uses little endian.
    let value = u16::from_ne_bytes([game.g_ram[address], game.g_ram[address + 1]]);
    game.g_ram[address..address + 2]
        .copy_from_slice(&(value & 0xff00).wrapping_add(slot).to_ne_bytes());
}

#[forbid(unsafe_code)]
fn spawn_extension(game: &mut Game) {
    control::spawn_extension(game, &mut FlightContext);
}
impl<'a> control::CreationContext<&'a mut Game> for FlightContext {
    fn occupied(&self, game: &&mut Game, slot: u16) -> bool {
        checked_word!(game.projectile_damage[(slot >> 1)]) != 0
    }
    #[forbid(unsafe_code)]
    fn prepare(&mut self, game: &'a mut Game, slot: u16) -> (&'a mut Game, bool) {
        let child = slot >> 1;
        checked_word!(
            game.projectile_type[child] = checked_word!(game.projectile_type[child]) | 0x8200
        );
        let parent = checked_word!(game.projectile_index) >> 1;
        checked_word!(game.projectile_x_pos[child] = checked_word!(game.projectile_x_pos[parent]));
        checked_word!(game.projectile_y_pos[child] = checked_word!(game.projectile_y_pos[parent]));
        checked_word!(game.projectile_dir[child] = checked_word!(game.projectile_dir[parent]));
        (game, true)
    }
    fn place(&mut self, game: &'a mut Game, slot: u16) -> &'a mut Game {
        unsafe { InitProjectilePositionDirection(slot, game) };
        game
    }
    fn initialize(&mut self, game: &'a mut Game, slot: u16) -> &'a mut Game {
        unsafe { initialize_super_missile(slot, game) };
        game
    }
    fn arm(&mut self, game: &mut &'a mut Game, slot: u16) {
        checked_word!(game.projectile_bomb_pre_instructions[(slot >> 1)] = 0xb075);
    }
    fn link(&mut self, game: &'a mut Game, slot: u16) -> &'a mut Game {
        link_extension(slot, game);
        game
    }
    fn count(&mut self, game: &mut &'a mut Game) {
        checked_word!(game.projectile_counter += 1);
    }
}
