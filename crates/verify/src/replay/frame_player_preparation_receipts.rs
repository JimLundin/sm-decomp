//! Named current-player owners and externally borrowed preparation effects.

use data::assets::samus::map;
use data::pose::{
    PoseId,
    transition::{Control, Controls, PoseTransitions},
};
use data::rom::{Rom, RomAddress};
use game::production::MovementContent;
use game::production::frame::borrowed_player_preparation::{
    ActorContentDisposition, ActorEffects, ActorGeometry, ActorRequiredDependencies, ActorScene,
    ActorSeed, ActorSolidity, ActorTouch, ActorUnselectedOrigin, ActorUpdateTerminal,
    ActorViewDisposition, AdditionalContactGate, AdministrativeCounter, ClockDisposition,
    CollectionClocks, ContactDamageDisposition, ContactResetOrigin, ContactResetTerminal,
    CounterEffect, DeferredActorUpdateReceipt, DeferredContactResetReceipt,
    DeferredProjectilesReceipt, DeferredSamusInteractionReceipt, DeferredSpriteObjectsReceipt,
    GraphicsUpdated, InstructionDisposition, InstructionSlotIdentity, InteractionDisposition,
    OrdinaryActorContact, ProjectileInstruction, ProjectilesOrigin, ProjectilesTerminal,
    SamusActivity, SamusContactGeometry, SamusInteractionOrigin, SamusInteractionTerminal,
    SolidContact, SpriteInstruction, SpriteObjectsOrigin, SpriteObjectsTerminal,
};
use game::production::frame::borrowed_player_preparation::{
    BombJumpDirection, DeferredEnvironmentReceipt, DeferredPoseInputReceipt,
    DeferredSuitPaletteReceipt, EnvironmentOrigin, InputBindings, LiquidSurface, PhysicalButtons,
    PoseCommand, PoseInputOrigin, PoseInputTerminal, PreparedEnvironment, SuitPaletteDisposition,
    SuitPaletteOrigin,
};
use serde::Deserialize;
use types::{Action, Buttons, Pose, button};

#[cfg(test)]
#[path = "frame_player_preparation_receipts/tests.rs"]
mod tests;

/// Invalid named current dependency, content binding or borrowed terminal.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlayerReceiptFault {
    /// Malformed projection.
    Format,
    /// Incomplete physical origin or invalid field extent.
    Length,
    /// Current owner differs from its complete admitted predicate.
    Dependency,
    /// Checked imported content differs from its admitted binding.
    Content,
    /// Unsupported owner meaning, recipe or causal contract.
    Unsupported,
}

#[derive(Deserialize)]
#[serde(transparent)]
struct ContractFlag(bool);

#[derive(Deserialize)]
struct CurrentField {
    owner: String,
    section: String,
    name: String,
    private_wram_offset: usize,
    quantity_width: u8,
    required_current_encoding: u32,
    ownership: String,
    revalidate_before_consumption: ContractFlag,
    #[serde(default)]
    semantic_decoder: String,
}

#[derive(Deserialize)]
struct TerminalField {
    owner: String,
    section: String,
    name: String,
    quantity_width: u8,
    ownership: String,
    qualified_borrowed_terminal_encoding: u32,
    current_complete_predicate_required: ContractFlag,
    #[serde(default)]
    semantic_decoder: String,
}

#[derive(Deserialize)]
struct SemanticDecoder {
    field: String,
    semantic_value: String,
    required_native_encoding: u16,
    quantity_width: u8,
    reject_other_encodings: ContractFlag,
}

#[derive(Deserialize)]
struct SemanticLabels {
    format: String,
    candidate_binding_sha256: String,
    decoder: Vec<SemanticDecoder>,
}

#[derive(Deserialize)]
struct EffectLink {
    owner: String,
    terminal_field: String,
    imported_operand_name: String,
    semantic_content_role: String,
    #[serde(default)]
    source_producer_kind: String,
    #[serde(default)]
    rom_bank: Option<u8>,
    #[serde(default)]
    rom_offset: Option<u16>,
    byte_width: u8,
    declared_encoding: u32,
    terminal_must_equal_checked_imported_operand: ContractFlag,
}

#[derive(Deserialize)]
struct EffectLinks {
    format: String,
    candidate_binding_sha256: String,
    effect_link: Vec<EffectLink>,
}

impl EffectLinks {
    fn parse(text: &str) -> Result<Self, PlayerReceiptFault> {
        let links: Self = toml::from_str(text).map_err(|_| PlayerReceiptFault::Format)?;
        if links.format != "player-entry-content-terminal-links-v1"
            || links.candidate_binding_sha256
                != "cb8c715ada04990e6bae72b4e63e7ea88e2ca02ac46d160b07451b75953e7254"
            || links.effect_link.len() != 3
        {
            return Err(PlayerReceiptFault::Unsupported);
        }
        Ok(links)
    }

    fn terminal(
        &self,
        source: &Projection,
        owner: &str,
        name: &str,
        operand: &str,
    ) -> Result<u16, PlayerReceiptFault> {
        let mut links = self
            .effect_link
            .iter()
            .filter(|link| link.owner == owner && link.terminal_field == name);
        let link = links.next().ok_or(PlayerReceiptFault::Unsupported)?;
        let value = source.terminal(owner, name)?;
        let role = if owner == "SuitPalette" {
            "PowerSuitPaletteDisposition"
        } else {
            name
        };
        if links.next().is_some()
            || link.semantic_content_role != role
            || link.imported_operand_name != operand
            || link.byte_width != 2
            || !link.terminal_must_equal_checked_imported_operand.0
            || link.declared_encoding != u32::from(value)
        {
            return Err(PlayerReceiptFault::Content);
        }
        if owner == "SuitPalette" {
            if link.source_producer_kind != "DirectZeroInitialization"
                || value != 0
                || link.rom_bank.is_some()
                || link.rom_offset.is_some()
            {
                return Err(PlayerReceiptFault::Unsupported);
            }
        } else {
            let mut content = source
                .imported_content
                .iter()
                .filter(|field| field.owner == owner && field.name == operand);
            let content_field = content.next().ok_or(PlayerReceiptFault::Content)?;
            if content.next().is_some()
                || !link.source_producer_kind.is_empty()
                || link.rom_bank != Some(content_field.rom_bank)
                || link.rom_offset != Some(content_field.rom_offset)
                || content_field.byte_width != link.byte_width
                || content_field.declared_encoding != link.declared_encoding
            {
                return Err(PlayerReceiptFault::Content);
            }
        }
        Ok(value)
    }
}

impl SemanticLabels {
    fn parse(text: &str) -> Result<Self, PlayerReceiptFault> {
        let labels: Self = toml::from_str(text).map_err(|_| PlayerReceiptFault::Format)?;
        if labels.format != "player-entry-exact-semantic-decoder-labels-v1"
            || labels.candidate_binding_sha256
                != "cb8c715ada04990e6bae72b4e63e7ea88e2ca02ac46d160b07451b75953e7254"
            || labels.decoder.len() != 8
        {
            return Err(PlayerReceiptFault::Unsupported);
        }
        Ok(labels)
    }

    fn validate(&self, field: &str, meaning: &str, actual: u16) -> Result<(), PlayerReceiptFault> {
        let mut decoders = self.decoder.iter().filter(|decoder| decoder.field == field);
        let decoder = decoders.next().ok_or(PlayerReceiptFault::Unsupported)?;
        if decoders.next().is_some()
            || decoder.semantic_value != meaning
            || decoder.quantity_width != 2
            || !decoder.reject_other_encodings.0
            || decoder.required_native_encoding != actual
        {
            return Err(PlayerReceiptFault::Unsupported);
        }
        Ok(())
    }
}

#[derive(Deserialize)]
struct ImportedContent {
    owner: String,
    name: String,
    rom_bank: u8,
    rom_offset: u16,
    byte_width: u8,
    declared_encoding: u32,
    importer_recipe: String,
    runtime_use: String,
    routine_dispatch_from_native_pointer: ContractFlag,
}

#[derive(Deserialize)]
struct TransitionRow {
    row_order: usize,
    required_pressed_controls: Vec<String>,
    required_held_controls: Vec<String>,
    required_pressed_encoding: u16,
    required_held_encoding: u16,
    target_pose_encoding: u16,
    admitted_at_actual_current_input: ContractFlag,
    importer_recipe: String,
}

#[derive(Deserialize)]
struct Projection {
    format: String,
    source_coordinates_role: String,
    selected_model_terminals_exported: ContractFlag,
    entry_overwritten_pending_pose_values_are_not_inputs: ContractFlag,
    prospective_pose_is_deferred_to_actual_later_pose_choice: ContractFlag,
    unused_source_scratch_is_not_gameplay_state: ContractFlag,
    named_imported_content_guard_required: ContractFlag,
    translated_pressed_actions: Vec<String>,
    translated_held_actions: Vec<String>,
    translated_actions_authority: String,
    pending_clears_then_prospective_selection_are_ordered_effects: ContractFlag,
    transition_witness_is_imported_content_not_runtime_selection: ContractFlag,
    current_field: Vec<CurrentField>,
    borrowed_terminal_field: Vec<TerminalField>,
    imported_content: Vec<ImportedContent>,
    transition_prefix: Vec<TransitionRow>,
}

fn scalar(bytes: &[u8], at: usize, width: u8) -> Result<u32, PlayerReceiptFault> {
    if !matches!(width, 1 | 2 | 4) {
        return Err(PlayerReceiptFault::Unsupported);
    }
    let end = at
        .checked_add(usize::from(width))
        .ok_or(PlayerReceiptFault::Length)?;
    let field = bytes.get(at..end).ok_or(PlayerReceiptFault::Length)?;
    match *field {
        [byte] => Ok(u32::from(byte)),
        [a, b] => Ok(u32::from(u16::from_le_bytes([a, b]))),
        [a, b, c, d] => Ok(u32::from_le_bytes([a, b, c, d])),
        _ => Err(PlayerReceiptFault::Unsupported),
    }
}

impl Projection {
    fn owner_shape(
        &self,
        owner: &str,
        currents: usize,
        terminals: usize,
    ) -> Result<(), PlayerReceiptFault> {
        if self
            .current_field
            .iter()
            .filter(|field| field.owner == owner)
            .count()
            != currents
            || self
                .borrowed_terminal_field
                .iter()
                .filter(|field| field.owner == owner)
                .count()
                != terminals
        {
            return Err(PlayerReceiptFault::Unsupported);
        }
        Ok(())
    }

    fn current_meaning(
        &self,
        owner: &str,
        name: &str,
        meaning: &str,
    ) -> Result<(), PlayerReceiptFault> {
        if self
            .current_field
            .iter()
            .filter(|field| field.owner == owner && field.name == name)
            .count()
            != 1
            || !self.current_field.iter().any(|field| {
                field.owner == owner && field.name == name && field.semantic_decoder == meaning
            })
        {
            return Err(PlayerReceiptFault::Unsupported);
        }
        Ok(())
    }

    fn parse(text: &str) -> Result<Self, PlayerReceiptFault> {
        let projection: Self = toml::from_str(text).map_err(|_| PlayerReceiptFault::Format)?;
        if projection.format != "player-entry-named-current-terminal-bindings-v2"
            || projection.source_coordinates_role != "ProvenanceOnly"
            || projection.selected_model_terminals_exported.0
            || !projection
                .entry_overwritten_pending_pose_values_are_not_inputs
                .0
            || !projection
                .prospective_pose_is_deferred_to_actual_later_pose_choice
                .0
            || !projection.unused_source_scratch_is_not_gameplay_state.0
            || !projection.named_imported_content_guard_required.0
            || !projection
                .pending_clears_then_prospective_selection_are_ordered_effects
                .0
            || !projection
                .transition_witness_is_imported_content_not_runtime_selection
                .0
            || projection.translated_actions_authority
                != "ActualCurrentPhysicalPadAndCheckedConfiguredBindings"
        {
            return Err(PlayerReceiptFault::Unsupported);
        }
        Ok(projection)
    }

    fn current(&self, bytes: &[u8], owner: &str, name: &str) -> Result<u16, PlayerReceiptFault> {
        if bytes.len() != 131_072 {
            return Err(PlayerReceiptFault::Length);
        }
        let mut fields = self
            .current_field
            .iter()
            .filter(|field| field.owner == owner && field.name == name);
        let field = fields.next().ok_or(PlayerReceiptFault::Unsupported)?;
        if fields.next().is_some()
            || field.section != format!("{owner}Origin")
            || field.ownership != "ActualCurrentOwnerAtThisOpportunity"
            || !field.revalidate_before_consumption.0
            || field.quantity_width != 2
        {
            return Err(PlayerReceiptFault::Unsupported);
        }
        let value = scalar(bytes, field.private_wram_offset, field.quantity_width)?;
        if value != field.required_current_encoding {
            return Err(PlayerReceiptFault::Dependency);
        }
        u16::try_from(value).map_err(|_| PlayerReceiptFault::Unsupported)
    }

    fn terminal(&self, owner: &str, name: &str) -> Result<u16, PlayerReceiptFault> {
        let mut fields = self
            .borrowed_terminal_field
            .iter()
            .filter(|field| field.owner == owner && field.name == name);
        let field = fields.next().ok_or(PlayerReceiptFault::Unsupported)?;
        if fields.next().is_some()
            || field.section != format!("{owner}Terminal")
            || field.ownership != "ActualBorrowedUnselectedOwnerCompletion"
            || !field.current_complete_predicate_required.0
            || field.quantity_width != 2
        {
            return Err(PlayerReceiptFault::Unsupported);
        }
        u16::try_from(field.qualified_borrowed_terminal_encoding)
            .map_err(|_| PlayerReceiptFault::Unsupported)
    }

    fn content(
        &self,
        rom: &Rom,
        owner: &str,
        expected_count: usize,
    ) -> Result<(), PlayerReceiptFault> {
        let fields: Vec<_> = self
            .imported_content
            .iter()
            .filter(|field| field.owner == owner)
            .collect();
        if fields.len() != expected_count {
            return Err(PlayerReceiptFault::Content);
        }
        for (index, field) in fields.iter().enumerate() {
            if fields
                .iter()
                .take(index)
                .any(|prior| prior.name == field.name)
                || field.importer_recipe != "ReadCheckedUnsignedValueAtNormalizedRomAddress"
                || field.runtime_use != "ImportedTypedContentOnly"
                || field.routine_dispatch_from_native_pointer.0
            {
                return Err(PlayerReceiptFault::Unsupported);
            }
            let bytes = rom
                .lorom()
                .read(
                    RomAddress::new(field.rom_bank, field.rom_offset),
                    usize::from(field.byte_width),
                )
                .map_err(|_| PlayerReceiptFault::Content)?;
            if scalar(bytes, 0, field.byte_width)? != field.declared_encoding {
                return Err(PlayerReceiptFault::Content);
            }
        }
        Ok(())
    }
}

fn pose(encoded: u16) -> Result<Pose, PlayerReceiptFault> {
    map::key_of(PoseId(encoded)).ok_or(PlayerReceiptFault::Unsupported)
}

fn ordinary_equipment(encoded: u16) -> Result<types::Set<types::Item>, PlayerReceiptFault> {
    let equipment = super::movement::items(encoded);
    if equipment[types::Item::GravitySuit] || equipment[types::Item::VariaSuit] {
        return Err(PlayerReceiptFault::Unsupported);
    }
    Ok(equipment)
}

fn actions(names: &[String]) -> Result<Buttons, PlayerReceiptFault> {
    if names != ["Right", "Run", "AimDown"] {
        return Err(PlayerReceiptFault::Unsupported);
    }
    Ok(Buttons(
        button::RIGHT.0 | button::RUN.0 | button::AIM_DOWN.0,
    ))
}

fn controls(names: &[String]) -> Result<Controls, PlayerReceiptFault> {
    let controls = names
        .iter()
        .map(|name| match name.as_str() {
            "Down" => Ok(Control::Down),
            "Jump" => Ok(Control::Jump),
            "Right" => Ok(Control::Right),
            "AimUp" => Ok(Control::AimUp),
            "AimDown" => Ok(Control::AimDown),
            _ => Err(PlayerReceiptFault::Unsupported),
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Controls::of(&controls))
}

/// Decode the complete current ordinary pose-input owner at its actual consumption edge.
/// Source coordinates and private scratch fields are never gameplay inputs.
/// # Errors
/// Rejects changed current predicates, malformed bindings and unsupported pose/action.
pub fn current_pose(bytes: &[u8], projection: &str) -> Result<PoseInputOrigin, PlayerReceiptFault> {
    let source = Projection::parse(projection)?;
    source.owner_shape("PoseInput", 10, 9)?;
    source.current_meaning(
        "PoseInput",
        "SAMUS_POSE",
        "CheckedImportedOrdinaryRunningPoseIdentity",
    )?;
    source.current_meaning(
        "PoseInput",
        "SAMUS_MOVEMENT_TYPE",
        "CheckedImportedRunningAction",
    )?;
    let read = |name| source.current(bytes, "PoseInput", name);
    let current_pose = pose(read("SAMUS_POSE")?)?;
    let _ = read("SAMUS_MOVEMENT_TYPE")?;
    if current_pose.action != Action::Running {
        return Err(PlayerReceiptFault::Unsupported);
    }
    Ok(PoseInputOrigin {
        pose: current_pose,
        action: Action::Running,
        held: PhysicalButtons(read("JOYPAD1_LASTKEYS")?),
        pressed: PhysicalButtons(read("JOYPAD1_NEWKEYS")?),
        bindings: InputBindings {
            shoot: PhysicalButtons(read("ButtonConfigShoot")?),
            jump: PhysicalButtons(read("ButtonConfigJump")?),
            run: PhysicalButtons(read("BUTTON_CONFIG_RUN_B")?),
            cancel: PhysicalButtons(read("ButtonConfigCancel")?),
            aim_up: PhysicalButtons(read("ButtonConfigAimUp")?),
            aim_down: PhysicalButtons(read("ButtonConfigAimDown")?),
        },
    })
}

/// Import external pose-input effects with actual current predicates and checked ROM content.
/// The four-row witness is compared to imported content; no transition selector is executed.
/// Completion remains deferred and requires fresh current-owner validation in game.
/// # Errors
/// Rejects malformed/changed dependencies, content witnesses and unsupported terminal arms.
pub fn pose_offer(
    bytes: &[u8],
    projection: &str,
    labels: &str,
    rom: &Rom,
    content: &MovementContent,
) -> Result<DeferredPoseInputReceipt, PlayerReceiptFault> {
    let source = Projection::parse(projection)?;
    let labels = SemanticLabels::parse(labels)?;
    let dependency = current_pose(bytes, projection)?;
    source.content(rom, "PoseInput", 19)?;
    let raw_pose = source.current(bytes, "PoseInput", "SAMUS_POSE")?;
    let transitions =
        PoseTransitions::load(rom.lorom()).map_err(|_| PlayerReceiptFault::Content)?;
    if source.transition_prefix.len() != 4 {
        return Err(PlayerReceiptFault::Content);
    }
    let expected = [
        (vec!["Down"], vec![]),
        (vec!["Jump"], vec![]),
        (vec![], vec!["Right", "AimUp"]),
        (vec![], vec!["Right", "AimDown"]),
    ];
    for (index, (witness, (pressed, held))) in
        source.transition_prefix.iter().zip(expected).enumerate()
    {
        let actual = transitions
            .of(PoseId(raw_pose))
            .get(index)
            .ok_or(PlayerReceiptFault::Content)?;
        if witness.row_order != index
            || witness.importer_recipe != "CheckedPoseTransitionsConsultedPrefix"
            || witness.admitted_at_actual_current_input.0 != (index == 3)
            || witness.required_pressed_controls != pressed
            || witness.required_held_controls != held
            || controls(&witness.required_pressed_controls)?.0 != witness.required_pressed_encoding
            || controls(&witness.required_held_controls)?.0 != witness.required_held_encoding
            || actual.pressed.0 != witness.required_pressed_encoding
            || actual.held.0 != witness.required_held_encoding
            || actual.pose.0 != witness.target_pose_encoding
        {
            return Err(PlayerReceiptFault::Content);
        }
    }
    let prospective = source.terminal("PoseInput", "ProspectivePose")?;
    if source
        .transition_prefix
        .last()
        .is_none_or(|row| row.target_pose_encoding != prospective)
    {
        return Err(PlayerReceiptFault::Unsupported);
    }
    for (field, meaning) in [
        ("SpecialProspectivePose", "NoSpecialProspectivePose"),
        (
            "SAMUS_NEW_POSE_TRANSITIONAL",
            "NoSuperSpecialProspectivePose",
        ),
        ("ProspectivePoseCommand", "NoProspectivePoseCommand"),
        (
            "SpecialProspectivePoseCommand",
            "NoSpecialProspectivePoseCommand",
        ),
        (
            "SAMUS_HURT_SWITCH_INDEX",
            "NoSuperSpecialProspectivePoseCommand",
        ),
        ("BombJumpDirection", "NoBombJumpDirection"),
    ] {
        labels.validate(field, meaning, source.terminal("PoseInput", field)?)?;
    }
    let radius = content
        .radius(dependency.pose)
        .ok_or(PlayerReceiptFault::Content)?;
    if radius.x.0 != source.terminal("PoseInput", "SAMUS_X_RADIUS")?
        || radius.y.0 != source.terminal("PoseInput", "SAMUS_Y_RADIUS")?
    {
        return Err(PlayerReceiptFault::Content);
    }
    Ok(DeferredPoseInputReceipt {
        dependency,
        terminal: PoseInputTerminal {
            pressed_actions: actions(&source.translated_pressed_actions)?,
            held_actions: actions(&source.translated_held_actions)?,
            prospective: pose(prospective)?,
            special: None,
            super_special: None,
            commands: [PoseCommand::None; 3],
            radius,
            bomb_jump: BombJumpDirection::None,
        },
    })
}

/// Decode current equipment at the suit-palette consumption edge.
/// # Errors
/// Rejects changed current predicates or suit alternatives.
pub fn current_suit(
    bytes: &[u8],
    projection: &str,
) -> Result<SuitPaletteOrigin, PlayerReceiptFault> {
    let source = Projection::parse(projection)?;
    source.owner_shape("SuitPalette", 1, 1)?;
    source.current_meaning(
        "SuitPalette",
        "EQUIPPED_ITEMS",
        "CheckedImportedEquipmentWithoutGravityOrVaria",
    )?;
    Ok(SuitPaletteOrigin {
        equipment: ordinary_equipment(source.current(bytes, "SuitPalette", "EQUIPPED_ITEMS")?)?,
    })
}

/// Import a separately qualified power-suit palette completion.
/// # Errors
/// Rejects changed dependencies, content bindings and unsupported palette meaning.
pub fn suit_offer(
    bytes: &[u8],
    projection: &str,
    links: &str,
    rom: &Rom,
) -> Result<DeferredSuitPaletteReceipt, PlayerReceiptFault> {
    let source = Projection::parse(projection)?;
    let links = EffectLinks::parse(links)?;
    let dependency = current_suit(bytes, projection)?;
    source.content(rom, "SuitPalette", 2)?;
    if !source.borrowed_terminal_field.iter().any(|field| {
        field.owner == "SuitPalette"
            && field.name == "SuitPaletteDisposition"
            && field.semantic_decoder == "CheckedImportedPowerSuitPaletteDisposition"
    }) {
        return Err(PlayerReceiptFault::Unsupported);
    }
    let _ = links.terminal(
        &source,
        "SuitPalette",
        "SuitPaletteDisposition",
        "CheckedPowerSuitDispositionCodec",
    )?;
    Ok(DeferredSuitPaletteReceipt {
        dependency,
        terminal: SuitPaletteDisposition::PowerSuit,
    })
}

/// Decode complete current equipment, pose, centre and inactive liquid owners.
/// # Errors
/// Rejects changed current predicates, suit alternatives or unsupported pose.
pub fn current_environment(
    bytes: &[u8],
    projection: &str,
    labels: &str,
) -> Result<EnvironmentOrigin, PlayerReceiptFault> {
    let source = Projection::parse(projection)?;
    source.owner_shape("Environment", 5, 2)?;
    let labels = SemanticLabels::parse(labels)?;
    source.current_meaning(
        "Environment",
        "EQUIPPED_ITEMS",
        "CheckedImportedEquipmentWithoutGravityOrVaria",
    )?;
    let read = |name| source.current(bytes, "Environment", name);
    labels.validate(
        "FxLiquidSurface",
        "InactiveLiquidSurface",
        read("FxLiquidSurface")?,
    )?;
    labels.validate(
        "LavaAcidSurface",
        "InactiveLiquidSurface",
        read("LavaAcidSurface")?,
    )?;
    Ok(EnvironmentOrigin {
        equipment: ordinary_equipment(read("EQUIPPED_ITEMS")?)?,
        pose: pose(read("SAMUS_POSE")?)?,
        centre_y: read("SAMUS_Y_POS")?,
        water: LiquidSurface::Inactive,
        lava_acid: LiquidSurface::Inactive,
    })
}

/// Import prepared ordinary acceleration with independently checked reference content.
/// # Errors
/// Rejects changed current dependencies or immutable reference content bindings.
pub fn environment_offer(
    bytes: &[u8],
    projection: &str,
    labels: &str,
    links: &str,
    rom: &Rom,
) -> Result<DeferredEnvironmentReceipt, PlayerReceiptFault> {
    let source = Projection::parse(projection)?;
    let links = EffectLinks::parse(links)?;
    let dependency = current_environment(bytes, projection, labels)?;
    source.content(rom, "Environment", 3)?;
    let acceleration = types::fixed(
        core::num::Wrapping(links.terminal(
            &source,
            "Environment",
            "VerticalAccelerationWhole",
            "EnvironmentSurfaceOrAccelerationContent_2",
        )?),
        links.terminal(
            &source,
            "Environment",
            "VerticalAccelerationFraction",
            "EnvironmentSurfaceOrAccelerationContent_1",
        )?,
    );
    Ok(DeferredEnvironmentReceipt {
        dependency,
        terminal: PreparedEnvironment { acceleration },
    })
}

/// Import the ordinary acceleration directly from its checked immutable ROM operands.
/// This never initializes Rules from a captured selected or borrowed terminal value.
/// # Errors
/// Rejects malformed effect-role links, mismatched ROM bindings or unavailable content.
pub fn reference_acceleration(
    projection: &str,
    links: &str,
    rom: &Rom,
) -> Result<types::Fixed, PlayerReceiptFault> {
    let source = Projection::parse(projection)?;
    source.content(rom, "Environment", 3)?;
    let links = EffectLinks::parse(links)?;
    let read = |name: &str, operand: &str| -> Result<u16, PlayerReceiptFault> {
        let _ = links.terminal(&source, "Environment", name, operand)?;
        let link = links
            .effect_link
            .iter()
            .find(|link| link.owner == "Environment" && link.terminal_field == name)
            .ok_or(PlayerReceiptFault::Content)?;
        let bytes = rom
            .lorom()
            .read(
                RomAddress::new(
                    link.rom_bank.ok_or(PlayerReceiptFault::Content)?,
                    link.rom_offset.ok_or(PlayerReceiptFault::Content)?,
                ),
                usize::from(link.byte_width),
            )
            .map_err(|_| PlayerReceiptFault::Content)?;
        u16::try_from(scalar(bytes, 0, link.byte_width)?).map_err(|_| PlayerReceiptFault::Content)
    };
    Ok(types::fixed(
        core::num::Wrapping(read(
            "VerticalAccelerationWhole",
            "EnvironmentSurfaceOrAccelerationContent_2",
        )?),
        read(
            "VerticalAccelerationFraction",
            "EnvironmentSurfaceOrAccelerationContent_1",
        )?,
    ))
}

#[derive(Deserialize)]
struct BorrowedCurrentField {
    #[serde(flatten)]
    field: CurrentField,
    reject_other_native_encodings: ContractFlag,
    #[serde(default)]
    runtime_value_authority: String,
}

#[derive(Deserialize)]
struct RetainedQuantity {
    owner: String,
    section: String,
    name: String,
    private_wram_offset: usize,
    quantity_width: u8,
    ownership: String,
    revalidate_before_consumption: ContractFlag,
    runtime_value_authority: String,
    not_an_admission_selector: ContractFlag,
}

#[derive(Deserialize)]
struct BorrowedTerminalField {
    owner: String,
    section: String,
    name: String,
    quantity_width: u8,
    ownership: String,
    #[serde(default)]
    qualified_borrowed_terminal_encoding: Option<u32>,
    current_complete_predicate_required: ContractFlag,
    #[serde(default)]
    terminal_relation: String,
    #[serde(default)]
    observed_encoding_is_not_runtime_terminal_input: bool,
}

#[derive(Deserialize)]
struct BorrowedOwner {
    name: String,
    origin_type: String,
    terminal_type: String,
    requires_complete_current_predicate: ContractFlag,
    requires_current_imported_content: ContractFlag,
    actual_selected_results_not_source_next_state: ContractFlag,
    depends_on: String,
    terminal_collection_relation: String,
    #[serde(default)]
    requires_actual_completed_s1_nonempty_valid_membership: Option<ContractFlag>,
    #[serde(default)]
    world_membership_authority: String,
    #[serde(default)]
    active_geometry_table_is_not_a_world_roster: Option<ContractFlag>,
}

#[derive(Deserialize)]
struct GeometryField {
    owner: String,
    member_relation: String,
    name: String,
    private_wram_offset: usize,
    quantity_width: u8,
    required_current_encoding: u32,
    runtime_terminal_relation: String,
    source_backend_writer_count: u32,
    terminal_native_next_state_exported: ContractFlag,
}

#[derive(Deserialize)]
struct ActorContentOperand {
    owner: String,
    name: String,
    rom_bank: u8,
    rom_offset: u16,
    byte_width: u8,
    declared_encoding: u32,
    semantic_decoder: String,
    runtime_use: String,
}

#[derive(Deserialize)]
struct GraphicsDecoder {
    member: String,
    current_field: String,
    terminal_field: String,
    private_wram_offset: usize,
    quantity_width: u8,
    required_current_encoding: u32,
    required_terminal_encoding: u32,
    current_semantic_disposition: String,
    terminal_semantic_disposition: String,
    instruction_processing: String,
    spritemap_kind: String,
    paralysis: String,
    reject_other_native_encodings: ContractFlag,
    preserve_other_meaningful_properties: ContractFlag,
}

#[derive(Deserialize)]
struct ViewWitness {
    member: String,
    property_source_field: String,
    actual_current_view_disposition: String,
    drawing_admission: ContractFlag,
    geometry_authority: String,
    requires_actual_current_geometry_and_view: ContractFlag,
}

#[derive(Deserialize)]
struct ContactDecoder {
    field: String,
    required_native_encoding: u32,
    quantity_width: u8,
    semantic_value: String,
    reject_other_native_encodings: ContractFlag,
}

#[derive(Deserialize)]
struct MembershipFact {
    source_backend_writer_count: u32,
    actual_active_order: String,
    actual_interactive_order: String,
    runtime_roster_authority: String,
    source_extent_has_no_prior_cpu_writer: ContractFlag,
}

#[derive(Deserialize)]
struct SelectedDependency {
    owner: String,
    field: String,
    authority: String,
    native_source_value_is_only_verifier_guard: ContractFlag,
    native_source_value_is_never_model_initializer: ContractFlag,
}

#[derive(Deserialize)]
struct BorrowedProjection {
    format: String,
    source_coordinates_role: String,
    runtime_source_executor: ContractFlag,
    selected_native_terminals_exported: ContractFlag,
    current_field: Vec<BorrowedCurrentField>,
    current_retained_quantity: Vec<RetainedQuantity>,
    borrowed_terminal_field: Vec<BorrowedTerminalField>,
    owner: Vec<BorrowedOwner>,
    actor_geometry: Vec<GeometryField>,
    imported_content: Vec<ActorContentOperand>,
    actor_graphics_decoder: Vec<GraphicsDecoder>,
    actor_view_witness: Vec<ViewWitness>,
    contact_decoder: Vec<ContactDecoder>,
    membership_source_fact: Vec<MembershipFact>,
    selected_dependency: Vec<SelectedDependency>,
}

impl BorrowedProjection {
    fn parse(text: &str) -> Result<Self, PlayerReceiptFault> {
        let source: Self = toml::from_str(text).map_err(|_| PlayerReceiptFault::Format)?;
        if source.format != "bounded-borrowed-player-current-terminal-bindings-v2"
            || source.source_coordinates_role != "ProvenanceOnly"
            || source.runtime_source_executor.0
            || source.selected_native_terminals_exported.0
        {
            return Err(PlayerReceiptFault::Unsupported);
        }
        Ok(source)
    }

    fn owner(
        &self,
        name: &str,
        dependency: &str,
        currents: usize,
        terminals: usize,
        relation: &str,
    ) -> Result<(), PlayerReceiptFault> {
        let mut owners = self.owner.iter().filter(|owner| owner.name == name);
        let owner = owners.next().ok_or(PlayerReceiptFault::Unsupported)?;
        if owners.next().is_some()
            || owner.origin_type != format!("{name}Origin")
            || owner.terminal_type != format!("{name}Terminal")
            || !owner.requires_complete_current_predicate.0
            || !owner.requires_current_imported_content.0
            || !owner.actual_selected_results_not_source_next_state.0
            || owner.depends_on != dependency
            || owner.terminal_collection_relation != relation
            || self
                .current_field
                .iter()
                .filter(|field| field.field.owner == name)
                .count()
                != currents
            || self
                .borrowed_terminal_field
                .iter()
                .filter(|field| field.owner == name)
                .count()
                != terminals
        {
            return Err(PlayerReceiptFault::Unsupported);
        }
        Ok(())
    }

    fn field(&self, owner: &str, name: &str) -> Result<&BorrowedCurrentField, PlayerReceiptFault> {
        let mut fields = self
            .current_field
            .iter()
            .filter(|field| field.field.owner == owner && field.field.name == name);
        let field = fields.next().ok_or(PlayerReceiptFault::Unsupported)?;
        if fields.next().is_some()
            || field.field.section != format!("{owner}Origin")
            || field.field.quantity_width != 2
            || !field.field.revalidate_before_consumption.0
            || field.field.ownership != "ActualCurrentOwnerAtThisOpportunity"
        {
            return Err(PlayerReceiptFault::Unsupported);
        }
        Ok(field)
    }

    fn current(&self, bytes: &[u8], owner: &str, name: &str) -> Result<u16, PlayerReceiptFault> {
        if bytes.len() != 131_072 {
            return Err(PlayerReceiptFault::Length);
        }
        let field = self.field(owner, name)?;
        if !field.reject_other_native_encodings.0 || !field.runtime_value_authority.is_empty() {
            return Err(PlayerReceiptFault::Unsupported);
        }
        let value = scalar(bytes, field.field.private_wram_offset, 2)?;
        if value != field.field.required_current_encoding {
            return Err(PlayerReceiptFault::Dependency);
        }
        u16::try_from(value).map_err(|_| PlayerReceiptFault::Unsupported)
    }

    /// Decode a validation-only predicate, never a model owner initializer.
    fn required(
        &self,
        bytes: &[u8],
        name: &str,
        authority: &str,
    ) -> Result<u16, PlayerReceiptFault> {
        if bytes.len() != 131_072 {
            return Err(PlayerReceiptFault::Length);
        }
        let field = self.field("ActorUpdate", name)?;
        if field.runtime_value_authority != authority || !field.reject_other_native_encodings.0 {
            return Err(PlayerReceiptFault::Unsupported);
        }
        let value = scalar(bytes, field.field.private_wram_offset, 2)?;
        if value != field.field.required_current_encoding {
            return Err(PlayerReceiptFault::Dependency);
        }
        u16::try_from(value).map_err(|_| PlayerReceiptFault::Unsupported)
    }

    fn bound(
        &self,
        owner: &str,
        name: &str,
        actual: u16,
        authority: &str,
    ) -> Result<(), PlayerReceiptFault> {
        let field = self.field(owner, name)?;
        if field.runtime_value_authority != authority
            || field.field.required_current_encoding != u32::from(actual)
        {
            return Err(PlayerReceiptFault::Dependency);
        }
        Ok(())
    }

    fn meaning(&self, owner: &str, name: &str, expected: &str) -> Result<(), PlayerReceiptFault> {
        if self.field(owner, name)?.field.semantic_decoder != expected {
            return Err(PlayerReceiptFault::Unsupported);
        }
        Ok(())
    }

    fn retained(
        &self,
        bytes: &[u8],
        owner: &str,
        name: &str,
    ) -> Result<AdministrativeCounter, PlayerReceiptFault> {
        if bytes.len() != 131_072 {
            return Err(PlayerReceiptFault::Length);
        }
        let mut quantities = self
            .current_retained_quantity
            .iter()
            .filter(|field| field.owner == owner && field.name == name);
        let field = quantities.next().ok_or(PlayerReceiptFault::Unsupported)?;
        if quantities.next().is_some()
            || field.section != format!("{owner}Origin")
            || field.quantity_width != 2
            || !field.revalidate_before_consumption.0
            || field.ownership != "ActualCurrentOwnerAtThisOpportunity"
            || field.runtime_value_authority != "ActualRetainedAdministrativeCounter"
            || !field.not_an_admission_selector.0
        {
            return Err(PlayerReceiptFault::Unsupported);
        }
        Ok(AdministrativeCounter(
            u16::try_from(scalar(bytes, field.private_wram_offset, 2)?)
                .map_err(|_| PlayerReceiptFault::Unsupported)?,
        ))
    }

    fn terminal_field(
        &self,
        owner: &str,
        name: &str,
    ) -> Result<&BorrowedTerminalField, PlayerReceiptFault> {
        let mut fields = self
            .borrowed_terminal_field
            .iter()
            .filter(|field| field.owner == owner && field.name == name);
        let field = fields.next().ok_or(PlayerReceiptFault::Unsupported)?;
        if fields.next().is_some()
            || field.section != format!("{owner}Terminal")
            || field.quantity_width != 2
            || !field.current_complete_predicate_required.0
            || field.ownership != "ActualBorrowedUnselectedOwnerCompletion"
        {
            return Err(PlayerReceiptFault::Unsupported);
        }
        Ok(field)
    }

    fn terminal(&self, owner: &str, name: &str) -> Result<u16, PlayerReceiptFault> {
        let field = self.terminal_field(owner, name)?;
        let required_relation = match (owner, name) {
            ("ActorUpdate", "ActorInstructionTimerFirst" | "ActorInstructionTimerSecond") => {
                "ActualCurrentNonExpiringInstructionTimerDecrementOne"
            }
            ("ActorUpdate", "ActorGraphicsFlagsFirst" | "ActorGraphicsFlagsSecond") => {
                "GraphicsUpdatedClearWithOtherMeaningfulPropertiesPreserved"
            }
            _ => "",
        };
        if field.terminal_relation != required_relation
            || field.observed_encoding_is_not_runtime_terminal_input
        {
            return Err(PlayerReceiptFault::Unsupported);
        }
        u16::try_from(
            field
                .qualified_borrowed_terminal_encoding
                .ok_or(PlayerReceiptFault::Unsupported)?,
        )
        .map_err(|_| PlayerReceiptFault::Unsupported)
    }

    fn counter_effect(&self, owner: &str, name: &str) -> Result<CounterEffect, PlayerReceiptFault> {
        let field = self.terminal_field(owner, name)?;
        if field.terminal_relation != "ActualRetainedCounterWrappingIncrementOne"
            || !field.observed_encoding_is_not_runtime_terminal_input
            || field.qualified_borrowed_terminal_encoding.is_some()
        {
            return Err(PlayerReceiptFault::Unsupported);
        }
        Ok(CounterEffect::WrappingIncrementOne)
    }

    fn clocks(&self, bytes: &[u8], owner: &str) -> Result<CollectionClocks, PlayerReceiptFault> {
        for name in ["SimulationClock", "EnemyClock"] {
            self.meaning(owner, name, "ActualRunningClock")?;
            let _ = self.current(bytes, owner, name)?;
        }
        Ok(CollectionClocks {
            simulation: ClockDisposition::Running,
            enemy: ClockDisposition::Running,
        })
    }

    fn activity(&self, bytes: &[u8], owner: &str) -> Result<SamusActivity, PlayerReceiptFault> {
        self.meaning(owner, "PROJECTILE_COUNTER", "NoProjectiles")?;
        self.meaning(owner, "BombCounter", "NoBombs")?;
        let projectiles = self.current(bytes, owner, "PROJECTILE_COUNTER")?;
        let bombs = self.current(bytes, owner, "BombCounter")?;
        if projectiles != 0 || bombs != 0 {
            return Err(PlayerReceiptFault::Unsupported);
        }
        Ok(SamusActivity { projectiles, bombs })
    }
}

/// Decode every actual ordered projectile instruction owner at its consumption edge.
/// # Errors
/// Rejects changed clocks/owners or incomplete source dependency contracts.
pub fn current_projectiles(
    bytes: &[u8],
    projection: &str,
) -> Result<ProjectilesOrigin, PlayerReceiptFault> {
    let source = BorrowedProjection::parse(projection)?;
    source.owner(
        "Projectiles",
        "ActualCompletedInsideAndHudWeapons",
        12,
        0,
        "ActualCurrentEmptyInstructionOwnersUnchanged",
    )?;
    let mut slots = Vec::with_capacity(10);
    for index in 0..10 {
        let name = format!("ProjectileInstructionSlot{index}");
        source.meaning(
            "Projectiles",
            &name,
            "EmptyInstructionOwnerForExactQualifiedEncoding",
        )?;
        let _ = source.current(bytes, "Projectiles", &name)?;
        slots.push(ProjectileInstruction {
            identity: InstructionSlotIdentity(index),
            instruction: InstructionDisposition::Empty,
        });
    }
    Ok(ProjectilesOrigin {
        slots: slots
            .try_into()
            .map_err(|_| PlayerReceiptFault::Unsupported)?,
        clocks: source.clocks(bytes, "Projectiles")?,
    })
}

/// Retain the qualified unchanged projectile collection without running an instruction body.
/// # Errors
/// Rejects incomplete or changed actual current collection predicates.
pub fn projectiles_offer(
    bytes: &[u8],
    projection: &str,
) -> Result<DeferredProjectilesReceipt, PlayerReceiptFault> {
    let dependency = current_projectiles(bytes, projection)?;
    let terminal = ProjectilesTerminal {
        slots: dependency.slots,
    };
    Ok(DeferredProjectilesReceipt {
        dependency,
        terminal,
    })
}

/// Decode enabled interaction and actual no-projectile/no-bomb owners.
/// # Errors
/// Rejects incomplete or changed actual current interaction predicates.
pub fn current_interaction(
    bytes: &[u8],
    projection: &str,
) -> Result<SamusInteractionOrigin, PlayerReceiptFault> {
    let source = BorrowedProjection::parse(projection)?;
    source.owner(
        "SamusInteraction",
        "ActualCompletedInsideHudWeaponsMotionResetAndProjectiles",
        4,
        0,
        "QualifiedActualBorrowedOwnerEffects",
    )?;
    for (name, meaning) in [
        ("SpriteInteractionMode", "EnabledSpriteInteraction"),
        (
            "SamusProjectileInteractionMode",
            "EnabledSamusProjectileInteraction",
        ),
    ] {
        source.meaning("SamusInteraction", name, meaning)?;
        let _ = source.current(bytes, "SamusInteraction", name)?;
    }
    Ok(SamusInteractionOrigin {
        sprite_interaction: InteractionDisposition::Enabled,
        projectile_interaction: InteractionDisposition::Enabled,
        activity: source.activity(bytes, "SamusInteraction")?,
    })
}

/// Retain actual unchanged no-activity owners; source processing cursors are omitted.
/// # Errors
/// Rejects incomplete or changed current owner predicates.
pub fn interaction_offer(
    bytes: &[u8],
    projection: &str,
) -> Result<DeferredSamusInteractionReceipt, PlayerReceiptFault> {
    let dependency = current_interaction(bytes, projection)?;
    let terminal = SamusInteractionTerminal {
        sprite_interaction: dependency.sprite_interaction,
        projectile_interaction: dependency.projectile_interaction,
        activity: dependency.activity,
    };
    Ok(DeferredSamusInteractionReceipt {
        dependency,
        terminal,
    })
}

/// Decode every actual ordered sprite instruction owner at its consumption edge.
/// # Errors
/// Rejects changed clocks/owners or incomplete source dependency contracts.
pub fn current_sprites(
    bytes: &[u8],
    projection: &str,
) -> Result<SpriteObjectsOrigin, PlayerReceiptFault> {
    let source = BorrowedProjection::parse(projection)?;
    source.owner(
        "SpriteObjects",
        "ActualCompletedActorUpdateAndActualRunningClocks",
        34,
        0,
        "ActualCurrentEmptyInstructionOwnersUnchanged",
    )?;
    let mut slots = Vec::with_capacity(32);
    for index in 0..32 {
        let name = format!("SpriteInstructionSlot{index}");
        source.meaning(
            "SpriteObjects",
            &name,
            "EmptyInstructionOwnerForExactQualifiedEncoding",
        )?;
        let _ = source.current(bytes, "SpriteObjects", &name)?;
        slots.push(SpriteInstruction {
            identity: InstructionSlotIdentity(index),
            instruction: InstructionDisposition::Empty,
        });
    }
    Ok(SpriteObjectsOrigin {
        slots: slots
            .try_into()
            .map_err(|_| PlayerReceiptFault::Unsupported)?,
        clocks: source.clocks(bytes, "SpriteObjects")?,
    })
}

/// Retain the qualified unchanged sprite collection without running an instruction body.
/// # Errors
/// Rejects incomplete or changed actual current collection predicates.
pub fn sprites_offer(
    bytes: &[u8],
    projection: &str,
) -> Result<DeferredSpriteObjectsReceipt, PlayerReceiptFault> {
    let dependency = current_sprites(bytes, projection)?;
    let terminal = SpriteObjectsTerminal {
        slots: dependency.slots,
    };
    Ok(DeferredSpriteObjectsReceipt {
        dependency,
        terminal,
    })
}

/// Read an actual retained contact counter without selecting a gameplay profile by its value.
/// # Errors
/// Rejects incomplete origins or invalid retained-owner contracts.
pub fn current_contact(
    bytes: &[u8],
    projection: &str,
) -> Result<ContactResetOrigin, PlayerReceiptFault> {
    let source = BorrowedProjection::parse(projection)?;
    source.owner(
        "ContactReset",
        "ActualCompletedActorUpdateAndSpriteObjects",
        0,
        9,
        "QualifiedActualBorrowedOwnerEffects",
    )?;
    Ok(ContactResetOrigin {
        execution_counter: source.retained(bytes, "ContactReset", "ContactExecutionCounter")?,
    })
}

/// Import explicit cleared contact meanings and a mechanical retained-counter effect.
/// Recorded counter witnesses are never runtime terminal inputs.
/// # Errors
/// Rejects missing semantic decoders or unsupported external terminal effects.
pub fn contact_offer(
    bytes: &[u8],
    projection: &str,
) -> Result<DeferredContactResetReceipt, PlayerReceiptFault> {
    let source = BorrowedProjection::parse(projection)?;
    let dependency = current_contact(bytes, projection)?;
    if source.contact_decoder.len() != 8 {
        return Err(PlayerReceiptFault::Unsupported);
    }
    for direction in ["Left", "Right", "Up", "Down"] {
        for (suffix, meaning) in [
            ("Actor", "NoSolidContactActor"),
            ("Distance", "NoSolidContactDistance"),
        ] {
            let name = format!("SolidContact{direction}{suffix}");
            let mut decoders = source
                .contact_decoder
                .iter()
                .filter(|decoder| decoder.field == name);
            let decoder = decoders.next().ok_or(PlayerReceiptFault::Unsupported)?;
            if decoders.next().is_some()
                || decoder.quantity_width != 2
                || decoder.semantic_value != meaning
                || !decoder.reject_other_native_encodings.0
                || u32::from(source.terminal("ContactReset", &name)?)
                    != decoder.required_native_encoding
            {
                return Err(PlayerReceiptFault::Unsupported);
            }
        }
    }
    Ok(DeferredContactResetReceipt {
        dependency,
        terminal: ContactResetTerminal {
            execution_counter: source.counter_effect("ContactReset", "ContactExecutionCounter")?,
            contacts: [SolidContact {
                actor: None,
                distance: None,
            }; 4],
        },
    })
}

impl BorrowedProjection {
    fn actor_content(&self, rom: &Rom) -> Result<(), PlayerReceiptFault> {
        if self.imported_content.len() != 21 {
            return Err(PlayerReceiptFault::Content);
        }
        for (index, field) in self.imported_content.iter().enumerate() {
            if field.owner != "ActorUpdate"
                || self
                    .imported_content
                    .iter()
                    .take(index)
                    .any(|prior| prior.name == field.name)
                || field.semantic_decoder != "KnownCeresDoorHeaderAndBoundedAiContactContent"
                || field.runtime_use != "ImportedTypedContentOnlyNoNativeRoutineDispatch"
            {
                return Err(PlayerReceiptFault::Unsupported);
            }
            let bytes = rom
                .lorom()
                .read(
                    RomAddress::new(field.rom_bank, field.rom_offset),
                    usize::from(field.byte_width),
                )
                .map_err(|_| PlayerReceiptFault::Content)?;
            if scalar(bytes, 0, field.byte_width)? != field.declared_encoding {
                return Err(PlayerReceiptFault::Content);
            }
        }
        Ok(())
    }

    fn selected(&self, name: &str, authority: &str, actual: u16) -> Result<(), PlayerReceiptFault> {
        let mut dependencies = self
            .selected_dependency
            .iter()
            .filter(|field| field.owner == "ActorUpdate" && field.field == name);
        let dependency = dependencies.next().ok_or(PlayerReceiptFault::Unsupported)?;
        if dependencies.next().is_some()
            || dependency.authority != authority
            || !dependency.native_source_value_is_only_verifier_guard.0
            || !dependency.native_source_value_is_never_model_initializer.0
        {
            return Err(PlayerReceiptFault::Unsupported);
        }
        self.bound(
            "ActorUpdate",
            name,
            actual,
            "ActualSelectedMovementStartingStateAndCheckedBodyContent",
        )
    }

    fn membership(
        &self,
        active: &[game::production::frame::borrowed_preparation::ActorSlotIdentity],
        interactive: &[game::production::frame::borrowed_preparation::ActorSlotIdentity],
    ) -> Result<(), PlayerReceiptFault> {
        let owner = self
            .owner
            .iter()
            .find(|owner| owner.name == "ActorUpdate")
            .ok_or(PlayerReceiptFault::Unsupported)?;
        if !owner
            .requires_actual_completed_s1_nonempty_valid_membership
            .as_ref()
            .is_some_and(|flag| flag.0)
            || owner.world_membership_authority
                != "ActualCompletedS1InteractiveMembershipAndCurrentSolidOrFrozenDispositions"
            || !owner
                .active_geometry_table_is_not_a_world_roster
                .as_ref()
                .is_some_and(|flag| flag.0)
        {
            return Err(PlayerReceiptFault::Unsupported);
        }
        if self.membership_source_fact.len() != 1
            || active.len() != 2
            || active.iter().any(|slot| slot.0 >= 32)
            || active.first() == active.last()
            || interactive != active.get(1..).ok_or(PlayerReceiptFault::Dependency)?
        {
            return Err(PlayerReceiptFault::Dependency);
        }
        let fact = self
            .membership_source_fact
            .first()
            .ok_or(PlayerReceiptFault::Unsupported)?;
        if fact.source_backend_writer_count != 0
            || fact.actual_active_order != "CompletedS1FirstThenSecond"
            || fact.actual_interactive_order != "CompletedS1SecondOnly"
            || fact.runtime_roster_authority != "ActualCompletedS1MembershipNotThisSourceTable"
            || !fact.source_extent_has_no_prior_cpu_writer.0
        {
            return Err(PlayerReceiptFault::Unsupported);
        }
        for (index, name) in ["ActiveMembershipFirst", "ActiveMembershipSecond"]
            .into_iter()
            .enumerate()
        {
            self.meaning(
                "ActorUpdate",
                name,
                "ActualCompletedS1MembershipIdentityOrTerminator",
            )?;
            let slot = active.get(index).ok_or(PlayerReceiptFault::Dependency)?;
            self.bound("ActorUpdate", name, u16::from(slot.0) * 64, "")?;
        }
        self.meaning(
            "ActorUpdate",
            "ActiveMembershipTerminator",
            "ActualCompletedS1MembershipIdentityOrTerminator",
        )?;
        self.bound("ActorUpdate", "ActiveMembershipTerminator", u16::MAX, "")
    }

    fn geometry(&self, bytes: &[u8], member: &str) -> Result<ActorGeometry, PlayerReceiptFault> {
        if bytes.len() != 131_072 {
            return Err(PlayerReceiptFault::Length);
        }
        let relation = format!("ActualCompletedS1{member}Member");
        if self.actor_geometry.len() != 12
            || self
                .actor_geometry
                .iter()
                .filter(|field| field.member_relation == relation)
                .count()
                != 6
        {
            return Err(PlayerReceiptFault::Unsupported);
        }
        let read = |name| -> Result<u16, PlayerReceiptFault> {
            let mut fields = self
                .actor_geometry
                .iter()
                .filter(|field| field.member_relation == relation && field.name == name);
            let field = fields.next().ok_or(PlayerReceiptFault::Unsupported)?;
            if fields.next().is_some()
                || field.owner != "ActorUpdate"
                || field.quantity_width != 2
                || field.runtime_terminal_relation != "ActualCurrentGeometryUnchanged"
                || field.source_backend_writer_count != 0
                || field.terminal_native_next_state_exported.0
            {
                return Err(PlayerReceiptFault::Unsupported);
            }
            let value = scalar(bytes, field.private_wram_offset, 2)?;
            if value != field.required_current_encoding {
                return Err(PlayerReceiptFault::Dependency);
            }
            u16::try_from(value).map_err(|_| PlayerReceiptFault::Unsupported)
        };
        Ok(ActorGeometry {
            position: types::Pair {
                x: types::fixed(core::num::Wrapping(read("XCenter")?), read("XFraction")?),
                y: types::fixed(core::num::Wrapping(read("YCenter")?), read("YFraction")?),
            },
            radius: types::Pair {
                x: core::num::Wrapping(read("XRadius")?),
                y: core::num::Wrapping(read("YRadius")?),
            },
        })
    }

    fn graphics(
        &self,
        bytes: &[u8],
        member: &str,
    ) -> Result<(GraphicsUpdated, GraphicsUpdated), PlayerReceiptFault> {
        if self.actor_graphics_decoder.len() != 2 {
            return Err(PlayerReceiptFault::Unsupported);
        }
        let mut decoders = self
            .actor_graphics_decoder
            .iter()
            .filter(|decoder| decoder.member == member);
        let decoder = decoders.next().ok_or(PlayerReceiptFault::Unsupported)?;
        let current_name = if member == "First" {
            "ActorGraphicsFlagsFirst"
        } else {
            "ActorExtraPropertiesSecond"
        };
        let terminal_name = format!("ActorGraphicsFlags{member}");
        let field = self.field("ActorUpdate", current_name)?;
        if decoders.next().is_some()
            || decoder.current_field != current_name
            || decoder.terminal_field != terminal_name
            || decoder.quantity_width != 2
            || decoder.private_wram_offset != field.field.private_wram_offset
            || decoder.current_semantic_disposition
                != "DefaultCeresDoorExtraPropertiesGraphicsUpdatedSet"
            || decoder.terminal_semantic_disposition
                != "DefaultCeresDoorExtraPropertiesGraphicsUpdatedClear"
            || decoder.instruction_processing != "Enabled"
            || decoder.spritemap_kind != "Basic"
            || decoder.paralysis != "None"
            || !decoder.reject_other_native_encodings.0
            || !decoder.preserve_other_meaningful_properties.0
            || u32::from(self.current(bytes, "ActorUpdate", current_name)?)
                != decoder.required_current_encoding
            || u32::from(self.terminal("ActorUpdate", &terminal_name)?)
                != decoder.required_terminal_encoding
        {
            return Err(PlayerReceiptFault::Unsupported);
        }
        Ok((GraphicsUpdated::Set, GraphicsUpdated::Clear))
    }

    fn view(&self, member: &str) -> Result<ActorViewDisposition, PlayerReceiptFault> {
        if self.actor_view_witness.len() != 2 {
            return Err(PlayerReceiptFault::Unsupported);
        }
        let mut witnesses = self
            .actor_view_witness
            .iter()
            .filter(|witness| witness.member == member);
        let witness = witnesses.next().ok_or(PlayerReceiptFault::Unsupported)?;
        let disposition = if member == "First" {
            "WithinPaddedViewButInvisible"
        } else {
            "BelowPaddedView"
        };
        if witnesses.next().is_some()
            || witness.property_source_field != format!("ActorProperties{member}")
            || witness.actual_current_view_disposition != disposition
            || witness.drawing_admission.0
            || witness.geometry_authority != "ActualCurrentActorAndCurrentCamera"
            || !witness.requires_actual_current_geometry_and_view.0
        {
            return Err(PlayerReceiptFault::Unsupported);
        }
        Ok(if member == "First" {
            ActorViewDisposition::WithinPaddedViewButInvisible
        } else {
            ActorViewDisposition::BelowPaddedView
        })
    }
}

/// Decode validation-only predicates without constructing future selected owners.
/// # Errors
/// Rejects invalid source bindings, dependency classifications or membership facts.
pub fn actor_required_dependencies(
    bytes: &[u8],
    projection: &str,
) -> Result<ActorRequiredDependencies, PlayerReceiptFault> {
    let source = BorrowedProjection::parse(projection)?;
    if source.selected_dependency.len() != 2 {
        return Err(PlayerReceiptFault::Unsupported);
    }
    let centre_x = source.required(
        bytes,
        "SAMUS_X_POS",
        "ActualSelectedMovementStartingStateAndCheckedBodyContent",
    )?;
    let radius_x = source.required(
        bytes,
        "SAMUS_X_RADIUS",
        "ActualSelectedMovementStartingStateAndCheckedBodyContent",
    )?;
    source.selected(
        "SAMUS_X_POS",
        "ActualMotionResetCompletionPreviousAndCurrentMovementState",
        centre_x,
    )?;
    source.selected(
        "SAMUS_X_RADIUS",
        "ActualStartingPoseAndCheckedMovementContentBodyRadius",
        radius_x,
    )?;
    let slot = |name| {
        let value = source.required(bytes, name, "")?;
        if value % 64 != 0 || value / 64 >= 32 {
            return Err(PlayerReceiptFault::Dependency);
        }
        let index = u8::try_from(value / 64).map_err(|_| PlayerReceiptFault::Dependency)?;
        Ok(game::production::frame::borrowed_preparation::ActorSlotIdentity(index))
    };
    let first = slot("ActiveMembershipFirst")?;
    let second = slot("ActiveMembershipSecond")?;
    let active = [first, second];
    if source.required(bytes, "ActiveMembershipTerminator", "")? != u16::MAX {
        return Err(PlayerReceiptFault::Dependency);
    }
    let interactive = vec![second];
    source.membership(&active, &interactive)?;
    Ok(ActorRequiredDependencies {
        samus: SamusContactGeometry {
            centre_x: std::num::Wrapping(centre_x),
            radius_x: std::num::Wrapping(radius_x),
        },
        camera_view: types::Pair {
            x: source.required(bytes, "LAYER1_X_POS", "")?,
            y: source.required(bytes, "LAYER1_Y_POS", "")?,
        },
        active_order: active,
        interactive_order: interactive,
        views: [source.view("First")?, source.view("Second")?],
    })
}

fn validate_actor_fields(
    bytes: &[u8],
    source: &BorrowedProjection,
) -> Result<(), PlayerReceiptFault> {
    source.owner("ActorUpdate", "ActualCompletedS1MembershipAndSamusInteractionAndActualSelectedInsideHudMotionResetSamusGeometry", 38, 6, "QualifiedActualBorrowedOwnerEffects")?;
    for field in source
        .current_field
        .iter()
        .filter(|field| field.field.owner == "ActorUpdate")
    {
        if matches!(
            field.field.name.as_str(),
            "ActiveMembershipFirst"
                | "ActiveMembershipSecond"
                | "ActiveMembershipTerminator"
                | "SAMUS_X_POS"
                | "SAMUS_X_RADIUS"
                | "LAYER1_X_POS"
                | "LAYER1_Y_POS"
        ) {
            continue;
        }
        let _ = source.current(bytes, "ActorUpdate", &field.field.name)?;
    }
    for (name, meaning) in [
        ("EnemyIndexToShake", "NoShakeRequest"),
        ("CERES_STATUS", "PreEscapeCeres"),
        (
            "ActorPropertiesFirst",
            "IntangibleInvisibleSolidMarkedNotDeletedActor",
        ),
        (
            "ActorPropertiesSecond",
            "TangibleVisibleSolidMarkedNotDeletedActor",
        ),
        (
            "ActorAiDispositionFirst",
            "CheckedCeresDoorOrdinaryAiDisposition",
        ),
        (
            "ActorAiDispositionSecond",
            "CheckedCeresDoorOrdinaryAiDisposition",
        ),
        ("ActorBankFirst", "CheckedCeresDoorContentIdentity"),
        ("ActorBankSecond", "CheckedCeresDoorContentIdentity"),
        ("ActorIdentityFirst", "CheckedCeresDoorContentIdentity"),
        ("ActorSpritemapSecond", "CheckedCeresDoorContentIdentity"),
        ("SpriteInteractionMode", "EnabledSpriteInteraction"),
        ("SamusContactDamageMode", "OrdinaryContact"),
        (
            "ActorExtraPropertiesSecond",
            "BasicSpritemapInstructionsEnabledNotParalysed",
        ),
        ("ActorInvincibilityTimerSecond", "NoInvincibility"),
        ("SamusInvincibilityTimer", "NoInvincibility"),
        (
            "ActorAdditionalContactGateSecond",
            "NoAdditionalContactGate",
        ),
    ] {
        source.meaning("ActorUpdate", name, meaning)?;
    }
    Ok(())
}

/// Decode current unselected actor owner seeds, excluding selected dependency predicates.
/// # Errors
/// Rejects changed current owner bindings or unsupported actor arms.
pub fn current_actors(
    bytes: &[u8],
    projection: &str,
) -> Result<ActorUnselectedOrigin, PlayerReceiptFault> {
    use types::Parts;
    let source = BorrowedProjection::parse(projection)?;
    validate_actor_fields(bytes, &source)?;
    let mut actors = Vec::with_capacity(2);
    for (index, member) in ["First", "Second"].into_iter().enumerate() {
        let timer_name = format!("ActorInstructionTimer{member}");
        let flash_name = format!("ActorFlashTimer{member}");
        source.meaning(
            "ActorUpdate",
            &timer_name,
            "QualifiedNonExpiringInstructionTimer",
        )?;
        source.meaning("ActorUpdate", &flash_name, "NoFlash")?;
        let geometry = source.geometry(bytes, member)?;
        for (quantity, value) in [
            ("ActorXCenter", geometry.position.x.pixel().0),
            ("ActorYCenter", geometry.position.y.pixel().0),
            ("ActorXRadius", geometry.radius.x.0),
        ] {
            if source.current(bytes, "ActorUpdate", &format!("{quantity}{member}"))? != value {
                return Err(PlayerReceiptFault::Dependency);
            }
        }
        actors.push(ActorSeed {
            content: ActorContentDisposition::OrdinaryCeresDoor,
            touch: if index == 0 {
                ActorTouch::Intangible
            } else {
                ActorTouch::Tangible
            },
            solidity: ActorSolidity::Solid,
            geometry,
            frame_counter: source.retained(
                bytes,
                "ActorUpdate",
                &format!("ActorFrameCounter{member}"),
            )?,
            instruction_timer: source.current(bytes, "ActorUpdate", &timer_name)?,
            graphics_updated: source.graphics(bytes, member)?.0,
            flash_timer: source.current(bytes, "ActorUpdate", &flash_name)?,
        });
    }
    Ok(ActorUnselectedOrigin {
        actors: actors
            .try_into()
            .map_err(|_| PlayerReceiptFault::Unsupported)?,
        clocks: source.clocks(bytes, "ActorUpdate")?,
        scene: ActorScene::PreEscape,
        shake_requested: false,
        contact: OrdinaryActorContact {
            actor_invincibility: source.current(
                bytes,
                "ActorUpdate",
                "ActorInvincibilityTimerSecond",
            )?,
            samus_invincibility: source.current(bytes, "ActorUpdate", "SamusInvincibilityTimer")?,
            additional_gate: AdditionalContactGate::None,
            damage_mode: ContactDamageDisposition::Ordinary,
            sprite_interaction: InteractionDisposition::Enabled,
            activity: source.activity(bytes, "ActorUpdate")?,
        },
    })
}

/// Import actual externally qualified actor effects, binding current geometry and checked content.
/// Native counter witnesses and selected next-state values are never consumed.
/// # Errors
/// Rejects changed source/content predicates or unsupported actor terminal relations.
pub fn actors_offer(
    bytes: &[u8],
    projection: &str,
    rom: &Rom,
) -> Result<DeferredActorUpdateReceipt, PlayerReceiptFault> {
    let source = BorrowedProjection::parse(projection)?;
    source.actor_content(rom)?;
    let dependency = current_actors(bytes, projection)?;
    let required = actor_required_dependencies(bytes, projection)?;
    let mut effects = Vec::with_capacity(2);
    for (index, member) in ["First", "Second"].into_iter().enumerate() {
        let actor = dependency
            .actors
            .get(index)
            .ok_or(PlayerReceiptFault::Dependency)?;
        effects.push(ActorEffects {
            geometry: actor.geometry,
            frame_counter: source
                .counter_effect("ActorUpdate", &format!("ActorFrameCounter{member}"))?,
            instruction_timer: source
                .terminal("ActorUpdate", &format!("ActorInstructionTimer{member}"))?,
            graphics_updated: source.graphics(bytes, member)?.1,
        });
    }
    Ok(DeferredActorUpdateReceipt {
        dependency,
        required,
        terminal: ActorUpdateTerminal {
            actors: effects
                .try_into()
                .map_err(|_| PlayerReceiptFault::Unsupported)?,
        },
    })
}

/// Import remaining-five current owners and distinct offers without future selected state.
/// # Errors
/// Rejects invalid source owner bindings, content or validation-only dependency predicates.
pub fn player_continuation(
    bytes: &[u8],
    projection: &str,
    rom: &Rom,
) -> Result<
    (
        game::production::frame::borrowed_player_preparation::PlayerContinuationOrigins,
        game::production::frame::borrowed_player_preparation::PlayerContinuationOffers,
    ),
    PlayerReceiptFault,
> {
    use game::production::frame::borrowed_player_preparation::{
        PlayerContinuationOffers, PlayerContinuationOrigins,
    };
    Ok((
        PlayerContinuationOrigins {
            projectiles: current_projectiles(bytes, projection)?,
            interaction: current_interaction(bytes, projection)?,
            actors: current_actors(bytes, projection)?,
            sprites: current_sprites(bytes, projection)?,
            contact: current_contact(bytes, projection)?,
        },
        PlayerContinuationOffers {
            projectiles: projectiles_offer(bytes, projection)?,
            interaction: interaction_offer(bytes, projection)?,
            actors: actors_offer(bytes, projection, rom)?,
            sprites: sprites_offer(bytes, projection)?,
            contact: contact_offer(bytes, projection)?,
        },
    ))
}
