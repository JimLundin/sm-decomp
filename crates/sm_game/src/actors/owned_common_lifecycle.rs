//! Shared actor lifecycle on the prepared Session. Rules have one existing body.
#![forbid(unsafe_code)]
use crate::actors::access::ActorKey;
use crate::actors::api::{DeathHandling, DeathKind, ShotHandling};
use crate::actors::content_records::ContentSource;
use crate::actors::deaths::Deaths;
use crate::actors::effects::ActorHost;
use crate::actors::host::{ActorRuntime, Death, Shot};
use crate::actors::owned_execution::PreparedActorState;
use crate::enemy_projectiles::host::{
    Effects as ProjectileEffects, Host as ProjectileHost, ShotMember,
};
use crate::geometry::{Point, Vector32};
use crate::session_access::SessionAccess;

/// Shared boss operations. Species keep their own finite field/slot contracts.
/// ActorRuntime is the finite movement/reaction API, not legacy ActorHost.
/// Native/odd aliases remain in those APIs' explicit host ports.
pub trait BossState:
    ActorHost + ActorRuntime + ProjectileHost + crate::room_mechanisms::host::Effects + Deaths + Sized
{
    fn react_to_shot(&mut self, shot: Shot) {
        crate::actors::combat::shot_reaction(
            self,
            match shot {
                Shot::ReactionOnly => ShotHandling::ReactionOnly,
                Shot::Flash => ShotHandling::Flash,
                Shot::Finalize => ShotHandling::Finalize,
            },
        );
    }
    fn react_to_contact(&mut self, death: Death) {
        crate::actors::combat::contact_reaction(
            self,
            match death {
                Death::Finalize => DeathHandling::Finalize,
                Death::LeavePending => DeathHandling::LeavePending,
            },
        );
    }
    fn place_projectile_spawn(&mut self, x: u16, y: u16) {
        self.session_mut().enemy_projectiles.spawn.point = Point { x, y };
    }
    fn spawn_room_eproj(&mut self, definition: u16, parameter: u16) -> u16 {
        crate::enemy_projectiles::lifecycle::spawn_room(self, definition, parameter)
    }
    fn spawn_actor_eproj(&mut self, parameter: u16, k: u16, definition: u16) -> u16 {
        crate::enemy_projectiles::lifecycle::spawn_from_enemy(self, parameter, k, definition)
    }
    fn roll_random(&mut self) -> u16 {
        ProjectileEffects::next_random(self)
    }
    fn player_direction(&mut self) -> crate::actors::api::PlayerDirection {
        crate::actors::queries::direction_to_player(self)
    }
    fn camera(&mut self) -> crate::camera::Camera {
        self.session().room.camera
    }
    fn current_enemy(&self) -> u16 {
        self.session().actors.controls.selected
    }
    fn area(&self) -> u16 {
        self.session().room.area
    }
    fn frame(&self) -> u16;
    fn scaled_cosine(&mut self, phase: u16, amplitude: u16) -> u16 {
        let phase = u16::from(phase.wrapping_add(64).to_le_bytes()[0]);
        let result = crate::actors::queries::sine_product(self, amplitude, phase);
        u16::try_from(result >> 16).expect("sine high word")
    }
    fn scaled_sine(&mut self, phase: u16, amplitude: u16) -> u16 {
        let phase = u16::from(phase.wrapping_add(128).to_le_bytes()[0]);
        let result = crate::actors::queries::sine_product(self, amplitude, phase);
        u16::try_from(result >> 16).expect("sine high word")
    }
    fn direction_step(&mut self, phase: u16, amplitude: u16) -> Vector32;
    fn step_along(&mut self, phase: u16, vector: Vector32) {
        crate::actors::movement::apply_vector(self, phase, vector);
    }
    fn set_step_scratch(&mut self, value: u16) {
        self.session_mut().compatibility_enemy_scratch = value;
    }
    fn actor_record(&mut self, bank: u8, address: u16) -> sm_data::cartridge::RomRecord;
    fn player_angle(&mut self, k: u16) -> u16 {
        crate::actors::queries::angle_to_player(self, ActorKey::from_legacy(k))
    }
    fn shot_direction(&self) -> u16 {
        crate::enemy_projectiles::access::shot_word(
            self,
            ShotMember::Direction,
            isize::try_from(self.session().room.collision_detection_index)
                .expect("full collision selector"),
        )
    }
    fn current_random(&self) -> u16 {
        self.session().timing.random_number
    }
    fn select_enemy(&mut self, k: u16) {
        self.session_mut().actors.controls.selected = k;
    }
    fn kill_current(&mut self, kind: DeathKind) {
        crate::actors::combat::die(self, kind);
    }
    fn area_boss_byte(&mut self) -> u8;
}

impl BossState for PreparedActorState<'_> {
    fn frame(&self) -> u16 {
        // Presentation owns this counter throughout a prepared actor interval.
        self.session()
            .presentation
            .owner()
            .expect("prepared actor presentation")
            .state
            .nmi
            .frame_word
    }
    fn direction_step(&mut self, phase: u16, amplitude: u16) -> Vector32 {
        self.ordinary_direction_vector(phase, amplitude)
    }
    fn actor_record(&mut self, bank: u8, address: u16) -> sm_data::cartridge::RomRecord {
        // Capture follows the actual low-window diagnostic; native acquisition
        // is a separate species port, not substituted for this loaded record.
        ContentSource::content_record(self, bank, address)
    }
    fn area_boss_byte(&mut self) -> u8 {
        crate::progress::boss_byte_at(self, self.session().room.area)
    }
}

impl Deaths for PreparedActorState<'_> {
    fn normal_death(&mut self, animation: u16) {
        self.guarded_actor_call(|host| {
            crate::actors::combat::die(host, DeathKind::Normal { animation });
        });
    }
    fn scatter_enemy_drop(
        &mut self,
        source: crate::enemy_projectiles::drops::DropSource,
        index: u16,
    ) {
        self.guarded_actor_call(|host| {
            crate::enemy_projectiles::drops::scatter(source, index, host);
        });
    }
}

impl PreparedActorState<'_> {
    pub fn lifecycle_prepare_room_assets(&mut self) {
        self.guarded_actor_call(crate::actors::staging::prepare_room_assets);
    }
    pub fn lifecycle_transfer_tiles_and_initialize(&mut self) {
        self.guarded_actor_call(crate::actors::staging::transfer_actor_tiles_and_initialize);
    }
    /// Called once at the room population phase, never when resuming a frame.
    pub fn lifecycle_install_population(&mut self) {
        self.guarded_actor_call(crate::actors::population::install_population);
    }
    /// Allocation/setup validates births before publication. A full population
    /// and partial initialization preserve the original selected-slot protocol.
    pub fn lifecycle_spawn(
        &mut self,
        request: crate::actors::api::ActorSpawnRequest,
    ) -> Result<
        Option<crate::actors::identity::InstanceHandle>,
        crate::actors::identity::ActorIdentityError,
    > {
        self.guarded_actor_call(|host| crate::actors::population::spawn(host, request))
    }
    /// Retained full-width content cursor for species whose actual setup still
    /// selects a legacy template; acquisition resolves the prepared catalog.
    pub fn lifecycle_spawn_template(&mut self, bank: u8, address: u16) -> u16 {
        self.guarded_actor_call(|host| crate::actors::population::spawn_legacy(host, bank, address))
    }
    pub fn lifecycle_respawn(&mut self, actor: ActorKey) {
        self.guarded_actor_call(|host| crate::actors::population::respawn(host, actor));
    }
    pub fn lifecycle_delete_connected(&mut self) {
        self.guarded_actor_call(crate::actors::population::delete_connected);
    }
    pub fn lifecycle_die(&mut self, kind: DeathKind) {
        self.guarded_actor_call(|host| crate::actors::combat::die(host, kind));
    }
    pub fn lifecycle_contact(&mut self, handling: DeathHandling) {
        self.guarded_actor_call(|host| crate::actors::combat::contact_reaction(host, handling));
    }
    pub fn lifecycle_shot(&mut self, handling: ShotHandling) {
        self.guarded_actor_call(|host| crate::actors::combat::shot_reaction(host, handling));
    }
    pub fn lifecycle_power_bomb(&mut self, handling: DeathHandling) {
        self.guarded_actor_call(|host| crate::actors::combat::power_bomb_reaction(host, handling));
    }
}
