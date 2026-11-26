//! Admit the interrupted shaft publication and compare its outputs before station dispatch.

use super::{carry, load_content};
use crate::replay::{blackout, compared, data_failure, explosion, result::Outcome};
use data::assets::countdown::IndirectHdmaStart;
use data::rom::{Lorom, Rom, RomAddress};
use game::exact::save_files::Format;
use game::production::played::{departure, landing_site};
use game::production::{
    Production, ProductionContent, ProductionRules, ProductionSeed, Scope, blackout as owner,
    escape_departure::{Content, Shaft, State},
};
use game::state::video::{DMA_CHANNELS, DmaChannel, Scroll};
use oracle_record::{Cut, FrameRecord, NmiKind};
use std::path::Path;

pub(in crate::replay) fn starts(record: &FrameRecord) -> Result<bool, Outcome> {
    Ok(explosion::word(record, 0x998)? == 33
        && explosion::word(record, 0x93f)? & 0x8000 != 0
        && matches!(record.cut,
            Cut::NmiReturn(nmi) if nmi.kind == NmiKind::Overrun
                && nmi.interrupted_pc == 0x0080_82ce))
}

pub(in crate::replay) fn admit(
    path: &Path,
    record: &FrameRecord,
) -> Result<(Production, ProductionContent, ProductionRules), Outcome> {
    let image = std::fs::read(path).map_err(|_| data_failure(None, "shaft ROM file"))?;
    let rom = Rom::new(image).map_err(|_| data_failure(None, "shaft ROM pin"))?;
    let rom = rom.lorom();
    let format = Format::load(rom).map_err(|_| data_failure(None, "cartridge format"))?;
    let figures = import::countdown::import(rom).map_err(|_| data_failure(None, "timer art"))?;
    let mut blackout = blackout::seed::read(record, rom, &format, true)?;
    blackout.video.batch.standing_strips = blackout::standing::pending(record, rom)?;
    let standing = game::samus_appearance_bridge::elevator::Art::load(rom)
        .map_err(|_| data_failure(Some(record.index), "standing body art"))?;
    blackout.video.dma = record.ppu.dma.map(crate::replay::landing_site::channel);
    blackout.video.batch.fixed_colour =
        crate::replay::landing_site::queued_colour(record)?.map(game::state::video::ColourWrite);
    setup_scroll(&mut blackout.video.batch.registers.scroll, record)?;
    let raster_opening = raster_opening(record, &blackout.video.dma, rom)?;
    let (placement, area, maps, shadow) = carry::room(record)?;
    let state = State::Shaft(Box::new(Shaft {
        blackout,
        room: placement,
        landing: landing_site::State::new(area, maps, shadow),
        progress: (&crate::replay::progress::decode(&record.wram)?).into(),
    }));
    let content = ProductionContent::Escape(Box::new(Content {
        shaft: owner::Content {
            figures,
            format,
            graphics: data::pose::tiles::Graphics::default(),
            standing: Some(standing),
        },
        raster_opening,
        departure: load_content(rom)?,
    }));
    let rules = ProductionRules::Departure(departure::Rules::REFERENCE);
    let production = Production::admit(
        ProductionSeed::Escape(Box::new(state)),
        content.clone(),
        rules.clone(),
    )
    .map_err(|why| data_failure(Some(record.index), &format!("shaft intake {why:?}")))?;
    Ok((production, content, rules))
}

fn setup_scroll(scroll: &mut [Scroll; 4], record: &FrameRecord) -> Result<(), Outcome> {
    const MIRRORS: [[usize; 2]; 4] = [[0xb1, 0xb3], [0xb5, 0xb7], [0xb9, 0xbb], [0xbd, 0xbf]];
    for (layer, [x, y]) in scroll.iter_mut().zip(MIRRORS) {
        layer.x = explosion::word(record, x)?;
        layer.y = explosion::word(record, y)?;
    }
    Ok(())
}

fn raster_opening(
    record: &FrameRecord,
    dma: &[DmaChannel; DMA_CHANNELS],
    rom: Lorom<'_>,
) -> Result<[Option<IndirectHdmaStart>; DMA_CHANNELS], Outcome> {
    const HDMA_ENABLE_MIRROR: usize = 0x85;
    let mask = explosion::byte(record, HDMA_ENABLE_MIRROR)?;
    let mut opening = [None; DMA_CHANNELS];
    for (i, (slot, channel)) in opening.iter_mut().zip(dma).enumerate() {
        if mask & (1 << i) != 0 {
            let source = RomAddress::new(channel.source_bank, channel.source);
            *slot = Some(
                import::countdown::indirect_hdma_start(rom, source)
                    .map_err(|_| data_failure(Some(record.index), "shaft HDMA table"))?,
            );
        }
    }
    Ok(opening)
}

pub(super) fn check(
    production: &Production,
    output: &owner::Output,
    previous: &FrameRecord,
    current: &FrameRecord,
) -> Result<(), Outcome> {
    compared(
        current.index,
        &blackout::observe::expected(previous, current, true)?.fields,
        &blackout::actual(production, output, true)?,
    )?;
    if output
        .requests
        .contains(&owner::Request::DisableShaftInterrupts)
    {
        let shaft = production
            .blackout()
            .ok_or_else(|| data_failure(Some(current.index), "shaft publication"))?;
        if let Some(i) = shaft
            .video
            .dma
            .iter()
            .zip(current.ppu.dma.map(crate::replay::landing_site::channel))
            .position(|(actual, expected)| *actual != expected)
        {
            return Err(Outcome::Difference {
                cut: current.index,
                field: format!("blackout.dma[{i}]"),
            });
        }
    }
    Ok(())
}

pub(super) fn check_cartridge(
    production: &Production,
    current: &FrameRecord,
    cut: u64,
) -> Result<(), Outcome> {
    if production.scope() != Scope::Escape {
        return Ok(());
    }
    let image = production
        .battery()
        .ok_or_else(|| data_failure(Some(cut), "departure cartridge"))?;
    if image.as_slice() != current.sram.as_ref() {
        return Err(Outcome::Difference {
            cut,
            field: "departure.cartridge".into(),
        });
    }
    Ok(())
}
