//! `sm_game::actors::boss_lease` for Game, over the RAM image and its effects,
//! and for Runtime, on the leased Session.
#![forbid(unsafe_code)]
use super::{api, combat, queries};
use crate::game::session_runtime::Runtime;
use sm_game::actors::access::ActorKey;
use sm_game::actors::boss_lease::{BossLease, BossState};
use sm_game::actors::host::{Death, Shot};
use sm_game::geometry::{Point, Vector32};

fn death(death: Death) -> api::DeathHandling {
    match death {
        Death::Finalize => api::DeathHandling::Finalize,
        Death::LeavePending => api::DeathHandling::LeavePending,
    }
}

fn shot(shot: Shot) -> api::ShotHandling {
    match shot {
        Shot::ReactionOnly => api::ShotHandling::ReactionOnly,
        Shot::Flash => api::ShotHandling::Flash,
        Shot::Finalize => api::ShotHandling::Finalize,
    }
}

// Each effect is the one the game ran on a lease of its own, here on this one.
impl BossState for Runtime<'_> {
    fn react_to_shot(&mut self, handling: Shot) {
        combat::shot_reaction(self, shot(handling));
    }
    fn react_to_contact(&mut self, handling: Death) {
        combat::contact_reaction(self, death(handling));
    }
    fn place_projectile_spawn(&mut self, x: u16, y: u16) {
        self.session_mut().enemy_projectiles.spawn.point = Point { x, y };
    }
    fn spawn_room_eproj(&mut self, definition: u16, parameter: u16) -> u16 {
        sm_game::enemy_projectiles::lifecycle::spawn_room(self, definition, parameter)
    }
    fn spawn_actor_eproj(&mut self, parameter: u16, k: u16, definition: u16) -> u16 {
        sm_game::enemy_projectiles::lifecycle::spawn_from_enemy(self, parameter, k, definition)
    }
    fn roll_random(&mut self) -> u16 {
        self.next_random_number()
    }
    fn decompress_into_ram(&mut self, source: u32, destination: usize) {
        self.owned_player_call(|runtime| {
            sm_game::decompression::decompress_to_ram(source, destination, runtime)
        });
    }
    fn player_direction(&mut self) -> api::PlayerDirection {
        api::direction_to_player(self)
    }
    fn camera(&mut self) -> sm_game::camera::Camera {
        self.session().room.camera
    }
    fn current_enemy(&self) -> u16 {
        self.session().actors.controls.selected
    }
    fn area(&self) -> u16 {
        self.session().room.area
    }
    fn frame(&self) -> u16 {
        // The display owner's word while the lease holds it, the live RAM otherwise.
        self.word(sm_game::ram::words::NMI_FRAME_COUNTER_WORD)
    }
    fn scaled_cosine(&mut self, phase: u16, amplitude: u16) -> u16 {
        (queries::sine_product(self, amplitude, phase.wrapping_add(64) as u8 as u16) >> 16) as u16
    }
    fn scaled_sine(&mut self, phase: u16, amplitude: u16) -> u16 {
        (queries::sine_product(self, amplitude, phase.wrapping_add(0x80) as u8 as u16) >> 16) as u16
    }
    fn direction_step(&mut self, phase: u16, amplitude: u16) -> Vector32 {
        // $A0:B7EE, the sine table the game's direction vector reads.
        let game = &*self;
        let [x, y] = [0x40u16, 0x80].map(|offset| {
            let index = (phase.wrapping_add(offset) & 0x7f) as isize;
            u32::from(sm_game::cartridge::rom_table_word!(game, 0xa0b7ee, index))
                * u32::from(amplitude)
        });
        Vector32 { x, y }
    }
    fn step_along(&mut self, phase: u16, vector: Vector32) {
        api::apply_vector(self, phase, vector);
    }
    fn set_step_scratch(&mut self, value: u16) {
        self.session_mut().compatibility_enemy_scratch = value;
    }
    fn actor_record(&mut self, bank: u8, address: u16) -> crate::rom::RomRecord {
        self.actor_legacy_record(bank, address)
    }
    fn player_angle(&mut self, k: u16) -> u16 {
        api::angle_to_player(self, ActorKey::from_legacy(k))
    }
    fn shot_direction(&self) -> u16 {
        use sm_game::ram::{word_arrays::PROJECTILE_DIR, words::COLLISION_DETECTION_INDEX};
        self.word(PROJECTILE_DIR + 2 * usize::from(self.word(COLLISION_DETECTION_INDEX)))
    }
    fn current_random(&self) -> u16 {
        self.session().timing.random_number
    }
    fn select_enemy(&mut self, k: u16) {
        self.session_mut().actors.controls.selected = k;
    }
    fn kill_current(&mut self, kind: api::DeathKind) {
        combat::die(self, kind);
    }
    fn area_boss_byte(&mut self) -> u8 {
        sm_game::progress::boss_byte_at(self, self.session().room.area)
    }
}

impl BossLease for Runtime<'_> {
    fn owned_step<T>(&mut self, action: impl FnOnce(&mut Self) -> T) -> T {
        self.owned_player_call(action)
    }
}

impl sm_game::cartridge::NativeDiagnostics for Runtime<'_> {
    fn native_rom_diagnostic(&mut self, address: u32) {
        self.report_native_diagnostic(address);
    }
}
