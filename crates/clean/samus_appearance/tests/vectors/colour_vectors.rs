//! Exercise the same palette authority across actual observation, completion and fresh restore.
use core::num::Wrapping;
use std::collections::BTreeMap;

use samus_appearance::{
    CannonDelay, ChargeGlowPosition, ColourEffect, ColourSequences, DryColourFacts, DryColourWork,
    Figure, GrappleActivity, LiquidLevel, Look, Palette, Palettes, RoomColours, ScrewAttack, Suit,
    SuspendedAppearance, UnsupportedDryColours, VisorDelay, VisorStep,
};
use serde::{Deserialize, Serialize};
use types::{Colour, Pair, px};

use super::{ACTIONS, behaviour, figure, named, read_file, suspended_vectors::make_work};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ColourVectors {
    observations: toml::Table,
    invented_palettes: BTreeMap<String, u16>,
    history: Vec<History>,
    boundary: Vec<Boundary>,
}

impl ColourVectors {
    pub(super) fn entries(&self) -> impl Iterator<Item = &str> {
        self.history
            .iter()
            .flat_map(|h| h.entries.iter().map(String::as_str))
    }

    fn palette(&self, name: &str) -> Palette {
        let red = *self
            .invented_palettes
            .get(name)
            .expect("named invented bank");
        assert!(red < 32);
        Palette(std::array::from_fn(|i| {
            let i = u16::try_from(i).expect("palette slot");
            Colour::from_channels([red, i % 32, i / 32])
        }))
    }

    /// The current facts of a history: the shared observations, then the history's own
    /// `observed` table, then `replacement`, each replacing keys already present. A boundary
    /// case's `figure`, given separately, replaces the history's figure.
    fn facts(
        &self,
        history: &History,
        figure_text: Option<&str>,
        replacement: &toml::Table,
    ) -> Option<DryColourFacts> {
        let mut input = self.observations.clone();
        for (key, value) in history.observed.iter().chain(replacement) {
            assert!(
                input.insert(key.clone(), value.clone()).is_some(),
                "{}: unknown replacement {key}",
                history.id
            );
        }
        let input: Observations = toml::Value::Table(input)
            .try_into()
            .expect("known observations");
        input.facts(
            figure(figure_text.unwrap_or(&history.figure)),
            history.position,
        )
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct History {
    id: String,
    source: String,
    entries: Vec<String>,
    figure: String,
    position: [u16; 2],
    #[serde(default)]
    observed: toml::Table,
    before: Sequences,
    after: Sequences,
    palettes: [String; 2],
    applications: usize,
    #[serde(default)]
    restore: Vec<Cut>,
    #[serde(default)]
    later_change: toml::Table,
    #[serde(default)]
    palette_updates: Vec<PaletteUpdate>,
}

#[derive(Clone, Copy, Deserialize)]
#[serde(deny_unknown_fields)]
struct Sequences {
    visor_delay: u8,
    visor_step: usize,
    charge_glow_position: usize,
}

impl From<Sequences> for ColourSequences {
    fn from(v: Sequences) -> Self {
        Self {
            visor_delay: VisorDelay(v.visor_delay),
            visor_step: VisorStep(v.visor_step),
            charge_glow_position: ChargeGlowPosition(v.charge_glow_position),
        }
    }
}

#[derive(Clone, Copy, Deserialize, PartialEq, Eq)]
enum Cut {
    Unobserved,
    Pending,
    Completed,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PaletteUpdate {
    bank: Bank,
    colours: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
enum Bank {
    Current,
    Target,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Boundary {
    id: String,
    source: String,
    based_on: String,
    outcome: String,
    applications: usize,
    replacements: Vec<toml::Table>,
}

/// A boundary case: its figure, if it names one, and its changed shared observations.
fn boundary_case(replacement: &toml::Table) -> (Option<String>, toml::Table) {
    let mut observations = replacement.clone();
    let figure = observations.remove("figure").map(|text| {
        text.as_str()
            .expect("a boundary figure is written as text")
            .to_owned()
    });
    (figure, observations)
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
#[expect(
    clippy::struct_excessive_bools,
    reason = "the vector format supplies independent owner facts as flags"
)]
struct Observations {
    action: String,
    screw_attack: bool,
    suit: Suit,
    room_colours: String,
    water_present: bool,
    lava_present: bool,
    grapple: String,
    stored_glow: bool,
    weapon_charging: bool,
    override_colours: bool,
    special_colours: bool,
    speed_glow: bool,
    hit_colour_pulse: bool,
}

impl Observations {
    fn facts(&self, figure: Figure, [x, y]: [u16; 2]) -> Option<DryColourFacts> {
        assert_eq!(
            figure.pose.action,
            named(ACTIONS, &self.action),
            "the observed action agrees with the figure"
        );
        let room = &self.room_colours;
        let grapple = &self.grapple;
        if room == "Unavailable" || grapple == "Unavailable" {
            return None;
        }
        Some(DryColourFacts {
            figure,
            position: Pair {
                x: px(Wrapping(x)),
                y: px(Wrapping(y)),
            },
            screw_attack: if self.screw_attack {
                ScrewAttack::On
            } else {
                ScrewAttack::Off
            },
            suit: self.suit,
            room_colours: named(
                &[
                    RoomColours::Standard,
                    RoomColours::RedBackground,
                    RoomColours::OrangeBackground,
                ],
                room,
            ),
            levels: [
                (LiquidLevel::Water, self.water_present),
                (LiquidLevel::Magma, self.lava_present),
            ]
            .into_iter()
            .collect(),
            grapple: named(&[GrappleActivity::Dormant, GrappleActivity::Busy], grapple),
            effects: [
                (ColourEffect::StoredGlow, self.stored_glow),
                (ColourEffect::WeaponCharging, self.weapon_charging),
                (ColourEffect::OverrideColours, self.override_colours),
                (ColourEffect::SpecialColours, self.special_colours),
                (ColourEffect::SpeedGlow, self.speed_glow),
                (ColourEffect::HitColourPulse, self.hit_colour_pulse),
            ]
            .into_iter()
            .collect(),
        })
    }
}

/// A caller's complete save, including unrelated work with its retained receipts and the
/// authoritative banks. The opportunity identity is the history's caller-owned ID.
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
struct Run {
    opportunity: String,
    look: Look,
    appearance: SuspendedAppearance,
    sequences: ColourSequences,
    palettes: Palettes,
    current: Option<DryColourFacts>,
    work: DryColourWork,
    palette_updates_consumed: usize,
    applications: usize,
}

fn saved(run: &Run) -> String {
    toml::to_string(run).expect("complete palette opportunity serializes")
}

fn fresh(run: Run) -> Run {
    let text = saved(&run);
    drop(run);
    let restored = toml::from_str(&text).expect("fresh palette and look owners deserialize");
    assert_eq!(saved(&restored), text);
    restored
}

fn checkpoint(run: Run, cuts: &[Cut], cut: Cut) -> Run {
    if cuts.contains(&cut) { fresh(run) } else { run }
}

fn start(file: &super::VectorFile, h: &History) -> Run {
    let vectors = &file.ordinary_dry_colours;
    let mut appearance = make_work(file, "Work", "C-contact-and-start");
    appearance.finish();
    let [current, target] = &h.palettes;
    let facts = vectors
        .facts(h, None, &toml::Table::new())
        .expect("known ordinary facts");
    assert_eq!(facts.figure, figure(&h.figure));
    Run {
        opportunity: h.id.clone(),
        look: Look {
            animation: appearance.state,
            cannon: CannonDelay::INITIAL,
        },
        appearance,
        sequences: h.before.into(),
        palettes: Palettes {
            current: vectors.palette(current),
            target: vectors.palette(target),
        },
        current: Some(facts),
        work: DryColourWork::UNOBSERVED,
        palette_updates_consumed: 0,
        applications: 0,
    }
}

fn update_banks(run: &mut Run, vectors: &ColourVectors, h: &History, expected: &mut Palettes) {
    while let Some(update) = h.palette_updates.get(run.palette_updates_consumed) {
        let colours = vectors.palette(&update.colours);
        match update.bank {
            Bank::Current => {
                run.palettes.current = colours.clone();
                expected.current = colours;
            }
            Bank::Target => {
                run.palettes.target = colours.clone();
                expected.target = colours;
            }
        }
        run.palette_updates_consumed = run.palette_updates_consumed.saturating_add(1);
        assert_eq!(
            &run.palettes, expected,
            "{}: both full banks after each write",
            h.id
        );
        assert_eq!(run.sequences, h.before.into());
        assert_eq!(run.applications, 0);
    }
}

fn complete(run: &mut Run) {
    if run.work.complete(&mut run.sequences) {
        run.applications = run.applications.saturating_add(1);
    }
}

fn run_history(file: &super::VectorFile, h: &History, cuts: &[Cut]) -> Run {
    let vectors = &file.ordinary_dry_colours;
    let mut run = start(file, h);
    let mut expected_palettes = run.palettes.clone();
    let look = run.look;
    let appearance = run.appearance.clone();
    assert!(!run.work.complete(&mut run.sequences));
    run = checkpoint(run, cuts, Cut::Unobserved);
    let actual = run.current;
    assert_eq!(run.work.observe(run.current), Ok(true));
    assert_eq!(run.sequences, h.before.into());
    assert_eq!(run.palettes, expected_palettes);
    assert_eq!(run.applications, 0);
    assert_eq!(run.work.pending().copied(), actual);
    // A neighbour changes while the reset is pending, the screw attack among them: the work
    // keeps the facts it admitted and is not re-decided from the newer ones.
    run.current = vectors.facts(h, None, &h.later_change);
    if !h.later_change.is_empty() {
        assert_ne!(run.current, actual, "{}: later_change changes a fact", h.id);
    }
    update_banks(&mut run, vectors, h, &mut expected_palettes);
    run = checkpoint(run, cuts, Cut::Pending);
    assert_eq!(run.work.observe(run.current), Ok(false));
    assert_eq!(run.work.pending().copied(), actual);
    assert_eq!(run.sequences, h.before.into());
    assert_eq!(run.palettes, expected_palettes);
    assert_eq!(run.applications, 0);
    let later = run.current;
    complete(&mut run);
    assert_eq!(run.applications, h.applications);
    run = checkpoint(run, cuts, Cut::Completed);
    assert_eq!(run.work.observe(actual), Ok(false));
    let mut later_sequences = h.before.into();
    assert!(!run.work.complete(&mut later_sequences));
    assert_eq!(
        later_sequences,
        h.before.into(),
        "completed work cannot write again"
    );
    assert_eq!(run.sequences, h.after.into());
    assert_eq!(run.palettes, expected_palettes);
    assert_eq!(run.applications, h.applications);
    assert_eq!(run.look, look);
    assert_eq!(run.appearance, appearance);
    assert_eq!(
        run.current, later,
        "selected figure and live facts remain available"
    );
    run
}

#[test]
#[expect(
    clippy::print_stdout,
    reason = "report the number of supplied palette histories"
)]
fn every_ordinary_palette_history() {
    let (file, _, _) = read_file();
    for h in &file.ordinary_dry_colours.history {
        assert_ne!(h.source, "");
        for entry in &h.entries {
            behaviour(entry);
        }
        let uninterrupted = run_history(&file, h, &[]);
        assert_eq!(
            run_history(&file, h, &h.restore),
            uninterrupted,
            "{}: complete restored result",
            h.id
        );
    }
    println!(
        "{} ordinary dry colour histories passed",
        file.ordinary_dry_colours.history.len()
    );
}

#[test]
#[expect(
    clippy::print_stdout,
    reason = "report the number of independent palette refusals"
)]
fn changed_current_facts_refuse_without_progress() {
    let (file, _, _) = read_file();
    let vectors = &file.ordinary_dry_colours;
    assert!(!vectors.boundary.is_empty(), "boundary sets are supplied");
    let mut refusals = 0_usize;
    for boundary in &vectors.boundary {
        assert_ne!(boundary.source, "");
        assert_eq!(boundary.outcome, "Unsupported");
        let h = vectors
            .history
            .iter()
            .find(|h| h.id == boundary.based_on)
            .expect("base history");
        let admitted = run_history(&file, h, &[]);
        assert_eq!(admitted.applications, h.applications);
        for replacement in &boundary.replacements {
            let (figure_text, observations) = boundary_case(replacement);
            let mut run = fresh(start(&file, h));
            run.current = vectors.facts(h, figure_text.as_deref(), &observations);
            let before = saved(&run);
            assert_eq!(
                run.work.observe(run.current),
                Err(UnsupportedDryColours),
                "{}: {replacement:?}",
                boundary.id
            );
            complete(&mut run);
            assert_eq!(run.applications, boundary.applications);
            assert_eq!(saved(&run), before, "refusal must not consume progress");
            assert_eq!(run.work, DryColourWork::UNOBSERVED);
            refusals = refusals.saturating_add(1);
        }
    }
    println!(
        "{refusals} independent ordinary colour boundary refusals passed in {} sets",
        vectors.boundary.len()
    );
}

#[test]
fn palette_import_rejects_unrepresentable_colours_and_incomplete_banks() {
    assert!(Colour::new(0x8000).is_none());
    assert!(Palette::try_from(vec![Colour::BLACK; 255]).is_err());
    assert!(Palette::try_from(vec![Colour::BLACK; 257]).is_err());
    let (file, _, _) = read_file();
    let run = start(
        &file,
        file.ordinary_dry_colours.history.first().expect("history"),
    );
    let text = saved(&run);
    assert!(text.contains("current = [1,"));
    assert!(toml::from_str::<Run>(&text.replace("current = [1,", "current = [32768,")).is_err());
}
