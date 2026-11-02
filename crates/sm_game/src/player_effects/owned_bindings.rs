//! Concrete player callbacks on the prepared actor execution owner.
#![forbid(unsafe_code)]
use crate::actors::owned_execution::{PreparedActorState, ProjectileRecordSource};
use crate::session_access::SessionAccess;

impl crate::projectile_block_reactions::Definitions for PreparedActorState<'_> {
    type Table = RomRecord;
    fn resolve(&mut self, address: u32) -> Self::Table {
        crate::room_mechanisms::host::Content::loaded_record(
            self,
            crate::room_mechanisms::host::ContentAddress {
                bank: address.to_le_bytes()[2],
                offset: u16::from_le_bytes([address.to_le_bytes()[0], address.to_le_bytes()[1]]),
            },
        )
    }
    fn bts(&self) -> u8 {
        crate::player_projectiles::access::room_bts(self, self.session().room.collision_cursor)
    }
    fn area(&self) -> u16 {
        self.session().room.area
    }
    fn spawn(&mut self, header: u16) {
        crate::player_projectiles::host::Host::spawn_mechanism(self, header);
    }
}
use crate::{
    player_collision::effects as c, player_effects as p, player_movement as m,
    player_transitions as t, visual_host as v, visual_state::PaletteBuffer,
};
use sm_data::cartridge::RomRecord;

/// These finite interactions still need their actual external consumer. Each
/// receives the current canonical owner; none reports success by default.
pub trait PlayerPorts {
    fn samus_sprite(
        &mut self,
        owner: &mut crate::persistent_session::PersistentSession,
        index: u16,
        x: u16,
        y: u16,
    );
    fn atmospheric_tile(
        &mut self,
        owner: &mut crate::persistent_session::PersistentSession,
        slot: usize,
        kind: u16,
    );
    fn visual_upload(
        &mut self,
        owner: &mut crate::persistent_session::PersistentSession,
        request: crate::visual_state::Upload,
    );
    fn player_hdma(
        &mut self,
        owner: &mut crate::persistent_session::PersistentSession,
        bank: u8,
        definition: crate::display::hdma_spawn::HdmaObjectDefinition,
    );
    fn player_hdma_record(
        &mut self,
        owner: &mut crate::persistent_session::PersistentSession,
        bank: u8,
        definition: RomRecord,
    );
    fn xray_window(
        &mut self,
        owner: &mut crate::persistent_session::PersistentSession,
        row: u16,
        value: u16,
    );
    fn xray_tables(&mut self, owner: &mut crate::persistent_session::PersistentSession);
    fn enable_hdma(&mut self, owner: &mut crate::persistent_session::PersistentSession);
    fn message_step(
        &mut self,
        owner: &mut crate::persistent_session::PersistentSession,
        allocations: &mut crate::actors::owned_execution::ActorAllocations,
        presentation: &crate::scene::actor_presentation::Binding,
        samus_content: &crate::scene::prepared_samus::PreparedSamusContent,
        request: u16,
    ) -> u8;
    fn message_poll(
        &mut self,
        owner: &mut crate::persistent_session::PersistentSession,
        request: u16,
    ) -> i32;
    fn sram_native_word(&self, byte: usize) -> u16;
    fn set_sram_native_word(&mut self, byte: usize, word: u16);
    fn sram_snapshot(&self) -> Box<[u8]>;
    fn deliver_save(
        &mut self,
        owner: &mut crate::persistent_session::PersistentSession,
        request: p::save::SaveRequest,
    ) -> p::save::Delivery;
    fn retain_pending_save(
        &mut self,
        owner: &mut crate::persistent_session::PersistentSession,
        request: p::save::SaveRequest,
    );
}

impl PreparedActorState<'_> {
    pub(crate) fn projectile_player_step(
        &mut self,
        step: crate::player_projectiles::host::PlayerStep,
    ) {
        p::player_step(self, step);
    }
    pub(crate) fn projectile_palette(
        &mut self,
        effect: crate::player_projectiles::host::PaletteEffect,
    ) -> u8 {
        p::palette(self, effect)
    }
    pub(crate) fn projectile_player_command(&mut self, code: u16) -> u16 {
        p::command(self, code)
    }
    pub(crate) fn projectile_spawn_power_bomb(&mut self) {
        p::spawn_power_bomb(self);
    }
    pub(crate) fn projectile_enable_hdma(&mut self) {
        self.aliases.enable_hdma(self.owner);
        self.refresh_inputs();
    }
    pub(crate) fn projectile_invalid_damage(&mut self) {
        self.owner.poison();
        std::panic::panic_any(PlayerFault::InvalidDamage)
    }
    pub(crate) fn projectile_unknown(&mut self) -> bool {
        self.unknown_function()
    }
    pub(crate) fn mechanism_player_control(&mut self, operation: u16) {
        p::command(self, operation);
    }
    pub(crate) fn mechanism_face_forward(&mut self) {
        t::equipment::face_forward(self);
    }
    pub(crate) fn mechanism_suit_pickup(
        &mut self,
        pickup: crate::room_mechanisms::host::SuitPickup,
    ) {
        match pickup {
            crate::room_mechanisms::host::SuitPickup::Varia => p::begin_suit(self, false),
            crate::room_mechanisms::host::SuitPickup::Gravity => p::begin_suit(self, true),
            crate::room_mechanisms::host::SuitPickup::Unknown(_) => self.unknown_function(),
        }
    }
    pub(crate) fn mechanism_display_message(&mut self, request: u16) -> u8 {
        let result = self.aliases.message_step(
            self.owner,
            self.allocations,
            self.presentation,
            &self.samus_content,
            request,
        );
        self.refresh_inputs();
        result
    }
    pub(crate) fn mechanism_poll_message(&mut self, request: u16) -> i32 {
        let result = self.aliases.message_poll(self.owner, request);
        self.refresh_inputs();
        result
    }
    pub(crate) fn mechanism_save(&mut self, slot: u16) {
        if let p::save::Delivery::Pending(request) = p::save::prepare(self, slot) {
            self.aliases.retain_pending_save(self.owner, request);
        }
    }
    pub(crate) fn mechanism_pickup_visual(&mut self, visual: crate::item_pickups::VisualUpdate) {
        use crate::item_pickups::VisualUpdate::*;
        let index = match visual {
            None => return,
            Beams => {
                self.prepared_beam_tiles_and_palette();
                self.refresh_inputs();
                return;
            }
            Missiles => 0,
            SuperMissiles => 1,
            PowerBombs => 2,
            Grapple => 3,
            Xray => 4,
        };
        // The optional native icon record is captured before the blank test.
        let native = (index != 0).then(|| {
            self.native_actor_record(0x80, 0x99a7 + 8 * u16::try_from(index).expect("HUD icon"))
        });
        let base = [0x14, 0x1c, 0x22, 0x28, 0x2e][index] / 2;
        let value = self
            .session()
            .presentation
            .owner()
            .expect("owned HUD")
            .state
            .hud
            .tiles[base];
        if value & 0x3ff != 0x0f {
            return;
        }
        let columns = if index == 0 { 3 } else { 2 };
        let missiles = [0x344b, 0x3449, 0x744b, 0x344c, 0x344a, 0x744c];
        for (i, &missile) in missiles.iter().enumerate().take(2 * columns) {
            let word = native
                .as_ref()
                .map_or_else(|| missile, |record| record.word(2 * i));
            self.session_mut()
                .presentation
                .owner_mut()
                .expect("owned HUD")
                .state
                .hud
                .tiles[base + 32 * (i / columns) + i % columns] = word;
        }
    }
}
#[derive(Debug)]
pub enum PlayerFault {
    InvalidDamage,
    InvalidDemo,
    InvalidBounce,
    InvalidPalette,
    InvalidAnimation,
    InvalidClearance,
}

impl p::Host for PreparedActorState<'_> {
    fn set_xray_window(&mut self, row: u16, value: u16) {
        self.aliases.xray_window(self.owner, row, value);
        self.refresh_inputs();
    }
    fn setup_xray_tables(&mut self) {
        self.aliases.xray_tables(self.owner);
        self.refresh_inputs();
    }
    fn spawn_player_hdma(
        &mut self,
        bank: u8,
        definition: crate::display::hdma_spawn::HdmaObjectDefinition,
    ) {
        self.aliases.player_hdma(self.owner, bank, definition);
        self.refresh_inputs();
    }
    fn spawn_player_hdma_record(&mut self, bank: u8, definition: RomRecord) {
        self.aliases
            .player_hdma_record(self.owner, bank, definition);
        self.refresh_inputs();
    }
    fn set_time_frozen(&mut self, frozen: u16) {
        self.session_mut().timing.time_frozen = frozen;
        self.refresh_inputs();
    }
    fn finish_ceres_arrival(&mut self) {
        let parameter = crate::enemy_projectiles::lifecycle::spawn_from_enemy(self, 0, 0, 0xa387);
        crate::enemy_projectiles::lifecycle::spawn_from_enemy(self, parameter, 0, 0xa395);
        self.session_mut().minimap.disabled = 0;
        crate::room_mechanisms::host::Effects::delayed_room_music(self, 32);
    }
}
impl p::input::Host for PreparedActorState<'_> {
    fn set_input_lane(&mut self, lane: p::input::InputLane, value: u16) {
        use p::input::InputLane::*;
        let input = &mut self
            .session_mut()
            .control
            .metadata_mut()
            .expect("owned input")
            .input_history;
        match lane {
            Held => input.ports[0].sampled = value,
            Pressed => input.ports[0].pressed = value,
            FilteredHeld => input.samus_held = value,
            FilteredPressed => input.samus_pressed = value,
        }
        self.refresh_inputs();
    }
    fn direction_button(&self, left: bool) -> u16 {
        self.current_word(if left { 0x9ae } else { 0x9b0 })
    }
    fn input_unknown(&mut self) -> bool {
        self.unknown_function()
    }
}
impl crate::player_collision::grapple::Scratch for PreparedActorState<'_> {
    fn collision_scratch_byte(&self, offset: usize) -> u8 {
        sm_data::table::check_index(offset, 14);
        self.current_byte(0xd82 + offset)
    }
    fn clear_collision_scratch(&mut self) {
        self.write_current_slice(0xd82, &[0; 14]);
    }
    fn store_collision_scratch_word(&mut self, offset: usize, value: u16) {
        sm_data::table::check_index(offset, 13);
        let [low, high] = value.to_le_bytes();
        self.set_current_byte(0xd82 + offset, low);
        self.set_current_byte(0xd83 + offset, high);
    }
}
impl p::grapple::Host for PreparedActorState<'_> {
    fn grapple_unreachable(&mut self) -> bool {
        self.unknown_function()
    }
    fn grapple_pose_command(&mut self, code: u16) -> u16 {
        p::command(self, code)
    }
    fn player_bottom(&self) -> u16 {
        let radius = self.allocations.loaded().read(
            usize::try_from(sm_data::cartridge::lorom_offset(0x91b629)).expect("pose table")
                + usize::from(self.session().player.pose) * 8
                + 6,
        );
        self.session()
            .player
            .y
            .wrapping_add(u16::from(radius))
            .wrapping_sub(1)
    }
    fn settle_grapple(&mut self) {
        crate::player_collision::operations::settle_after_grapple(self);
    }
    fn block_behavior_word(&self, index: usize) -> i16 {
        i16::from_ne_bytes([
            self.current_byte(0x16402 + index),
            self.current_byte(0x16403 + index),
        ])
    }
    fn periodic_damage(&self) -> (u16, u16) {
        let player = &self.session().player;
        (
            u16::from_ne_bytes(player.periodic_damage_fraction.to_le_bytes()),
            u16::from_ne_bytes(player.periodic_damage.to_le_bytes()),
        )
    }
}
impl c::Content for PreparedActorState<'_> {
    type Record = RomRecord;
    fn pose_table(&mut self) -> RomRecord {
        v::Content::table(self, v::Source::Native, 0x91b629)
    }
    fn walked_pose_table(&mut self) -> RomRecord {
        v::Content::table(self, v::Source::Loaded, 0x91eb74)
    }
    fn special_header(&mut self, behavior: u8, solid: bool) -> c::MechanismHeader {
        let header = if behavior & 0x80 != 0 {
            let table = self.loaded_table_word(
                if solid { 0x9492e9 } else { 0x9492d9 },
                isize::try_from(self.session().room.area).expect("area"),
            );
            if table & 0x8000 == 0 {
                self.native_actor_record(0x94, table);
            }
            self.loaded_table_word(0x948000 | u32::from(table), isize::from(behavior & 0x7f))
        } else {
            self.loaded_table_word(0x949139, isize::from(behavior))
        };
        c::MechanismHeader(header)
    }
    fn bomb_header(&mut self, behavior: u8) -> c::MechanismHeader {
        c::MechanismHeader(self.loaded_table_word(0x94936b, isize::from(behavior)))
    }
}
impl c::ImportedCollisionReads for PreparedActorState<'_> {
    fn imported_level_word(&self, offset: usize) -> u16 {
        let [low, high] = crate::player_collision::geometry::level_word_bytes(offset);
        u16::from_le_bytes([self.current_byte(low), self.current_byte(high)])
    }
    fn imported_behavior_byte(&self, index: usize) -> u8 {
        self.current_byte(0x16402 + index)
    }
    fn imported_interactive_word(&self, offset: usize) -> u16 {
        self.current_word(0x17ec + offset)
    }
}
impl c::Effects for PreparedActorState<'_> {
    fn apply_mechanism(&mut self, header: c::MechanismHeader, displacement: &mut i32) -> u8 {
        crate::room_mechanisms::creation::spawn(
            self,
            crate::room_mechanisms::host::SpawnSource::Collision { header: header.0 },
            &mut crate::room_mechanisms::host::Contact::Local(displacement),
        )
    }
    fn enter_door(&mut self) -> crate::room_entry::api::EntryPoll {
        let result = self.room_begin(crate::room_entry::api::ArrivalRequest::Contact(
            crate::room_entry::api::ContactSite::CurrentCollision,
        ));
        self.refresh_inputs();
        result
    }
    fn cancel_speed_boost(&mut self) {
        crate::samus_palette::cancel_speed_boost(self);
    }
    fn boss_defeated(&mut self) -> bool {
        crate::progress::boss_defeated(self, 1)
    }
    fn vertical_slope_bts(&mut self, block: u16) -> u16 {
        let result = self.room_slope_pair(block);
        self.refresh_inputs();
        result
    }
    fn invalid_clearance_flags(&mut self, _flags: u16) {
        self.owner.poison();
        std::panic::panic_any(PlayerFault::InvalidClearance)
    }
}
impl t::content::Content for PreparedActorState<'_> {
    fn loaded_pose_table(&mut self) -> RomRecord {
        v::Content::table(self, v::Source::Loaded, 0x91b629)
    }
    fn transition_table(&mut self, table: t::content::TransitionTable) -> RomRecord {
        use t::content::TransitionTable::*;
        let (source, address) = match table {
            InputPrograms => (v::Source::Loaded, 0x919ee2),
            BlockFallPoses => (v::Source::Loaded, 0x91e921),
            BlockLandPoses => (v::Source::Loaded, 0x91e9f3),
            StandingTurn => (v::Source::Native, 0x91f9c2),
            CrouchingTurn => (v::Source::Native, 0x91f9cc),
            JumpingTurn => (v::Source::Native, 0x91f9d6),
            FallingTurn => (v::Source::Native, 0x91f9e0),
            MoonwalkJumpTurn => (v::Source::Native, 0x91f9ea),
        };
        v::Content::table(self, source, address)
    }
    fn animation_table(&mut self, table: t::content::AnimationTable) -> RomRecord {
        use t::content::AnimationTable::*;
        v::Content::table(
            self,
            v::Source::Loaded,
            match table {
                PosePrograms => 0x91b010,
                RunningWithoutBooster => 0x91b5d1,
                RunningBoosterStages => 0x91b5de,
                BoosterStageDurations => 0x91b61f,
            },
        )
    }
    fn animation_program(
        &mut self,
        program: t::content::AnimationProgram,
        advance: u16,
    ) -> RomRecord {
        self.native_actor_record(0x91, program.0.wrapping_add(advance))
    }
    fn landing_region(&self) -> t::content::LandingRegion {
        t::landing::area_region(self.session().room.area)
    }
    fn landing_room_rule(&self, region: t::content::LandingRegion) -> t::content::LandingRule {
        t::landing::room_rule(region, self.session().room.entry.room_index)
    }
}
impl t::effects::Effects for PreparedActorState<'_> {
    fn invalid_demo_instruction(&mut self) -> u16 {
        self.owner.poison();
        std::panic::panic_any(PlayerFault::InvalidDemo)
    }
    fn random_word(&mut self) -> u16 {
        crate::enemy_projectiles::host::Effects::next_random(self)
    }
    fn grapple_transition(&mut self, transition: t::effects::GrappleTransition) {
        match transition {
            t::effects::GrappleTransition::SwingPose => p::grapple::position_swing(self),
            t::effects::GrappleTransition::AnchoredPose => p::grapple::position_anchored(self),
        }
    }
    fn invalid_bounce(&mut self, _site: t::effects::BounceFault, _value: u16) {
        self.owner.poison();
        std::panic::panic_any(PlayerFault::InvalidBounce)
    }
}
impl t::effects::Host for PreparedActorState<'_> {}
impl m::content::Content for PreparedActorState<'_> {
    fn speed_record(&mut self, profile: m::content::SpeedRecord) -> RomRecord {
        use m::content::SpeedRecord::*;
        self.native_actor_record(
            0x90,
            match profile {
                Selected(cursor) => cursor,
                Bomb => 0x9f25,
                GrappleAir => 0x9f31,
                GrappleWater => 0x9f3d,
                GrappleLava => 0x9f49,
            },
        )
    }
    fn liquid_speed_table(&self, liquid: crate::liquid_surface::LiquidAtHeight) -> Option<u16> {
        use crate::liquid_surface::LiquidAtHeight::*;
        match liquid {
            None => Option::None,
            Water => Some(0xa08d),
            LavaAcid => Some(0xa1dd),
        }
    }
    fn spin_sound_frames(&self) -> RomRecord {
        v::Content::table(self, v::Source::Native, 0x90a514)
    }
    fn footstep_region(&self) -> m::content::FootstepRegion {
        use m::content::FootstepRegion::*;
        [
            Crateria, Default, Default, Default, Sand, Default, Default, Default,
        ][usize::from(self.session().room.area)]
    }
    fn footstep_room_rule(&self) -> m::content::FootstepRule {
        use m::content::FootstepRule::*;
        let room = self.session().room.entry.room_index;
        if room.wrapping_sub(16) & 0x8000 == 0 {
            return Default;
        }
        let flags = [1, 0, 0, 0, 0, 2, 0, 4, 0, 4, 4, 4, 4, 0, 4, 0][usize::from(room)];
        if flags & 1 != 0 {
            SandWhenFxTen
        } else if flags & 2 != 0 {
            SandAtOrBelow944
        } else if flags & 4 != 0 {
            Sand
        } else {
            Default
        }
    }
}
impl m::effects::Effects for PreparedActorState<'_> {
    fn boss_active(&self) -> bool {
        self.current_word(0x179c) != 0
    }
    fn initialize_shinespark_echo(&mut self, slot: u16) {
        crate::player_projectiles::animation::initialize(
            self,
            slot,
            crate::projectile_initialization::Initialization::Echo,
        );
    }
    fn write_echo_coordinate(&mut self, axis: m::effects::EchoAxis, index: usize, value: u16) {
        let base: usize = match axis {
            m::effects::EchoAxis::X => 0xab0,
            m::effects::EchoAxis::Y => 0xab8,
        };
        self.write_current_word_le(
            base.checked_add(index.checked_mul(2).expect("echo extent"))
                .expect("echo address"),
            value,
        );
    }
}
impl v::Content for PreparedActorState<'_> {
    type Record = RomRecord;
    fn record(&mut self, source: v::Source, bank: u8, address: u16) -> RomRecord {
        self.projectile_record(
            match source {
                v::Source::Loaded => ProjectileRecordSource::PlayerLoaded,
                v::Source::Native => ProjectileRecordSource::PlayerNative,
            },
            bank,
            address,
        )
    }
    fn table(&self, source: v::Source, address: u32) -> RomRecord {
        RomRecord {
            image: match source {
                v::Source::Loaded => self.allocations.loaded().clone(),
                v::Source::Native => self.allocations.native().clone(),
            },
            base: usize::try_from(sm_data::cartridge::lorom_offset(address)).expect("visual table"),
        }
    }
}

fn palette_address(member: v::PaletteMember, handle: u16) -> usize {
    use v::PaletteMember::*;
    let base = match member {
        Definition => 0x1e7d,
        ColorOffset => 0x1e8d,
        PreAction => 0x1ead,
        Program => 0x1ebd,
        Delay => 0x1ecd,
        Counter => 0x1edd,
    };
    base + usize::from(handle & !1)
}
fn animation_address(member: v::AnimationMember, handle: u16) -> usize {
    use v::AnimationMember::*;
    let base = match member {
        Definition => 0x1ef5,
        Counter => 0x1f01,
        Program => 0x1f0d,
        Delay => 0x1f19,
        Source => 0x1f25,
    };
    base + usize::from(handle & !1)
}
impl v::Aliases for PreparedActorState<'_> {
    fn palette_word(&self, member: v::PaletteMember, handle: u16) -> u16 {
        use v::PaletteMember::*;
        if let Some(slot) = self
            .session()
            .presentation
            .visual()
            .palette_effects
            .slots
            .get(usize::from(handle >> 1))
        {
            match member {
                Definition => slot.definition,
                ColorOffset => slot.color_offset,
                PreAction => slot.pre_action,
                Program => slot.program,
                Delay => slot.delay,
                Counter => slot.counter,
            }
        } else {
            self.current_word(palette_address(member, handle))
        }
    }
    fn set_palette_word_slice(&mut self, member: v::PaletteMember, handle: u16, value: u16) {
        use v::PaletteMember::*;
        if let Some(slot) = self
            .session_mut()
            .presentation
            .visual_mut()
            .palette_effects
            .slots
            .get_mut(usize::from(handle >> 1))
        {
            *match member {
                Definition => &mut slot.definition,
                ColorOffset => &mut slot.color_offset,
                PreAction => &mut slot.pre_action,
                Program => &mut slot.program,
                Delay => &mut slot.delay,
                Counter => &mut slot.counter,
            } = value;
            self.refresh_inputs();
        } else {
            self.write_current_slice(palette_address(member, handle), &value.to_le_bytes());
        }
    }
    fn set_palette_counter_byte(&mut self, handle: u16, value: u8) {
        self.set_current_byte(0x1edd + usize::from(handle), value);
    }
    fn palette_flag_native(&self, displacement: u16) -> u16 {
        let at = 0x1e79 + usize::from(displacement);
        u16::from_ne_bytes([self.current_byte(at), self.current_byte(at + 1)])
    }
    fn animation_word(&self, member: v::AnimationMember, handle: u16) -> u16 {
        use v::AnimationMember::*;
        if let Some(slot) = self
            .session()
            .presentation
            .visual()
            .animated_tiles
            .slots
            .get(usize::from(handle >> 1))
        {
            match member {
                Definition => slot.definition,
                Counter => slot.counter,
                Program => slot.program,
                Delay => slot.delay,
                Source => slot.source,
            }
        } else {
            self.current_word(animation_address(member, handle))
        }
    }
    fn set_animation_word_prefix(&mut self, member: v::AnimationMember, handle: u16, value: u16) {
        use v::AnimationMember::*;
        if let Some(slot) = self
            .session_mut()
            .presentation
            .visual_mut()
            .animated_tiles
            .slots
            .get_mut(usize::from(handle >> 1))
        {
            *match member {
                Definition => &mut slot.definition,
                Counter => &mut slot.counter,
                Program => &mut slot.program,
                Delay => &mut slot.delay,
                Source => &mut slot.source,
            } = value;
            self.refresh_inputs();
        } else {
            self.write_current_word_le(animation_address(member, handle), value);
        }
    }
    fn set_animation_counter_byte(&mut self, handle: u16, value: u8) {
        self.set_current_byte(0x1f01 + usize::from(handle), value);
    }
    fn set_palette_color_slice(&mut self, displacement: u16, value: u16) {
        let index = usize::from(displacement >> 1);
        if let Some(color) = self
            .session_mut()
            .presentation
            .visual_mut()
            .colors
            .get_mut(index)
        {
            *color = value;
        } else {
            self.write_current_slice(
                0xc000 + usize::from(displacement & !1),
                &value.to_le_bytes(),
            );
        }
    }
    fn set_color_prefix(&mut self, buffer: PaletteBuffer, index: usize, value: u16) {
        let visual = self.session_mut().presentation.visual_mut();
        let colors = match buffer {
            PaletteBuffer::Current => &mut visual.colors,
            PaletteBuffer::Target => &mut visual.targets,
        };
        if let Some(color) = colors.get_mut(index) {
            *color = value;
        } else {
            let base: usize = match buffer {
                PaletteBuffer::Current => 0xc000,
                PaletteBuffer::Target => 0xc200,
            };
            self.write_current_word_le(
                base.checked_add(index.checked_mul(2).expect("palette index"))
                    .expect("palette address"),
                value,
            );
        }
    }
}
impl v::Effects for PreparedActorState<'_> {
    fn draw_samus_sprite(&mut self, index: u16, x: u16, y: u16) {
        self.aliases.samus_sprite(self.owner, index, x, y);
        self.refresh_inputs();
    }
    fn draw_atmospheric_tile(&mut self, slot: usize, kind: u16) {
        self.aliases.atmospheric_tile(self.owner, slot, kind);
        self.refresh_inputs();
    }
    fn sound(&mut self, channel: u8, limit: u8, sound: u16) -> u16 {
        crate::player_projectiles::host::Host::sound(self, channel, limit, sound)
    }
    fn music_delayed8(&mut self, track: u16) {
        crate::music_requests::queue_delayed_8(track, self);
    }
    fn resume_movement_sound(&mut self) {
        p::command(self, 0x1c);
    }
    fn upload(&mut self, request: crate::visual_state::Upload) {
        self.aliases.visual_upload(self.owner, request);
        self.refresh_inputs();
    }
    fn invalid_palette_instruction(&mut self) {
        self.owner.poison();
        std::panic::panic_any(PlayerFault::InvalidPalette)
    }
    fn invalid_animation_instruction(&mut self) -> u16 {
        self.owner.poison();
        std::panic::panic_any(PlayerFault::InvalidAnimation)
    }
    fn color_written(&mut self, _index: u16, _value: u16) {}
}
impl v::Composition for PreparedActorState<'_> {
    fn spawn_palette(&mut self, definition: u16) {
        crate::palette_effects::execution::spawn(self, definition);
    }
    fn spawn_statue_projectile(&mut self, kind: v::StatueProjectile, parameter: u16) {
        let definition = match kind {
            v::StatueProjectile::EyeGlow => 0xba6a,
            v::StatueProjectile::Soul => 0xba94,
        };
        crate::enemy_projectiles::lifecycle::spawn_room(self, definition, parameter);
    }
}
impl v::Host for PreparedActorState<'_> {}
impl crate::effects::SoundEffects for PreparedActorState<'_> {
    fn queue_sound(&mut self, channel: usize, sound: u8, max_queued: u8) {
        crate::music_requests::queue_sfx(channel, sound, max_queued, self)
            .expect("owned sound enqueue");
    }
}

fn save_address(lane: p::save::Lane, index: usize) -> usize {
    use p::save::Lane::*;
    match lane {
        PlayerSource => 0xa00 - index * 2,
        SavedPlayer => 0xd7c0 + index,
        CurrentMap => 0x7f7 + index,
        AreaMap => 0xcd52 + index,
        PackedMap => 0xd91c + index,
        Payload => 0xd7c0 + index,
    }
}
impl p::save::Host for PreparedActorState<'_> {
    type Map = RomRecord;
    fn save_byte(&self, lane: p::save::Lane, index: usize) -> u8 {
        let at = save_address(lane, index);
        if let Some(value) = self.session().save_state.byte(at) {
            value
        } else if matches!(lane, p::save::Lane::CurrentMap) && index < 256 {
            self.session().minimap.explored[index]
        } else {
            self.current_byte(at)
        }
    }
    fn set_save_byte(&mut self, lane: p::save::Lane, index: usize, value: u8) {
        let at = save_address(lane, index);
        if !self.session_mut().save_state.set_byte(at, value) {
            self.set_current_byte(at, value);
        }
    }
    fn save_native_word(&self, lane: p::save::Lane, index: usize) -> u16 {
        let at = match lane {
            p::save::Lane::PlayerSource => save_address(lane, index),
            _ => save_address(lane, index * 2),
        };
        let read = |at| {
            self.session()
                .save_state
                .byte(at)
                .unwrap_or_else(|| self.current_byte(at))
        };
        u16::from_ne_bytes([read(at), read(at + 1)])
    }
    fn save_prefix_word(&mut self, lane: p::save::Lane, index: usize, value: u16) {
        let at = save_address(lane, index * 2);
        let [low, high] = value.to_le_bytes();
        if self.session().save_state.byte(at).is_some() {
            self.session_mut().save_state.set_byte(at, low);
            self.session_mut().save_state.set_byte(at + 1, high);
        } else {
            self.write_current_word_le(at, value);
        }
    }
    fn area_map_count(&self, area: usize) -> u8 {
        self.allocations.native().read(
            usize::try_from(sm_data::cartridge::lorom_offset(0x818131)).expect("map counts") + area,
        )
    }
    fn area_map_record(&mut self, area: usize) -> RomRecord {
        let cursor =
            crate::menu_lists::area_map_record(self, isize::try_from(area).expect("map area"));
        self.native_actor_record(0x81, cursor)
    }
    fn area_map_index(&self, record: &RomRecord, cursor: usize) -> u8 {
        record.byte(cursor)
    }
    fn area_map_bit_offset(&self, area: usize) -> u16 {
        crate::menu_lists::area_map_bit_offset(self, isize::try_from(area).expect("map area"))
    }
    fn save_slot_offset(&self, slot: u16) -> u16 {
        self.loaded_table_word(0x81812b, isize::try_from(slot).expect("save slot"))
    }
    fn sram_native_word(&self, byte: usize) -> u16 {
        self.aliases.sram_native_word(byte)
    }
    fn set_sram_native_word(&mut self, byte: usize, value: u16) {
        self.aliases.set_sram_native_word(byte, value);
    }
    fn sram_snapshot(&self) -> Box<[u8]> {
        self.aliases.sram_snapshot()
    }
    fn deliver_save(&mut self, request: p::save::SaveRequest) -> p::save::Delivery {
        self.aliases.deliver_save(self.owner, request)
    }
}
