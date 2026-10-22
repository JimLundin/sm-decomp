//! Launch speed and flight of ordinary beams, missiles and their linked extensions.
use crate::game::projectile_animation::initialize_super_missile;
use crate::game::sm_90::{ClearProjectile, InitProjectilePositionDirection};
use crate::game::sm_94::{
    BlockCollMissileHoriz, BlockCollMissileVert, BlockCollNoWaveBeamHoriz, BlockCollNoWaveBeamVert,
    BlockCollWaveBeamHoriz, BlockCollWaveBeamVert,
};
use crate::game_state::Game;
use crate::ram_map::{checked_word, word, word_arrays as arrays};
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
        unsafe { accelerate_missile(slot, game) };
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

const RomPtr_90: unsafe fn(u16, &mut Game) -> *const u8 = crate::rtl::rom_bank_ptr::<0x90>;
use crate::game::constants::bank_90::{
    addr_kSuperMissileAccelerations, addr_kSuperMissileAccelerations2,
};

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

unsafe fn accelerate_missile(k: u16, game: &mut Game) {
    unsafe {
        static word_90C301: u16 = 0x100;
        let mut v3: u16 = 0;
        let v1: i32 = k as i32 >> 1;
        if (word!(game.projectile_variables[v1]) & 0xff00) != 0 {
            if (word!(game.projectile_type[v1]) & 0x200) != 0 {
                v3 = addr_kSuperMissileAccelerations2 as u16;
            } else {
                v3 = addr_kSuperMissileAccelerations as u16;
            }
            let v4 = RomPtr_90(
                (v3 as i32 + 4 * (word!(game.projectile_dir[v1]) as i32 & 0xf)) as u16,
                game,
            );
            word!(game.projectile_bomb_x_speed[v1] += *(v4 as *mut u16));
            word!(game.projectile_bomb_y_speed[v1] += *(v4.offset(2) as *mut u16));
        } else {
            let v2: u16 = (word_90C301).wrapping_add(word!(game.projectile_variables[v1]));
            word!(game.projectile_variables[v1] = v2);
            if (v2 & 0xff00) != 0 {
                initialize_speed(k, word!(game.projectile_variables[v1]), game);
                if (word!(game.projectile_type[v1]) & 0x200) != 0 {
                    spawn_extension(game);
                }
            }
        };
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

unsafe fn spawn_extension(game: &mut Game) {
    unsafe {
        let mut v0: u16 = 0;
        while word!(game.projectile_damage[(v0 >> 1)]) != 0 {
            v0 = (v0).wrapping_add(2);
            if ((v0) as i16).wrapping_sub(10) >= 0 {
                return;
            }
        }
        let r20: u16 = v0;
        let v1: i32 = v0 as i32 >> 1;
        word!(game.projectile_type[v1] |= (0x8200u16));
        let v2: i32 = word!(game.projectile_index) as i32 >> 1;
        word!(game.projectile_x_pos[v1] = word!(game.projectile_x_pos[v2]));
        word!(game.projectile_y_pos[v1] = word!(game.projectile_y_pos[v2]));
        word!(game.projectile_dir[v1] = word!(game.projectile_dir[v2]));
        InitProjectilePositionDirection(r20, game);
        initialize_super_missile(v0, game);
        word!(game.projectile_bomb_pre_instructions[v1] = ((0x90b075i32) as u16));
        link_extension(v0, game);
        word!(game.projectile_counter += 1);
    }
}
