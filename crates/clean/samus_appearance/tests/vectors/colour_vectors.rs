//! The ordinary dry palette histories and their refusal boundaries, each a frame with a palette
//! opportunity played through the one update. The shared current and target palettes are the
//! caller's, saved and restored with her appearance State.

use core::num::Wrapping;
use std::collections::BTreeMap;

use samus_appearance::{
    AppearanceState, CannonDelay, ChargeGlowPosition, ColourEffect, ColourSequences,
    DryColourFacts, DryColourReset, Figure, Frame, FrameOutputs, GrappleActivity, LiquidLevel,
    PaletteOpportunity, RoomColours, Rules, ScrewAttack, Suit, VisorDelay, VisorStep, update,
};
use serde::{Deserialize, Serialize};
use types::{Colour, PALETTE_SLOTS, Pair, Palette, Palettes, px};

use super::{
    ACTIONS, Library, VectorContent, bare_frame, behaviour, figure, named, read_file, state,
};

/// The levels of each colour channel, 0 to 31: the invented fixtures' slot pattern.
const LEVELS: u16 = 32;

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
            .flat_map(|history| history.entries.iter().map(String::as_str))
    }

    /// An invented bank: slot `i` holds red at the fixture's level, green `i % 32` and blue
    /// `i / 32`, so every slot and every fixture differs.
    fn palette(&self, name: &str) -> Palette {
        let red = *self
            .invented_palettes
            .get(name)
            .unwrap_or_else(|| panic!("no invented bank {name}"));
        assert!(red < LEVELS, "{name}: red within a channel");
        Palette(std::array::from_fn(|slot| {
            let slot = u16::try_from(slot).expect("a palette slot");
            Colour::from_channels([red, slot % LEVELS, slot / LEVELS])
        }))
    }

    /// The frame's palette opportunity for `history`: the shared observations, then the
    /// history's own `observed` table, then `replacement`, each replacing keys already present.
    /// A boundary case's `figure`, given apart, replaces the history's figure.
    fn opportunity(
        &self,
        history: &History,
        figure_text: Option<&str>,
        replacement: &toml::Table,
    ) -> PaletteOpportunity {
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
        input.opportunity(
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
    fn from(given: Sequences) -> Self {
        Self {
            visor_delay: VisorDelay(given.visor_delay),
            visor_step: VisorStep(given.visor_step),
            charge_glow_position: ChargeGlowPosition(given.charge_glow_position),
        }
    }
}

/// A change another producer makes to one bank in the frame.
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
    outcome: Outcome,
    applications: usize,
    replacements: Vec<toml::Table>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
enum Outcome {
    Unsupported,
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

/// An owner's fact the file gives as unavailable.
const UNAVAILABLE: &str = "Unavailable";

impl Observations {
    /// The opportunity these observations make, with `figure` where `position` says: one whose
    /// owners couldn't supply a fact is unrecognized.
    fn opportunity(&self, figure: Figure, [x, y]: [u16; 2]) -> PaletteOpportunity {
        assert_eq!(
            figure.pose.action,
            named(ACTIONS, &self.action),
            "the observed action agrees with the figure"
        );
        let (room, grapple) = (&self.room_colours, &self.grapple);
        if room == UNAVAILABLE || grapple == UNAVAILABLE {
            return PaletteOpportunity::Unrecognized;
        }
        PaletteOpportunity::Observed(DryColourFacts {
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

/// What a caller keeps between frames for her colours: her whole appearance State and the
/// shared current and target palettes, saved and restored together.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct Kept {
    appearance: AppearanceState,
    palettes: Palettes,
}

/// `kept` saved whole with serde, and restored fresh from the saved text alone, the same.
fn restored(kept: &Kept) -> Kept {
    let text = toml::to_string(kept).expect("the State and palettes save");
    let fresh: Kept = toml::from_str(&text).expect("the State and palettes restore");
    assert_eq!(toml::to_string(&fresh).expect("restored saves"), text);
    fresh
}

/// Where a palette history starts: its sequences, its banks, and an animation and cannon its
/// frame leaves alone.
fn start(vectors: &ColourVectors, history: &History) -> Kept {
    let [current, target] = &history.palettes;
    restored(&Kept {
        appearance: AppearanceState {
            animation: state(2, 3, 1, 1),
            cannon: CannonDelay::INITIAL,
            colours: history.before.into(),
        },
        palettes: Palettes {
            current: vectors.palette(current),
            target: vectors.palette(target),
        },
    })
}

/// Plays the frame of `opportunity` from `kept`: the frame's other producers write the banks
/// they write, and the opportunity readies or leaves her sequences. Gives what the caller then
/// keeps, restored, and the frame's outputs.
fn play_frame(
    mut kept: Kept,
    opportunity: PaletteOpportunity,
    updates: &[PaletteUpdate],
    vectors: &ColourVectors,
    library: &Library,
) -> (Kept, FrameOutputs) {
    for update in updates {
        let colours = vectors.palette(&update.colours);
        match update.bank {
            Bank::Current => kept.palettes.current = colours,
            Bank::Target => kept.palettes.target = colours,
        }
    }
    let frame = Frame {
        palette: opportunity,
        ..bare_frame()
    };
    let content = VectorContent {
        clips: Vec::new(),
        paces: &library.paces,
    };
    let (appearance, outputs) = update(kept.appearance, &frame, &content, &Rules::VANILLA);
    let after = restored(&Kept {
        appearance,
        palettes: kept.palettes,
    });
    (after, outputs)
}

/// The frame's applications of the reset: 1 where it applied, 0 where it was refused.
fn applications(outputs: &FrameOutputs) -> usize {
    match outputs.dry_colours {
        DryColourReset::Applied => 1,
        DryColourReset::Unsupported | DryColourReset::NoOpportunity => 0,
    }
}

/// The frame asks for nothing and leaves her animation and cannon as they were.
fn check_quiet(before: &Kept, after: &Kept, outputs: &FrameOutputs, id: &str) {
    assert!(outputs.sounds.is_empty(), "{id}: no requests");
    assert_eq!(outputs.contact, None, "{id}: no graphics");
    assert_eq!(outputs.ahead, None, "{id}: no animation update");
    assert_eq!(
        after.appearance.animation, before.appearance.animation,
        "{id}: animation"
    );
    assert_eq!(
        after.appearance.cannon, before.appearance.cannon,
        "{id}: cannon"
    );
}

fn run_history(vectors: &ColourVectors, history: &History, library: &Library) {
    let id = &history.id;
    let before = start(vectors, history);
    let mut expected_palettes = before.palettes.clone();
    for update in &history.palette_updates {
        let colours = vectors.palette(&update.colours);
        match update.bank {
            Bank::Current => expected_palettes.current = colours,
            Bank::Target => expected_palettes.target = colours,
        }
    }
    let opportunity = vectors.opportunity(history, None, &toml::Table::new());
    let (after, outputs) = play_frame(
        before.clone(),
        opportunity,
        &history.palette_updates,
        vectors,
        library,
    );
    assert_eq!(
        after.appearance.colours,
        history.after.into(),
        "{id}: the three quantities"
    );
    assert_eq!(after.palettes, expected_palettes, "{id}: both whole banks");
    assert_eq!(after.palettes.current.0.len(), PALETTE_SLOTS, "{id}");
    assert_eq!(applications(&outputs), history.applications, "{id}");
    check_quiet(&before, &after, &outputs, id);
}

#[test]
#[expect(
    clippy::print_stdout,
    reason = "report the number of supplied palette histories"
)]
fn every_ordinary_palette_history() {
    let (file, library) = read_file();
    let vectors = &file.ordinary_dry_colours;
    for history in &vectors.history {
        assert_ne!(history.source, "", "{}", history.id);
        for entry in &history.entries {
            behaviour(entry);
        }
        run_history(vectors, history, &library);
    }
    println!(
        "{} ordinary dry colour histories passed",
        vectors.history.len()
    );
}

#[test]
#[expect(
    clippy::print_stdout,
    reason = "report the number of independent palette refusals"
)]
fn every_refusal_boundary_leaves_everything() {
    let (file, library) = read_file();
    let vectors = &file.ordinary_dry_colours;
    assert!(!vectors.boundary.is_empty(), "boundary sets are supplied");
    let mut refusals = 0_usize;
    for boundary in &vectors.boundary {
        assert_ne!(boundary.source, "", "{}", boundary.id);
        assert_eq!(boundary.outcome, Outcome::Unsupported, "{}", boundary.id);
        let history = vectors
            .history
            .iter()
            .find(|history| history.id == boundary.based_on)
            .unwrap_or_else(|| panic!("{}: no history {}", boundary.id, boundary.based_on));
        for replacement in &boundary.replacements {
            let case = format!("{}: {replacement:?}", boundary.id);
            let (figure_text, observations) = boundary_case(replacement);
            let opportunity = vectors.opportunity(history, figure_text.as_deref(), &observations);
            let before = start(vectors, history);
            let (after, outputs) = play_frame(before.clone(), opportunity, &[], vectors, &library);
            assert_eq!(outputs.dry_colours, DryColourReset::Unsupported, "{case}");
            assert_eq!(applications(&outputs), boundary.applications, "{case}");
            assert_eq!(
                after, before,
                "{case}: the State and both banks as they were"
            );
            check_quiet(&before, &after, &outputs, &case);
            refusals = refusals.saturating_add(1);
        }
    }
    println!(
        "{refusals} independent ordinary colour refusals passed in {} sets",
        vectors.boundary.len()
    );
}

#[test]
fn palettes_restore_only_whole_and_representable() {
    assert!(Colour::new(0x8000).is_none());
    assert!(Palette::try_from(vec![Colour::BLACK; PALETTE_SLOTS - 1]).is_err());
    assert!(Palette::try_from(vec![Colour::BLACK; PALETTE_SLOTS + 1]).is_err());
    let (file, _) = read_file();
    let vectors = &file.ordinary_dry_colours;
    let kept = start(vectors, vectors.history.first().expect("a history"));
    let text = toml::to_string(&kept).expect("the State and palettes save");
    assert!(text.contains("current = [1,"), "{text}");
    assert!(toml::from_str::<Kept>(&text.replace("current = [1,", "current = [32768,")).is_err());
}
