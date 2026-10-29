//! Enemy gameplay interfaces over the shared Session and explicit external effects.
#![forbid(unsafe_code)]
use crate::geometry::Vector32;
use crate::session_access::SessionAccess;
use sm_data::cartridge::RomImage;

#[derive(Clone, Copy)]
pub enum DefinitionField {
    Initialize,
    PreAction,
    Program,
    Radius,
    Properties,
    HitProgram,
    ShotProgram,
    EnemyDropTable,
}
pub trait LoadedRecord {
    fn byte(&self, byte_offset: isize) -> u8;
    fn native_word(&self, byte_offset: isize) -> u16;
    fn definition_word(&self, field: DefinitionField) -> u16;
}
pub trait NativeRecord {
    fn byte(&self, byte_offset: isize) -> u8;
    fn word_le(&self, word_index: isize) -> u16;
    fn long_le(&self, byte_offset: isize) -> u32;
}
pub trait Content {
    type Loaded: LoadedRecord;
    type Native: NativeRecord;
    // Capture loaded allocation for script frame; no low-window diagnostic.
    fn script_image(&self) -> RomImage;
    // Diagnostic before loaded/native allocation capture for low bank window.
    fn loaded_record(&mut self, bank: u8, bank_address: u16) -> Self::Loaded;
    fn native_record(&mut self, bank: u8, bank_address: u16) -> Self::Native;
    // Reproduce current loaded table capture and read boundaries.
    fn screen_shake_word(&self, word_index: isize) -> u16;
    fn waffle_sine_sample(&mut self, phase: u8) -> u8;
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EnemyMember {
    Definition,
    Graphics,
    Counter,
    PreAction,
    XFraction,
    X,
    YFraction,
    Y,
    VelocityX,
    VelocityY,
    BehaviorA,
    BehaviorB,
    Program,
    Spritemap,
    Delay,
    Radius,
    Properties,
    BehaviorC,
    Angle,
    DropFlags,
    DropDefinition,
    KilledEnemy,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShotMember {
    Kind,
    Damage,
    Direction,
    Variable,
    X,
    Y,
    XRadius,
    YRadius,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MechanismMember {
    Block,
    RoomArgument,
}
pub trait ImportedAliases {
    // Exceptional indices only. Normal accesses are ordinary Session members.
    fn enemy_word(&self, member: EnemyMember, signed_slot: isize) -> u16;
    fn set_enemy_word(&mut self, member: EnemyMember, signed_slot: isize, value: u16);
    fn enemy_byte(&self, member: EnemyMember, byte_index: isize) -> u8;
    fn set_enemy_byte(&mut self, member: EnemyMember, byte_index: isize, value: u8);
    fn shot_word(&self, member: ShotMember, signed_slot: isize) -> u16;
    fn set_shot_direction(&mut self, signed_slot: isize, value: u16);
    fn mechanism_word(&self, member: MechanismMember, signed_slot: isize) -> u16;
    fn terrain_word(&self, block_index: u16) -> u16;
    fn behavior_byte(&self, block_index: u16) -> u8;
}
#[derive(Clone, Copy)]
pub enum WaffleField {
    Behavior,
    Reverse,
    Radius,
    PhaseFraction,
    Phase,
    StartPhase,
    PhaseSpeedFraction,
    PhaseSpeed,
    CenterX,
    CenterY,
    PartCursor,
    Spacing,
    PhaseLead,
    EndPhase,
    FirstTransition,
    SecondTransition,
    HeadStage,
    Palette,
    Projectile,
    PartStage,
    Sprite,
    SpriteStage,
}
#[derive(Clone, Copy)]
pub enum BotwoonField {
    HistoryCursor,
    HistorySpacing,
    Projectile,
    Visible,
    TransitionPending,
    TransitionCursor,
    TailState,
    Pattern,
}
pub trait SpeciesActors {
    fn mini_kraid_spit_x_velocity(&self, index: u16) -> u16;
    fn mini_kraid_spit_y_velocity(&self, index: u16) -> u16;
    fn mini_kraid_spit_x_offset(&self, index: u16) -> u16;
    fn kraid_unk9000(&self) -> u16;
    fn nuclear_waffle_part_offset(&self, index: u16) -> u16;
    fn enemy_projectile_origin_x(&self, index: u16) -> u16;
    fn enemy_projectile_origin_y(&self, index: u16) -> u16;
    fn maw_body_part_index(&self, index: u16) -> u16;
    fn kraid_health_eighth(&self, index: u16) -> u16;
    fn mini_kraid_spike_variant(&self, index: u16) -> u16;
    fn set_enemy_linked_projectile(&mut self, index: u16, value: u16);
    fn set_botwoon_body_active(&mut self, index: u16, value: u16);
    fn set_maw_body_link(&mut self, index: u16, value: u16);
    fn mother_brain_active_bombs(&self) -> u16;
    fn mother_brain_blue_ring_contacts(&self) -> u16;
    fn mother_brain_target_enemy(&self) -> u16;
    fn mother_brain_turret_destroyed(&self) -> u16;
    fn botwoon_death_phase(&self) -> u16;
    fn ridley_fireball_x_velocity(&self) -> u16;
    fn ridley_fireball_y_velocity(&self) -> u16;
    fn set_mother_brain_active_bombs(&mut self, value: u16);
    fn set_mother_brain_blue_ring_contacts(&mut self, value: u16);
    fn set_mother_brain_hurt_frame(&mut self, value: u16);
    fn set_mother_brain_purple_breath_active(&mut self, value: u16);
    fn mother_brain_beam_x_fraction(&self) -> u16;
    fn set_mother_brain_beam_x_fraction(&mut self, value: u16);
    fn mother_brain_beam_x(&self) -> u16;
    fn set_mother_brain_beam_x(&mut self, value: u16);
    fn mother_brain_beam_y_fraction(&self) -> u16;
    fn set_mother_brain_beam_y_fraction(&mut self, value: u16);
    fn mother_brain_beam_y(&self) -> u16;
    fn set_mother_brain_beam_y(&mut self, value: u16);
    fn mother_brain_beam_velocity_x(&self) -> u16;
    fn set_mother_brain_beam_velocity_x(&mut self, value: u16);
    fn mother_brain_beam_velocity_y(&self) -> u16;
    fn set_mother_brain_beam_velocity_y(&mut self, value: u16);
    fn mother_brain_beam_angle(&self) -> u16;
    fn set_mother_brain_beam_angle(&mut self, value: u16);
    fn set_enemy_extra_health_fraction(&mut self, index: u16, element: usize, value: u16);
    fn set_botwoon_body_landed(&mut self, value: u16);
    fn spore_stem_anchor_x(&self) -> u16;
    fn spore_stem_anchor_y(&self) -> u16;
    fn set_spore_stem_control_x(&mut self, part: u8, value: u16);
    fn set_spore_stem_control_y(&mut self, part: u8, value: u16);
    fn maw_segment_handle(&self, enemy: u16, part: u8) -> u16;
    fn maw_anchor_x(&self, enemy: u16) -> u16;
    fn maw_anchor_y(&self, enemy: u16) -> u16;
    fn maw_segment_offset_x(&self, enemy: u16, part: u8) -> u16;
    fn maw_segment_offset_y(&self, enemy: u16, part: u8) -> u16;
    fn maw_palette(&self, enemy: u16) -> u16;
    fn expanding_square_origin(&self) -> u16;
    fn waffle_field(&self, enemy: usize, field: WaffleField) -> u16;
    fn set_waffle_field(&mut self, enemy: usize, field: WaffleField, value: u16);
    fn botwoon_field(&self, enemy: usize, field: BotwoonField) -> u16;
    fn set_botwoon_field(&mut self, enemy: usize, field: BotwoonField, value: u16);
    fn botwoon_history_x(&self, cursor: u16) -> u16;
    fn set_botwoon_history_x(&mut self, cursor: u16, value: u16);
    fn botwoon_history_y(&self, cursor: u16) -> u16;
    fn set_botwoon_history_y(&mut self, cursor: u16, value: u16);
}
pub trait Effects {
    fn next_random(&mut self) -> u16;
    fn grapple_pickup_query(&mut self) -> u16;
    fn resume_player_after_elevator(&mut self) -> u16;
    fn calculate_angle_from_xy(&mut self, x: u16, y: u16) -> u16;
    fn damage_player(&mut self, damage: u16) -> ();
    fn ridley_offscreen(&mut self, x: u16, y: u16) -> u8;
    fn queue_music(&mut self, track: u16) -> ();
    fn convert_angle_to_xy(&mut self, angle: u16, amplitude: u16) -> Vector32;
    fn queue_sound(&mut self, channel: u8, max: u8, sound: u16);
    fn unreachable(&mut self);
    fn draw_spritemap(
        &mut self,
        offscreen: bool,
        bank: u8,
        map: u16,
        x: u16,
        y: u16,
        tile: u16,
        graphics: u16,
    );
    fn respawn_enemy(&mut self, index: u16);
    fn reset_room_enemy_workspace(&mut self);
}
pub trait Graphics {
    fn vram_write_queue_tail(&self) -> u16;
    fn set_vram_write_queue_tail(&mut self, value: u16);
    fn set_vram_queue_size(&mut self, cursor: u16, value: u16);
    fn set_vram_queue_source(&mut self, cursor: u16, value: u16);
    fn set_vram_queue_bank_word(&mut self, cursor: u16, value: u16);
    fn set_vram_queue_destination(&mut self, cursor: u16, value: u16);
    fn set_bg2_y_scroll(&mut self, value: u16);
    fn set_palette_buffer(&mut self, index: isize, value: u16);
}
pub trait Host:
    SessionAccess
    + Content
    + ImportedAliases
    + SpeciesActors
    + crate::actors::access::ActorAccess
    + Effects
    + Graphics
    + crate::sprite_effect_host::Host
    + crate::gameplay_flag_host::Host
{
}
