//! The ordinary dry palette histories and their refusal boundaries, each a frame with a palette
//! opportunity played through the one update. The shared current and target palettes are the
//! caller's, saved and restored with her appearance State.

use std::collections::BTreeMap;

use samus_appearance::{
    AppearanceState, CannonDelay, ChargeGlowPosition, ColourEffect, ColourSequences,
    DryColourOutcome, DryColourReset, Frame, FrameOutputs, LiquidLevel, PaletteObservations,
    PaletteOpportunity, Rules, Suit, Unrecognized, VisorDelay, VisorStep, update,
};
use serde::{Deserialize, Serialize};
use types::{Action, Colour, PALETTE_SLOTS, Palette, Palettes, Set};

use super::{Library, VectorContent, VectorInputs, animation, behaviour, frame, named, read_file};

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
            .unwrap_or_else(|| panic!("expected an invented bank {name}"));
        assert!(
            red < LEVELS,
            "{name}: expected red within a channel's 0 to 31"
        );
        Palette(std::array::from_fn(|slot| {
            let slot = u16::try_from(slot).expect("expected a palette slot below 256");
            Colour::from_channels([red, slot % LEVELS, slot / LEVELS])
        }))
    }

    /// The frame's palette opportunity for `history`: the shared observations, then the
    /// history's own `observed` table, then `replacement`'s observations, each replacing keys
    /// already present.
    fn opportunity(&self, history: &History, replacement: &toml::Table) -> PaletteOpportunity {
        let mut input = self.observations.clone();
        for (key, value) in history.observed.iter().chain(replacement) {
            assert!(
                input.insert(key.clone(), value.clone()).is_some(),
                "{}: expected a replacement of a shared observation, found {key}",
                history.id
            );
        }
        let input: Observations = toml::Value::Table(input)
            .try_into()
            .expect("expected the observations the file's header names");
        input.opportunity()
    }
}

/// The frame's inputs a history reads: her pose as the frame ends, which is her pose as it
/// starts too, since the frame has no movement, and the items switched on.
fn frame_inputs(pose_after: &str, items: &[String]) -> VectorInputs {
    VectorInputs {
        pose: pose_after.to_owned(),
        items: items.to_vec(),
        ..VectorInputs::default()
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct History {
    id: String,
    source: String,
    entries: Vec<String>,
    pose_after: String,
    #[serde(default)]
    items: Vec<String>,
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
    visor_step: u8,
    charge_glow_position: u8,
}

impl From<Sequences> for ColourSequences {
    fn from(given: Sequences) -> Self {
        Self {
            visor_delay: VisorDelay::new(given.visor_delay),
            visor_step: VisorStep::new(given.visor_step),
            charge_glow_position: ChargeGlowPosition::new(given.charge_glow_position),
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

/// A boundary's outcome, as the file names it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
enum Outcome {
    /// The reset refused, an ordinary result of the frame.
    Refused,
    /// A fact the rule can't read.
    Unrecognized,
}

impl Outcome {
    /// The model's outcome of a frame's palette opportunity for this one.
    const fn expected(self) -> DryColourOutcome {
        match self {
            Self::Refused => Ok(DryColourReset::Refused),
            Self::Unrecognized => Err(Unrecognized),
        }
    }
}

/// A boundary case: the frame's inputs, with the history's pose after and items where the case
/// doesn't replace them, and its changed shared observations.
fn boundary_case(history: &History, replacement: &toml::Table) -> (VectorInputs, toml::Table) {
    let mut observations = replacement.clone();
    let pose_after = observations.remove("pose_after").map_or_else(
        || history.pose_after.clone(),
        |text| {
            text.as_str()
                .expect("expected a boundary pose written as text")
                .to_owned()
        },
    );
    let items = observations
        .remove("items")
        .map_or_else(
            || Ok(history.items.clone()),
            toml::Value::try_into::<Vec<String>>,
        )
        .expect("expected boundary items written as names");
    (frame_inputs(&pose_after, &items), observations)
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
#[expect(
    clippy::struct_excessive_bools,
    reason = "the vector format supplies independent owner facts as flags"
)]
struct Observations {
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
    /// The opportunity these observations make: one whose owners couldn't supply a fact has a
    /// fact the rule can't read.
    fn opportunity(&self) -> PaletteOpportunity {
        let (room, grapple) = (&self.room_colours, &self.grapple);
        if room == UNAVAILABLE || grapple == UNAVAILABLE {
            return PaletteOpportunity::Unreadable;
        }
        PaletteOpportunity::Observed(PaletteObservations {
            suit: self.suit,
            room_colours: named(room),
            levels: [
                (LiquidLevel::Water, self.water_present),
                (LiquidLevel::Lava, self.lava_present),
            ]
            .into_iter()
            .collect(),
            grapple: named(grapple),
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
    let text = toml::to_string(kept).expect("expected the State and palettes to save");
    let fresh: Kept = toml::from_str(&text).expect("expected the State and palettes to restore");
    assert_eq!(
        toml::to_string(&fresh).expect("expected the restored State and palettes to save"),
        text,
        "expected them restored as saved"
    );
    fresh
}

/// Where a palette history starts: its sequences, its banks, and an animation and cannon its
/// frame leaves alone.
fn start(vectors: &ColourVectors, history: &History) -> Kept {
    let [current, target] = &history.palettes;
    restored(&Kept {
        appearance: AppearanceState {
            animation: animation(2, 3, 1, 1),
            kept_action: Action::WallJumping,
            cannon: CannonDelay::INITIAL,
            colours: history.before.into(),
        },
        palettes: Palettes {
            current: vectors.palette(current),
            target: vectors.palette(target),
        },
    })
}

/// Plays the frame of `opportunity`, with `inputs`, from `kept`: the frame's other producers
/// write the banks they write, and the opportunity readies or leaves her sequences. Gives what
/// the caller then keeps, restored, and the frame's outputs.
fn play_frame(
    mut kept: Kept,
    inputs: &VectorInputs,
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
        ..frame(inputs, Set::default())
    };
    let content = VectorContent {
        clips: Vec::new(),
        paces: &library.paces,
    };
    let (appearance, outputs) = update(kept.appearance, &frame, &content, &Rules::VANILLA)
        .expect("expected a frame without movement to read no clip");
    let after = restored(&Kept {
        appearance,
        palettes: kept.palettes,
    });
    (after, outputs)
}

/// The frame's applications of the reset: 1 where it applied, 0 where it was refused or a fact
/// was unrecognized.
fn applications(outputs: &FrameOutputs) -> usize {
    match outputs.dry_colours {
        Some(Ok(DryColourReset::Applied)) => 1,
        Some(Ok(DryColourReset::Refused) | Err(Unrecognized)) | None => 0,
    }
}

/// The frame asks for nothing and leaves her animation and cannon as they were.
fn check_quiet(before: &Kept, after: &Kept, outputs: &FrameOutputs, id: &str) {
    assert!(outputs.sounds.is_empty(), "{id}: expected no requests");
    assert_eq!(outputs.contact, None, "{id}: expected no graphics");
    assert_eq!(
        after.appearance.animation, before.appearance.animation,
        "{id}: expected her animation unchanged"
    );
    assert_eq!(
        after.appearance.cannon, before.appearance.cannon,
        "{id}: expected her cannon delay unchanged"
    );
    assert_eq!(
        after.appearance.kept_action, before.appearance.kept_action,
        "{id}: expected her kept action unchanged"
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
    let inputs = frame_inputs(&history.pose_after, &history.items);
    let opportunity = vectors.opportunity(history, &toml::Table::new());
    let (after, outputs) = play_frame(
        before.clone(),
        &inputs,
        opportunity,
        &history.palette_updates,
        vectors,
        library,
    );
    assert_eq!(
        after.appearance.colours,
        history.after.into(),
        "{id}: expected the three quantities `after` gives"
    );
    assert_eq!(
        after.palettes, expected_palettes,
        "{id}: expected both whole banks as the frame's producers leave them"
    );
    assert_eq!(
        after.palettes.current.0.len(),
        PALETTE_SLOTS,
        "{id}: expected 256 slots"
    );
    assert_eq!(
        applications(&outputs),
        history.applications,
        "{id}: expected the applications `applications` gives"
    );
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
        assert_ne!(history.source, "", "{}: expected a source", history.id);
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
    assert!(
        !vectors.boundary.is_empty(),
        "expected boundary sets in the file"
    );
    let mut cases = 0_usize;
    for boundary in &vectors.boundary {
        assert_ne!(boundary.source, "", "{}: expected a source", boundary.id);
        let history = vectors
            .history
            .iter()
            .find(|history| history.id == boundary.based_on)
            .unwrap_or_else(|| panic!("{}: expected a history {}", boundary.id, boundary.based_on));
        for replacement in &boundary.replacements {
            let case = format!("{}: {replacement:?}", boundary.id);
            let (inputs, observations) = boundary_case(history, replacement);
            let opportunity = vectors.opportunity(history, &observations);
            let before = start(vectors, history);
            let (after, outputs) =
                play_frame(before.clone(), &inputs, opportunity, &[], vectors, &library);
            assert_eq!(
                outputs.dry_colours,
                Some(boundary.outcome.expected()),
                "{case}: expected the outcome `outcome` gives"
            );
            assert_eq!(
                applications(&outputs),
                boundary.applications,
                "{case}: expected the applications `applications` gives"
            );
            assert_eq!(
                after, before,
                "{case}: expected the State and both banks as they were"
            );
            check_quiet(&before, &after, &outputs, &case);
            cases = cases.saturating_add(1);
        }
    }
    println!(
        "{cases} independent ordinary colour boundary cases passed in {} sets",
        vectors.boundary.len()
    );
}

#[test]
fn palettes_restore_only_whole_and_representable() {
    assert!(
        Colour::new(0x8000).is_none(),
        "expected a sixteenth bit refused"
    );
    assert!(
        Palette::try_from(vec![Colour::BLACK; PALETTE_SLOTS - 1]).is_err(),
        "expected 255 colours refused"
    );
    assert!(
        Palette::try_from(vec![Colour::BLACK; PALETTE_SLOTS + 1]).is_err(),
        "expected 257 colours refused"
    );
    let (file, _) = read_file();
    let vectors = &file.ordinary_dry_colours;
    let kept = start(
        vectors,
        vectors.history.first().expect("expected a history"),
    );
    let text = toml::to_string(&kept).expect("expected the State and palettes to save");
    assert!(
        text.contains("current = [1,"),
        "expected the current bank saved: {text}"
    );
    assert!(
        toml::from_str::<Kept>(&text.replace("current = [1,", "current = [32768,")).is_err(),
        "expected an unrepresentable restored colour refused"
    );
}

/// Her colour sequences save as their numbers and restore only within their types' ranges.
#[test]
fn colour_sequences_restore_only_in_range() {
    let sequences = ColourSequences {
        visor_delay: VisorDelay::new(u8::MAX),
        visor_step: VisorStep::new(u8::MAX),
        charge_glow_position: ChargeGlowPosition::new(u8::MAX),
    };
    let text = toml::to_string(&sequences).expect("expected the sequences to save");
    assert_eq!(
        toml::from_str::<ColourSequences>(&text).expect("expected the sequences to restore"),
        sequences,
        "expected the sequences restored as saved"
    );
    for key in ["visor_delay", "visor_step", "charge_glow_position"] {
        let beyond = text.replacen(&format!("{key} = 255"), &format!("{key} = 256"), 1);
        assert_ne!(beyond, text, "expected {key} saved as 255: {text}");
        assert!(
            toml::from_str::<ColourSequences>(&beyond).is_err(),
            "expected {key} of 256 refused"
        );
    }
}
