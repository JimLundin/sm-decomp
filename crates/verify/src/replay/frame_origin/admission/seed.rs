//! Load the independently captured first cut and its required preparation owners.

use super::super::{
    Manifest, Member, Origin, borrowed_receipts, decode, hardware_seed, interrupt_inputs,
    layer_blend_receipt, preparation_inputs, preparation_receipts, profiles, random_inputs,
    sender_seed, validate_countdown_scene_scope,
};
use crate::lock::Sha256;
use crate::replay::{
    self, Sources, data_failure, frame_player_preparation_receipts as player_preparation_receipts,
    index::{Case, Start},
    movement,
    result::Outcome,
};
use canonical::model::Model;
use game::production::{frame, timing::PadChange};

pub(super) fn read_manifest(case: &Case, sources: &Sources) -> Result<Manifest, Outcome> {
    let Start::NativeSeed { path, sha256 } = &case.start else {
        return Err(data_failure(None, "frame native origin"));
    };
    let path = if path.is_absolute() {
        path.clone()
    } else {
        sources.native.join(path)
    };
    let root = path
        .parent()
        .ok_or_else(|| data_failure(None, "frame manifest directory"))?
        .to_path_buf();
    let mut manifest: Manifest = toml::from_str(
        &Member {
            path,
            sha256: sha256.clone(),
        }
        .text()?,
    )
    .map_err(|_| data_failure(None, "frame origin manifest"))?;
    manifest.resolve(&root);
    if manifest.final_owners.is_some() != case.compares_final_frame_owners() {
        return Err(data_failure(
            None,
            "matched final frame owner declaration and expectation",
        ));
    }
    if manifest.format != 1
        || manifest.host_budget == 0
        || manifest.current_pad_row == manifest.preview_pad_row
    {
        return Err(data_failure(None, "frame origin manifest contract"));
    }
    Ok(manifest)
}

fn countdown_scene(
    case: &Case,
    manifest: &Manifest,
    wram: &[u8],
) -> Result<
    Option<(
        frame::countdown_scene::Admission,
        frame::countdown_scene::Current,
    )>,
    Outcome,
> {
    let scene_countdown = if let Some(member) = &manifest.countdown_scene_current {
        validate_countdown_scene_scope(case)?;
        let expected =
            Sha256::try_from("a9513e8f8dbe8dfe4cfc6e1f34c1c8071c2f9c22f44a9766b1c339524df82555")
                .map_err(|_| data_failure(None, "countdown scene projection identity"))?;
        let origin =
            Sha256::try_from("bfee6ac13da413a36ad42194a36aa3224af6a84e818092b1e7040ed4782f03db")
                .map_err(|_| data_failure(None, "countdown scene origin identity"))?;
        if member.pin()? != expected || manifest.wram.pin()? != origin {
            return Err(data_failure(None, "countdown scene source identity"));
        }
        Some(
            replay::frame_countdown_scene::current(wram, &member.text()?)
                .map_err(|_| data_failure(None, "current separate countdown scene owner"))?,
        )
    } else {
        None
    };
    Ok(scene_countdown)
}

struct MachineSeed {
    hardware: Vec<u8>,
    timing: game::production::timing::State,
    next_pad: PadChange,
    senders: frame::Senders,
    initial: Vec<data::timing::ContentRegion>,
}

fn machine_seed(
    manifest: &Manifest,
    sources: &Sources,
    wram: &[u8],
) -> Result<MachineSeed, Outcome> {
    let pads = manifest.pads.text()?;
    let current = pads
        .lines()
        .nth(manifest.current_pad_row)
        .ok_or_else(|| data_failure(None, "frame current pad row"))?;
    let preview = pads
        .lines()
        .nth(manifest.preview_pad_row)
        .ok_or_else(|| data_failure(None, "frame preview pad row"))?;
    let (current, preview) = hardware_seed::pads(current, preview)
        .map_err(|_| data_failure(None, "frame physical pads"))?;
    let hardware = manifest.hardware.read()?;
    let timing = hardware_seed::decode(&hardware, data::timing::IrqPolicy::Eligible, current)
        .map_err(|_| data_failure(None, "frame completed-return hardware origin"))?;
    let next_pad = PadChange {
        at: timing.hardware.now,
        pad: preview,
    };
    let senders = sender_seed::decode(wram, &manifest.apuram.read()?)
        .map_err(|_| data_failure(None, "frame current sender origins"))?;
    let mut initial = manifest
        .initial
        .iter()
        .map(|initial| {
            Ok(data::timing::ContentRegion {
                start: data::dma::BusAddress {
                    bank: initial.bank,
                    offset: initial.offset,
                },
                bytes: initial.member.read()?,
            })
        })
        .collect::<Result<Vec<_>, Outcome>>()?;
    if let Some(member) = &manifest.initial_hdma_rom_backing {
        let image = std::fs::read(&sources.rom)
            .map_err(|_| data_failure(None, "frame checked initial HDMA ROM"))?;
        initial.extend(
            replay::frame_hdma_content::admit(&image, &timing, &member.text()?)
                .map_err(|_| data_failure(None, "frame bounded initial HDMA ROM content"))?,
        );
    }
    Ok(MachineSeed {
        hardware,
        timing,
        next_pad,
        senders,
        initial,
    })
}

struct PlayerEntry {
    current: frame::borrowed_player_preparation::PlayerEntryOrigins,
    offers: frame::borrowed_player_preparation::PlayerEntryOffers,
    labels: String,
    reference_acceleration: types::Fixed,
}

fn player_entry(
    manifest: &Manifest,
    wram: &[u8],
    checked_rom: &data::rom::Rom,
) -> Result<PlayerEntry, Outcome> {
    let entry_text = manifest.player_entry_current.text()?;
    let entry_labels = manifest.player_entry_labels.text()?;
    let entry_links = manifest.player_entry_links.text()?;
    let entry_current = frame::borrowed_player_preparation::PlayerEntryOrigins {
        pose_input: player_preparation_receipts::current_pose(wram, &entry_text)
            .map_err(|_| data_failure(None, "frame actual current pose-input dependency"))?,
        suit_palette: player_preparation_receipts::current_suit(wram, &entry_text)
            .map_err(|_| data_failure(None, "frame actual current suit-palette dependency"))?,
        environment: player_preparation_receipts::current_environment(
            wram,
            &entry_text,
            &entry_labels,
        )
        .map_err(|_| data_failure(None, "frame actual current environment dependency"))?,
    };
    let entry_offers = frame::borrowed_player_preparation::PlayerEntryOffers {
        suit_palette: player_preparation_receipts::suit_offer(
            wram,
            &entry_text,
            &entry_links,
            checked_rom,
        )
        .map_err(|_| data_failure(None, "frame qualified deferred suit-palette effect"))?,
        environment: player_preparation_receipts::environment_offer(
            wram,
            &entry_text,
            &entry_labels,
            &entry_links,
            checked_rom,
        )
        .map_err(|_| data_failure(None, "frame qualified deferred environment effects"))?,
    };
    let reference_acceleration =
        player_preparation_receipts::reference_acceleration(&entry_text, &entry_links, checked_rom)
            .map_err(|_| data_failure(None, "frame actual imported reference acceleration"))?;
    Ok(PlayerEntry {
        current: entry_current,
        offers: entry_offers,
        labels: entry_labels,
        reference_acceleration,
    })
}

struct PreludeWork {
    layer_defaults: frame::layer_blend::LayerBlendDefaults,
    drawing: frame::interrupts::DrawingState,
}

fn prelude_work(
    manifest: &Manifest,
    wram: &[u8],
    checked_rom: &data::rom::Rom,
) -> Result<PreludeWork, Outcome> {
    let layer_text = manifest.layer_blend.text()?;
    let layer_defaults = layer_blend_receipt::defaults(checked_rom, &layer_text)
        .map_err(|_| data_failure(None, "frame immutable layer defaults"))?;

    let drawing_current = interrupt_inputs::current(wram)
        .map_err(|_| data_failure(None, "frame current drawing owner"))?;
    let [begin, end] = interrupt_inputs::receipts(drawing_current, &manifest.irq_current.text()?)
        .map_err(|_| data_failure(None, "frame complete drawing dependencies"))?;
    let [after_begin, after_end] =
        interrupt_inputs::terminal_schedule(&manifest.irq_terminal.text()?)
            .map_err(|_| data_failure(None, "frame drawing terminal schedule"))?;
    let drawing_state = frame::interrupts::DrawingState {
        current: drawing_current,
        begin: Some(begin),
        end: Some(end),
        active: None,
        after_begin: Some(after_begin),
        after_end: Some(after_end),
    };
    Ok(PreludeWork {
        layer_defaults,
        drawing: drawing_state,
    })
}

fn templates(manifest: &Manifest) -> Result<profiles::Templates, Outcome> {
    let texts = manifest
        .profiles
        .iter()
        .map(Member::text)
        .collect::<Result<Vec<_>, _>>()?;
    let texts: [&str; 6] = texts
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>()
        .try_into()
        .map_err(|_| data_failure(None, "frame physical profile members"))?;
    let templates = profiles::decode(texts)
        .map_err(|_| data_failure(None, "frame physical profile contracts"))?;
    Ok(templates)
}

fn palette(
    manifest: &Manifest,
    wram: &[u8],
    templates: &profiles::Templates,
    initial: &mut Vec<data::timing::ContentRegion>,
) -> Result<
    (
        Option<frame::samus_palette::Palette>,
        Option<frame::samus_palette::Contract>,
    ),
    Outcome,
> {
    let (palette_state, samus_palette) = match (&manifest.palette_locations, &manifest.palette_edge)
    {
        (Some(locations), Some(edge)) => {
            let (state, contract) = replay::frame_palette::origin(
                wram,
                &locations.text()?,
                &edge.text()?,
                templates,
                initial,
            )?;
            (Some(state), Some(contract))
        }
        (None, None) => (None, None),
        _ => return Err(data_failure(None, "paired ordinary palette sources")),
    };
    Ok((palette_state, samus_palette))
}

fn bind_player_entry(
    manifest: &Manifest,
    wram: &[u8],
    checked_rom: &data::rom::Rom,
    state: &mut frame::State,
    entry_current: frame::borrowed_player_preparation::PlayerEntryOrigins,
    entry_offers: frame::borrowed_player_preparation::PlayerEntryOffers,
) -> Result<Option<frame::hud_controls_inputs::Bindings>, Outcome> {
    let frame::Caller::AwaitingAcknowledgment {
        prelude: frame::PreludeSource::Origin(prelude),
        ..
    } = &mut state.caller
    else {
        return Err(data_failure(None, "frame first entry owner boundary"));
    };
    state.pose_input_configuration = Some(frame::pose_input::Configuration::new(
        entry_current.pose_input.bindings,
    ));
    prelude.player_entry_current = Some(Model(Box::new(entry_current)));
    prelude.player_entry_offers = Some(Model(Box::new(entry_offers)));
    if let Some(member) = &manifest.player_continuation_current {
        let accepted =
            Sha256::try_from("6ca142b3ff1f22a64eb17f2f13486f9854c0d2acfadf017760010a1907324f16")
                .map_err(|_| data_failure(None, "frame remaining player source identity"))?;
        if member.pin()? != accepted {
            return Err(data_failure(None, "frame remaining player source identity"));
        }
        let (current, offers) =
            player_preparation_receipts::player_continuation(wram, &member.text()?, checked_rom)
                .map_err(|_| {
                    data_failure(None, "frame current unselected remaining player owners")
                })?;
        prelude.player_continuation_current = Some(Model(Box::new(current)));
        prelude.player_continuation_offers = Some(Model(Box::new(offers)));
    }
    let early_hud_controls = if let Some(member) = &manifest.hud_current_controls {
        let proof = member.text()?;
        let bindings = replay::frame_hud_controls_inputs::qualified_bindings(wram, &proof)
            .map_err(|_| data_failure(None, "frame current early HUD physical bindings"))?;
        prelude.hud_inventory = Some(
            replay::frame_hud_controls_inputs::qualified_inventory(wram, &proof)
                .map_err(|_| data_failure(None, "frame independently current HUD inventory"))?,
        );
        Some(bindings)
    } else {
        None
    };
    Ok(early_hud_controls)
}

pub(super) struct EntrySeed {
    pub(super) state: frame::State,
    pub(super) next_pad: PadChange,
    pub(super) hardware: Vec<u8>,
    pub(super) image: Vec<u8>,
    pub(super) checked_rom: data::rom::Rom,
    pub(super) entry_labels: String,
    pub(super) reference_acceleration: types::Fixed,
    pub(super) layer_defaults: frame::layer_blend::LayerBlendDefaults,
    pub(super) templates: profiles::Templates,
    pub(super) random_requests: frame::random::SeedRequests,
    pub(super) random: data::timing::Template,
    pub(super) palette_state: Option<frame::samus_palette::Palette>,
    pub(super) samus_palette: Option<frame::samus_palette::Contract>,
    pub(super) scene_countdown: Option<(
        frame::countdown_scene::Admission,
        frame::countdown_scene::Current,
    )>,
    pub(super) early_hud_controls: Option<frame::hud_controls_inputs::Bindings>,
}

pub(super) fn entry_seed(
    case: &Case,
    sources: &Sources,
    manifest: &Manifest,
    content: &movement::Content,
    wram: &[u8],
) -> Result<EntrySeed, Outcome> {
    let scene_countdown = countdown_scene(case, manifest, wram)?;
    let MachineSeed {
        hardware,
        timing,
        next_pad,
        senders,
        mut initial,
    } = machine_seed(manifest, sources, wram)?;
    let music =
        borrowed_receipts::qualified_music_completion(wram, &manifest.music_completion.text()?)
            .map_err(|_| data_failure(None, "frame borrowed current music completion"))?;
    let image = std::fs::read(&sources.rom).map_err(|_| data_failure(None, "frame checked ROM"))?;
    let checked_rom = data::rom::Rom::new(image.clone())
        .map_err(|_| data_failure(None, "frame checked ROM identity"))?;
    let entry = player_entry(manifest, wram, &checked_rom)?;
    let PreludeWork {
        layer_defaults,
        drawing,
    } = prelude_work(manifest, wram, &checked_rom)?;
    let random_current = preparation_receipts::random_origin(wram)
        .map_err(|_| data_failure(None, "frame current original Random seed"))?;
    let preparation = preparation_inputs(wram, manifest)?;
    let templates = templates(manifest)?;
    let (random_requests, random) = random_inputs::decode(
        random_current,
        &checked_rom,
        &manifest.random_requests.text()?,
        &manifest.random_import.text()?,
        &templates,
    )
    .map_err(|_| {
        data_failure(
            None,
            "frame checked Random requests and full physical owner",
        )
    })?;
    let (palette_state, samus_palette) = palette(manifest, wram, &templates, &mut initial)?;
    let initial = frame::TransferContent::new(initial)
        .map_err(|_| data_failure(None, "frame initial content ownership"))?;
    let mut state = decode(
        Origin {
            wram,
            timing,
            senders,
            content: initial,
            music: Some(music),
            drawing: Some(drawing),
            random: random_current,
            preparation,
        },
        content,
    )?;
    let early_hud_controls = bind_player_entry(
        manifest,
        wram,
        &checked_rom,
        &mut state,
        entry.current,
        entry.offers,
    )?;
    Ok(EntrySeed {
        state,
        next_pad,
        hardware,
        image,
        checked_rom,
        entry_labels: entry.labels,
        reference_acceleration: entry.reference_acceleration,
        layer_defaults,
        templates,
        random_requests,
        random,
        palette_state,
        samus_palette,
        scene_countdown,
        early_hud_controls,
    })
}
