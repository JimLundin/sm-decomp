// Crate-root declarations for the detached verification entrypoint.
mod animation;
mod animation_independent_checks;
mod composition_checks;
mod enemy_checks;
mod enemy_host;
mod flag_checks;
mod flag_host;
pub(crate) mod game;
mod palette;
mod palette_independent_checks;
mod player;
mod player_checks;
mod room_host;
mod session_host;
mod sprite_checks;
mod sprite_host;
mod visual_checks;
mod visual_host;
use game::room_mechanisms as room;
#[allow(
    unused_imports,
    unused_macros,
    reason = "Only RAM layout constants participate; raw compatibility macros have no consumers in this owned carrier."
)]
#[path = "../../ram_map.rs"]
pub(crate) mod ram_map;
#[path = "../../rom.rs"]
pub(crate) mod rom;
#[path = "../../verify_probe.rs"]
pub(crate) mod verify_probe;
use animation::execution as animated;
use game::visual_state::Upload;
use palette::execution as colors;
use rom::{RomImage, RomRecord};

#[derive(Clone, Debug, PartialEq)]
struct World {
    player: game::session::Session,
    player_inputs: game::external_inputs::ExternalInputs,
    flag_external: flag_host::ExternalBytes,
}
impl Default for World {
    fn default() -> Self {
        Self {
            player: game::session::Session::with_world(
                game::room_entry::world::WorldBinding::canonical_legacy(),
            ),
            player_inputs: Default::default(),
            flag_external: Default::default(),
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
enum Event {
    AtmosphericTile(usize, u16, u16, u16, u16),
    SamusSprite(u16, u16, u16),
    EnemySound(u8, u8, u16),
    EnemyRandom,
    /// A byte stored through an enemy projectile member past the slots.
    EnemyAlias(game::enemy_projectiles::host::EnemyMember, isize, u8),
    EnemyGrapple,
    RoomSound(u8, u8, u16),
    RoomMessage(u16, u16),
    RoomVisual(u16),
    RoomMusic(u16),
    RoomControl(u16, u16, u16),
    PlayerSound(u8, u8, u16),
    PlayerMechanism(u16),
    PlayerRecord(u8, u16),
    Trail(usize, bool, bool, u16, u16),
    Shot(u16, u16, u16),
    Beam(u16, u16, u16),
    PaletteSpawn(u16),
    Color(usize, u16),
    Music(u16),
    Sound(u8, u16),
    Projectile(game::visual_host::StatueProjectile, u16),
    Upload(Upload),
    Sprite(u16, u16, u16, u16, u16),
}
struct Composed {
    world: World,
    collision_contract: game::player_collision_host::CollisionContract,
    transition_contract: game::player_transitions_host::TransitionContract,
    image: RomImage,
    events: Vec<Event>,
    fail_upload: bool,
    animation_invalid_result: Option<u16>,
    late_message: Option<u8>,
    message_yields: std::collections::VecDeque<u8>,
    on_upload: Option<fn(&mut World)>,
    on_enemy_random: Option<fn(&mut Composed)>,
    on_enemy_sound: Option<fn(&mut Composed)>,
    expected_drop_award: Option<u16>,
    on_room_sound: Option<fn(&mut Composed)>,
    enemy_random: std::collections::VecDeque<u16>,
    on_visual_sound: Option<fn(&mut Composed)>,
    on_visual_music: Option<fn(&mut Composed)>,
    on_visual_record: Option<fn(&mut Composed, game::visual_host::Source, u8, u16)>,
    visual_records: Vec<(game::visual_host::Source, u8, u16)>,
    on_sprite_draw: Option<fn(&mut Composed)>,
    on_sprite_record: Option<fn(&mut Composed, u16)>,
    on_player_record: Option<fn(&mut Composed, u8, u16) -> Option<RomRecord>>,
    on_player_trail: Option<fn(&mut Composed, usize, bool, bool)>,
    on_player_unknown: Option<fn(&mut Composed) -> bool>,
    on_player_mechanism: Option<fn(&mut Composed, u16)>,
}
impl Composed {
    fn new() -> Self {
        Self {
            world: World::default(),
            collision_contract: Default::default(),
            transition_contract: Default::default(),
            image: RomImage::zeroed(0x300000),
            events: Vec::new(),
            fail_upload: false,
            animation_invalid_result: None,
            late_message: None,
            message_yields: Default::default(),
            on_upload: None,
            on_enemy_random: None,
            on_enemy_sound: None,
            expected_drop_award: None,
            on_room_sound: None,
            enemy_random: Default::default(),
            on_visual_sound: None,
            on_visual_music: None,
            on_visual_record: None,
            visual_records: Vec::new(),
            on_sprite_draw: None,
            on_sprite_record: None,
            on_player_record: None,
            on_player_trail: None,
            on_player_unknown: None,
            on_player_mechanism: None,
        }
    }
    fn byte(&self, bank: u8, cursor: u16, byte: u8) {
        let base = usize::from(bank & 0x7f) * 0x8000;
        self.image.write(base + usize::from(cursor & 0x7fff), byte);
    }
    fn word(&self, bank: u8, cursor: u16, value: u16) {
        let base = usize::from(bank & 0x7f) * 0x8000 + usize::from(cursor & 0x7fff);
        for (offset, byte) in value.to_ne_bytes().into_iter().enumerate() {
            self.image.write(base + offset, byte);
        }
    }
    fn tick(&mut self) {
        animated::advance(self);
        colors::advance(self);
        animated::upload_pending(self);
    }
}
mod checks;
