//! Independent recorded inputs and observations for the production flight to Zebes.

use data::rom::Rom;
use game::production::played::zebes::{Flow, LoadDestination, Output, State};
use game::production::{
    AdvanceInput, Production, ProductionContent, ProductionFault, ProductionOutputs,
    ProductionRules, ProductionSeed, Unsupported,
};
use game::state::owned::OwnedState;
use game::state::video::{Colour, Oam};
use oracle_record::{Cut, FrameRecord, NmiKind};
use screens_zebes::{ColourEffectRequest, FrameAdvance};
use types::zebes::{Brightness, FlightView, ScenePicture, StarField, SubtitleChoice, ZebesStage};

use super::compare::Observation;
use super::index::{Case, ObservationId, Responsibility, Start};
use super::readers::{
    ActualReader, AdmittedCase, ExpectedCut, ExpectedReader, InputReader, record,
};
use super::result::{Counts, Outcome};
use super::{Sources, data_failure};

const GAME_FLOW: usize = 0x0998;
const SCENE: usize = 0x1f51;
const SCENE_FLOW: u16 = 0x22;
const LOADING_FLOW: u16 = 6;
const FINISHED: usize = 0x05b6;
const ENERGY: usize = 0x09c2;
const ENERGY_CAPACITY: usize = 0x09c4;
const DESTINATION: usize = 0xd914;
const LANDING_SITE: u16 = 0x22;
const CERES_START: u16 = 0x1f;
const SUBTITLES: usize = 0x09e2;

pub(super) const SCOPED_FIELDS: &[ObservationId] = &[
    ObservationId::ZebesStage,
    ObservationId::ZebesPicture,
    ObservationId::ZebesMotion,
];
pub(super) const SCOPED_OUTPUTS: &[ObservationId] = &[ObservationId::ZebesRequests];
pub(super) const INTEGRATED_FIELDS: &[ObservationId] = &[
    ObservationId::ZebesStage,
    ObservationId::ZebesPicture,
    ObservationId::ZebesMotion,
    ObservationId::ZebesHealth,
    ObservationId::ZebesSchedule,
];
pub(super) const INTEGRATED_OUTPUTS: &[ObservationId] = &[
    ObservationId::ZebesRequests,
    ObservationId::ZebesPages,
    ObservationId::ZebesOam,
    ObservationId::ZebesPalette,
];

// Every cut compares light. A finished cut adds the scene, transform and seven pictures.
const SCENE_FIELDS_ON_FINISH: usize = 14;
const RECEIVING_FIELDS: usize = 5;
const DRAW_OUTPUTS_ON_FINISH: usize = 2;
const SETUP_AND_JOIN_OUTPUTS: usize = 6;

pub(super) fn valid_counts(case: &Case, counts: &Counts) -> bool {
    let integrated = case
        .responsibilities
        .contains(&Responsibility::ZebesDrawing);
    let base_fields = if integrated {
        RECEIVING_FIELDS.saturating_add(1)
    } else {
        1
    };
    let Some(finished_fields) = counts
        .fields
        .checked_sub(counts.cuts.saturating_mul(base_fields))
    else {
        return false;
    };
    let finished = finished_fields / SCENE_FIELDS_ON_FINISH;
    let outputs = if integrated {
        counts
            .cuts
            .saturating_add(finished.saturating_mul(DRAW_OUTPUTS_ON_FINISH))
            .saturating_add(SETUP_AND_JOIN_OUTPUTS)
    } else {
        counts.cuts
    };
    finished <= counts.cuts
        && finished_fields.is_multiple_of(SCENE_FIELDS_ON_FINISH)
        && counts.outputs == outputs
        && counts.publications == 0
        && counts.replies == 0
        && counts.pending_boundaries == 0
}

const PICTURES: [(ScenePicture, usize, u16); 7] = [
    (ScenePicture::Planet, 30, 0x9558),
    (ScenePicture::Stars(StarField::First), 28, 0x975e),
    (ScenePicture::Stars(StarField::Second), 26, 0x979c),
    (ScenePicture::Stars(StarField::Third), 24, 0x97bc),
    (ScenePicture::Stars(StarField::Fourth), 22, 0x97d2),
    (ScenePicture::English, 20, 0x9654),
    (ScenePicture::Japanese, 18, 0x968d),
];

pub(super) struct Inputs {
    start: u64,
    cuts: Vec<FrameAdvance>,
}

pub(super) struct Expected {
    start: u64,
    cuts: Vec<ExpectedCut>,
}

impl Inputs {
    pub fn advance(
        &self,
        production: &mut Production,
        cut: u64,
    ) -> Result<ProductionOutputs, Outcome> {
        let index = offset(self.start, cut)?;
        let input = self
            .cuts
            .get(index)
            .ok_or_else(|| data_failure(Some(cut), "flight cut input"))?;
        advance(production, *input, cut)
    }
}

impl Expected {
    pub fn read(&self, cut: u64) -> Result<ExpectedCut, Outcome> {
        self.cuts
            .get(offset(self.start, cut)?)
            .cloned()
            .ok_or_else(|| data_failure(Some(cut), "flight expected cut"))
    }
}

fn offset(start: u64, cut: u64) -> Result<usize, Outcome> {
    cut.checked_sub(start)
        .and_then(|offset| offset.checked_sub(1))
        .and_then(|offset| usize::try_from(offset).ok())
        .ok_or_else(|| data_failure(Some(cut), "flight cut range"))
}

pub(super) fn starts(seed: &FrameRecord) -> Result<bool, Outcome> {
    Ok(word(seed, GAME_FLOW)? == SCENE_FLOW
        && word(seed, SCENE)? == 0xc699
        && byte(seed, 0x0051)? & 0x80 != 0)
}

pub(super) fn content(rom: &std::path::Path) -> Result<data::assets::zebes::Zebes, Outcome> {
    let image = std::fs::read(rom).map_err(|_| data_failure(None, "flight ROM file"))?;
    let rom = Rom::new(image).map_err(|_| data_failure(None, "flight ROM pin"))?;
    import::zebes::import(rom.lorom())
        .map_err(|fault| data_failure(None, &format!("flight content {fault:?}")))
}

pub(super) fn admit_seed(
    rom: &std::path::Path,
    seed: &FrameRecord,
) -> Result<(Production, ProductionContent, ProductionRules), Outcome> {
    if !starts(seed)? {
        return Err(data_failure(
            Some(seed.index),
            "blank station-to-Zebes landing context",
        ));
    }
    let destination = match word(seed, DESTINATION)? {
        LANDING_SITE => LoadDestination::LandingSite,
        CERES_START => LoadDestination::CeresStart,
        _ => return Err(data_failure(Some(seed.index), "flight loading context")),
    };
    let content = ProductionContent::Zebes(Box::new(content(rom)?));
    let rules = ProductionRules::Zebes(screens_zebes::Rules::REFERENCE);
    let mut palette = [Colour(0); 256];
    if palette.len() != seed.cgram.len() {
        return Err(data_failure(Some(seed.index), "flight initial palette"));
    }
    for (out, &value) in palette.iter_mut().zip(&seed.cgram) {
        *out = Colour(value);
    }
    let mut oam = Oam::default();
    let low = seed
        .oam
        .get(..512)
        .ok_or_else(|| data_failure(Some(seed.index), "flight seed low OAM"))?;
    for (out, &[x, y, tile, attributes]) in oam.low.iter_mut().zip(low.as_chunks::<4>().0) {
        *out = game::state::video::OamEntry {
            x,
            y,
            tile,
            attributes,
        };
    }
    let high = seed
        .oam
        .get(512..544)
        .ok_or_else(|| data_failure(Some(seed.index), "flight seed high OAM"))?;
    oam.high.copy_from_slice(high);
    let state = State::start(
        word(seed, FINISHED)?,
        super::health_tail::decode(&seed.wram, Some(seed.index))?,
        word(seed, ENERGY_CAPACITY)?,
        palette,
        oam,
        if word(seed, SUBTITLES)? == 0 {
            SubtitleChoice::EnglishOnly
        } else {
            SubtitleChoice::WithJapanese
        },
        destination,
    );
    let production = Production::admit(
        ProductionSeed::Zebes(Box::new(state)),
        content.clone(),
        rules.clone(),
    )
    .map_err(|fault| {
        data_failure(
            Some(seed.index),
            &format!("flight production admission {fault:?}"),
        )
    })?;
    Ok((production, content, rules))
}

const fn cadence(record: &FrameRecord) -> FrameAdvance {
    if matches!(record.cut, Cut::NmiReturn(nmi) if matches!(nmi.kind, NmiKind::Requested)) {
        FrameAdvance::Finished
    } else {
        FrameAdvance::BorrowedLag
    }
}

fn advance(
    production: &mut Production,
    input: FrameAdvance,
    cut: u64,
) -> Result<ProductionOutputs, Outcome> {
    match production.advance(AdvanceInput::Zebes(input)) {
        Ok(output @ ProductionOutputs::Zebes(_)) => Ok(output),
        Err(ProductionFault::Unsupported(Unsupported::Zebes(fault))) => Err(Outcome::Unsupported {
            cut: Some(cut),
            dependency: format!("Zebes flight {fault:?}"),
        }),
        _ => Err(data_failure(Some(cut), "flight production scope")),
    }
}

pub(super) fn admit(case: &Case, sources: &Sources) -> Result<AdmittedCase, Outcome> {
    let Start::CacheCut { frame: start } = case.start else {
        return Err(data_failure(None, "flight cache start"));
    };
    let lock = std::fs::read_to_string(&sources.lock)
        .ok()
        .and_then(|text| crate::lock::Lock::parse(&text).ok())
        .ok_or_else(|| data_failure(None, "flight record lock"))?;
    let row = lock
        .row(&crate::lock::RowId(case.row.clone()))
        .ok_or_else(|| data_failure(None, "flight record row"))?;
    let cache = crate::cache::Cache::open(&sources.cache, row)
        .map_err(|_| data_failure(None, "flight pinned cache"))?;
    let mut previous = record(&cache, start)?;
    let (production, content, rules) = admit_seed(&sources.rom, &previous)?;
    let integrated = case
        .responsibilities
        .contains(&Responsibility::ZebesDrawing);
    let first = start
        .checked_add(1)
        .ok_or_else(|| data_failure(None, "flight first cut"))?;
    let schedule: Vec<_> = (first..=case.end.cut_inclusive).collect();
    let mut input_cuts = Vec::with_capacity(schedule.len());
    let mut expected_cuts = Vec::with_capacity(schedule.len());
    let landing = integrated
        .then(|| landing_seed(&lock, sources))
        .transpose()?;
    for &cut in &schedule {
        let current = record(&cache, cut)?;
        input_cuts.push(cadence(&current));
        let mut observed = expected(&previous, &current, integrated)?;
        if let Some(landing) = &landing
            && word(&current, GAME_FLOW)? == LOADING_FLOW
        {
            observed.fields.push(landing_join(landing)?);
            observed.output_count = observed.output_count.saturating_add(1);
        }
        expected_cuts.push(observed);
        previous = current;
    }
    let ProductionContent::Zebes(imported) = &content else {
        return Err(data_failure(None, "flight content scope"));
    };
    let actual = ActualReader::Zebes {
        integrated,
        scene: imported.scene,
    };
    Ok(AdmittedCase {
        production,
        content,
        rules,
        schedule,
        actual,
        inputs: InputReader::Zebes(Inputs {
            start,
            cuts: input_cuts,
        }),
        expected: ExpectedReader::Zebes(Expected {
            start,
            cuts: expected_cuts,
        }),
    })
}

fn byte(record: &FrameRecord, index: usize) -> Result<u8, Outcome> {
    record
        .wram
        .get(index)
        .copied()
        .ok_or_else(|| data_failure(Some(record.index), "flight WRAM byte"))
}

fn word(record: &FrameRecord, index: usize) -> Result<u16, Outcome> {
    Ok(u16::from_le_bytes([
        byte(record, index)?,
        byte(record, index.saturating_add(1))?,
    ]))
}

fn quantity(field: &str, value: impl Into<i64>) -> Observation {
    Observation::quantity(format!("zebes.{field}"), value.into())
}

fn packed(field: &str, values: impl IntoIterator<Item = u16>) -> Observation {
    bytes(
        field,
        values.into_iter().flat_map(u16::to_le_bytes).collect(),
    )
}

fn bytes(field: &str, bytes: Vec<u8>) -> Observation {
    Observation {
        field: format!("zebes.{field}"),
        bytes,
    }
}

fn state(production: &Production) -> Result<&State, Outcome> {
    if let OwnedState::Zebes(state) = production.state() {
        Ok(state)
    } else if let Some(run) = production.departure()
        && let game::production::played::departure::Scene::Flight(state) = &run.scene
    {
        Ok(state)
    } else {
        Err(data_failure(None, "flight actual state"))
    }
}

pub(super) fn initial(
    production: &Production,
    scene: &types::zebes::ZebesScene,
    integrated: bool,
) -> Result<Vec<Observation>, Outcome> {
    let state = state(production)?;
    let output = Output {
        scene: screens_zebes::FrameReport {
            picture: screens_zebes::report::publish(&state.scene.0, scene),
            requests: screens_zebes::Requests::NONE,
        },
        pages: None,
    };
    actual(production, &output, integrated)
}

pub(super) fn actual(
    production: &Production,
    output: &Output,
    integrated: bool,
) -> Result<Vec<Observation>, Outcome> {
    let state = state(production)?;
    let owned = &state.scene.0;
    let picture = &output.scene.picture;
    let (phase, counter, beat) = match owned.stage {
        ZebesStage::Pending => (0, 0, 0),
        ZebesStage::Revealing { fade, beat } => (
            1,
            u16::from(fade == types::zebes::FadeWait::OneFrame),
            u16::from(beat.get()),
        ),
        ZebesStage::Caption { age } => (2, age.0, 0),
        ZebesStage::Rightward => (3, 0, 0),
        ZebesStage::Banking => (4, 0, 0),
        ZebesStage::Leftward => (5, 0, 0),
        ZebesStage::Still { hold } => (6, hold.get(), 0),
        ZebesStage::Sliding => (7, 0, 0),
        ZebesStage::Leaving => (8, 0, 0),
        ZebesStage::Complete => (9, 0, 0),
    };
    let mut fields = vec![
        quantity("stage", phase),
        quantity(
            "light",
            match picture.light {
                Brightness::Blanked => 0x80,
                Brightness::Shown(level) => level.get(),
            },
        ),
        quantity("mosaic", picture.mosaic.get()),
        quantity(
            "view",
            match picture.view {
                FlightView::MosaicPage => 0,
                FlightView::ShipAndSprites => 1,
                FlightView::SpritesOnly => 2,
            },
        ),
        packed("counter", [counter, beat]),
        packed(
            "pose",
            [
                u16::from(owned.pose.angle.0),
                owned.pose.zoom.0,
                owned.pose.scroll.x.pixel.cast_unsigned(),
                owned.pose.scroll.x.fraction,
                owned.pose.scroll.y.pixel.cast_unsigned(),
                owned.pose.scroll.y.fraction,
            ],
        ),
        packed(
            "scroll",
            [
                picture.plane.scroll.x.cast_unsigned(),
                picture.plane.scroll.y.cast_unsigned(),
            ],
        ),
        packed(
            "matrix",
            [
                picture.plane.matrix.a,
                picture.plane.matrix.b,
                picture.plane.matrix.c,
                picture.plane.matrix.d,
            ]
            .into_iter()
            .map(plane_word),
        ),
        quantity("energy", state.health.0.energy.whole.get()),
        quantity("maximum", state.maximum),
        quantity("finished", state.finished),
        quantity("flow", flow_word(state.flow)),
        quantity("destination", destination_word(state.flow.destination())),
    ];
    for (which, _, _) in PICTURES {
        let at = owned.pictures[which].map(|motion| {
            [
                motion.position.x.pixel.cast_unsigned(),
                motion.position.x.fraction,
                motion.position.y.pixel.cast_unsigned(),
                motion.position.y.fraction,
                motion.speed.0,
            ]
        });
        fields.push(packed(
            &format!("motion.{which:?}"),
            at.into_iter().flatten(),
        ));
    }
    fields.extend(actual_outputs(state, output, integrated));
    Ok(fields)
}

fn actual_outputs(state: &State, output: &Output, integrated: bool) -> Vec<Observation> {
    let owned = &state.scene.0;
    let picture = &output.scene.picture;
    let mut fields = Vec::new();
    let request = match (
        output.scene.requests.colour_effect,
        output.scene.requests.handoff.is_some(),
    ) {
        (Some(ColourEffectRequest::ReplaceActive { .. }), _) => 1,
        (Some(ColourEffectRequest::CaptionBrightens(_)), _) => 2,
        (Some(ColourEffectRequest::CaptionDims(_)), _) => 3,
        (None, true) => 4,
        (None, false) => 0,
    };
    fields.push(quantity("request", request));
    if !integrated {
        return fields;
    }
    if let Flow::Loading(destination) = state.flow {
        fields.push(packed(
            "landing.join",
            [
                state.finished,
                flow_word(state.flow),
                state.health.0.energy.whole.get(),
                state.maximum,
                destination_word(destination),
                if picture.light == Brightness::Blanked {
                    0x80
                } else {
                    0
                },
                u16::from(owned.pose.angle.0),
                owned.pose.zoom.0,
                picture.plane.scroll.x.cast_unsigned(),
                picture.plane.scroll.y.cast_unsigned(),
                plane_word(picture.plane.matrix.a),
                plane_word(picture.plane.matrix.b),
                plane_word(picture.plane.matrix.c),
                plane_word(picture.plane.matrix.d),
            ],
        ));
    }
    if let Some(pages) = &output.pages {
        fields.push(bytes("pages.ship", pages.ship_map.clone()));
        fields.push(bytes(
            "pages.ship-tiles",
            pages.inherited_ship_tiles.clone(),
        ));
        fields.push(packed("pages.flat", pages.flat_map.iter().copied()));
        fields.push(packed("pages.tiles", pages.zebes_tiles.iter().copied()));
        fields.push(packed(
            "palette.setup",
            state.palette.iter().map(|colour| colour.0),
        ));
    }
    fields.push(packed(
        "palette",
        state.palette.iter().map(|colour| colour.0),
    ));
    fields.push(bytes(
        "oam",
        oam_bytes(
            state
                .oam
                .low
                .iter()
                .map(|entry| [entry.x, entry.y, entry.tile, entry.attributes]),
            &state.oam.high,
        ),
    ));
    fields
}

pub(super) const fn flow_word(flow: Flow) -> u16 {
    match flow {
        Flow::Flight(_) => SCENE_FLOW,
        Flow::Loading(_) => LOADING_FLOW,
    }
}

pub(super) const fn destination_word(destination: LoadDestination) -> u16 {
    match destination {
        LoadDestination::LandingSite => LANDING_SITE,
        LoadDestination::CeresStart => CERES_START,
    }
}

const fn plane_word(value: i32) -> u16 {
    let [lo, hi, ..] = value.to_le_bytes();
    u16::from_le_bytes([lo, hi])
}

fn oam_bytes(entries: impl IntoIterator<Item = [u8; 4]>, high: &[u8]) -> Vec<u8> {
    let mut visible = Vec::new();
    for (index, [x, y, tile, attributes]) in entries.into_iter().enumerate() {
        if y == game::exact::oam::HIDDEN_Y {
            continue;
        }
        let bits = high.get(index / 4).copied().unwrap_or_default() >> (index % 4).wrapping_mul(2);
        visible.extend([
            u8::try_from(index).unwrap_or_default(),
            x,
            y,
            tile,
            attributes,
            bits & 3,
        ]);
    }
    visible
}

pub(super) fn expected(
    previous: &FrameRecord,
    current: &FrameRecord,
    integrated: bool,
) -> Result<ExpectedCut, Outcome> {
    let w = |at| word(current, at);
    let b = |at| byte(current, at);
    let loading = w(GAME_FLOW)? == LOADING_FLOW;
    let held = cadence(current) == FrameAdvance::BorrowedLag;
    let setup = !held && word(previous, SCENE)? == 0xc699;
    let stage = stage(current, loading)?;
    let raw_light = b(0x0051)?;
    let mut semantics = vec![quantity(
        "light",
        if raw_light & 0x80 != 0 {
            0x80
        } else {
            raw_light & 15
        },
    )];
    if integrated {
        semantics.extend([
            quantity("energy", w(ENERGY)?),
            quantity("maximum", w(ENERGY_CAPACITY)?),
            quantity("finished", w(FINISHED)?),
            quantity("flow", w(GAME_FLOW)?),
            quantity("destination", w(DESTINATION)?),
        ]);
    }
    if !held {
        semantics.extend(expected_picture(current, stage)?);
    }
    let outputs = expected_outputs(previous, current, integrated, held, setup)?;
    let semantic_count = semantics.len();
    let output_count = outputs.len();
    let fields = semantics.iter().chain(&outputs).cloned().collect();
    Ok(ExpectedCut {
        fields,
        semantics,
        semantic_count,
        output_count,
        replies: 0,
    })
}

fn stage(current: &FrameRecord, loading: bool) -> Result<i32, Outcome> {
    Ok(if loading {
        9
    } else {
        match word(current, SCENE)? {
            0xc699 => 0,
            0xc79c | 0xc7ca => 1,
            0xa38f => 2,
            0xc9f9 => 3,
            0xca36 => 4,
            0xca85 => 5,
            0xcad0 => 6,
            0xcade => 7,
            0xcadf => 8,
            _ => return Err(data_failure(Some(current.index), "flight observed stage")),
        }
    })
}

fn expected_picture(current: &FrameRecord, stage: i32) -> Result<Vec<Observation>, Outcome> {
    let w = |at| word(current, at);
    let b = |at| byte(current, at);
    let mut semantics = Vec::new();
    let caption_age = if stage == 2 {
        let end = match w(0x1b31)? {
            0xccbf => 64u16,
            0xccc5 => 96,
            0xcccb => 288,
            0xccd1 => 384,
            _ => {
                return Err(data_failure(
                    Some(current.index),
                    "flight observed caption hold",
                ));
            }
        };
        end.checked_sub(w(0x1b71)?)
            .ok_or_else(|| data_failure(Some(current.index), "flight observed caption age"))?
    } else {
        0
    };
    let counter = match stage {
        1 => [w(0x0725)?, w(0x1a51)? & 3],
        2 => [caption_age, 0],
        6 => [w(0x1a49)?, 0],
        _ => [0, 0],
    };
    semantics.extend([
        quantity("stage", stage),
        quantity("mosaic", (b(0x0057)? >> 4).saturating_add(1)),
        quantity(
            "view",
            if b(0x0055)? == 1 {
                0
            } else if b(0x0069)? == 17 {
                1
            } else {
                2
            },
        ),
        packed("counter", counter),
        packed(
            "pose",
            words_at(current, [0x198d, 0x198f, 0x1993, 0x1991, 0x1997, 0x1995])?,
        ),
        packed(
            "matrix",
            words_at(current, [0x0078, 0x007a, 0x007c, 0x007e])?,
        ),
        packed("scroll", words_at(current, [0x00b1, 0x00b3])?),
    ]);
    for (which, slot, picture) in PICTURES {
        let word = w(0x1a5dusize.saturating_add(slot))?;
        let at = match word {
            0 => None,
            _ if word == picture => Some(words_at(
                current,
                [0x1a7dusize, 0x1add, 0x1a9d, 0x1afd, 0x1b7d].map(|base| base.saturating_add(slot)),
            )?),
            _ => {
                return Err(data_failure(
                    Some(current.index),
                    "flight observed sprite picture",
                ));
            }
        };
        semantics.push(packed(
            &format!("motion.{which:?}"),
            at.into_iter().flatten(),
        ));
    }
    Ok(semantics)
}

fn expected_outputs(
    previous: &FrameRecord,
    current: &FrameRecord,
    integrated: bool,
    held: bool,
    setup: bool,
) -> Result<Vec<Observation>, Outcome> {
    let active = |name| has_effect(current, name);
    let incoming = |name| has_effect(previous, name);
    let request = if setup && active(0xe1a8)? && incoming(0xe1b8)? {
        1
    } else if active(0xe1b0)? && !incoming(0xe1b0)? {
        2
    } else if active(0xe1b4)? && !incoming(0xe1b4)? {
        3
    } else if word(current, GAME_FLOW)? == LOADING_FLOW
        && word(previous, GAME_FLOW)? != LOADING_FLOW
    {
        4
    } else {
        0
    };
    let mut outputs = vec![quantity("request", request)];
    if integrated && setup {
        outputs.extend([
            bytes(
                "pages.ship",
                vram(current, 0..0x300)?
                    .iter()
                    .map(|word| word.to_le_bytes().first().copied().unwrap_or_default())
                    .collect(),
            ),
            bytes(
                "pages.ship-tiles",
                vram(current, 0..0x4000)?
                    .iter()
                    .map(|word| word.to_le_bytes().last().copied().unwrap_or_default())
                    .collect(),
            ),
            packed("pages.flat", vram(current, 0x5c00..0x6000)?.iter().copied()),
            packed(
                "pages.tiles",
                vram(current, 0x6000..0x8000)?.iter().copied(),
            ),
            packed("palette.setup", previous.cgram.iter().copied()),
        ]);
    }
    if integrated && !held {
        let low = current
            .oam
            .get(..512)
            .ok_or_else(|| data_failure(Some(current.index), "flight record OAM"))?;
        let high = current
            .oam
            .get(512..544)
            .ok_or_else(|| data_failure(Some(current.index), "flight record high OAM"))?;
        outputs.push(bytes(
            "oam",
            oam_bytes(low.as_chunks::<4>().0.iter().copied(), high),
        ));
        outputs.push(packed("palette", current.cgram.iter().copied()));
    }
    Ok(outputs)
}

fn has_effect(record: &FrameRecord, id: u16) -> Result<bool, Outcome> {
    for address in (0x1e7d..0x1e8d).step_by(2) {
        if word(record, address)? == id {
            return Ok(true);
        }
    }
    Ok(false)
}

fn landing_seed(lock: &crate::lock::Lock, sources: &Sources) -> Result<FrameRecord, Outcome> {
    let row = lock
        .row(&crate::lock::RowId("any-zebes-arrival".to_owned()))
        .ok_or_else(|| data_failure(None, "receiving Landing Site row"))?;
    let cache = crate::cache::Cache::open(&sources.cache, row)
        .map_err(|_| data_failure(None, "receiving Landing Site cache pins"))?;
    record(&cache, 0)
}

fn landing_join(record: &FrameRecord) -> Result<Observation, Outcome> {
    let values = words_at(
        record,
        [FINISHED, GAME_FLOW, ENERGY, ENERGY_CAPACITY, DESTINATION],
    )?;
    let plane = words_at(
        record,
        [
            0x198d, 0x198f, 0x00b1, 0x00b3, 0x0078, 0x007a, 0x007c, 0x007e,
        ],
    )?;
    Ok(packed(
        "landing.join",
        values
            .into_iter()
            .chain([u16::from(byte(record, 0x0051)?)])
            .chain(plane),
    ))
}

fn words_at<const N: usize>(record: &FrameRecord, at: [usize; N]) -> Result<Vec<u16>, Outcome> {
    at.into_iter().map(|index| word(record, index)).collect()
}

fn vram(record: &FrameRecord, range: core::ops::Range<usize>) -> Result<&[u16], Outcome> {
    record
        .vram
        .get(range)
        .ok_or_else(|| data_failure(Some(record.index), "flight recorded drawing pages"))
}

/// Advance the indexed recipe from the same station hand-off for ordinary played replay.
pub(super) fn play_cut(
    production: &mut Production,
    previous: &FrameRecord,
    current: &FrameRecord,
    cut: u64,
) -> Result<(), Outcome> {
    let ProductionOutputs::Zebes(output) = advance(production, cadence(current), cut)? else {
        return Err(data_failure(Some(cut), "flight production scope"));
    };
    super::compared(
        cut,
        &expected(previous, current, true)?.fields,
        &actual(production, &output, true)?,
    )
}
