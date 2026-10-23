//! Direct beam charge/fire/launch bindings over the same owner for every slot.
use super::{Context, Transaction};
use crate::game::beam_firing::{self as shared, Control, Cue, Field, InitialCollision, Input, Mode};
use crate::game::{beam_definitions as definitions, projectile_room::Axis};
use crate::ram_map::{word_arrays as arrays, words};
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
impl<'g> shared::Context<Transaction<'g>> for Context {
    type Sound = u16;
    fn read(&self, g: &Transaction<'g>, slot: u16, field: Field) -> u16 {
        g.read_le(field_offset(field) + usize::from(slot & !1))
    }
    fn write(&mut self, g: &mut Transaction<'g>, slot: u16, field: Field, value: u16) {
        g.write_slice_le(field_offset(field) + usize::from(slot & !1), value);
    }
    fn control(&self, g: &Transaction<'g>, field: Control) -> u16 {
        g.read_le(control_offset(field))
    }
    fn set_control(&mut self, g: &mut Transaction<'g>, field: Control, value: u16) {
        g.write_slice_le(control_offset(field), value);
    }
    fn input(&self, g: &Transaction<'g>, field: Input) -> u16 {
        g.read_le(match field {
            Input::Hyper => words::hyper_beam_flag,
            Input::EquippedBeams => words::equipped_beams,
            Input::ShootButton => words::button_config_shoot_x,
            Input::Held => words::joypad1_lastkeys,
            Input::Pressed => words::joypad1_newkeys,
            Input::FilteredPressed => words::joypad1_newinput_samusfilter,
            Input::PendingAim => words::new_projectile_direction_changed_pose,
        })
    }
    fn flare(&mut self, g: &mut Transaction<'g>, index: usize, timer: bool, value: u16) {
        g.write_slice_le(
            words::flare_animation_frame + 2 * index + if timer { 6 } else { 0 },
            value,
        );
    }
    fn wave(&self, g: &Transaction<'g>, slot: u16) -> bool {
        g.read_le(arrays::projectile_type + usize::from(slot & !1)) & 1 != 0
    }
    fn is_beam(&self, g: &Transaction<'g>, slot: u16) -> bool {
        let at = arrays::projectile_type + usize::from(slot & !1);
        g.read_native(at) & 0xf00 == 0
    }
    fn place(&mut self, mut g: Transaction<'g>, slot: u16) -> (Transaction<'g>, bool) {
        super::placement::place(slot, g)
    }
    fn arm(&mut self, mut g: Transaction<'g>, slot: u16, mode: Mode) -> (Transaction<'g>, u16) {
        let at = arrays::projectile_type + usize::from(slot & !1);
        let sound = match mode {
            Mode::Hyper => {
                g.write_slice_le(at, 0x9018);
                definitions::CHARGED_SOUND[8]
            }
            Mode::Charged => {
                let kind = word!(g.equipped_beams) & 0x100f | 0x8010;
                g.write_slice_le(at, kind);
                definitions::CHARGED_SOUND[usize::from(kind & 15)]
            }
            Mode::Ordinary => {
                let sound_index = word!(g.equipped_beams) & 15;
                let kind = word!(g.equipped_beams) | 0x8000;
                g.write_slice_le(at, kind);
                definitions::UNCHARGED_SOUND[usize::from(sound_index)]
            }
        };
        (g, sound)
    }
    fn sound(&mut self, mut g: Transaction<'g>, cue: Cue<u16>) -> Transaction<'g> {
        g.foreign(|g| {
            crate::game::sm_80::QueueSfx1_Max15(
                match cue {
                    Cue::Fire(sound) => sound,
                    Cue::CancelCharge => 2,
                },
                g,
            )
        })
        .0
    }
    fn initialize(&mut self, mut g: Transaction<'g>, slot: u16) -> Transaction<'g> {
        super::animation::initialize(
            slot,
            crate::game::projectile_initialization::Initialization::Projectile,
            g,
        )
    }
    fn cooldown(
        &mut self,
        mut g: Transaction<'g>,
        slot: u16,
        mode: Mode,
    ) -> (Transaction<'g>, u16) {
        let manual = mode == Mode::Charged
            || word!(g.equipped_beams) & 0x1000 != 0
            || word!(g.button_config_shoot_x) & word!(g.joypad1_newkeys) != 0
            || word!(g.button_config_shoot_x) & word!(g.joypad1_newinput_samusfilter) != 0;
        let kind = g.read_le(arrays::projectile_type + usize::from(slot & !1));
        let table: &[u8] = if manual {
            &definitions::MANUAL_COOLDOWN
        } else {
            &definitions::AUTO_COOLDOWN
        };
        (g, u16::from(table[usize::from(kind & 0x3f)]))
    }
    fn collide(&mut self, mut g: Transaction<'g>, slot: u16, wave: bool) -> Transaction<'g> {
        shared::initial_collision(g, slot, wave, self)
    }
    fn motion(&mut self, mut g: Transaction<'g>, slot: u16, mode: Mode) -> Transaction<'g> {
        let motion = if mode == Mode::Hyper {
            definitions::Motion::Hyper
        } else {
            let index =
                usize::from(g.read_le(arrays::projectile_type + usize::from(slot & !1)) & 15);
            definitions::motion(mode == Mode::Charged, index).unwrap_or_else(|| {
                panic!("index out of bounds: the len is 12 but the index is {index}")
            })
        };
        g.write_slice_le(
            arrays::projectile_bomb_pre_instructions + usize::from(slot & !1),
            definitions::instruction(motion),
        );
        g
    }
    fn launch(&mut self, mut g: Transaction<'g>, slot: u16) -> Transaction<'g> {
        super::motion::initialize_beam_speed(slot, g)
    }
    fn special(&mut self, mut g: Transaction<'g>) -> (Transaction<'g>, u8) {
        super::specials::fire(g)
    }
    fn palette(&mut self, mut g: Transaction<'g>) -> Transaction<'g> {
        g.foreign(|g| unsafe { crate::game::sm_91::Samus_LoadSuitPalette(g) })
            .0
    }
}
impl<'g> InitialCollision<Transaction<'g>> for Context {
    fn direction(&self, g: &Transaction<'g>, caller: u16) -> u16 {
        g.read_le(arrays::projectile_dir + usize::from(caller & !1))
    }
    fn selected(&self, g: &Transaction<'g>) -> u16 {
        word!(g.projectile_index)
    }
    fn write_x_speed_native(&mut self, g: &mut Transaction<'g>, slot: u16, value: u16) {
        let at = arrays::projectile_bomb_x_speed + usize::from(slot & !1);
        g.write_slice_native(at, value);
    }
    fn scan(
        &mut self,
        mut g: Transaction<'g>,
        slot: u16,
        wave: bool,
        axis: Axis,
    ) -> (Transaction<'g>, u8) {
        let kind = if wave {
            crate::game::projectile_flight_control::Flight::Wave
        } else {
            crate::game::projectile_flight_control::Flight::NonWave
        };
        super::collision::scan(slot, kind, axis, g)
    }
    fn invalid(&mut self, _: Transaction<'g>, index: usize, length: usize) -> ! {
        panic!("index out of bounds: the len is {length} but the index is {index}")
    }
}
pub(super) fn update_input(g: Transaction<'_>) -> Transaction<'_> {
    shared::update_input(g, &mut Context::default())
}
pub(super) fn clear_animation(g: &mut Transaction<'_>) {
    shared::clear_animation(g, &mut Context::default());
}
pub fn charged_instruction(index: usize) -> u16 {
    definitions::instruction(
        definitions::motion(true, index).unwrap_or_else(|| {
            panic!("index out of bounds: the len is 12 but the index is {index}")
        }),
    )
}
