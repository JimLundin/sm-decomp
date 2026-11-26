//! The cannon readiness histories, played a frame at a time through the one update, with her
//! appearance State saved and restored between frames.

use samus_appearance::{
    AppearanceState, CannonDelay, ControlEligibility, Frame, FramePart, HurtCount,
    ItemControlAnswer, ItemControls, Rules, SelectedItem,
};
use serde::Deserialize;
use types::{Action, Set, set_of};

use super::{
    Library, UNTOUCHED_COLOURS, VectorContent, VectorFile, VectorInputs, animation, behaviour,
    frame, kept, moving_content, play, read_file,
};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct CannonHistory {
    id: String,
    source: String,
    pub(super) entries: Vec<String>,
    initial_delay: u8,
    steps: Vec<HistoryStep>,
}

/// What a step of a history is, as the file's comment names it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
enum Operation {
    /// A frame whose item controls run.
    Controls,
    /// A frame whose item controls don't run.
    Skip,
    /// A frame in which Samus is initialized.
    Initialize,
    /// A selection change outside item controls.
    OutsideSelection,
    /// A change that is not her cannon's: a pose, a clip, or a passage without initialization.
    Unrelated,
    /// The picture of her gun reading readiness as the frame leaves it.
    Draw,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
enum UnrelatedEvent {
    PoseChange,
    ClipChange,
    PassageWithoutInitialization,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
struct HistoryStep {
    kind: Operation,
    item_before: Option<SelectedItem>,
    item_after: Option<SelectedItem>,
    eligible: Option<bool>,
    one_shot_before: Option<bool>,
    one_shot_after: Option<bool>,
    switched: Option<bool>,
    world_paused: Option<bool>,
    event: Option<UnrelatedEvent>,
    expected_delay: u8,
    expected_may_follow: bool,
}

impl HistoryStep {
    /// The item controls' answer: the identities they start from and leave, and whether they
    /// may switch. The HUD's one-shot mode, its switch event and frozen time are not part of it.
    fn answer(&self) -> ItemControlAnswer {
        ItemControlAnswer {
            before: self
                .item_before
                .expect("expected the identity the controls start from"),
            after: self
                .item_after
                .expect("expected the identity the controls leave"),
            eligibility: if self
                .eligible
                .expect("expected whether the controls may switch")
            {
                ControlEligibility::Eligible
            } else {
                ControlEligibility::Ineligible
            },
        }
    }

    /// Each field belongs to the operations that give it, so none takes a meaning elsewhere.
    fn validate(&self) {
        let controls = self.kind == Operation::Controls;
        let identities = controls || self.kind == Operation::OutsideSelection;
        let fields = "expected each field only on the operations that give it";
        assert_eq!(self.item_before.is_some(), identities, "{fields}: {self:?}");
        assert_eq!(self.item_after.is_some(), identities, "{fields}: {self:?}");
        assert_eq!(self.eligible.is_some(), controls, "{fields}: {self:?}");
        assert_eq!(
            self.event.is_some(),
            self.kind == Operation::Unrelated,
            "{fields}: {self:?}"
        );
        let controls_only = [
            self.one_shot_before,
            self.one_shot_after,
            self.switched,
            self.world_paused,
        ];
        assert!(
            controls || controls_only.iter().all(Option::is_none),
            "{fields}: {self:?}"
        );
        assert!(
            CannonDelay::new(self.expected_delay).is_some(),
            "expected a delay of 0 to 2 frames: {self:?}"
        );
    }
}

/// The inputs of an animation vector an unrelated frame borrows its movement from.
fn borrowed<'a>(file: &'a VectorFile, id: &str) -> &'a VectorInputs {
    &file
        .vector
        .iter()
        .find(|vector| vector.id == id)
        .unwrap_or_else(|| panic!("expected an animation vector {id}"))
        .inputs
}

/// The frame a step plays, with the clips it reads, if the step is a frame of play.
///
/// A selection change outside item controls and a drawing are a history's own steps: it plays
/// no frame for them, and checks the delay as it stands. A pose or clip change is a frame whose
/// movement takes another pose or clip and whose controls don't run; a passage without
/// initialization is a frame with a passage entry and no controls. A frame of her item controls
/// or her initialization alone brings her standing still.
fn frame_of<'a>(
    step: &HistoryStep,
    file: &VectorFile,
    library: &'a Library,
) -> Option<(Frame, VectorContent<'a>)> {
    let standing = VectorInputs {
        pose: "Upright Still Right Forward".to_owned(),
        ..VectorInputs::default()
    };
    let running = VectorInputs {
        pose: "Upright Running Right Forward".to_owned(),
        ..VectorInputs::default()
    };
    let (given, parts, item_controls): (&VectorInputs, Set<FramePart>, ItemControls) =
        match step.kind {
            Operation::Controls => (&standing, Set::default(), ItemControls::Ran(step.answer())),
            Operation::Skip => (&standing, Set::default(), ItemControls::Skipped),
            Operation::Initialize => (
                &standing,
                set_of([FramePart::Initialization]),
                ItemControls::Skipped,
            ),
            Operation::OutsideSelection | Operation::Draw => return None,
            Operation::Unrelated => match step.event.expect("expected an unrelated event") {
                UnrelatedEvent::PoseChange => (
                    borrowed(file, "C-start"),
                    set_of([FramePart::Movement]),
                    ItemControls::Skipped,
                ),
                UnrelatedEvent::ClipChange => (
                    borrowed(file, "C-end"),
                    set_of([FramePart::Movement]),
                    ItemControls::Skipped,
                ),
                UnrelatedEvent::PassageWithoutInitialization => (
                    &running,
                    set_of([FramePart::PassageEntry]),
                    ItemControls::Skipped,
                ),
            },
        };
    let played = Frame {
        item_controls,
        ..frame(given, parts)
    };
    Some((played, moving_content("a cannon frame", given, library)))
}

/// Plays a history frame by frame from its initial delay, checking the delay after each step,
/// and at each drawing whether the frame left the cannon free to follow. Her kept action and
/// colours stay as they were, until an initialization sets her whole State as at
/// initialization.
fn run_history(history: &CannonHistory, file: &VectorFile, library: &Library) {
    let id = &history.id;
    let mut appearance = AppearanceState {
        animation: animation(2, 3, 1, 1),
        kept_action: Action::Spinning,
        cannon: CannonDelay::new(history.initial_delay)
            .unwrap_or_else(|| panic!("{id}: expected an initial delay of 0 to 2 frames")),
        colours: UNTOUCHED_COLOURS,
        hurt_count: HurtCount::NONE,
        dust: None,
    };
    let mut drawn = appearance.cannon.may_follow();
    for (at, step) in history.steps.iter().enumerate() {
        step.validate();
        if let Some((frame, content)) = frame_of(step, file, library) {
            let (after, outputs) = play(appearance, &frame, &content, &Rules::VANILLA);
            let unchanged = if step.kind == Operation::Initialize {
                assert_eq!(
                    after,
                    AppearanceState::INITIAL,
                    "{id} step {at}: expected her whole State at initialization"
                );
                AppearanceState::INITIAL
            } else {
                appearance
            };
            assert_eq!(
                after.colours, unchanged.colours,
                "{id} step {at}: expected her colours unchanged"
            );
            assert_eq!(
                after.kept_action, unchanged.kept_action,
                "{id} step {at}: expected her kept action unchanged"
            );
            assert!(
                outputs.sounds.len() <= 1,
                "{id} step {at}: expected a passage's sound at most"
            );
            drawn = outputs.cannon_may_follow;
            appearance = after;
        } else {
            appearance = kept(appearance);
        }
        let delay = appearance.cannon;
        assert_eq!(
            delay.remaining(),
            step.expected_delay,
            "{id} step {at}: expected the delay `expected_delay` gives"
        );
        assert_eq!(
            delay.may_follow(),
            step.expected_may_follow,
            "{id} step {at}: expected readiness as `expected_may_follow` gives it"
        );
        if step.kind == Operation::Draw {
            assert_eq!(
                drawn, step.expected_may_follow,
                "{id} step {at}: expected the picture to read readiness as the frame left it"
            );
        }
    }
}

#[test]
#[expect(
    clippy::print_stdout,
    reason = "report the number of supplied cannon histories"
)]
fn every_cannon_history() {
    let (file, library) = read_file();
    for history in &file.cannon_readiness {
        assert_ne!(history.source, "", "{}: expected a source", history.id);
        for entry in &history.entries {
            behaviour(entry);
        }
        run_history(history, &file, &library);
    }
    println!(
        "{} cannon readiness histories passed",
        file.cannon_readiness.len()
    );
}

#[test]
fn strict_cannon_parser_rejects_unknown_fields_and_names() {
    let text = "id='test'\nsource='constructed'\nentries=[]\ninitial_delay=0\nsteps=[{kind='draw',expected_delay=0,expected_may_follow=true}]";
    let refused = "expected the parser to refuse it";
    assert!(
        toml::from_str::<CannonHistory>(text).is_ok(),
        "expected a well-formed history to read"
    );
    assert!(
        toml::from_str::<CannonHistory>(&format!("{text}\nunknown=1")).is_err(),
        "an unknown key: {refused}"
    );
    assert!(
        toml::from_str::<CannonHistory>(&text.replace("kind='draw'", "kind='draw',unknown=1"))
            .is_err(),
        "an unknown step key: {refused}"
    );
    assert!(
        toml::from_str::<CannonHistory>(&text.replace("kind='draw'", "kind='redraw'")).is_err(),
        "an unknown kind: {refused}"
    );
    assert!(
        toml::from_str::<CannonHistory>(&text.replace(
            "kind='draw'",
            "kind='outside_selection',item_before='Missile',item_after='None'"
        ))
        .is_err(),
        "an unknown item: {refused}"
    );
    assert!(
        toml::from_str::<VectorFile>(&format!("{}\nunknown=1", super::VECTORS)).is_err(),
        "an unknown key of the file: {refused}"
    );
}

#[test]
fn the_delay_is_built_and_restored_only_in_its_range() {
    for frames in 0..=2 {
        let delay = CannonDelay::new(frames).expect("expected each delay of 0 to 2 frames");
        assert_eq!(delay.remaining(), frames, "expected the delay built");
        assert_eq!(
            delay.may_follow(),
            frames == 0,
            "expected readiness at 0 alone"
        );
        let appearance = AppearanceState {
            animation: animation(3, 7, 2, 1),
            kept_action: Action::WallJumping,
            cannon: delay,
            colours: UNTOUCHED_COLOURS,
            hurt_count: HurtCount::NONE,
            dust: None,
        };
        assert_eq!(
            kept(appearance),
            appearance,
            "expected it restored as saved"
        );
    }
    assert_eq!(CannonDelay::new(3), None, "expected a delay of 3 refused");
    assert_eq!(
        CannonDelay::new(u8::MAX),
        None,
        "expected a delay of 255 refused"
    );
    let initial = AppearanceState::INITIAL;
    let text = toml::to_string(&initial).expect("expected the appearance State to save");
    assert!(
        text.contains("\"Two\""),
        "expected the delay of 2 saved: {text}"
    );
    assert!(
        toml::from_str::<AppearanceState>(&text.replace("\"Two\"", "\"Three\"")).is_err(),
        "expected a restored delay of 3 refused"
    );
}
