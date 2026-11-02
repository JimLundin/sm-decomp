//! Actual finite provider bodies for the remaining actor registry.
//! Global Phantoon and physical message/save providers are composition prerequisites.
#![forbid(unsafe_code)]
use crate::actors::owned_execution::PreparedActorState;
use crate::actors::remaining_owned as owned;
use crate::session_access::SessionAccess;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Diagnostic {
    Reflection,
    LavaHeight(i32),
}
/// The native diagnostic runs after all preceding actor effects on this owner.
/// The supplied sm emitter retains the original printf and its return edge.
pub trait Ports {
    fn remaining_actor_diagnostic(
        &mut self,
        owner: &mut crate::persistent_session::PersistentSession,
        diagnostic: Diagnostic,
    );
    /// Actual Kraid verifier callback on the same imported Game resource owner.
    fn kraid_flags_observer(
        &mut self,
        owner: &mut crate::persistent_session::PersistentSession,
        step: u8,
    );
}

impl owned::torizo::Ai for PreparedActorState<'_> {
    fn run_function(&mut self, address: u32) {
        pre_instruction(address, self);
    }
    fn spawn_palette_effect(&mut self, definition: u16) {
        crate::palette_effects::execution::spawn(self, definition);
    }
    fn spawn_haze(&mut self) {
        self.hardware_initialize_bomb_torizo_haze();
    }
}
impl owned::morph_ball_eye::Host for PreparedActorState<'_> {
    fn start_beam(&mut self) {
        self.hardware_initialize_morph_eye_beam();
    }
}
impl owned::reflec::Host for PreparedActorState<'_> {
    fn warn_reflection(&mut self) {
        self.aliases
            .remaining_actor_diagnostic(self.owner, Diagnostic::Reflection);
        self.refresh_inputs();
    }
}
impl owned::magdollite::Host for PreparedActorState<'_> {
    fn warn_lava_height(&mut self, value: i32) {
        self.aliases
            .remaining_actor_diagnostic(self.owner, Diagnostic::LavaHeight(value));
        self.refresh_inputs();
    }
}

fn part_y_address(slot: i32, fraction: bool) -> usize {
    // Original part() widens current u16 before adding 64/128. No u16 wrap.
    let slot = usize::try_from(slot).expect("gunship part address");
    (if fraction { 0xf80usize } else { 0xf7eusize })
        .checked_add(slot)
        .expect("gunship part extent")
}
impl owned::gunship::Host for PreparedActorState<'_> {
    fn gunship_control(&self, field: owned::gunship::ControlWord) -> u16 {
        use owned::gunship::ControlWord::*;
        let session = self.session();
        match field {
            Mode => session.game_loop.mode,
            DustCursor => session.shared_sequence.stage,
            SuitBeam => session.shared_sequence.value,
            UsedSaveStations => u16::from_le_bytes([
                session.gameplay_flags.used_stations[0],
                session.gameplay_flags.used_stations[1],
            ]),
        }
    }
    fn set_gunship_control(&mut self, field: owned::gunship::ControlWord, value: u16) {
        use owned::gunship::ControlWord::*;
        match field {
            Mode => self.session_mut().game_loop.mode = value,
            DustCursor => self.session_mut().shared_sequence.stage = value,
            SuitBeam => self.session_mut().shared_sequence.value = value,
            UsedSaveStations => {
                let bytes = value.to_le_bytes();
                self.session_mut().gameplay_flags.used_stations[0] = bytes[0];
                self.session_mut().gameplay_flags.used_stations[1] = bytes[1];
            }
        }
        self.refresh_inputs();
    }
    fn gunship_part_y(&self, slot: i32, fraction: bool) -> u16 {
        self.current_native_word(part_y_address(slot, fraction))
    }
    fn set_gunship_part_y_native(&mut self, slot: i32, fraction: bool, value: u16) {
        self.write_current_slice(part_y_address(slot, fraction), &value.to_ne_bytes());
    }
    fn clear_gunship_color_alias(&mut self, word: usize) {
        // DF0/DF2 name player palette fields, not colors in the C000 palette.
        let address = [0xdf0usize, 0xdf2usize][word];
        for (lane, byte) in 0u16.to_ne_bytes().into_iter().enumerate() {
            crate::scene::actor_presentation::write_byte(
                &mut self.owner.gameplay,
                self.aliases.presentation_diagnostics(),
                address + lane,
                byte,
            )
            .expect("admitted gunship palette byte");
            self.refresh_inputs();
        }
    }
    fn spawn_palette_effect(&mut self, definition: u16) {
        crate::palette_effects::execution::spawn(self, definition);
    }
    fn face_player_forward(&mut self) {
        self.mechanism_face_forward();
    }
    fn message_box_response(&mut self, kind: u16) -> i32 {
        self.mechanism_poll_message(kind)
    }
    fn save_game(&mut self, slot: u16) {
        self.mechanism_save(slot);
    }
    fn run_other_pre_instruction(&mut self, address: u32) {
        pre_instruction(address, self);
    }
}

impl crate::screen_fade::Owner for PreparedActorState<'_> {
    fn with_screen_fade<R>(
        &mut self,
        action: impl FnOnce(&mut crate::screen_fade::ScreenFade) -> R,
    ) -> R {
        action(&mut self.session_mut().presentation.visual_mut().screen_fade)
    }
    fn cinematic_screen_fade(&mut self, fade: crate::screen_fade::CinematicFade) -> bool {
        if !self.with_screen_fade(|screen| screen.cinematic_ready(fade)) {
            return false;
        }
        // This is the original RAW sample, not the staged brightness projection.
        let brightness = self.aliases.actor_alias_byte(&self.owner.gameplay, 0x51);
        let Some((next, complete)) =
            self.with_screen_fade(|screen| screen.cinematic_brightness(fade, brightness))
        else {
            return true;
        };
        self.aliases
            .set_actor_alias_byte(&mut self.owner.gameplay, 0x51, next);
        let actual = self.aliases.actor_alias_byte(&self.owner.gameplay, 0x51);
        crate::session_encoding::write_projected_byte(&mut self.owner.gameplay, 0x51, actual);
        self.refresh_inputs();
        self.with_screen_fade(|screen| screen.cinematic_finish(fade, complete));
        complete
    }
}

/// Exhaustive original species table. Full address matching precedes the sole
/// low-word extraction in a recognized Phantoon arm; no bank-wide fallback.
pub(crate) fn pre_instruction(address: u32, host: &mut PreparedActorState<'_>) {
    let selected = host.session().actors.controls.selected;
    let routine = crate::actors::content::ActorRoutine(address);
    if crate::actors::prepared_remaining_species::try_gunship_pre_instruction(
        address, selected, host,
    ) || crate::actors::prepared_remaining_species::try_pre_instruction(address, selected, host)
        || crate::actors::prepared_remaining_species::try_b3_pre_instruction(
            routine, selected, host,
        )
    {
        return;
    }
    match address {
        10518651 | 10649723 | 10652277 | 10657819 | 10659799 | 10676097 | 10730689 | 10735455
        | 10739850 | 10741441 | 10998952 | 11047388 | 11192670 | 11197610 | 11200422 | 11174011
        | 11200474 | 11197611 => return,
        10999114 | 10999271 | 10999309 | 10999416 | 10999481 | 10999522 | 10999597 | 10999615
        | 10999655 | 10999765 | 10999850 | 10999900 | 10999953 | 10999980 | 11000086 | 11000110
        | 11000136 | 11000401 | 11000454 | 11000535 | 11000637 | 10999388 | 10998953 | 10999022
        | 10999048 | 10999190 | 10999508 | 10999688 | 10999799 | 10999924 | 11000203 => {
            let [low, high, _, _] = address.to_le_bytes();
            crate::actors::phantoon_owned::callback(
                host,
                crate::actors::phantoon_owned::Callback::PreInstruction {
                    function: u16::from_le_bytes([low, high]),
                    actor: selected,
                },
            );
            return;
        }
        _ => {}
    }
    if crate::actors::prepared_major_bosses::try_pre_instruction(
        routine,
        crate::actors::access::ActorKey::from_legacy(selected),
        host,
    ) {
        return;
    }
    host.unknown_function();
}
