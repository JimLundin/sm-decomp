//! Staged vectors drive the carried HUD and independently mutable external inputs.

use interface_hud::{EventCursor, HudOpportunity, MovementInputs, Observation, StatusInputs};
use serde::{Deserialize, Serialize};

use super::super::shape::{CutIn, OwnerIn, Stage};
use super::{
    Energy, FrameCount, HudEvent, HudPictures, HudState, HudWork, ItemControlInputs, Pair,
    ResumeIn, Rules, VectorFile, WorkIn, chosen_controls, chosen_inputs, differences, merge, state,
    work_expected,
};

#[derive(Clone)]
struct Current {
    controls: ItemControlInputs,
    movement: MovementInputs,
    status: StatusInputs,
}

impl Current {
    fn owner(&self) -> OwnerIn {
        OwnerIn {
            position: [self.movement.position.x, self.movement.position.y],
            energy: [self.status.energy.now, self.status.energy.capacity],
        }
    }

    fn choose(&self, stage: Stage, work: &mut HudWork, pictures: &HudPictures) -> bool {
        let rules = Rules::reference();
        match stage {
            Stage::ItemControls => work.choose_controls(self.controls),
            Stage::MiniMapUpdate => work.choose_mini_map(&rules, pictures, self.movement.clone()),
            Stage::StatusRefresh => work.choose_status(&rules, pictures, self.status),
        }
    }
}

const STAGES: [Stage; 3] = [
    Stage::ItemControls,
    Stage::MiniMapUpdate,
    Stage::StatusRefresh,
];

fn inputs_for(stage: Stage, input: &toml::Table) -> Result<toml::Table, String> {
    let keys: &[&str] = match stage {
        Stage::ItemControls => &[
            "select",
            "cancel",
            "can_switch",
            "missiles",
            "super_missiles",
            "power_bombs",
            "equipped",
        ],
        Stage::MiniMapUpdate => &[
            "runs",
            "turned_on",
            "mother_brain_falls",
            "position",
            "room",
            "map",
        ],
        Stage::StatusRefresh => &[
            "energy",
            "reserve",
            "auto_reserve",
            "missiles",
            "super_missiles",
            "power_bombs",
            "muted_by",
            "collected",
            "dropped",
        ],
    };
    if let Some(key) = input.keys().find(|key| !keys.contains(&key.as_str())) {
        return Err(format!("{stage:?} does not read {key}"));
    }
    if stage == Stage::MiniMapUpdate && !input.is_empty() {
        return Ok(toml::Table::from_iter([(
            "mini_map".to_owned(),
            input.clone().into(),
        )]));
    }
    Ok(input.clone())
}

fn parse_current(
    [controls, movement, status]: &[toml::Table; 3],
    pictures: &HudPictures,
) -> Result<Current, String> {
    let controls = chosen_controls(&inputs_for(Stage::ItemControls, controls)?, pictures)?;
    let map = chosen_inputs(&inputs_for(Stage::MiniMapUpdate, movement)?, pictures)?;
    let status = chosen_inputs(&inputs_for(Stage::StatusRefresh, status)?, pictures)?;
    Ok(Current {
        controls,
        movement: MovementInputs::capture(&map.mini_map(), map.mother_brain_falls),
        status: status.status_inputs(),
    })
}

type Snapshot = (HudState, Vec<HudEvent>);

pub(super) struct WorkCase<'a> {
    base: &'a WorkIn,
    before: Snapshot,
    expected: [Snapshot; 3],
    initial: Current,
    patched: Option<(Stage, CutIn, Current)>,
}

impl<'a> WorkCase<'a> {
    pub(super) fn load(
        base: &'a WorkIn,
        resume: Option<&ResumeIn>,
        pictures: &HudPictures,
    ) -> Result<Self, String> {
        let initial_tables = [
            base.controls.clone(),
            base.mini_map.clone(),
            base.status.clone(),
        ];
        let initial = parse_current(&initial_tables, pictures)?;
        let before = state(&base.before, pictures)?;
        let mut controls_after = base.controls_after.clone();
        let mut map_after = base
            .map_after
            .clone()
            .unwrap_or_else(|| controls_after.clone());
        let mut after = base.after.clone();
        let patched = if let Some(resume) = resume {
            let mut fresh = initial_tables;
            let [controls, movement, status] = &mut fresh;
            let selected = match resume.opportunity {
                Stage::ItemControls => controls,
                Stage::MiniMapUpdate => movement,
                Stage::StatusRefresh => status,
            };
            merge(selected, &resume.patch);
            merge(&mut controls_after, &resume.expect_controls_after);
            merge(&mut map_after, &resume.expect_map_after);
            merge(&mut after, &resume.expect_after);
            Some((
                resume.opportunity,
                resume.cut,
                parse_current(&fresh, pictures)?,
            ))
        } else {
            None
        };
        Ok(Self {
            expected: [
                work_expected(&controls_after, &before)?,
                work_expected(&map_after, &before)?,
                work_expected(&after, &before)?,
            ],
            before: (before, vec![]),
            base,
            initial,
            patched,
        })
    }

    fn patch(&self, current: &mut Current, stage: Stage, cut: CutIn) {
        let Some((at, when, fresh)) = &self.patched else {
            return;
        };
        if (*at, *when) != (stage, cut) {
            return;
        }
        match stage {
            Stage::ItemControls => current.controls = fresh.controls,
            Stage::MiniMapUpdate => current.movement = fresh.movement.clone(),
            Stage::StatusRefresh => current.status = fresh.status,
        }
    }

    fn run(&self, pictures: &HudPictures) -> Vec<String> {
        let frame = FrameCount(self.base.frame);
        let mut live = LiveWork {
            work: HudWork::begin(self.before.0.clone(), frame),
            delivery: EventCursor::default(),
        };
        let mut current = self.initial.clone();
        let mut energy_at_movement = current.status.energy;
        let mut problems = Vec::new();
        let mut previous = &self.before;
        for (stage, expected) in STAGES.into_iter().zip(&self.expected) {
            self.advance_owners(stage, &mut current);
            if stage == Stage::MiniMapUpdate {
                energy_at_movement = current.status.energy;
            }
            let (fresh, stage_problems) =
                self.run_stage(live, stage, previous, expected, &mut current, pictures);
            live = fresh;
            problems.extend(stage_problems);
            assert_eq!(live.work.frame_count(), Some(frame));
            assert_eq!(
                live.work.initial_highlight(),
                self.before.0.shown.highlighted
            );
            previous = expected;
        }
        assert_eq!(
            live.work.publication(),
            Some((live.work.state(), live.work.events()))
        );
        self.compare_ordinary(&live.work, pictures);
        if self.patched.is_none() && self.base.pose_after.is_some() {
            self.verify_wrong_times(
                &live.work,
                energy_at_movement,
                current.movement.position,
                pictures,
            );
        }
        problems
    }

    fn advance_owners(&self, stage: Stage, current: &mut Current) {
        let expected = match stage {
            Stage::ItemControls => None,
            Stage::MiniMapUpdate => {
                if let Some([now, capacity]) = self.base.energy_at_map {
                    current.status.energy = Energy { now, capacity };
                }
                self.base.owner_at_map.as_ref()
            }
            Stage::StatusRefresh => {
                current.status.energy = self.initial.status.energy;
                self.base.owner_at_status.as_ref()
            }
        };
        if let Some(expected) = expected {
            assert_eq!(&current.owner(), expected);
        }
    }

    fn run_stage(
        &self,
        live: LiveWork,
        stage: Stage,
        previous: &Snapshot,
        expected: &Snapshot,
        current: &mut Current,
        pictures: &HudPictures,
    ) -> (LiveWork, Vec<String>) {
        let mut live = live.restore();
        let offered = match stage {
            Stage::ItemControls => HudOpportunity::ItemControls,
            Stage::MiniMapUpdate => HudOpportunity::MiniMapUpdate,
            Stage::StatusRefresh => HudOpportunity::StatusRefresh,
        };
        assert_eq!(live.work.opportunity(), Some(offered));
        assert!(live.work.publication().is_none());
        assert!(!live.work.apply());
        assert_eq!(live.delivery.next_event(&live.work), None);
        let mut problems = compare(previous, &live.work);
        self.patch(current, stage, CutIn::BeforeAnswer);
        let at_choice = current.clone();
        assert!(current.choose(stage, &mut live.work, pictures));
        inspect_answer(&live.work, stage, &at_choice, previous, expected);
        problems.extend(compare(previous, &live.work));
        assert_eq!(live.delivery.next_event(&live.work), None);
        self.place_after_choice(stage, current);
        self.patch(current, stage, CutIn::AfterAnswer);
        live = live.restore();
        assert!(live.work.publication().is_none());
        assert!(!current.choose(stage, &mut live.work, pictures));
        inspect_answer(&live.work, stage, &at_choice, previous, expected);
        problems.extend(compare(previous, &live.work));
        assert!(live.work.apply());
        problems.extend(compare(expected, &live.work));
        live.deliver(&expected.1[previous.1.len()..]);
        live = live.restore(); // includes acknowledgement of the actual early switch
        self.patch(current, stage, CutIn::AfterEffect);
        assert!(!current.choose(stage, &mut live.work, pictures));
        self.place_after_choice(stage, current);
        assert!(!live.work.apply());
        assert_eq!(live.delivery.next_event(&live.work), None);
        problems.extend(compare(expected, &live.work));
        (live, problems)
    }

    fn place_after_choice(&self, stage: Stage, current: &mut Current) {
        if stage == Stage::MiniMapUpdate
            && let Some([x, y]) = self.base.pose_after
        {
            current.movement.position = Pair { x, y };
        }
    }

    fn compare_ordinary(&self, work: &HudWork, pictures: &HudPictures) {
        let chosen = completed_inputs(work);
        let (ordinary, events) = self.run_ordinary(work.frame_count().unwrap(), &chosen, pictures);
        assert_eq!(&ordinary, work.state());
        assert_eq!(events, work.events());
    }

    fn verify_wrong_times(
        &self,
        work: &HudWork,
        energy_at_movement: Energy,
        position_at_status: Pair<u16>,
        pictures: &HudPictures,
    ) {
        let chosen = completed_inputs(work);
        let mut early_energy = chosen.clone();
        early_energy.status.energy = energy_at_movement;
        let mut late_position = chosen;
        late_position.movement.position = position_at_status;
        let frame = work.frame_count().unwrap();
        let [_, _, (expected, expected_events)] = &self.expected;
        let (wrong_energy, events) = self.run_contrast(frame, &early_energy, pictures);
        assert_eq!(&events, expected_events);
        assert_eq!(wrong_energy.explored, expected.explored);
        assert_ne!(wrong_energy.shown.energy, expected.shown.energy);
        assert_ne!(
            wrong_energy.grid.get(super::point([6, 2])),
            expected.grid.get(super::point([6, 2]))
        );
        let (wrong_map, events) = self.run_contrast(frame, &late_position, pictures);
        assert_eq!(&events, expected_events);
        assert_eq!(wrong_map.shown, expected.shown);
        assert_ne!(wrong_map.explored, expected.explored);
        assert_ne!(
            wrong_map.grid.get(super::point([28, 1])),
            expected.grid.get(super::point([28, 1]))
        );
    }

    // Run each wrong observation independently against the worked final expectation, through
    // the same public opportunities as the correct run. Only one external input differs.
    fn run_contrast(
        &self,
        frame: FrameCount,
        current: &Current,
        pictures: &HudPictures,
    ) -> Snapshot {
        let mut work = HudWork::begin(self.before.0.clone(), frame);
        for stage in STAGES {
            assert!(current.choose(stage, &mut work, pictures));
            assert!(work.apply());
        }
        let (hud, events) = work.publication().expect("contrast completed");
        (hud.clone(), events.to_vec())
    }

    fn run_ordinary(
        &self,
        frame: FrameCount,
        chosen: &Current,
        pictures: &HudPictures,
    ) -> Snapshot {
        let mut ordinary = self.before.0.clone();
        // FrameInputs shares stock between controls and status. Run the existing controls entry
        // first when those two observations can differ, then run the frame with no button edge.
        let mut events: Vec<_> = ordinary
            .item_controls(&chosen.controls)
            .into_iter()
            .collect();
        events.extend(draw_ordinary(
            &mut ordinary,
            frame,
            &chosen.movement,
            chosen.status,
            pictures,
        ));
        (ordinary, events)
    }
}

fn completed_inputs(work: &HudWork) -> Current {
    let Observation::Completed(controls) = work.controls() else {
        panic!("controls complete")
    };
    let Observation::Completed(map) = work.mini_map() else {
        panic!("map complete")
    };
    let Observation::Completed(status) = work.status() else {
        panic!("status complete")
    };
    Current {
        controls: controls.inputs,
        movement: map.inputs.clone(),
        status: status.inputs,
    }
}

fn draw_ordinary(
    hud: &mut HudState,
    frame: FrameCount,
    map: &MovementInputs,
    status: StatusInputs,
    pictures: &HudPictures,
) -> Vec<HudEvent> {
    let inputs = super::FrameInputs {
        frame,
        status: super::Status {
            energy: status.energy,
            reserve: status.reserve,
            ammunition: status.ammunition,
            equipped: super::Equipped::default(),
        },
        controls: super::Controls {
            select: super::SelectButton::NotPressed,
            cancel: super::CancelButton::NotPressed,
            can_switch: false,
            muted_by: status.muted_by,
        },
        dropped_before_status_bar: status.dropped_before_status_bar,
        dropped_after_status_bar: status.dropped_after_status_bar,
        collected: status.collected,
        mother_brain_falls: map.mother_brain_falls,
        mini_map: map.as_mini_map(),
    };
    hud.frame_of_play(&Rules::reference(), pictures, &inputs)
}

fn inspect_answer(
    work: &HudWork,
    stage: Stage,
    current: &Current,
    previous: &Snapshot,
    expected: &Snapshot,
) {
    let new_events = &expected.1[previous.1.len()..];
    match (stage, work.controls(), work.mini_map(), work.status()) {
        (
            Stage::ItemControls,
            Observation::Chosen(answer),
            Observation::Unread,
            Observation::Unread,
        ) => {
            assert_eq!(answer.inputs, current.controls);
            assert_eq!(answer.selection, expected.0.selection);
            assert_eq!(answer.switched.as_slice(), new_events);
        }
        (
            Stage::MiniMapUpdate,
            Observation::Completed(_),
            Observation::Chosen(answer),
            Observation::Unread,
        ) => {
            assert_eq!(answer.inputs, current.movement);
            assert_eq!(answer.after, expected.0);
            assert_eq!(new_events, []);
        }
        (
            Stage::StatusRefresh,
            Observation::Completed(_),
            Observation::Completed(_),
            Observation::Chosen(answer),
        ) => {
            assert_eq!(answer.inputs, current.status);
            assert_eq!(answer.after, expected.0);
            assert_eq!(answer.click.as_slice(), new_events);
        }
        _ => panic!("incorrect pending answer at {stage:?}"),
    }
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
struct LiveWork {
    work: HudWork,
    delivery: EventCursor,
}

impl LiveWork {
    fn restore(self) -> Self {
        let text = toml::to_string(&self).expect("serialize work and delivery position");
        drop(self);
        let fresh: Self = toml::from_str(&text).expect("fresh work and delivery position");
        assert_eq!(toml::to_string(&fresh).unwrap(), text);
        fresh
    }

    fn deliver(&mut self, expected: &[HudEvent]) {
        // Hand the cursor from the early switch reader to the later reader; the latter sees
        // only a newly available click, not the switch in the complete history.
        for &event in expected {
            assert_eq!(self.delivery.next_event(&self.work), Some(event));
        }
        assert_eq!(self.delivery.next_event(&self.work), None);
    }
}

fn compare(expected: &Snapshot, work: &HudWork) -> Vec<String> {
    differences(&expected.0, work.state(), &expected.1, work.events())
}

impl WorkIn {
    pub fn run(&self, pictures: &HudPictures) -> Result<Vec<String>, String> {
        Ok(WorkCase::load(self, None, pictures)?.run(pictures))
    }
}

impl ResumeIn {
    pub(super) fn base<'a>(&self, file: &'a VectorFile) -> Result<&'a WorkIn, String> {
        file.hud_work
            .iter()
            .find(|work| work.id == self.base)
            .ok_or_else(|| format!("unknown base {}", self.base))
    }

    pub fn run(&self, file: &VectorFile, pictures: &HudPictures) -> Result<Vec<String>, String> {
        Ok(WorkCase::load(self.base(file)?, Some(self), pictures)?.run(pictures))
    }
}
