//! Play both departing cinematics from the single elevator blackout.

pub(in crate::replay) mod carry;
mod shaft;

use super::{Report, Stop, rows::Rows};
use crate::replay::{compared, data_failure, explosion, result::Outcome, zebes};
use game::production::played::departure::{Content, Fault, Input, Rules, Run, Scene};
use game::production::played::landing_site::{self, Work};
use game::production::{
    AdvanceInput, Production, ProductionContent, ProductionFault, ProductionOutputs,
    ProductionRules, ProductionSeed, Unsupported,
};
use oracle_record::FrameRecord;
use std::path::Path;

pub(super) fn play(
    rows: &mut Rows<'_>,
    path: &Path,
    previous: FrameRecord,
    row: &str,
    start: u64,
    end: u64,
) -> Result<Report, Outcome> {
    let (held, stop, seed_held) = if shaft::starts(&previous)? {
        let (shaft, assets) = shaft::seed(path, &previous)?;
        let (held, stop) =
            through_shaft(shaft, assets, rows, previous, start.saturating_add(1)..=end)?;
        (held, stop, SHAFT_SEED)
    } else {
        let (production, content, _) = admit_seed(path, &previous)?;
        let ProductionContent::Departure(assets) = &content else {
            return Err(data_failure(Some(start), "departure assets"));
        };
        let (held, stop) = through(
            production,
            assets,
            rows,
            previous,
            start.saturating_add(1)..=end,
            None,
        )?;
        (held, stop, STATION_SEED)
    };
    Ok(Report {
        row: row.to_owned(),
        start,
        held,
        stop,
        seed_held,
        borrowed: &[
            "T1 work completion",
            "post-service music queue reply",
            "T1 accepted and selected bytes during direct DMA",
        ],
        masked: &[
            "in-progress cinematic setup transfers",
            "E3 flash scanline composition and retained HDMA channels 2 and 3",
        ],
    })
}

/// Admit complete live shaft owners and continue on the caller's row chain.
pub(super) fn through_shaft(
    shaft: game::production::escape_departure::Shaft,
    assets: game::production::escape_departure::Content,
    rows: &mut Rows<'_>,
    previous: FrameRecord,
    cuts: core::ops::RangeInclusive<u64>,
) -> Result<(u64, Stop), Outcome> {
    let (production, content, rules) =
        shaft::admit_owned(shaft, assets, cuts.start().saturating_sub(1))?;
    let ProductionContent::Escape(assets) = &content else {
        return Err(data_failure(None, "escape departure assets"));
    };
    through(
        production,
        &assets.departure,
        rows,
        previous,
        cuts,
        Some((&content, &rules)),
    )
}

/// Continue the selected shaft or cinematic on the caller's existing row chain.
/// The preceding record supplies timing and observations. Production retains its
/// owned state across row changes.
fn through(
    mut production: Production,
    assets: &Content,
    rows: &mut Rows<'_>,
    mut previous: FrameRecord,
    cuts: core::ops::RangeInclusive<u64>,
    shaft_context: Option<(&ProductionContent, &ProductionRules)>,
) -> Result<(u64, Stop), Outcome> {
    let mut observer = explosion::Observed::default();
    let mut held = 0_u64;
    for cut in cuts {
        let current = rows.record(cut)?;
        if current.video_frame != previous.video_frame.wrapping_add(1) {
            return Err(data_failure(Some(cut), "departure row join"));
        }
        let timing = explosion::timing(&current)?;
        let input = match production.departure() {
            Some(run) if run.loading() => {
                Input::Loader(landing_work(run, &current, &assets.landing)?)
            }
            Some(_) => Input::Scene(timing),
            None => Input::Shaft {
                work: crate::replay::blackout::work(&previous, &current, false)?,
                station: timing,
            },
        };
        let restored = shaft_checkpoint(&production, shaft_context, input, cut)?;
        let output =
            match production.advance(AdvanceInput::Departure(input)) {
                Ok(output) => output,
                Err(ProductionFault::Unsupported(Unsupported::Departure(Fault::Landing(
                    landing_site::Fault::Dma(game::exact::direct_dma::Fault::Source),
                )))) => return Ok((
                    held,
                    Stop::At(Outcome::Unsupported {
                        cut: Some(cut),
                        dependency:
                            "Landing Site sprite DMA needs the live source after the ROM bank wraps"
                                .into(),
                    }),
                )),
                Err(fault) => {
                    return Err(data_failure(
                        Some(cut),
                        &format!("departure advance {fault:?}"),
                    ));
                }
            };
        if let Some(mut restored) = restored {
            let resumed = restored.advance(AdvanceInput::Departure(input));
            if resumed.as_ref() != Ok(&output) || restored != production {
                return Ok((held, Stop::At(Outcome::CheckpointFailure { cut })));
            }
        }
        if let Err(why) = check_outputs(
            &production,
            &output,
            cut,
            [&previous, &current],
            &assets.landing,
            rows,
            &mut observer,
        )
        .and_then(|()| {
            if let Some(run) = production.departure() {
                carry::check(run, &current, timing, cut)?;
            }
            shaft::check_cartridge(&production, &current, cut)
        }) {
            return Ok((held, Stop::At(why)));
        }
        held = held.saturating_add(1);
        previous = current;
    }
    Ok((held, Stop::End))
}

fn shaft_checkpoint(
    production: &Production,
    admitted: Option<(&ProductionContent, &ProductionRules)>,
    input: Input,
    cut: u64,
) -> Result<Option<Production>, Outcome> {
    use game::production::blackout::{Cut, Work};
    let Some((content, rules)) = admitted else {
        return Ok(None);
    };
    if !matches!(
        input,
        Input::Shaft {
            work: Cut {
                work: Work::VerticalEnded | Work::SaveCompleted,
                ..
            },
            ..
        }
    ) {
        return Ok(None);
    }
    let fail = |_| Outcome::CheckpointFailure { cut };
    let saved = production.save().map_err(fail)?;
    let restored = Production::restore(&saved, content.clone(), rules.clone())
        .map_err(|_| Outcome::CheckpointFailure { cut })?;
    if restored != *production {
        return Err(Outcome::CheckpointFailure { cut });
    }
    Ok(Some(restored))
}

fn check_outputs(
    production: &Production,
    output: &ProductionOutputs,
    cut: u64,
    [previous, current]: [&FrameRecord; 2],
    graphics: &data::assets::landing_site::Graphics,
    rows: &mut Rows<'_>,
    observer: &mut explosion::Observed,
) -> Result<(), Outcome> {
    if let ProductionOutputs::Blackout(output) = output {
        shaft::check(production, output, previous, current, cut)
    } else if let ProductionOutputs::Explosion(output) = output {
        compared(
            cut,
            &observer.read(previous, current)?.fields,
            &explosion::actual(production, Some(output))?,
        )
    } else if let ProductionOutputs::Zebes(output) = output {
        compared(
            cut,
            &zebes::expected(previous, current, true)?.fields,
            &zebes::actual(production, output, true)?,
        )
    } else if matches!(output, ProductionOutputs::LandingSite) {
        crate::replay::departure::check_receiving(
            production, previous, current, graphics, rows, cut,
        )
    } else {
        Err(data_failure(Some(cut), "departure output scope"))
    }
}

const STATION_SEED: &[&str] = &[
    "elevator blackout, completed save, Health, destination, caption, shaft room, explored maps, RNG and video memory",
];
const SHAFT_SEED: &[&str] = &[
    "shaft awaiting vblank end, including the queued picture, cartridge before the write, room and live video",
];

pub(in crate::replay) use shaft::{
    admit as admit_shaft, interrupted as shaft_interrupted, starts as shaft_starts,
};

// The external T1 receipt measures source-bus work. No destination memory or game
// state from the receiving record is supplied to the simulation.
pub(in crate::replay) fn receiving_work(
    record: &FrameRecord,
    graphics: &data::assets::landing_site::Graphics,
) -> Result<Work, Outcome> {
    let channel = crate::replay::landing_site::channel(record.ppu.dma[1]);
    let accepted = graphics
        .sprite_length
        .checked_sub(channel.count)
        .ok_or_else(|| data_failure(Some(record.index), "landing DMA remaining count"))?;
    let ahead = channel
        .source
        .wrapping_sub(graphics.sprite_source.offset)
        .wrapping_sub(accepted);
    if channel.source_bank != graphics.sprite_source.bank
        || ahead > 1
        || !matches!(record.cut, oracle_record::Cut::VblankWithoutNmi)
    {
        return Err(data_failure(
            Some(record.index),
            "landing DMA read selection",
        ));
    }
    Ok(Work::Open {
        accepted,
        selected: ahead == 1,
    })
}

fn landing_work(
    run: &Run,
    record: &FrameRecord,
    graphics: &data::assets::landing_site::Graphics,
) -> Result<Work, Outcome> {
    match run.carry.landing.phase {
        landing_site::Phase::Pending => receiving_work(record, graphics),
        landing_site::Phase::Sprites(transfer) => {
            let channel = crate::replay::landing_site::channel(record.ppu.dma[1]);
            if channel.source_bank != transfer.source.bank {
                // Reprogramming the channel for a later transfer requires the old DMA to finish.
                return Ok(Work::Transfer {
                    accepted: transfer.length,
                    selected: false,
                });
            }
            let Work::Open { accepted, selected } = receiving_work(record, graphics)? else {
                return Err(data_failure(Some(record.index), "landing transfer receipt"));
            };
            Ok(Work::Transfer { accepted, selected })
        }
        landing_site::Phase::Ready => Err(data_failure(
            Some(record.index),
            "Landing Site graphics already drained",
        )),
    }
}

pub(in crate::replay) fn state(production: &Production, cut: u64) -> Result<&Run, Outcome> {
    let Some(run) = production.departure() else {
        return Err(data_failure(Some(cut), "departure state"));
    };
    Ok(run)
}

pub(in crate::replay) fn admit_seed(
    path: &Path,
    record: &FrameRecord,
) -> Result<(Production, ProductionContent, ProductionRules), Outcome> {
    let image = std::fs::read(path).map_err(|_| data_failure(None, "departure ROM file"))?;
    let rom = data::rom::Rom::new(image).map_err(|_| data_failure(None, "departure ROM pin"))?;
    let rom = rom.lorom();
    let content = ProductionContent::Departure(Box::new(load_content(rom)?));
    let rules = ProductionRules::Departure(Rules::REFERENCE);
    let station = explosion::seed_state(record)?;
    let carry = carry::origin(&station, record)?;
    let run = Run {
        scene: Scene::Station(Box::new(station)),
        carry,
    };
    let production = Production::admit(
        ProductionSeed::Departure(Box::new(run)),
        content.clone(),
        rules.clone(),
    )
    .map_err(|why| data_failure(Some(record.index), &format!("departure intake {why:?}")))?;
    Ok((production, content, rules))
}

fn load_content(rom: data::rom::Lorom<'_>) -> Result<Content, Outcome> {
    let station = import::explosion::import(rom)
        .map_err(|_| data_failure(None, "departure station content"))?;
    let flight =
        import::zebes::import(rom).map_err(|_| data_failure(None, "departure flight content"))?;
    let landing = import::landing_site::import(rom)
        .map_err(|_| data_failure(None, "Landing Site standard graphics"))?;
    Ok(Content {
        station,
        flight,
        landing,
    })
}

pub(super) fn live_content(
    path: &Path,
    content: &game::production::played::Content,
    format: game::exact::save_files::Format,
    hud: game::production::blackout::HudContent,
) -> Result<game::production::escape_departure::Content, Outcome> {
    use data::assets::countdown::shaft_opening;
    use data::pose::tiles::{GRAPHICS_BANKS, Graphics};
    use data::rom::Rom;
    use game::production::{blackout, escape_departure};

    let image = Rom::new(std::fs::read(path).map_err(|_| data_failure(None, "departure ROM"))?)
        .map_err(|_| data_failure(None, "departure ROM pin"))?;
    let figures = content
        .clock_figures
        .clone()
        .ok_or_else(|| data_failure(None, "departure timer art"))?;
    let programs = content
        .shaft_raster
        .ok_or_else(|| data_failure(None, "departure shaft raster programs"))?;
    Ok(escape_departure::Content {
        shaft: blackout::Content {
            figures,
            graphics: Graphics::load(image.lorom(), GRAPHICS_BANKS)
                .map_err(|_| data_failure(None, "departure sprite graphics"))?,
            format,
            standing: content.standing.clone(),
            hud: Some(hud),
        },
        raster_opening: shaft_opening(programs),
        departure: load_content(image.lorom())?,
    })
}

#[cfg(test)]
mod tests {
    #[test]
    #[ignore = "pinned record cache and player ROM"]
    fn shaft_bg2_scroll_reaches_the_station_and_flight() {
        use super::super::{
            Run, Stop, appearance, connected, movement, open_rows, read_lock, seed_controller,
        };

        let sources = super::super::tests::sources();
        let lock = read_lock(&sources).unwrap();
        let row = "chain-any-0010-DF8D";
        let mut rows = open_rows(&sources, &lock, row).unwrap();
        let start = rows.record(0).unwrap();
        let content = connected::Content {
            movement: movement::Content::load(&sources.rom).unwrap(),
            appearance: appearance::Content::load(&sources.rom).unwrap(),
        };
        let mut room = Run::admit(&start, &content, &sources.rom, row).unwrap();
        let controller = seed_controller(&start).unwrap();
        assert_eq!(
            room.through(&mut rows, start, controller, 1..=518).unwrap(),
            (518, Stop::End)
        );
        assert_eq!(
            rows.place(518).unwrap(),
            ("chain-any-0011-DF45".into(), 239)
        );
        let prepared = rows.record(518).unwrap();
        let video = &room.state.room_video.video;
        super::super::room_video::compare_shaft_bg2_scroll(video, &prepared, 13_311).unwrap();

        // Admit the other late-shaft owners at the handoff. Both BG2 scroll values
        // come from the running room and continue through the actual NMI and scenes.
        let (mut shaft, assets) = super::shaft::seed(&sources.rom, &prepared).unwrap();
        shaft.blackout.video.committed.scroll[1] = video.committed.scroll[1];
        shaft.blackout.video.batch.registers.scroll[1] = video.batch.registers.scroll[1];
        let mut movie = open_rows(&sources, &lock, "any-power-on").unwrap();
        assert_eq!(
            super::through_shaft(shaft, assets, &mut movie, prepared, 13_312..=15_196).unwrap(),
            (1885, Stop::End)
        );
        let prefix = super::super::play(&sources, "any-power-on", 0, 13_311).unwrap();
        assert_eq!((prefix.held, prefix.stop), (13_311, Stop::End));
    }

    // The reviewed run covers cut 15,197. This run must finish at the preceding cut.
    #[test]
    #[ignore = "pinned record cache and player ROM"]
    fn power_on_reaches_the_landing_site_before_the_rom_bank_boundary() {
        let sources = super::super::tests::sources();
        let ending = super::super::play(&sources, "any-power-on", 0, 15_196).unwrap();
        assert_eq!((ending.held, ending.stop), (15_196, super::Stop::End));
        assert!(ending.seed_held.is_empty());
    }

    #[test]
    #[ignore = "pinned record cache and player ROM"]
    fn owned_shaft_uses_the_existing_movie_coordinates() {
        let sources = super::super::tests::sources();
        let lock = super::super::read_lock(&sources).unwrap();
        let mut rows = super::super::open_rows(&sources, &lock, "any-power-on").unwrap();
        assert_eq!(
            rows.place(13_311).unwrap(),
            ("chain-any-0011-DF45".into(), 239)
        );
        let previous = rows.record(13_311).unwrap();
        // This scoped routing witness supplies the same admitted state as the
        // cut-239 run. The shared continuation receives no state from later records.
        let (shaft, assets) = super::shaft::seed(&sources.rom, &previous).unwrap();
        let (held, stop) =
            super::through_shaft(shaft, assets, &mut rows, previous, 13_312..=15_197).unwrap();
        assert_eq!(held, 1885);
        assert!(matches!(
            stop,
            super::Stop::At(super::Outcome::Unsupported { cut: Some(15_197), dependency })
                if dependency == "Landing Site sprite DMA needs the live source after the ROM bank wraps"
        ));
    }
}
