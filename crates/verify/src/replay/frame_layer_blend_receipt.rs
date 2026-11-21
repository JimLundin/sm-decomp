//! One-way import of immutable default layer-blend operands.
use data::rom::{Rom, RomAddress};
use game::production::frame::layer_blend::LayerBlendDefaults;
use game::state::video::{ColourMathLayers, ColourMathSelect, Layers};
use serde::Deserialize;
use std::marker::PhantomData;

/// Invalid source projection, current content or completed Haze dependency.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LayerBlendFault {
    /// Malformed external projection.
    Format,
    /// Missing or changed actual default content.
    Content,
    /// Unsupported dependency, terminal setting or consumption contract.
    Unsupported,
}
#[derive(Deserialize)]
#[serde(transparent)]
struct ContractFlag(bool);
#[derive(Deserialize)]
struct DefaultContent {
    main_screen: u8,
    sub_screen: u8,
    colour_math_select: u8,
    colour_math_layers: u8,
}
fn layers(bits: u8) -> Result<Layers, LayerBlendFault> {
    if bits & 0xe0 != 0 {
        return Err(LayerBlendFault::Unsupported);
    }
    Ok(Layers {
        bits: u32::from(bits),
        members: PhantomData,
    })
}
impl DefaultContent {
    fn decode(&self) -> Result<LayerBlendDefaults, LayerBlendFault> {
        Ok(LayerBlendDefaults {
            main_screen: layers(self.main_screen)?,
            sub_screen: layers(self.sub_screen)?,
            colour_math_select: ColourMathSelect(self.colour_math_select),
            colour_math_layers: ColourMathLayers(self.colour_math_layers),
        })
    }
    const fn fields(&self) -> [(&'static str, u8); 4] {
        [
            ("main_screen", self.main_screen),
            ("sub_screen", self.sub_screen),
            ("colour_math_select", self.colour_math_select),
            ("colour_math_layers", self.colour_math_layers),
        ]
    }
}
#[derive(Deserialize)]
struct Operand {
    semantic_field: String,
    private_rom_file_offset: usize,
    quantity_width: u8,
    actual_content_encoding: u8,
    origin_kind: String,
}
#[derive(Deserialize)]
struct SelectedHaze {
    actual_haze_selected_blend_encoding: u16,
}
#[derive(Deserialize)]
struct Projection {
    format: String,
    required_haze_disposition: String,
    actual_current_dependencies_required: ContractFlag,
    expected_callbacks_used_as_inputs: ContractFlag,
    read_only_at_actual_operation_completion: ContractFlag,
    consume_once: ContractFlag,
    preserve_unwritten_window_edges_and_logic: ContractFlag,
    preserve_actual_haze_fixed_colour_owner: ContractFlag,
    previous_output_mirrors_are_default_content_authority: ContractFlag,
    dependency: SelectedHaze,
    actual_default_content_dependency: DefaultContent,
    immutable_default_input: Vec<Operand>,
}
impl Projection {
    fn validate(&self) -> Result<(), LayerBlendFault> {
        if self.format != "borrowed-layer-blend-terminal-receipt-v4"
            || self.required_haze_disposition != "NoBackgroundThree"
            || !self.actual_current_dependencies_required.0
            || self.expected_callbacks_used_as_inputs.0
            || !self.read_only_at_actual_operation_completion.0
            || !self.consume_once.0
            || !self.preserve_unwritten_window_edges_and_logic.0
            || !self.preserve_actual_haze_fixed_colour_owner.0
            || self.previous_output_mirrors_are_default_content_authority.0
            || self.dependency.actual_haze_selected_blend_encoding != 44
        {
            return Err(LayerBlendFault::Unsupported);
        }
        Ok(())
    }
    fn content(&self, rom: &[u8]) -> Result<LayerBlendDefaults, LayerBlendFault> {
        if self.immutable_default_input.len() != 4 {
            return Err(LayerBlendFault::Content);
        }
        for (input, (field, expected)) in self
            .immutable_default_input
            .iter()
            .zip(self.actual_default_content_dependency.fields())
        {
            if input.semantic_field != field
                || input.quantity_width != 1
                || input.origin_kind != "CheckedRomImmediateProducerOperand"
                || input.actual_content_encoding != expected
                || rom.get(input.private_rom_file_offset).copied() != Some(expected)
            {
                return Err(LayerBlendFault::Content);
            }
        }
        self.actual_default_content_dependency.decode()
    }
}

/// Import actual immutable operands from the player's normalized ROM at the content edge.
/// Admission separately pins the operand locations and qualified source artifact.
/// Source record coordinates never select runtime operations; output mirrors are not inputs.
/// # Errors
/// Rejects malformed projections, changed content and unsupported qualified contracts.
pub fn defaults(
    rom: &Rom,
    external_projection: &str,
) -> Result<LayerBlendDefaults, LayerBlendFault> {
    let projection: Projection =
        toml::from_str(external_projection).map_err(|_| LayerBlendFault::Format)?;
    projection.validate()?;
    projection.content(
        rom.lorom()
            .from(RomAddress::new(0x80, 0x8000))
            .map_err(|_| LayerBlendFault::Content)?,
    )
}

/// Project the nine recorded setting writes for comparison after the actual layer work.
#[cfg(test)]
pub fn observed(external_projection: &str) -> Result<[u8; 9], LayerBlendFault> {
    #[derive(Deserialize)]
    struct Observations {
        terminal: Terminal,
    }

    #[derive(Deserialize)]
    #[serde(rename_all = "SCREAMING_SNAKE_CASE")]
    struct Terminal {
        reg_w12sel: u8,
        reg_w34sel: u8,
        reg_wobjsel: u8,
        reg_tm: u8,
        reg_ts: u8,
        reg_tmw: u8,
        reg_tsw: u8,
        next_gameplay_cgwsel: u8,
        next_gameplay_cgadsub: u8,
    }

    let Observations { terminal } =
        toml::from_str(external_projection).map_err(|_| LayerBlendFault::Format)?;
    Ok([
        terminal.reg_w12sel,
        terminal.reg_w34sel,
        terminal.reg_wobjsel,
        terminal.reg_tm,
        terminal.reg_ts,
        terminal.reg_tmw,
        terminal.reg_tsw,
        terminal.next_gameplay_cgwsel,
        terminal.next_gameplay_cgadsub,
    ])
}
