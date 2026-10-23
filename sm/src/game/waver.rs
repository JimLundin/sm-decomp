//! Waver: horizontal 16.16 motion with a sine-wave vertical phase.
//! RAM views are scoped between legacy movement calls. Animation words are
//! read from the loaded ROM; no ROM-derived data is stored in this module.
use crate::game::enemy_layouts::enemy_layout;
use crate::game::enemy_script::{EnemyScript, bank_base};
use crate::game::sm_a0::{Enemy_MoveDown, Enemy_MoveRight_IgnoreSlopes, SineMult8bit};
use crate::game_state::Game;
use crate::types::EnemyBase;

const INITIAL_INSTRUCTION: u16 = 0x86a7;
const ANIMATION_TABLE: u32 = 0xa386db;

enemy_layout! {
    Enemy_Waver {
        base: EnemyBase,
        words: [
            velocity_fraction, velocity_integer, previous_animation, sine_phase,
            animation_finished, animation, initial_direction, unused_parameter,
        ],
    }
}

/// A byte-backed view of one Waver slot; no typed pointers or alignment assumptions.
struct WaverRam<'a> {
    bytes: &'a mut [u8],
}

impl<'a> WaverRam<'a> {
    fn new(game: &'a mut Game, index: u16) -> Self {
        let start = 0xf78 + usize::from(index);
        Self {
            bytes: &mut game.g_ram[start..start + core::mem::size_of::<Enemy_Waver>()],
        }
    }

    fn read(&self, offset: usize) -> u16 {
        u16::from_le_bytes([self.bytes[offset], self.bytes[offset + 1]])
    }

    fn write(&mut self, offset: usize, value: u16) {
        self.bytes[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
    }

    fn velocity_fraction(&self) -> u16 {
        self.read(core::mem::offset_of!(Enemy_Waver, velocity_fraction))
    }

    fn set_velocity_fraction(&mut self, value: u16) {
        self.write(core::mem::offset_of!(Enemy_Waver, velocity_fraction), value);
    }

    fn velocity_integer(&self) -> u16 {
        self.read(core::mem::offset_of!(Enemy_Waver, velocity_integer))
    }

    fn set_velocity_integer(&mut self, value: u16) {
        self.write(core::mem::offset_of!(Enemy_Waver, velocity_integer), value);
    }

    fn previous_animation(&self) -> u16 {
        self.read(core::mem::offset_of!(Enemy_Waver, previous_animation))
    }

    fn set_previous_animation(&mut self, value: u16) {
        self.write(
            core::mem::offset_of!(Enemy_Waver, previous_animation),
            value,
        );
    }

    fn sine_phase(&self) -> u16 {
        self.read(core::mem::offset_of!(Enemy_Waver, sine_phase))
    }

    fn set_sine_phase(&mut self, value: u16) {
        self.write(core::mem::offset_of!(Enemy_Waver, sine_phase), value);
    }

    fn animation_finished(&self) -> u16 {
        self.read(core::mem::offset_of!(Enemy_Waver, animation_finished))
    }

    fn set_animation_finished(&mut self, value: u16) {
        self.write(
            core::mem::offset_of!(Enemy_Waver, animation_finished),
            value,
        );
    }

    fn animation(&self) -> u16 {
        self.read(core::mem::offset_of!(Enemy_Waver, animation))
    }

    fn set_animation(&mut self, value: u16) {
        self.write(core::mem::offset_of!(Enemy_Waver, animation), value);
    }

    fn initial_direction(&self) -> u16 {
        self.read(core::mem::offset_of!(Enemy_Waver, initial_direction))
    }

    fn set_instruction(&mut self, instruction: u16) {
        self.write(
            core::mem::offset_of!(EnemyBase, current_instruction),
            instruction,
        );
    }

    fn restart_instruction(&mut self, instruction: u16) {
        self.set_instruction(instruction);
        self.write(core::mem::offset_of!(EnemyBase, instruction_timer), 1);
        self.write(core::mem::offset_of!(EnemyBase, timer), 0);
    }
}

pub fn Waver_Instr_1(_k: u16, jp: EnemyScript, game: &mut Game) -> EnemyScript {
    let index = game.cur_enemy_index();
    WaverRam::new(game, index).set_animation_finished(1);
    jp
}

pub fn Waver_Init(game: &mut Game) {
    let index = game.cur_enemy_index();
    {
        let mut enemy = WaverRam::new(game, index);
        enemy.set_velocity_integer(1);
        enemy.set_velocity_fraction(0x8000);
        if enemy.initial_direction() & 1 == 0 {
            enemy.set_velocity_integer(0xfffe);
            // SignExtend8(0x8000) in the original retains 0x8000, not zero.
            enemy.set_velocity_fraction(0x8000);
        }
        enemy.set_animation(0);
        enemy.set_previous_animation(0);
        enemy.set_animation_finished(0);
        enemy.set_instruction(INITIAL_INSTRUCTION);
        enemy.set_animation(enemy.initial_direction() & 1);
    }
    Waver_Func_1(game);
}

pub unsafe fn Waver_Main(game: &mut Game) {
    unsafe {
        let index = game.cur_enemy_index();
        let horizontal_velocity = {
            let enemy = WaverRam::new(game, index);
            ((u32::from(enemy.velocity_integer()) << 16) | u32::from(enemy.velocity_fraction()))
                as i32
        };
        if Enemy_MoveRight_IgnoreSlopes(index, horizontal_velocity, game) != 0 {
            {
                let mut enemy = WaverRam::new(game, index);
                // The ROM reverses the middle two bytes of the 16.16 velocity.
                let middle = (enemy.velocity_fraction() >> 8) | (enemy.velocity_integer() << 8);
                let reversed = middle.wrapping_neg();
                enemy.set_velocity_integer((reversed >> 8) as u8 as i8 as i16 as u16);
                enemy.set_velocity_fraction(
                    ((middle as u8 as i8 as i16).wrapping_neg() as u16) << 8,
                );
                enemy.set_animation((enemy.animation() ^ 1) & 1);
            }
            Waver_Func_1(game);
        } else {
            // Read the current index again at the same point as the original.
            let vertical_index = game.cur_enemy_index();
            let phase = WaverRam::new(game, index).sine_phase() as u8 as u16;
            let vertical_velocity = i32::from(SineMult8bit(phase, 4, game) as i16) << 16;
            let collided = Enemy_MoveDown(vertical_index, vertical_velocity, game) != 0;
            let mut enemy = WaverRam::new(game, index);
            if collided {
                enemy.set_sine_phase(enemy.sine_phase().wrapping_add(0x80) as u8 as u16);
            } else {
                enemy.set_sine_phase(enemy.sine_phase().wrapping_add(2));
            }
        }
        if WaverRam::new(game, index).sine_phase() & 0x7f == 56 {
            {
                let mut enemy = WaverRam::new(game, index);
                enemy.set_animation(enemy.animation() | 2);
            }
            Waver_Func_1(game);
        }
        if WaverRam::new(game, index).animation_finished() != 0 {
            {
                let mut enemy = WaverRam::new(game, index);
                enemy.set_animation_finished(0);
                enemy.set_animation(enemy.animation() & 1);
            }
            Waver_Func_1(game);
        }
    }
}

pub fn Waver_Func_1(game: &mut Game) {
    let index = game.cur_enemy_index();
    let animation = WaverRam::new(game, index).animation();
    if animation != WaverRam::new(game, index).previous_animation() {
        WaverRam::new(game, index).set_previous_animation(animation);
        let instruction = animation_instruction(game, animation);
        WaverRam::new(game, index).restart_instruction(instruction);
    }
}

/// Preserve the original unmasked animation index and linear ROM offset.
fn animation_instruction(game: &Game, animation: u16) -> u16 {
    game.read_rom_word(ANIMATION_TABLE, usize::from(animation) * 2)
}
