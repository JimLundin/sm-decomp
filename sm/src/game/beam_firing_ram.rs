//! Checked beam state and synchronous runtime dependencies.
use crate::game::beam_firing::{
    self as shared, Context, Control, Cue, Field, InitialCollision, Input, Mode,
};
use crate::game::{beam_definitions as definitions, projectile_room::Axis};
use crate::game_state::Game;
use crate::ram_map::{checked_word as word, word_arrays as arrays, words};
pub struct Runtime;
fn field_offset(field: Field) -> usize {
    match field {
        Field::Damage => arrays::projectile_damage,
        Field::TrailTimer => arrays::projectile_timers,
        Field::XSpeed => arrays::projectile_bomb_x_speed,
        Field::YSpeed => arrays::projectile_bomb_y_speed,
    }
}
fn control_offset(field: Control) -> usize {
    match field {
        Control::Count => words::projectile_counter,
        Control::Cooldown => words::cooldown_timer,
        Control::Selected => words::projectile_index,
        Control::PreviousCharge => words::prev_beam_charge_counter,
        Control::Charge => words::flare_counter,
        Control::Protection => words::projectile_invincibility_timer,
        Control::ResumeSound => words::play_resume_charging_beam_sfx,
        Control::Glow => words::charged_shot_glow_timer,
    }
}
fn read(g: &Game, at: usize) -> u16 {
    u16::from_le_bytes([g.g_ram[at], g.g_ram[at + 1]])
}
fn write(g: &mut Game, at: usize, value: u16) {
    g.g_ram[at..at + 2].copy_from_slice(&value.to_le_bytes());
}
impl<'g> Context<&'g mut Game> for Runtime {
    type Sound = u16;
    fn read(&self, g: &&'g mut Game, slot: u16, field: Field) -> u16 {
        read(g, field_offset(field) + usize::from(slot & !1))
    }
    fn write(&mut self, g: &mut &'g mut Game, slot: u16, field: Field, value: u16) {
        write(g, field_offset(field) + usize::from(slot & !1), value);
    }
    fn control(&self, g: &&'g mut Game, field: Control) -> u16 {
        read(g, control_offset(field))
    }
    fn set_control(&mut self, g: &mut &'g mut Game, field: Control, value: u16) {
        write(g, control_offset(field), value);
    }
    fn input(&self, g: &&'g mut Game, field: Input) -> u16 {
        read(
            g,
            match field {
                Input::Hyper => words::hyper_beam_flag,
                Input::EquippedBeams => words::equipped_beams,
                Input::ShootButton => words::button_config_shoot_x,
                Input::Held => words::joypad1_lastkeys,
                Input::Pressed => words::joypad1_newkeys,
                Input::FilteredPressed => words::joypad1_newinput_samusfilter,
                Input::PendingAim => words::new_projectile_direction_changed_pose,
            },
        )
    }
    fn flare(&mut self, g: &mut &'g mut Game, index: usize, timer: bool, value: u16) {
        write(
            g,
            words::flare_animation_frame + 2 * index + if timer { 6 } else { 0 },
            value,
        );
    }
    fn wave(&self, g: &&'g mut Game, slot: u16) -> bool {
        read(g, arrays::projectile_type + usize::from(slot & !1)) & 1 != 0
    }
    fn is_beam(&self, g: &&'g mut Game, slot: u16) -> bool {
        let at = arrays::projectile_type + usize::from(slot & !1);
        u16::from_ne_bytes([g.g_ram[at], g.g_ram[at + 1]]) & 0xf00 == 0
    }
    fn place(&mut self, g: &'g mut Game, slot: u16) -> (&'g mut Game, bool) {
        let placed = crate::game::projectile_placement_ram::place(slot, g) & 1 == 0;
        (g, placed)
    }
    fn arm(&mut self, g: &'g mut Game, slot: u16, mode: Mode) -> (&'g mut Game, u16) {
        let at = arrays::projectile_type + usize::from(slot & !1);
        let sound = match mode {
            Mode::Hyper => {
                write(g, at, 0x9018);
                definitions::CHARGED_SOUND[8]
            }
            Mode::Charged => {
                let kind = word!(g.equipped_beams) & 0x100f | 0x8010;
                write(g, at, kind);
                definitions::CHARGED_SOUND[usize::from(kind & 15)]
            }
            Mode::Ordinary => {
                let sound_index = word!(g.equipped_beams) & 15;
                let kind = word!(g.equipped_beams) | 0x8000;
                write(g, at, kind);
                definitions::UNCHARGED_SOUND[usize::from(sound_index)]
            }
        };
        (g, sound)
    }
    fn sound(&mut self, g: &'g mut Game, cue: Cue<u16>) -> &'g mut Game {
        crate::game::sm_80::QueueSfx1_Max15(
            match cue {
                Cue::Fire(sound) => sound,
                Cue::CancelCharge => 2,
            },
            g,
        );
        g
    }
    fn initialize(&mut self, g: &'g mut Game, slot: u16) -> &'g mut Game {
        crate::game::projectile_initialization_ram::initialize(
            slot,
            crate::game::projectile_initialization::Initialization::Projectile,
            g,
        );
        g
    }
    fn cooldown(&mut self, g: &'g mut Game, slot: u16, mode: Mode) -> (&'g mut Game, u16) {
        let manual = mode == Mode::Charged
            || word!(g.equipped_beams) & 0x1000 != 0
            || word!(g.button_config_shoot_x) & word!(g.joypad1_newkeys) != 0
            || word!(g.button_config_shoot_x) & word!(g.joypad1_newinput_samusfilter) != 0;
        let kind = read(g, arrays::projectile_type + usize::from(slot & !1));
        let table: &[u8] = if manual {
            &definitions::MANUAL_COOLDOWN
        } else {
            &definitions::AUTO_COOLDOWN
        };
        (g, u16::from(table[usize::from(kind & 0x3f)]))
    }
    fn collide(&mut self, g: &'g mut Game, slot: u16, wave: bool) -> &'g mut Game {
        shared::initial_collision(g, slot, wave, self)
    }
    fn motion(&mut self, g: &'g mut Game, slot: u16, mode: Mode) -> &'g mut Game {
        let motion = if mode == Mode::Hyper {
            definitions::Motion::Hyper
        } else {
            let index = usize::from(read(g, arrays::projectile_type + usize::from(slot & !1)) & 15);
            definitions::motion(mode == Mode::Charged, index).unwrap_or_else(|| {
                panic!("index out of bounds: the len is 12 but the index is {index}")
            })
        };
        write(
            g,
            arrays::projectile_bomb_pre_instructions + usize::from(slot & !1),
            definitions::instruction(motion),
        );
        g
    }
    fn launch(&mut self, g: &'g mut Game, slot: u16) -> &'g mut Game {
        unsafe { crate::game::projectile_flight::initialize_beam_speed(slot, g) };
        g
    }
    fn special(&mut self, g: &'g mut Game) -> (&'g mut Game, u8) {
        let result = unsafe { crate::game::special_attacks_ram::fire(g) };
        (g, result)
    }
    fn palette(&mut self, g: &'g mut Game) -> &'g mut Game {
        unsafe { crate::game::sm_91::Samus_LoadSuitPalette(g) };
        g
    }
}
impl<'g> InitialCollision<&'g mut Game> for Runtime {
    fn direction(&self, g: &&'g mut Game, caller: u16) -> u16 {
        read(g, arrays::projectile_dir + usize::from(caller & !1))
    }
    fn selected(&self, g: &&'g mut Game) -> u16 {
        word!(g.projectile_index)
    }
    fn write_x_speed_native(&mut self, g: &mut &'g mut Game, slot: u16, value: u16) {
        let at = arrays::projectile_bomb_x_speed + usize::from(slot & !1);
        g.g_ram[at..at + 2].copy_from_slice(&value.to_ne_bytes());
    }
    fn scan(&mut self, g: &'g mut Game, slot: u16, wave: bool, axis: Axis) -> (&'g mut Game, u8) {
        use crate::game::sm_94::*;
        let value = unsafe {
            match (wave, axis) {
                (false, Axis::X) => BlockCollNoWaveBeamHoriz(slot, g),
                (false, Axis::Y) => BlockCollNoWaveBeamVert(slot, g),
                (true, Axis::X) => BlockCollWaveBeamHoriz(slot, g),
                (true, Axis::Y) => BlockCollWaveBeamVert(slot, g),
            }
        };
        (g, value)
    }
    fn invalid(&mut self, _: &'g mut Game, index: usize, length: usize) -> ! {
        panic!("index out of bounds: the len is {length} but the index is {index}")
    }
}
pub fn update_input(g: &mut Game) {
    shared::update_input(g, &mut Runtime);
}
pub fn clear_animation(mut g: &mut Game) {
    shared::clear_animation(&mut g, &mut Runtime);
}
pub fn prepare_hyper(slot: u16, g: &mut Game) {
    shared::prepare_hyper(g, slot, &mut Runtime);
}
pub fn finish_beam(slot: u16, charged: bool, wave: bool, g: &mut Game) {
    shared::finish(
        g,
        slot,
        if charged {
            Mode::Charged
        } else {
            Mode::Ordinary
        },
        wave,
        &mut Runtime,
    );
}
pub fn initial_collision(slot: u16, wave: bool, g: &mut Game) {
    shared::initial_collision(g, slot, wave, &mut Runtime);
}
pub fn charged_instruction(index: usize) -> u16 {
    definitions::instruction(
        definitions::motion(true, index).unwrap_or_else(|| {
            panic!("index out of bounds: the len is 12 but the index is {index}")
        }),
    )
}
