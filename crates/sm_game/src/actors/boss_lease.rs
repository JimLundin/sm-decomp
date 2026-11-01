//! A boss's per-frame lease: the state steps of its main AI run on the owned
//! Session, with the shared reactions and spawns it needs, and `foreign` runs
//! the rest on the game.
//!
//! # Design
//!
//! A species module writes its state steps against [`BossState`]: slot words
//! through `session_slot!`, named RAM through `word!`, sounds, the cartridge
//! and records, Samus's position, the shared actor operations, plus the few
//! effects every boss's frame uses. The game
//! implements it over the RAM image, delegating to its existing effects, and
//! the lease implements it on the Session, running those effects on the same
//! lease (they were leases of their own on the game). So a step behaves the same
//! called from either, and a main AI that opens one lease per frame keeps its
//! steps there, grouping what still needs the game into `foreign` calls.
use crate::actors::host::{Death, Shot};
use crate::actors::session_slots::SlotAccess;

/// What a boss's state steps read, write and call.
pub trait BossState:
    SlotAccess
    + crate::ram::CompatibilityRam
    + crate::effects::SoundEffects
    + crate::cartridge::LoadedImage
    + crate::cartridge::NativeDiagnostics
    + crate::records::Records
    + crate::player_position::PlayerPosition
    + crate::actors::host::ActorRuntime
    + crate::word_transfers::LoadedRom
    + crate::display::transfer::TilemapQueue
    + crate::sprite_drawing::DisplayObjects
{
    /// The shared shot reaction for the current actor.
    fn react_to_shot(&mut self, shot: Shot);
    /// The shared contact reaction for the current actor.
    fn react_to_contact(&mut self, death: Death);
    /// Where the next enemy projectile spawns.
    fn place_projectile_spawn(&mut self, x: u16, y: u16);
    /// Spawn the room-graphics enemy projectile `definition` with `parameter`.
    fn spawn_room_eproj(&mut self, definition: u16, parameter: u16) -> u16;
    /// Spawn enemy projectile `definition` from actor `k` with `parameter`.
    fn spawn_actor_eproj(&mut self, parameter: u16, k: u16, definition: u16) -> u16;
    /// The next random number.
    fn roll_random(&mut self) -> u16;
    /// Decompress the cartridge data at `source` into RAM at `destination`.
    fn decompress_into_ram(&mut self, source: u32, destination: usize);
    /// Which way Samus is from the current actor.
    fn player_direction(&mut self) -> crate::actors::api::PlayerDirection;
    /// The room camera.
    fn camera(&mut self) -> crate::camera::Camera;
    /// The current actor's slot offset, as the scheduler selected it.
    fn current_enemy(&self) -> u16;
    /// The current room's area.
    fn area(&self) -> u16;
    /// The NMI frame counter.
    fn frame(&self) -> u16;
    /// The high word of `amplitude` times the cosine of `phase`.
    fn scaled_cosine(&mut self, phase: u16, amplitude: u16) -> u16;
    /// The high word of `amplitude` times the sine of `phase`.
    fn scaled_sine(&mut self, phase: u16, amplitude: u16) -> u16;
    /// The (x, y) step of `amplitude` along direction `phase`.
    fn direction_step(&mut self, phase: u16, amplitude: u16) -> crate::geometry::Vector32;
    /// Move the current actor by `vector` along `phase`.
    fn step_along(&mut self, phase: u16, vector: crate::geometry::Vector32);
    /// The enemy scratch word the vector movement reads.
    fn set_step_scratch(&mut self, value: u16);
    /// The actor content record at `bank:address`, as the original actor read.
    fn actor_record(&mut self, bank: u8, address: u16) -> sm_data::cartridge::RomRecord;
    /// The angle from actor `k` to Samus.
    fn player_angle(&mut self, k: u16) -> u16;
    /// The direction of the projectile the current actor collided with.
    fn shot_direction(&self) -> u16;
    /// The random number, as last rolled.
    fn current_random(&self) -> u16;
    /// Make actor `k` the current actor.
    fn select_enemy(&mut self, k: u16);
}

/// A lease on a boss's game. The game is its own lease, whose `foreign` runs
/// the action on itself, so steps written against a lease also run unleased.
pub trait BossLease: BossState {
    type Game: ?Sized;
    fn foreign<T>(&mut self, action: impl FnOnce(&mut Self::Game) -> T) -> T;
}

/// The bosses' named Session words: their shared RAM, which the Session holds
/// as `actors.extended`'s last block, and the single words of other owners,
/// read and stored through the lease's projected bytes.
pub mod boss_ram {
    use crate::actors::session_slots::SlotAccess;
    /// A named boss RAM word, by its RAM address.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum Word {
        QuakeSoundTimer = 0x9000,
        CrocomireBridge1 = 0x9002,
        CrocomireBridge2 = 0x9006,
        CrocomireUnread = 0x900a,
        CrocomireAcidSmoke = 0x9018,
        PhantoonMaterialisationSound = 0x9032,
        /// `actors.drawing.tilemap_pending`.
        EnemyTilemapPending = 0x0e1e,
        /// `actors.drawing.tilemap_size`.
        EnemyTilemapSize = 0x179a,
        /// `actors.schedule.boss_id`.
        BossId = 0x179c,
        /// `actors.schedule.compatibility_179e[0]`.
        Unused179E = 0x179e,
        /// `minimap.disabled`.
        MinimapDisabled = 0x05f7,
        /// `room.liquid.lava_acid.whole`.
        LavaAcidY = 0x1962,
        /// `room.entry.enemy_interlock`.
        DoorTransitionEnemies = 0x0797,
        /// `room.camera.distance_index`.
        CameraDistance = 0x0941,
        /// `timing.random_number`.
        Random = 0x05e5,
        /// `room.fx_layer_flags`.
        FxLayerFlags = 0x1988,
        /// `messages.box_scratch[1]`, which Crocomire's melt borrows.
        MessageBoxScratch1 = 0x05a6,
        /// `REMOVED_R18`.
        RemovedR18 = 0x0012,
        /// `REMOVED_R20`.
        RemovedR20 = 0x0014,
        /// `REMOVED_R22`.
        RemovedR22 = 0x0016,
        /// `REG_M7A`.
        RegM7a = 0x0078,
        /// `REG_BG1HOFS`.
        RegBg1hofs = 0x00b1,
        /// `VRAM_WRITE_QUEUE_TAIL`.
        VramWriteQueueTail = 0x0330,
        /// `AREA_INDEX`.
        AreaIndex = 0x079f,
        /// `ROOM_WIDTH_IN_BLOCKS`.
        RoomWidthInBlocks = 0x07a5,
        /// `SCROLLING_FINISHED_HOOK`.
        ScrollingFinishedHook = 0x07e9,
        /// `CERES_STATUS`.
        CeresStatus = 0x093f,
        /// `TIMER_STATUS`.
        TimerStatus = 0x0943,
        /// `EQUIPPED_ITEMS`.
        EquippedItems = 0x09a2,
        /// `COLLECTED_ITEMS`.
        CollectedItems = 0x09a4,
        /// `EQUIPPED_BEAMS`.
        EquippedBeams = 0x09a6,
        /// `COLLECTED_BEAMS`.
        CollectedBeams = 0x09a8,
        /// `SAMUS_HEALTH`.
        SamusHealth = 0x09c2,
        /// `SAMUS_MAX_HEALTH`.
        SamusMaxHealth = 0x09c4,
        /// `SAMUS_MISSILES`.
        SamusMissiles = 0x09c6,
        /// `SAMUS_MAX_MISSILES`.
        SamusMaxMissiles = 0x09c8,
        /// `SAMUS_SUPER_MISSILES`.
        SamusSuperMissiles = 0x09ca,
        /// `SAMUS_MAX_SUPER_MISSILES`.
        SamusMaxSuperMissiles = 0x09cc,
        /// `SAMUS_POWER_BOMBS`.
        SamusPowerBombs = 0x09ce,
        /// `SAMUS_MAX_POWER_BOMBS`.
        SamusMaxPowerBombs = 0x09d0,
        /// `SAMUS_MAX_RESERVE_HEALTH`.
        SamusMaxReserveHealth = 0x09d4,
        /// `SAMUS_RESERVE_HEALTH`.
        SamusReserveHealth = 0x09d6,
        /// `JAPANESE_TEXT_FLAG`.
        JapaneseTextFlag = 0x09e2,
        /// `SAMUS_POSE`.
        SamusPose = 0x0a1c,
        /// `KNOCKBACK_X_DIR`.
        KnockbackXDir = 0x0a54,
        /// `SAMUS_GRAPPLE_FLAGS`.
        SamusGrappleFlags = 0x0a64,
        /// `SAMUS_X_SPEED_DIVISOR`.
        SamusXSpeedDivisor = 0x0a66,
        /// `SAMUS_X_POS`.
        SamusXPos = 0x0af6,
        /// `SAMUS_Y_POS`.
        SamusYPos = 0x0afa,
        /// `SAMUS_X_RADIUS`.
        SamusXRadius = 0x0afe,
        /// `SAMUS_Y_RADIUS`.
        SamusYRadius = 0x0b00,
        /// `EXTRA_SAMUS_X_SUBDISPLACEMENT`.
        ExtraSamusXSubdisplacement = 0x0b56,
        /// `EXTRA_SAMUS_X_DISPLACEMENT`.
        ExtraSamusXDisplacement = 0x0b58,
        /// `EXTRA_SAMUS_Y_DISPLACEMENT`.
        ExtraSamusYDisplacement = 0x0b5c,
        /// `PROJECTILE_COUNTER`.
        ProjectileCounter = 0x0cce,
        /// `POWER_BOMB_EXPLOSION_X_POS`.
        PowerBombExplosionXPos = 0x0ce2,
        /// `POWER_BOMB_EXPLOSION_Y_POS`.
        PowerBombExplosionYPos = 0x0ce4,
        /// `POWER_BOMB_FLAG`.
        PowerBombFlag = 0x0cee,
        /// `GRAPPLE_BEAM_FUNCTION`.
        GrappleBeamFunction = 0x0d32,
        /// `REMOVED_VAR_E22`.
        RemovedVarE22 = 0x0e22,
        /// `REMOVED_VAR_E2_A`.
        RemovedVarE2A = 0x0e2a,
        /// `RANDOM_ENEMY_COUNTER`.
        RandomEnemyCounter = 0x0e44,
        /// `CUR_ENEMY_INDEX`.
        CurEnemyIndex = 0x0e54,
        /// `FLAG_PROCESS_ALL_ENEMIES`.
        FlagProcessAllEnemies = 0x183c,
        /// `EARTHQUAKE_TYPE`.
        EarthquakeType = 0x183e,
        /// `EARTHQUAKE_TIMER`.
        EarthquakeTimer = 0x1840,
        /// `COLLISION_DETECTION_INDEX`.
        CollisionDetectionIndex = 0x18a6,
        /// `SAMUS_INVINCIBILITY_TIMER`.
        SamusInvincibilityTimer = 0x18a8,
        /// `SAMUS_KNOCKBACK_TIMER`.
        SamusKnockbackTimer = 0x18aa,
        /// `EPROJ_INIT_PARAM_2`.
        EprojInitParam2 = 0x1995,
        /// `PALETTE_CHANGE_NUM`.
        PaletteChangeNum = 0xc400,
    }
    /// `word` as the lease's projection holds it, low byte then high.
    pub fn get<H: SlotAccess + ?Sized>(h: &H, word: Word) -> u16 {
        u16::from_le_bytes([h.slot_byte(word as usize), h.slot_byte(word as usize + 1)])
    }
    /// Store `value` to `word`, low byte then high, as the named word store did.
    pub fn set<H: SlotAccess + ?Sized>(h: &mut H, word: Word, value: u16) {
        let [low, high] = value.to_le_bytes();
        h.set_slot_byte(word as usize, low);
        h.set_slot_byte(word as usize + 1, high);
    }
    /// A named Session-owned word array, by its RAM address.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum Array {
        ProjectileXPos = 0x0b64,
        ProjectileYPos = 0x0b78,
        ProjectileXRadius = 0x0bb4,
        ProjectileYRadius = 0x0bc8,
        ProjectileDir = 0x0c04,
        ProjectileType = 0x0c18,
        PaletteBuffer = 0xc000,
    }
    fn at<I: TryInto<usize, Error: core::fmt::Debug>>(array: Array, index: I) -> usize {
        array as usize + 2 * index.try_into().expect("word array index")
    }
    /// Word `index` of `array`, as the lease's projection holds it.
    pub fn get_at<H: SlotAccess + ?Sized, I: TryInto<usize, Error: core::fmt::Debug>>(
        h: &H,
        array: Array,
        index: I,
    ) -> u16 {
        let at = at(array, index);
        u16::from_le_bytes([h.slot_byte(at), h.slot_byte(at + 1)])
    }
    /// Store `value` to word `index` of `array`.
    pub fn set_at<H: SlotAccess + ?Sized, I: TryInto<usize, Error: core::fmt::Debug>>(
        h: &mut H,
        array: Array,
        index: I,
        value: u16,
    ) {
        let at = at(array, index);
        let [low, high] = value.to_le_bytes();
        h.set_slot_byte(at, low);
        h.set_slot_byte(at + 1, high);
    }
}

/// The BG2 tilemap staging RAM ($7E:2000..$7E:2FFF) by word index, which the
/// Session holds as `actors.extended`'s last two blocks and Ridley keeps its
/// tail in, through the lease's projected bytes.
pub mod bg2_staging {
    use crate::actors::session_slots::SlotAccess;
    const BASE: usize = 0x2000;
    /// Word `index`, low byte then high, as the named word read it.
    pub fn get<H: SlotAccess + ?Sized>(h: &H, index: usize) -> u16 {
        let at = BASE + 2 * index;
        u16::from_le_bytes([h.slot_byte(at), h.slot_byte(at + 1)])
    }
    /// Store `value` to word `index`, low byte then high.
    pub fn set<H: SlotAccess + ?Sized>(h: &mut H, index: usize, value: u16) {
        let [low, high] = value.to_le_bytes();
        h.set_slot_byte(BASE + 2 * index, low);
        h.set_slot_byte(BASE + 2 * index + 1, high);
    }
    /// Store `value` to each word index in `words`, in its order.
    pub fn fill<H: SlotAccess + ?Sized>(h: &mut H, words: impl Iterator<Item = usize>, value: u16) {
        for index in words {
            set(h, index, value);
        }
    }
}
