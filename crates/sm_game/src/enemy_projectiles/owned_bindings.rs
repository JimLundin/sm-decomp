//! Projectile execution on the actor interval's single current Session.
//! Include as a child of actors::owned_execution; no second owner is opened.
use super::{PreparedActorState, ProjectileRecordSource as RecordEdge};
use crate::actors::content_records::ContentSource;
use crate::enemy_projectiles::{content, host as enemy};
use crate::player_projectiles::{
    access::{ShotLane, Store},
    host as player,
};
use crate::session_access::SessionAccess;
use sm_data::cartridge::{RomImage, RomRecord};

fn indexed(base: usize, index: isize) -> usize {
    base.checked_add_signed(index.checked_mul(2).expect("projectile index overflow"))
        .expect("projectile extent overflow")
}
fn enemy_base(member: enemy::EnemyMember) -> usize {
    use enemy::EnemyMember::*;
    match member {
        Definition => 0x1997,
        Graphics => 0x19bb,
        Counter => 0x19df,
        PreAction => 0x1a03,
        XFraction => 0x1a27,
        X => 0x1a4b,
        YFraction => 0x1a6f,
        Y => 0x1a93,
        VelocityX => 0x1ab7,
        VelocityY => 0x1adb,
        BehaviorA => 0x1aff,
        BehaviorB => 0x1b23,
        Program => 0x1b47,
        Spritemap => 0x1b6b,
        Delay => 0x1b8f,
        Radius => 0x1bb3,
        Properties => 0x1bd7,
        BehaviorC => 0x1bfb,
        Angle => 0x97dc,
        DropFlags => 0xf380,
        DropDefinition => 0xf3c8,
        KilledEnemy => 0xf410,
    }
}
fn shot_lane(member: enemy::ShotMember) -> ShotLane {
    use enemy::ShotMember::*;
    match member {
        Kind => ShotLane::Kind,
        Damage => ShotLane::Damage,
        Direction => ShotLane::Direction,
        Variable => ShotLane::Variable,
        X => ShotLane::X,
        Y => ShotLane::Y,
        XRadius => ShotLane::XRadius,
        YRadius => ShotLane::YRadius,
    }
}
impl PreparedActorState<'_> {
    fn projectile_store(&mut self, at: usize, value: u16, store: Store) {
        let bytes = match store {
            Store::SliceNative => value.to_ne_bytes(),
            Store::SliceLe | Store::PrefixLe => value.to_le_bytes(),
        };
        match store {
            Store::PrefixLe => {
                self.set_current_byte(at, bytes[0]);
                self.set_current_byte(at.checked_add(1).expect("projectile word extent"), bytes[1]);
            }
            Store::SliceLe | Store::SliceNative => self.write_current_slice(at, &bytes),
        }
    }
    /// Original full-width variable byte selector; unlike shot words it keeps
    /// the selector's low bit and may cross into a different canonical owner.
    pub(crate) fn projectile_variable_byte(&self, raw: u16, high: bool) -> u8 {
        self.current_byte(0xc7c + usize::from(raw) + usize::from(high))
    }
}
impl enemy::Content for PreparedActorState<'_> {
    type Loaded = content::LoadedRecord;
    type Native = content::NativeRecord;
    fn script_image(&self) -> RomImage {
        self.content_image()
    }
    fn loaded_record(&mut self, bank: u8, address: u16) -> Self::Loaded {
        let record = self.projectile_record(RecordEdge::EnemyLoaded, bank, address);
        content::LoadedRecord::new(record.image, record.base)
    }
    fn native_record(&mut self, bank: u8, address: u16) -> Self::Native {
        let record = self.projectile_record(RecordEdge::EnemyNative, bank, address);
        content::NativeRecord::new(record.image, record.base)
    }
    fn screen_shake_word(&self, index: isize) -> u16 {
        self.loaded_table_word(0x86846b, index)
    }
    fn fire_arc_sine_sample(&mut self, phase: u8) -> u8 {
        // The original read_rom_byte uses the loaded allocation, captured only
        // when this read is reached; no native sine allocation substitution.
        self.content_record(0xa0, 0xb143).byte(usize::from(phase))
    }
}
impl enemy::ImportedAliases for PreparedActorState<'_> {
    fn enemy_word(&self, member: enemy::EnemyMember, index: isize) -> u16 {
        self.current_word(indexed(enemy_base(member), index))
    }
    fn set_enemy_word(&mut self, member: enemy::EnemyMember, index: isize, value: u16) {
        self.projectile_store(indexed(enemy_base(member), index), value, Store::SliceLe);
    }
    fn enemy_byte(&self, member: enemy::EnemyMember, index: isize) -> u8 {
        self.current_byte(
            enemy_base(member)
                .checked_add_signed(index)
                .expect("projectile byte extent"),
        )
    }
    fn set_enemy_byte(&mut self, member: enemy::EnemyMember, index: isize, value: u8) {
        self.set_current_byte(
            enemy_base(member)
                .checked_add_signed(index)
                .expect("projectile byte extent"),
            value,
        );
    }
    fn shot_word(&self, member: enemy::ShotMember, index: isize) -> u16 {
        // Signed -1 is a real failed-search alias. Do not convert to u16 or
        // route through ShotLane::address, which would discard its identity.
        self.current_word(indexed(shot_lane(member).address(0), index))
    }
    fn set_shot_direction(&mut self, index: isize, value: u16) {
        self.projectile_store(
            indexed(ShotLane::Direction.address(0), index),
            value,
            Store::SliceLe,
        );
    }
    fn mechanism_word(&self, member: enemy::MechanismMember, index: isize) -> u16 {
        let base = match member {
            enemy::MechanismMember::Block => 0x1c87,
            enemy::MechanismMember::RoomArgument => 0x1dc7,
        };
        self.current_word(indexed(base, index))
    }
    fn terrain_word(&self, block: u16) -> u16 {
        crate::room_terrain::level_word(
            self,
            usize::from(block) * 2,
            crate::room_mechanisms::host::ByteOrder::Little,
        )
    }
    fn behavior_byte(&self, block: u16) -> u8 {
        crate::room_terrain::bts(self, usize::from(block))
    }
}

impl crate::session_owners::AudioHost for PreparedActorState<'_> {
    fn audio_byte(&self, at: usize) -> u8 {
        self.current_byte(at)
    }
    fn audio_store_byte(&mut self, at: usize, value: u8) {
        self.set_current_byte(at, value);
    }
    fn audio_mutation<R>(
        &mut self,
        operation: impl FnOnce(
            &mut crate::session_owners::AudioRequests,
            &mut crate::scene::start::InitialAudioIntent,
            crate::session_owners::AudioStamp,
        ) -> Result<R, crate::scene::state::DisplayError>,
    ) -> Result<Option<R>, crate::scene::state::DisplayError> {
        use crate::scene::state::{DisplayError, DisplayPhase};
        let state = self.session_mut();
        let producing = state.presentation.owner()?.state.phase == DisplayPhase::Producing;
        let crate::session_owners::Control::Owned(control) = &mut state.control else {
            return Err(DisplayError::Phase);
        };
        let result = control.audio_mutation(producing, operation);
        if result.is_err() {
            state.presentation.owner_mut()?.state.phase = DisplayPhase::Failed;
        }
        result.map(Some)
    }
}

impl player::Host for PreparedActorState<'_> {
    fn loaded_image(&self) -> Option<RomImage> {
        Some(self.content_image())
    }
    fn native_image(&self) -> RomImage {
        self.native_content_image()
    }
    fn record(&mut self, bank: u8, cursor: u16) -> RomRecord {
        self.projectile_record(RecordEdge::PlayerLoaded, bank, cursor)
    }
    fn native_record(&mut self, bank: u8, cursor: u16) -> RomRecord {
        self.projectile_record(RecordEdge::PlayerNative, bank, cursor)
    }
    fn sound(&mut self, channel: u8, limit: u8, sound: u16) -> u16 {
        if !matches!(
            (channel, limit),
            (1, 6 | 9 | 15) | (2, 6) | (3, 3 | 6 | 9 | 15)
        ) {
            unreachable!("unknown projectile sound queue");
        }
        crate::music_requests::queue_sfx(
            usize::from(channel - 1),
            sound.to_le_bytes()[0],
            limit,
            self,
        )
        .expect("audio sound enqueue")
    }
    fn player_step(&mut self, step: player::PlayerStep) {
        if matches!(step, player::PlayerStep::SuitIndex) {
            crate::samus_palette::update_suit_index(self.session_mut());
        } else if matches!(step, player::PlayerStep::Radius) {
            let offset = usize::from(self.session().player.pose) * 8 + 6;
            let radius = u16::from(self.content_record(0x91, 0xb629).byte(offset));
            self.session_mut().player.contact.radius = radius;
            self.session_mut().player.contact.x_radius = 5;
            self.refresh_inputs();
        } else {
            self.projectile_player_step(step);
        }
    }
    fn palette(&mut self, effect: player::PaletteEffect) -> u8 {
        self.projectile_palette(effect)
    }
    fn enable_hdma(&mut self) {
        self.projectile_enable_hdma();
    }
    fn spawn_power_bomb(&mut self) {
        self.projectile_spawn_power_bomb();
    }
    fn spawn_mechanism(&mut self, header: u16) {
        crate::room_mechanisms::creation::spawn(
            self,
            crate::room_mechanisms::host::SpawnSource::Collision { header },
            &mut crate::room_mechanisms::host::Contact::Absent,
        );
    }
    fn draw(&mut self, drawing: player::Drawing) {
        self.projectile_draw(drawing);
    }
    fn draw_trail(&mut self, pair: usize, right: bool, frozen: bool) {
        self.projectile_draw_trail(pair, right, frozen);
    }
    fn invalid_damage(&mut self) {
        self.projectile_invalid_damage();
    }
    fn unknown(&mut self) -> bool {
        self.projectile_unknown()
    }
    fn room_level(&self, cell: u16) -> u16 {
        crate::room_terrain::level_word(
            self,
            usize::from(cell) * 2,
            crate::room_mechanisms::host::ByteOrder::Little,
        )
    }
    fn room_bts(&self, cell: u16) -> u8 {
        crate::room_terrain::bts(self, usize::from(cell))
    }
    fn alias_shot_read(&self, lane: ShotLane, raw: u16) -> u16 {
        self.current_word(lane.address(raw))
    }
    fn alias_shot_write(&mut self, lane: ShotLane, raw: u16, value: u16, store: Store) {
        self.projectile_store(lane.address(raw), value, store);
    }
    fn variable_byte(&self, raw: u16, high: bool) -> u8 {
        self.projectile_variable_byte(raw, high)
    }
    fn set_echo_x(&mut self, index: usize, value: u16) {
        self.projectile_store(0xab0 + index * 2, value, Store::PrefixLe);
    }
    fn set_echo_y(&mut self, index: usize, value: u16) {
        self.projectile_store(0xab8 + index * 2, value, Store::PrefixLe);
    }
    fn set_echo_index_native(&mut self, raw: u16, value: u16) {
        self.projectile_store(0xaae + usize::from(raw), value, Store::SliceNative);
    }
}

impl enemy::Effects for PreparedActorState<'_> {
    fn next_random(&mut self) -> u16 {
        let result = crate::random::next(self.session().timing.random_number);
        self.session_mut().timing.random_number = result;
        self.refresh_inputs();
        result
    }
    fn grapple_pickup_query(&mut self) -> u16 {
        self.projectile_player_command(0xd)
    }
    fn resume_player_after_elevator(&mut self) -> u16 {
        self.projectile_player_command(0xe)
    }
    fn calculate_angle_from_xy(&mut self, x: u16, y: u16) -> u16 {
        crate::actors::queries::angle_from_delta(x, y)
    }
    fn damage_player(&mut self, damage: u16) {
        crate::samus_periodic_damage::contact(self, damage);
    }
    fn ridley_offscreen(&mut self, x: u16, y: u16) -> u8 {
        self.projectile_ridley_solid(x, y)
    }
    fn queue_music(&mut self, track: u16) {
        crate::music_requests::queue_delayed_8(track, self);
    }
    fn convert_angle_to_xy(&mut self, angle: u16, amplitude: u16) -> crate::geometry::Vector32 {
        let [x, y] = [0x40_u16, 0x80].map(|offset| {
            let index = usize::from(angle.wrapping_add(offset) & 0x7f) * 2;
            let record = self.content_record(0xa0, 0xb7ee);
            let end = record
                .base
                .checked_add(index)
                .and_then(|at| at.checked_add(2))
                .expect("angle table extent");
            assert!(end <= record.image.len(), "angle table extent");
            let sample = u16::from_le_bytes([record.byte(index), record.byte(index + 1)]);
            u32::from(sample) * u32::from(amplitude)
        });
        crate::geometry::Vector32 { x, y }
    }
    fn queue_sound(&mut self, channel: u8, max: u8, sound: u16) {
        if !(1..=3).contains(&channel) || !matches!(max, 1 | 3 | 6 | 9 | 15) {
            unreachable!("invalid projectile sound channel/limit");
        }
        crate::music_requests::queue_sfx(
            usize::from(channel - 1),
            sound.to_le_bytes()[0],
            max,
            self,
        )
        .expect("audio sound enqueue");
    }
    fn unreachable(&mut self) {
        self.projectile_unknown();
    }
    fn draw_spritemap(
        &mut self,
        offscreen: bool,
        bank: u8,
        map: u16,
        position: crate::geometry::Point,
        tile: u16,
        graphics: u16,
    ) {
        self.projectile_draw_enemy(offscreen, bank, map, position, tile, graphics);
    }
    fn respawn_enemy(&mut self, index: u16) {
        crate::actors::population::respawn(
            self,
            crate::actors::access::ActorKey::from_legacy(index),
        );
    }
    fn reset_room_enemy_workspace(&mut self) {
        // The original fixed fill has one structurally valid complete span and
        // no callback or failure after admission. Clear its actual typed owners
        // in span order. Codec-omitted shadows retain their checkpoint values.
        // No dynamic alias query or partial allocation occurs in this interval.
        let session = self.session_mut();
        for record in &mut session.actors.saved {
            *record = Default::default();
        }
        session.actors.extended.clear_words(0, 0..1024);
        session.actors.extended.clear_words(1, 0..15);
        session.actors.extended.botwoon_head[0].pending_hit = 0;
        session.actors.extended.botwoon_head[0].active = 0;
        session.actors.extended.clear_words(1, 17..30);
        session.actors.extended.botwoon_head[0].tail_ready = 0;
        session.actors.extended.clear_words(1, 31..1024);
        session.actors.extended.clear_words(2, 0..12);
        session.actors.extended.botwoon_path[0].previous_health = 0;
        session.actors.extended.clear_words(2, 13..31);
        session.actors.extended.botwoon_path[0].body_landed = 0;
        session.actors.extended.clear_words(2, 32..1024);
        session.actors.extended.clear_words(3, 0..1006);
        for slot in &mut session.enemy_projectiles.slots {
            slot.angle = 0;
        }
        self.refresh_inputs();
    }
}
impl enemy::Host for PreparedActorState<'_> {}

impl PreparedActorState<'_> {
    /// The existing stem body uses the same prepared Host and exact actor,
    /// projectile and species owners, with no compatibility lease or selector
    /// recapture between its ordered X and Y passes.
    pub fn update_spore_stem(&mut self) {
        crate::enemy_projectiles::attachments_spore_maw::update_spore_stem(self);
    }
    /// Preparation retains Normal/Demo's different input/suit order, the
    /// projectile-only entry and the exact launch-history completion boundary.
    pub fn prepare_player_projectiles(&mut self, entry: crate::session::PrepareEntry) {
        crate::player_projectiles::preparation::prepare(self, entry);
    }
    pub fn advance_player_projectiles(&mut self) {
        crate::player_projectiles::motion::advance_all(self);
    }
    pub fn reset_player_projectiles(&mut self) {
        crate::player_projectiles::motion::reset(self);
    }
    pub fn clear_player_projectile(&mut self, shot: u16) {
        crate::player_projectiles::motion::clear(self, shot);
    }
    pub fn impact_player_projectile(&mut self, shot: u16) {
        crate::player_projectiles::lifecycle::impact(self, shot);
    }
    pub fn retire_player_projectile(&mut self, shot: u16) {
        crate::player_projectiles::lifecycle::retire(self, shot);
    }
    pub fn reflect_player_projectile(&mut self, shot: u16) {
        crate::player_projectiles::motion::reflect(self, shot);
    }
    pub fn initialize_player_projectile(
        &mut self,
        shot: u16,
        kind: crate::projectile_initialization::Initialization,
    ) {
        crate::player_projectiles::animation::initialize(self, shot, kind);
    }
    pub fn set_enemy_projectiles_enabled(&mut self, enabled: bool) {
        crate::enemy_projectiles::lifecycle::set_enabled(self, enabled);
    }
    pub fn clear_enemy_projectiles(&mut self) {
        crate::enemy_projectiles::lifecycle::clear(self);
    }
    pub fn advance_enemy_projectiles(&mut self) {
        crate::enemy_projectiles::lifecycle::advance(self);
    }
    pub fn spawn_room_projectile(&mut self, definition: u16, parameter: u16) -> u16 {
        crate::enemy_projectiles::lifecycle::spawn_room(self, definition, parameter)
    }
    pub fn spawn_actor_projectile(&mut self, parameter: u16, actor: u16, definition: u16) -> u16 {
        crate::enemy_projectiles::lifecycle::spawn_from_enemy(self, parameter, actor, definition)
    }
    pub fn resolve_hostile_projectile_player_contacts(&mut self) {
        crate::enemy_projectiles::contact::enemy_projectiles_hit_samus(self);
    }
    pub fn resolve_shot_hostile_projectile_contacts(&mut self) {
        crate::enemy_projectiles::contact::shoot_enemy_projectiles(self);
    }
    pub fn spawn_projectile_drop(&mut self, definition: u16, index: u16) {
        crate::enemy_projectiles::drops::spawn(definition, index, self);
    }
    pub fn scatter_projectile_drop(
        &mut self,
        source: crate::enemy_projectiles::drops::DropSource,
        index: u16,
    ) {
        crate::enemy_projectiles::drops::scatter(source, index, self);
    }
    pub fn reset_projectile_drop_metadata(&mut self) {
        crate::enemy_projectiles::drops::reset_room_metadata(self);
    }
}
