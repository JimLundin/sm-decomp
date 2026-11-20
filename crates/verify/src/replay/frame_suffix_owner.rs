//! Qualify source-only suffix ownership without importing future prepared publications.

use super::frame_origin::profiles::{ProfileName, Templates};
use data::timing::{Template, TemplateEffect};
use game::production::frame::suffix_caller::NmiQuantity;
use serde::Deserialize;

/// Unsupported source ownership, physical profile or actual producer readiness.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fault {
    /// The source classification or full binding contract is incompatible.
    Source,
    /// The complete physical template or effect binding is incompatible.
    Profile,
    /// A required actual completed producer capability is unavailable or stale.
    Ready,
}

/// Select the accepted complete unconditional software request writer, never by old request state.
pub fn preparation_template(templates: &Templates) -> Result<Template, Fault> {
    let mut portions = templates.portions.iter().filter(|portion| {
        portion.semantic_owner == "S6.FrameToWaitCaller" && portion.profile == ProfileName::PreWait
    });
    let portion = portions.next().ok_or(Fault::Source)?;
    if portions.next().is_some()
        || portion.receipt_contract != "FramePreparationToNmiWait caller and software request"
    {
        return Err(Fault::Source);
    }
    let mut profiles = templates
        .profiles
        .iter()
        .filter(|profile| profile.name == ProfileName::PreWait);
    let profile = profiles.next().ok_or(Fault::Profile)?;
    if profiles.next().is_some() {
        return Err(Fault::Profile);
    }
    let template = profile
        .slice(portion.first_exposure, portion.end_exposure)
        .map_err(|_| Fault::Profile)?
        .template()
        .map_err(|_| Fault::Profile)?;
    if template
        .runs
        .iter()
        .any(|run| run.effect != TemplateEffect::None)
    {
        return Err(Fault::Profile);
    }
    Ok(template)
}

/// Complete immutable pieces of the source-qualified single-idle-poll NMI family.
/// The controller suffix already contains native RTI; do not append a second return.
#[derive(Clone, Debug)]
pub struct IdleNmiPollParts {
    /// Full NMI work preceding the actual controller-busy reader.
    pub prefix: Template,
    /// Actual status read, mask and untaken branch; Busy remains Unsupported.
    pub poll: Template,
    /// Actual controller reads and complete existing native-return tail.
    pub controller_and_return: Template,
}

/// Import source slices and its actual-busy-bit branch contract, without forging a sample.
pub fn idle_nmi_poll_parts(text: &str, templates: &Templates) -> Result<IdleNmiPollParts, Fault> {
    #[derive(Deserialize)]
    struct Projection {
        format: u8,
        profile: ProfileName,
        predicate: String,
        guard_required: bool,
        busy_action: String,
        idle_action: String,
        dynamic_repetitions_owner: String,
        dynamic_repeat_admitted: bool,
        native_return_count: u8,
        final_poll_retained: bool,
        slice: Vec<Slice>,
    }
    #[derive(Deserialize)]
    struct Slice {
        name: String,
        first_exposure: u64,
        end_exposure: u64,
    }
    let source: Projection = toml::from_str(text).map_err(|_| Fault::Source)?;
    if source.format != 1
        || source.profile != ProfileName::NmiVideo
        || source.predicate != "ActualBlankingControllerBusy"
        || !source.guard_required
        || source.busy_action != "UnsupportedBeforeControllerEffects"
        || source.idle_action != "ControllerReadsThenNativeReturn"
        || source.dynamic_repetitions_owner != "Hardware"
        || source.dynamic_repeat_admitted
        || source.native_return_count != 1
        || !source.final_poll_retained
        || source.slice.len() != 4
    {
        return Err(Fault::Source);
    }
    let mut profiles = templates
        .profiles
        .iter()
        .filter(|profile| profile.name == ProfileName::NmiVideo);
    let profile = profiles.next().ok_or(Fault::Profile)?;
    if profiles.next().is_some() {
        return Err(Fault::Profile);
    }
    let member = |name| {
        let mut found = source.slice.iter().filter(|slice| slice.name == name);
        let value = found.next().ok_or(Fault::Source)?;
        if found.next().is_some() {
            return Err(Fault::Source);
        }
        Ok(value)
    };
    let prefix = member("prefix")?;
    let poll = member("poll")?;
    let controller = member("controller_and_return_suffix")?;
    let native = member("native_return")?;
    if prefix.first_exposure != 0
        || prefix.end_exposure != poll.first_exposure
        || poll.end_exposure != controller.first_exposure
        || controller.end_exposure != native.first_exposure
        || native.end_exposure != profile.exposures
        || native.first_exposure >= native.end_exposure
    {
        return Err(Fault::Source);
    }
    let template = |slice: &Slice| {
        profile
            .slice(slice.first_exposure, slice.end_exposure)
            .map_err(|_| Fault::Profile)?
            .template()
            .map_err(|_| Fault::Profile)
    };
    let parts = IdleNmiPollParts {
        prefix: template(prefix)?,
        poll: template(poll)?,
        controller_and_return: profile
            .slice(controller.first_exposure, native.end_exposure)
            .and_then(|profile| profile.template())
            .map_err(|_| Fault::Profile)?,
    };
    let samples: Vec<_> = parts
        .poll
        .runs
        .iter()
        .flat_map(|run| {
            (0..run.count).filter_map(move |_| {
                if let TemplateEffect::Sample(sample) = run.effect {
                    Some(sample)
                } else {
                    None
                }
            })
        })
        .collect();
    if samples.as_slice() != [data::timing::Sample::Blanking] {
        return Err(Fault::Profile);
    }
    Ok(parts)
}

/// Decode only independently admitted initial video fields from the corrected first cut.
/// Prepared S1 layer/haze and S6 camera scroll/upload requests are never initialized here.
pub fn initial_borrowed_video(
    bytes: &[u8],
) -> Result<game::production::frame::suffix_caller::InitialBorrowedVideo, Fault> {
    if bytes.len() != 0x20000 {
        return Err(Fault::Source);
    }
    let byte = |offset| bytes.get(offset).copied().ok_or(Fault::Source);
    let word = |offset: usize| {
        let high = offset.checked_add(1).ok_or(Fault::Source)?;
        Ok::<_, Fault>(u16::from_le_bytes([byte(offset)?, byte(high)?]))
    };
    Ok(
        game::production::frame::suffix_caller::InitialBorrowedVideo {
            background_character_bases_one_two: byte(93)?,
            background_character_bases_three_four: byte(94)?,
            background_four_horizontal: word(189)?,
            background_four_map: byte(92)?,
            background_four_vertical: word(191)?,
            background_mode: byte(85)?,
            background_one_map: byte(88)?,
            background_three_horizontal: word(185)?,
            background_three_map: byte(90)?,
            background_three_vertical: word(187)?,
            background_two_horizontal: word(181)?,
            background_two_map: byte(89)?,
            background_window_combination: byte(103)?,
            color_math: byte(114)?,
            color_window: byte(111)?,
            display: byte(81)?,
            dma_channel_seven_source_offset: word(6370)?,
            dma_channel_two_source_offset: word(6360)?,
            interrupt_enable: byte(132)?,
            mode_seven_settings: byte(95)?,
            mosaic: byte(87)?,
            object_color_window_combination: byte(104)?,
            object_layout: byte(82)?,
            screen_settings: byte(119)?,
            window_one_left: byte(99)?,
            window_one_right: byte(100)?,
            window_two_left: byte(101)?,
            window_two_right: byte(102)?,
        },
    )
}

/// Decode current background inputs independently of future prepared scroll outputs.
pub fn background_scroll_inputs(
    bytes: &[u8],
) -> Result<game::production::frame::suffix_caller::BackgroundScrollInputs, Fault> {
    if bytes.len() != 0x20000 {
        return Err(Fault::Source);
    }
    let byte = |offset| bytes.get(offset).copied().ok_or(Fault::Source);
    let word = |offset: usize| {
        let high = offset.checked_add(1).ok_or(Fault::Source)?;
        Ok::<_, Fault>(u16::from_le_bytes([byte(offset)?, byte(high)?]))
    };
    Ok(
        game::production::frame::suffix_caller::BackgroundScrollInputs {
            running: word(0xA78)? == 0,
            horizontal: scroll_disposition(byte(0x91B)?),
            vertical: scroll_disposition(byte(0x91C)?),
            one_x_offset: word(2333)?,
            one_y_offset: word(2335)?,
            two_y_offset: word(2339)?,
            vertical_mode: byte(2332)?,
        },
    )
}

fn portable_tiles(
    bound: super::frame_tiles::BoundFrame,
) -> game::production::frame::suffix_caller::SelectedTileUploads {
    let canonical = |upload: super::frame_tiles::BoundUpload| {
        game::production::frame::suffix_caller::SelectedTileUpload {
            definition: upload.definition,
            first_length: upload.first_length,
            second_length: upload.second_length,
            first: upload.first,
            second: upload.second,
        }
    };
    game::production::frame::suffix_caller::SelectedTileUploads {
        top: canonical(bound.top),
        bottom: bound.bottom.map(canonical),
    }
}

/// Import checked immutable HUD descriptor operands; the bank is a native word.
/// Truncating its high byte would accept an incompatible source binding.
pub fn hud_transfer_template(
    rom: data::rom::Lorom<'_>,
) -> Result<game::production::frame::suffix_caller::HudTransferTemplate, Fault> {
    let word = |offset| {
        let bytes = rom
            .read(data::rom::RomAddress::new(0x80, offset), 2)
            .map_err(|_| Fault::Source)?;
        let [low, high] = bytes else {
            return Err(Fault::Source);
        };
        Ok::<_, Fault>(u16::from_le_bytes([*low, *high]))
    };
    let length = word(0x9CAE)?;
    let source_offset = word(0x9CB5)?;
    let source_bank = u8::try_from(word(0x9CBC)?).map_err(|_| Fault::Source)?;
    let target = word(0x9CC2)?;
    if length != 192 {
        return Err(Fault::Source);
    }
    Ok(
        game::production::frame::suffix_caller::HudTransferTemplate {
            source: data::dma::BusAddress {
                bank: source_bank,
                offset: source_offset,
            },
            length,
            target: game::state::video::VramWord(target),
        },
    )
}

/// Import the accepted whole background preparation, including both actual product readers.
pub fn background_scroll_template(templates: &Templates) -> Result<Template, Fault> {
    let mut profiles = templates
        .profiles
        .iter()
        .filter(|profile| profile.name == ProfileName::PreWait);
    let profile = profiles.next().ok_or(Fault::Profile)?;
    if profiles.next().is_some() {
        return Err(Fault::Profile);
    }
    profile
        .slice(24604, 24834)
        .map_err(|_| Fault::Profile)?
        .template()
        .map_err(|_| Fault::Profile)
}

/// Admit only the actual current empty queue; a future descriptor cursor cannot seed it.
pub fn initial_hud_queue(
    bytes: &[u8],
) -> Result<game::production::frame::suffix_caller::HudQueue, Fault> {
    if bytes.len() != 0x20000 {
        return Err(Fault::Source);
    }
    let low = *bytes.get(0x330).ok_or(Fault::Source)?;
    let high = *bytes.get(0x331).ok_or(Fault::Source)?;
    game::production::frame::suffix_caller::HudQueue::from_initial_cursor(u16::from_le_bytes([
        low, high,
    ]))
    .map_err(|_| Fault::Source)
}

pub(super) fn nmi_pictures(
    rendering: &game::production::frame::rendering_stage::Content,
    clips: &game::samus_appearance_bridge::Clips,
    cartridge: &data::rom::Rom,
) -> Result<Vec<game::production::frame::suffix_caller::PictureNmi>, Fault> {
    use data::pose::{
        Poses,
        animation::AnimationDelays,
        tiles::{GRAPHICS_BANKS, Graphics, SamusTiles},
    };
    use game::production::frame::suffix_caller::PictureNmi;
    let rom = cartridge.lorom();
    let poses = Poses::load(rom).map_err(|_| Fault::Source)?;
    let delays = AnimationDelays::load(rom, &poses).map_err(|_| Fault::Source)?;
    let tiles = SamusTiles::load(rom, &delays, &poses).map_err(|_| Fault::Source)?;
    let graphics = Graphics::load(rom, GRAPHICS_BANKS).map_err(|_| Fault::Source)?;
    let imported = import::samus::import_with_bindings(rom).map_err(|_| Fault::Source)?;
    let content = super::frame_tiles::Content {
        imported: &imported,
        tiles: &tiles,
        graphics: &graphics,
        clips,
    };
    let mut pictures = Vec::new();
    for clip in clips
        .poses()
        .iter()
        .filter(|clip| clip.figure.pose.action == types::Action::Running)
    {
        let mut step = samus_appearance::Step(0);
        while clip.clip.at(step).is_some() {
            if matches!(
                clip.clip.at(step),
                Some(samus_appearance::Instruction::Show(_))
            ) {
                let outputs = rendering
                    .picture_uploads(clip.figure, step)
                    .map_err(|_| Fault::Source)?;
                if !outputs.is_empty() {
                    let bound = content
                        .bind_figure(clip.figure, step, &outputs)
                        .map_err(|_| Fault::Source)?;
                    content.source(&bound.top, rom).map_err(|_| Fault::Source)?;
                    if let Some(bottom) = &bound.bottom {
                        content.source(bottom, rom).map_err(|_| Fault::Source)?;
                        let selected = portable_tiles(bound);
                        let immutable = nmi_operands(rom, &selected)?;
                        pictures.push(PictureNmi {
                            figure: clip.figure,
                            step,
                            selected,
                            immutable,
                        });
                    }
                }
            }
            let Some(next) = step.next() else {
                break;
            };
            step = next;
        }
    }
    if pictures.is_empty() {
        return Err(Fault::Source);
    }
    Ok(pictures)
}

fn nmi_operands(
    rom: data::rom::Lorom<'_>,
    selected: &game::production::frame::suffix_caller::SelectedTileUploads,
) -> Result<game::production::frame::suffix_caller::ImmutableNmiContent, Fault> {
    let bottom = selected.bottom.as_ref().ok_or(Fault::Ready)?;
    let address = |long: u32| {
        let bank = u8::try_from(long >> 16).map_err(|_| Fault::Source)?;
        let offset = u16::try_from(long & 65535).map_err(|_| Fault::Source)?;
        Ok::<_, Fault>(data::rom::RomAddress::new(bank, offset))
    };
    let read = |long, width| {
        let bytes = rom.read(address(long)?, width).map_err(|_| Fault::Source)?;
        match bytes {
            [byte] => Ok(u16::from(*byte)),
            [low, high] => Ok(u16::from_le_bytes([*low, *high])),
            _ => Err(Fault::Source),
        }
    };
    let immediate = |long: u32, width, opcode| {
        let site = long.checked_sub(1).ok_or(Fault::Source)?;
        if read(site, 1)? != opcode {
            return Err(Fault::Source);
        }
        read(long, width)
    };
    let descriptor = |upload: &game::production::frame::suffix_caller::SelectedTileUpload| {
        let [_, low] = upload.first_length.to_be_bytes();
        u16::from_le_bytes([upload.first.start.bank, low])
    };
    let immutable = game::production::frame::suffix_caller::ImmutableNmiContent {
        object_upload_control_and_target: immediate(0x80_933B, 2, 0xA9)?,
        object_upload_source_offset: immediate(0x80_9341, 2, 0xA9)?,
        object_upload_source_bank: u8::try_from(immediate(0x80_9347, 1, 0xA2)?)
            .map_err(|_| Fault::Source)?,
        object_upload_length: immediate(0x80_934C, 2, 0xA9)?,
        palette_upload_control_and_target: immediate(0x80_9355, 2, 0xA9)?,
        palette_upload_source_offset: immediate(0x80_935B, 2, 0xA9)?,
        palette_upload_source_bank: u8::try_from(immediate(0x80_9361, 1, 0xA2)?)
            .map_err(|_| Fault::Source)?,
        palette_upload_length: immediate(0x80_9366, 2, 0xA9)?,
        palette_start: u8::try_from(immediate(0x80_936C, 1, 0xA2)?).map_err(|_| Fault::Source)?,
        object_palette_upload_enable: u8::try_from(immediate(0x80_9371, 1, 0xA2)?)
            .map_err(|_| Fault::Source)?,
        samus_tile_upload_first_layout: u8::try_from(immediate(0x80_937E, 1, 0xA0)?)
            .map_err(|_| Fault::Source)?,
        samus_tile_upload_first_target: immediate(0x80_938F, 2, 0xA9)?,
        samus_tile_upload_first_control_and_target: immediate(0x80_9395, 2, 0xA9)?,
        samus_tile_upload_first_source_offset: selected.top.first.start.offset,
        samus_tile_upload_first_bank_and_length_low: descriptor(&selected.top),
        samus_tile_upload_first_length: selected.top.first_length,
        samus_tile_upload_first_enable: u8::try_from(immediate(0x80_937C, 1, 0xA2)?)
            .map_err(|_| Fault::Source)?,
        samus_tile_upload_first_second_segment_target: immediate(0x80_93B7, 2, 0xA9)?,
        samus_tile_upload_first_second_segment_length: selected.top.second_length,
        samus_tile_upload_second_target: immediate(0x80_93D8, 2, 0xA9)?,
        samus_tile_upload_second_control_and_target: immediate(0x80_93DE, 2, 0xA9)?,
        samus_tile_upload_second_source_offset: bottom.first.start.offset,
        samus_tile_upload_second_bank_and_length_low: descriptor(bottom),
        samus_tile_upload_second_length: bottom.first_length,
        samus_tile_upload_second_second_segment_target: immediate(0x80_9400, 2, 0xA9)?,
        samus_tile_upload_second_second_segment_length: bottom.second_length,
        hud_upload_control_and_target: immediate(0x80_8C8E, 2, 0xA9)?,
        hud_upload_layout_and_target_low: immediate(0x80_8CAB, 2, 0xA9)?,
        hud_upload_enable: u8::try_from(immediate(0x80_8CBB, 1, 0xA9)?)
            .map_err(|_| Fault::Source)?,
        vram_layout_after_upload: u8::try_from(immediate(0x80_8CD9, 1, 0xA9)?)
            .map_err(|_| Fault::Source)?,
        vram_layout_after_buffer: u8::try_from(immediate(0x80_8DAD, 1, 0xA9)?)
            .map_err(|_| Fault::Source)?,
    };
    Ok(immutable)
}

/// Complete source-qualified recipes and full NMI work; contains no source locations or values.
#[derive(Clone, Debug)]
pub struct NmiRecipes {
    /// Whole immutable body including its original controller/native-return tail.
    pub whole: Template,
    /// Exact ordered physical writes from actual typed quantities.
    pub publications: Vec<game::production::frame::suffix_caller::PublicationRecipe>,
}

fn nmi_quantity(owner: &str, quantity: &str) -> Result<NmiQuantity, Fault> {
    Ok(match (owner, quantity) {
        ("CheckedImmutableImportedContent", name) => return immutable_nmi_quantity(name),
        ("NamedZeroHardwareSetting", "ObjectAddressReset") => NmiQuantity::ObjectAddressReset,
        ("ActualUploadSegmentContinuation", "SamusTileUploadFirstAdvancedSourceOffset") => {
            NmiQuantity::FirstAdvancedSourceOffset
        }
        ("ActualUploadSegmentContinuation", "SamusTileUploadSecondAdvancedSourceOffset") => {
            NmiQuantity::SecondAdvancedSourceOffset
        }
        ("InitialBorrowedVideoOwner", name) => return initial_video_quantity(name),
        ("LayerBlendTerminal", "BackgroundWindowOneTwo") => {
            NmiQuantity::LayerBlendBackgroundWindowOneTwo
        }
        ("LayerBlendTerminal", "BackgroundWindowThreeFour") => {
            NmiQuantity::LayerBlendBackgroundWindowThreeFour
        }
        ("LayerBlendTerminal", "ObjectColorWindow") => NmiQuantity::LayerBlendObjectColorWindow,
        ("LayerBlendTerminal", "MainLayers") => NmiQuantity::LayerBlendMainLayers,
        ("LayerBlendTerminal", "MainWindowLayers") => NmiQuantity::LayerBlendMainWindowLayers,
        ("LayerBlendTerminal", "SubLayers") => NmiQuantity::LayerBlendSubLayers,
        ("LayerBlendTerminal", "SubWindowLayers") => NmiQuantity::LayerBlendSubWindowLayers,
        ("HazeFixedColour", "RedWire") => NmiQuantity::HazeRedWire,
        ("HazeFixedColour", "GreenWire") => NmiQuantity::HazeGreenWire,
        ("HazeFixedColour", "BlueWire") => NmiQuantity::HazeBlueWire,
        ("CurrentBackgroundScroll", "BackgroundOneHorizontal") => {
            NmiQuantity::BackgroundOneHorizontal
        }
        ("CurrentBackgroundScroll", "BackgroundOneVertical") => NmiQuantity::BackgroundOneVertical,
        ("CurrentBackgroundScroll", "BackgroundTwoVertical") => NmiQuantity::BackgroundTwoVertical,
        ("ActualCompletedHudTilemapTransferRequest", "Length") => NmiQuantity::HudLength,
        ("ActualCompletedHudTilemapTransferRequest", "SourceOffset") => {
            NmiQuantity::HudSourceOffset
        }
        ("ActualCompletedHudTilemapTransferRequest", "SourceHighBankWireWord") => {
            NmiQuantity::HudSourceHighBankWireWord
        }
        ("ActualCompletedHudTilemapTransferRequest", "Target") => NmiQuantity::HudTarget,
        ("HazeTerminal", "EnabledChannels") => NmiQuantity::HazeEnabledChannels,
        _ => return Err(Fault::Source),
    })
}

fn immutable_nmi_quantity(quantity: &str) -> Result<NmiQuantity, Fault> {
    Ok(match quantity {
        "ObjectUploadControlAndTarget" => NmiQuantity::ImmutableObjectUploadControlAndTarget,
        "ObjectUploadSourceOffset" => NmiQuantity::ImmutableObjectUploadSourceOffset,
        "ObjectUploadSourceBank" => NmiQuantity::ImmutableObjectUploadSourceBank,
        "ObjectUploadLength" => NmiQuantity::ImmutableObjectUploadLength,
        "PaletteUploadControlAndTarget" => NmiQuantity::ImmutablePaletteUploadControlAndTarget,
        "PaletteUploadSourceOffset" => NmiQuantity::ImmutablePaletteUploadSourceOffset,
        "PaletteUploadSourceBank" => NmiQuantity::ImmutablePaletteUploadSourceBank,
        "PaletteUploadLength" => NmiQuantity::ImmutablePaletteUploadLength,
        "PaletteStart" => NmiQuantity::ImmutablePaletteStart,
        "ObjectPaletteUploadEnable" => NmiQuantity::ImmutableObjectPaletteUploadEnable,
        "SamusTileUploadFirstLayout" => NmiQuantity::ImmutableSamusTileUploadFirstLayout,
        "SamusTileUploadFirstTarget" => NmiQuantity::ImmutableSamusTileUploadFirstTarget,
        "SamusTileUploadFirstControlAndTarget" => {
            NmiQuantity::ImmutableSamusTileUploadFirstControlAndTarget
        }
        "SamusTileUploadFirstSourceOffset" => {
            NmiQuantity::ImmutableSamusTileUploadFirstSourceOffset
        }
        "SamusTileUploadFirstBankAndLengthLow" => {
            NmiQuantity::ImmutableSamusTileUploadFirstBankAndLengthLow
        }
        "SamusTileUploadFirstLength" => NmiQuantity::ImmutableSamusTileUploadFirstLength,
        "SamusTileUploadFirstEnable" => NmiQuantity::ImmutableSamusTileUploadFirstEnable,
        "SamusTileUploadFirstSecondSegmentTarget" => {
            NmiQuantity::ImmutableSamusTileUploadFirstSecondSegmentTarget
        }
        "SamusTileUploadFirstSecondSegmentLength" => {
            NmiQuantity::ImmutableSamusTileUploadFirstSecondSegmentLength
        }
        "SamusTileUploadSecondTarget" => NmiQuantity::ImmutableSamusTileUploadSecondTarget,
        "SamusTileUploadSecondControlAndTarget" => {
            NmiQuantity::ImmutableSamusTileUploadSecondControlAndTarget
        }
        "SamusTileUploadSecondSourceOffset" => {
            NmiQuantity::ImmutableSamusTileUploadSecondSourceOffset
        }
        "SamusTileUploadSecondBankAndLengthLow" => {
            NmiQuantity::ImmutableSamusTileUploadSecondBankAndLengthLow
        }
        "SamusTileUploadSecondLength" => NmiQuantity::ImmutableSamusTileUploadSecondLength,
        "SamusTileUploadSecondSecondSegmentTarget" => {
            NmiQuantity::ImmutableSamusTileUploadSecondSecondSegmentTarget
        }
        "SamusTileUploadSecondSecondSegmentLength" => {
            NmiQuantity::ImmutableSamusTileUploadSecondSecondSegmentLength
        }
        "HudUploadControlAndTarget" => NmiQuantity::ImmutableHudUploadControlAndTarget,
        "HudUploadLayoutAndTargetLow" => NmiQuantity::ImmutableHudUploadLayoutAndTargetLow,
        "HudUploadEnable" => NmiQuantity::ImmutableHudUploadEnable,
        "VramLayoutAfterUpload" => NmiQuantity::ImmutableVramLayoutAfterUpload,
        "VramLayoutAfterBuffer" => NmiQuantity::ImmutableVramLayoutAfterBuffer,
        _ => return Err(Fault::Source),
    })
}

fn initial_video_quantity(quantity: &str) -> Result<NmiQuantity, Fault> {
    Ok(match quantity {
        "InterruptEnable" => NmiQuantity::InitialInterruptEnable,
        "Display" => NmiQuantity::InitialDisplay,
        "ObjectLayout" => NmiQuantity::InitialObjectLayout,
        "BackgroundMode" => NmiQuantity::InitialBackgroundMode,
        "Mosaic" => NmiQuantity::InitialMosaic,
        "BackgroundOneMap" => NmiQuantity::InitialBackgroundOneMap,
        "BackgroundTwoMap" => NmiQuantity::InitialBackgroundTwoMap,
        "BackgroundThreeMap" => NmiQuantity::InitialBackgroundThreeMap,
        "BackgroundFourMap" => NmiQuantity::InitialBackgroundFourMap,
        "BackgroundCharacterBasesOneTwo" => NmiQuantity::InitialBackgroundCharacterBasesOneTwo,
        "BackgroundCharacterBasesThreeFour" => {
            NmiQuantity::InitialBackgroundCharacterBasesThreeFour
        }
        "ModeSevenSettings" => NmiQuantity::InitialModeSevenSettings,
        "WindowOneLeft" => NmiQuantity::InitialWindowOneLeft,
        "WindowOneRight" => NmiQuantity::InitialWindowOneRight,
        "WindowTwoLeft" => NmiQuantity::InitialWindowTwoLeft,
        "WindowTwoRight" => NmiQuantity::InitialWindowTwoRight,
        "BackgroundWindowCombination" => NmiQuantity::InitialBackgroundWindowCombination,
        "ObjectColorWindowCombination" => NmiQuantity::InitialObjectColorWindowCombination,
        "ColorWindow" => NmiQuantity::InitialColorWindow,
        "ColorMath" => NmiQuantity::InitialColorMath,
        "ScreenSettings" => NmiQuantity::InitialScreenSettings,
        "BackgroundTwoHorizontal" => NmiQuantity::InitialBackgroundTwoHorizontal,
        "BackgroundThreeHorizontal" => NmiQuantity::InitialBackgroundThreeHorizontal,
        "BackgroundThreeVertical" => NmiQuantity::InitialBackgroundThreeVertical,
        "BackgroundFourHorizontal" => NmiQuantity::InitialBackgroundFourHorizontal,
        "BackgroundFourVertical" => NmiQuantity::InitialBackgroundFourVertical,
        "DmaChannelTwoSourceOffset" => NmiQuantity::InitialDmaChannelTwoSourceOffset,
        "DmaChannelSevenSourceOffset" => NmiQuantity::InitialDmaChannelSevenSourceOffset,
        _ => return Err(Fault::Source),
    })
}

/// Import the accepted immutable identity projections and join them to actual physical ports.
pub fn nmi_recipes(
    wire_text: &str,
    s6_text: &str,
    templates: &Templates,
) -> Result<NmiRecipes, Fault> {
    #[derive(Deserialize)]
    struct Wire {
        format: String,
        publication: Vec<Row>,
    }
    #[derive(Deserialize)]
    struct S6 {
        format: String,
        publication_binding: Vec<Row>,
    }
    #[derive(Deserialize)]
    struct Row {
        publication_ordinal: usize,
        device: String,
        typed_owner: String,
        typed_quantity: String,
        word_part: String,
    }
    let pin = |text: &str, expected: &str| {
        let expected = crate::lock::Sha256::try_from(expected).map_err(|_| Fault::Source)?;
        if crate::lock::Sha256::of(text.as_bytes()) != expected {
            return Err(Fault::Source);
        }
        Ok::<_, Fault>(())
    };
    pin(
        wire_text,
        "536b43b39adf688bef78bdfede3dca0e0a00ec035830b3b91dee14f4e8ea6a0e",
    )?;
    pin(
        s6_text,
        "70bd3f2e8049e7dc197d3ddf519c6c24ee71dc3227d8410e71c9cae7b7e4d235",
    )?;
    let wire: Wire = toml::from_str(wire_text).map_err(|_| Fault::Source)?;
    let s6: S6 = toml::from_str(s6_text).map_err(|_| Fault::Source)?;
    if wire.format != "nmi-immutable-wire-identity-projection-v1"
        || wire.publication.len() != 108
        || s6.format != "bounded-suffix-s6-typed-producer-bindings-v2"
        || s6.publication_binding.len() != 14
    {
        return Err(Fault::Source);
    }
    let mut rows = std::collections::BTreeMap::new();
    for row in wire.publication.into_iter().chain(s6.publication_binding) {
        if rows.insert(row.publication_ordinal, row).is_some() {
            return Err(Fault::Source);
        }
    }
    let mut profiles = templates
        .profiles
        .iter()
        .filter(|profile| profile.name == ProfileName::NmiVideo);
    let profile = profiles.next().ok_or(Fault::Profile)?;
    if profiles.next().is_some() {
        return Err(Fault::Profile);
    }
    let mut physical = profile.runs.iter().flat_map(|run| {
        (0..run.count).filter_map(move |_| {
            if let super::frame_origin::profiles::Effect::Publication(publication) = &run.effect {
                Some(publication)
            } else {
                None
            }
        })
    });
    let mut publications = Vec::new();
    for (ordinal, row) in rows {
        let publication = physical.next().ok_or(Fault::Profile)?;
        if publication.device != row.device {
            return Err(Fault::Source);
        }
        if ordinal != publications.len() {
            return Err(Fault::Source);
        }
        let quantity = nmi_quantity(&row.typed_owner, &row.typed_quantity)?;
        let part = match row.word_part.as_str() {
            "Byte" => game::production::frame::suffix_caller::WordPart::Byte,
            "Low" => game::production::frame::suffix_caller::WordPart::Low,
            "High" => game::production::frame::suffix_caller::WordPart::High,
            _ => return Err(Fault::Source),
        };
        publications.push(game::production::frame::suffix_caller::PublicationRecipe {
            quantity,
            part,
            port: publication.port,
        });
    }
    if physical.next().is_some() || publications.len() != 122 {
        return Err(Fault::Profile);
    }
    Ok(NmiRecipes {
        whole: profile.template().map_err(|_| Fault::Profile)?,
        publications,
    })
}

const fn scroll_disposition(raw: u8) -> game::production::frame::suffix_caller::ScrollDisposition {
    match raw {
        0 => game::production::frame::suffix_caller::ScrollDisposition::Follow,
        1 => game::production::frame::suffix_caller::ScrollDisposition::Preserve,
        _ => game::production::frame::suffix_caller::ScrollDisposition::Scaled,
    }
}

impl NmiRecipes {
    pub(super) fn prefix_template(&self, parts: &IdleNmiPollParts) -> Result<Template, Fault> {
        let prefix = Template {
            runs: parts
                .prefix
                .runs
                .iter()
                .chain(&parts.poll.runs)
                .copied()
                .collect(),
        };
        let opportunities = |template: &Template| {
            template
                .runs
                .iter()
                .flat_map(|run| {
                    (0..run.count).map(move |_| (run.cycle, run.kind, run.poll_before, run.effect))
                })
                .collect::<Vec<_>>()
        };
        let source = opportunities(&self.whole);
        let candidate = opportunities(&prefix);
        let suffix = opportunities(&parts.controller_and_return);
        if source.get(..candidate.len()) != Some(candidate.as_slice())
            || source.get(candidate.len()..) != Some(suffix.as_slice())
        {
            return Err(Fault::Profile);
        }
        Ok(prefix)
    }
}
