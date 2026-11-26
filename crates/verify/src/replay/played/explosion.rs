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
    mut previous: FrameRecord,
    row: &str,
    start: u64,
    end: u64,
) -> Result<Report, Outcome> {
    let (mut production, content, _) = if shaft::starts(&previous)? {
        shaft::admit(path, &previous)?
    } else {
        admit_seed(path, &previous)?
    };
    let (assets, seed_held) = if let ProductionContent::Departure(assets) = &content {
        (assets.as_ref(), STATION_SEED)
    } else if let ProductionContent::Escape(assets) = &content {
        (&assets.departure, SHAFT_SEED)
    } else {
        return Err(data_failure(Some(start), "departure assets"));
    };
    let mut observer = explosion::Observed::default();
    let report = |held, stop| Report {
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
    };
    let mut held = 0_u64;
    for cut in start.saturating_add(1)..=end {
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
        let output =
            match production.advance(AdvanceInput::Departure(input)) {
                Ok(output) => output,
                Err(ProductionFault::Unsupported(Unsupported::Departure(Fault::Landing(
                    landing_site::Fault::Dma(game::exact::direct_dma::Fault::Source),
                )))) => return Ok(report(
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
        if let Err(why) = check_outputs(
            &production,
            &output,
            cut,
            [&previous, &current],
            &assets.landing,
            input,
            &mut observer,
        )
        .and_then(|()| {
            if let Some(run) = production.departure() {
                carry::check(run, &current, timing, cut)?;
            }
            shaft::check_cartridge(&production, &current, cut)
        }) {
            return Ok(report(held, Stop::At(why)));
        }
        held = held.saturating_add(1);
        previous = current;
    }
    Ok(report(held, Stop::End))
}

fn check_outputs(
    production: &Production,
    output: &ProductionOutputs,
    cut: u64,
    [previous, current]: [&FrameRecord; 2],
    graphics: &data::assets::landing_site::Graphics,
    input: Input,
    observer: &mut explosion::Observed,
) -> Result<(), Outcome> {
    if let ProductionOutputs::Blackout(output) = output {
        shaft::check(production, output, previous, current)
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
            production, previous, current, graphics, input, cut,
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

pub(in crate::replay) use shaft::{admit as admit_shaft, starts as shaft_starts};

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
