//! The step: one frame of the doors, from the state a caller holds and what the frame brings to
//! what the frame answers.

use types::{Heading, Position, Set, px, set_of};

use crate::colours::{ColourGroup, FadeTime, Palette, fade, head_for_dark, light_visor};
use crate::io::{
    Activity, Cost, CostEvent, Exports, Finishing, HerSound, Inputs, Report, RoomCall, RoomSound,
    Scene, SlideScreen,
};
use crate::passage::{
    Arrival, Doors, Fetching, LAST_BEFORE_THE_SLIDE, LAST_LANDING, Passage, Phase, WorkFrames,
};
use crate::rules::{Axis, Rules};
use crate::view;

impl Doors {
    /// One frame of the doors: the state at the frame's start, the scene as the frame before
    /// left it and what this frame brings, to the state at its end, the scene as it leaves it and
    /// what it reports.
    ///
    /// In play, a touch of a door to a room lets the frame play out as usual and starts the
    /// passage on the next (`TheTouch`). While passing, nothing she touches is read.
    #[must_use]
    pub fn step(self, scene: Scene, inputs: &Inputs, rules: &Rules) -> Exports {
        match self {
            Self::Play => Exports {
                doors: if inputs.touched {
                    Self::Passing(Passage::START)
                } else {
                    Self::Play
                },
                scene,
                report: Report::play(),
            },
            Self::Passing(passage) => passage.step(scene, inputs, rules),
        }
    }
}

impl Passage {
    /// One frame of the passage. Two clocks run in it (`PassageTime`):
    /// - every frame, whatever the work, at the frame's start: she is placed as `Arriving`
    ///   begins, and on each frame after the slide takes a frame (`Arriving`, `Sliding`);
    /// - then the frame of work in progress, which ends this frame when the platform says its
    ///   cost is spent, or, with no cost, when the slide has ended; only when it ends does the
    ///   phase's entry act and the phase move on. Until then nothing else moves.
    fn step(mut self, mut scene: Scene, inputs: &Inputs, rules: &Rules) -> Exports {
        let finishes_anyway = self.phase.finishes_anyway();
        let event = self.phase.cost_event();
        let mut report = Report::of_passage(event.map_or(Cost::OwnWait, Cost::Event));

        // Every frame, whatever the work: the placing, then the slide.
        if let Phase::Arriving { placed, fetching } = self.phase {
            if !placed {
                scene.her = view::place(scene.her, scene.camera);
                scene.seen = view::pixels(scene.her);
                self.phase = Phase::Arriving {
                    placed: true,
                    fetching,
                };
            } else if !self.slide_ended(rules) {
                self.slide(&mut scene, &mut report, rules);
            }
        }

        // Each frame of work: done when its cost is spent, or, with none, as the slide ends.
        let work_ends = event.map_or_else(|| self.slide_ended(rules), |_| inputs.work_done);
        report.finishing = if finishes_anyway {
            Finishing::Restarts(SlideScreen::below_the_hud(inputs.items_dark))
        } else if work_ends {
            Finishing::Finished
        } else {
            Finishing::Overrun
        };
        let doors = if work_ends {
            self.work(&mut scene, &mut report, inputs, rules)
                .map_or(Doors::Play, |phase| Doors::Passing(Self { phase, ..self }))
        } else {
            Doors::Passing(self)
        };
        Exports {
            doors,
            scene,
            report,
        }
    }

    /// The end of a frame of work: what runs, and the phase's entry, which answers the next
    /// phase, or none when play goes on.
    fn work(
        &mut self,
        scene: &mut Scene,
        report: &mut Report,
        inputs: &Inputs,
        rules: &Rules,
    ) -> Option<Phase> {
        let steps = rules.fade_steps;
        report.runs = self.phase.what_runs(inputs, steps);
        match self.phase {
            Phase::Quieting { asked: false } => Some(leave(&mut scene.palette, report, inputs)),
            Phase::Quieting { asked: true } => Some(if inputs.sounds_sent {
                Phase::Darkening(FadeTime::START)
            } else {
                self.phase
            }),
            Phase::Darkening(t) => {
                fade(&mut scene.palette, t, steps);
                Some(t.next(steps).map_or(Phase::Squaring, Phase::Darkening))
            }
            Phase::Squaring => Some(self.square(&mut scene.camera, inputs.doorway, report)),
            Phase::BeforeTheSlide(n) => Some(self.before_the_slide(n, scene, report, rules)),
            Phase::Arriving { placed, fetching } => Some(fetch(
                placed,
                fetching,
                &mut scene.palette,
                &inputs.groups,
                report,
            )),
            Phase::Landing(n) => Some(self.land(n, &mut scene.her, report)),
            Phase::Music { started } => Some(music(started, &mut scene.her, inputs, report)),
            Phase::Brightening(t) => brighten(t, &mut scene.palette, report, steps),
        }
    }

    /// `Squaring`: with no arrival held, the doorway's is taken and held and the line effects'
    /// objects are let go (with no doorway given, it waits); then each frame of work creeps the
    /// view a pixel toward a screen's edge, and the one that finds it on an edge ends the
    /// squaring.
    fn square(
        &mut self,
        camera: &mut Position,
        doorway: Option<Arrival>,
        report: &mut Report,
    ) -> Phase {
        let Some(arrival) = self.arrival else {
            if doorway.is_some() {
                self.arrival = doorway;
                report.call = Some(RoomCall::LeaveRoom);
            }
            return Phase::Squaring;
        };
        let Some(crept) = view::squared(*camera, arrival.way) else {
            return Phase::BeforeTheSlide(WorkFrames::FIRST);
        };
        *camera = crept;
        Phase::Squaring
    }

    /// `Building`: the first frame of work after the squaring prepares a slide up, the second
    /// builds the next room, and the third sets the view off (`SettingOff`).
    fn before_the_slide(
        &mut self,
        n: WorkFrames<LAST_BEFORE_THE_SLIDE>,
        scene: &mut Scene,
        report: &mut Report,
        rules: &Rules,
    ) -> Phase {
        match n.next() {
            Some(next) if n == WorkFrames::FIRST => {
                if self
                    .arrival
                    .is_some_and(|arrival| arrival.way == Heading::Up)
                {
                    report.call = Some(RoomCall::DrawTopRow);
                    self.slide(scene, report, rules);
                }
                Phase::BeforeTheSlide(next)
            }
            Some(next) => {
                report.call = Some(RoomCall::BuildRoom);
                Phase::BeforeTheSlide(next)
            }
            None => {
                if let Some(arrival) = self.arrival.as_mut() {
                    scene.camera = view::setting_off(arrival, scene.camera);
                }
                self.slide(scene, report, rules);
                Phase::Arriving {
                    placed: false,
                    fetching: Fetching::Tiles,
                }
            }
        }
    }

    /// `Sliding`, `Carrying`: one slide frame, which counts on the slide's frames.
    ///
    /// Each frame but a slide down or up's frame 0 moves the view the slide's step the way she
    /// leaves, layer 2 with it, and carries her that way at the doorway's speed or the passage's
    /// own; the camera sees her where she is carried. The frame that brings the count to the
    /// slide's length puts the view on the arrival, wherever the steps left it: the slide ends.
    fn slide(&mut self, scene: &mut Scene, report: &mut Report, rules: &Rules) {
        let Some(arrival) = self.arrival else {
            return;
        };
        let axis = Axis::of(arrival.way);
        let moves = self.slide_frames > 0 || first_frame_moves(axis);
        self.slide_frames = self.slide_frames.saturating_add(1);
        if moves {
            let step = view::toward(arrival.way, px(rules.slide_step));
            scene.camera = view::plus(scene.camera, step);
            report.layer_2 = Some(step);
            let speed = arrival.speed.unwrap_or(rules.carry_speed[axis]);
            scene.her = view::plus(scene.her, view::toward(arrival.way, speed));
            scene.seen = view::pixels(scene.her);
        }
        if self.slide_frames >= rules.slide_length[axis] {
            scene.camera = view::at_pixels(arrival.view, scene.camera);
        }
    }

    /// `Sliding`: whether the slide has ended, its frames at its length; with no arrival there
    /// is no slide to wait for.
    fn slide_ended(&self, rules: &Rules) -> bool {
        self.arrival
            .is_none_or(|arrival| self.slide_frames >= rules.slide_length[Axis::of(arrival.way)])
    }

    /// `Landing`: the first frame of work after the slide settles her across (`WhereSheLands`);
    /// the second runs the animated tiles once, and the dark room waits for its track.
    fn land(&self, n: WorkFrames<LAST_LANDING>, her: &mut Position, report: &mut Report) -> Phase {
        let Some(next) = n.next() else {
            report.call = Some(RoomCall::AnimateTiles);
            return Phase::Music { started: false };
        };
        if let Some(arrival) = self.arrival {
            *her = view::settled(*her, arrival.way);
        }
        Phase::Landing(next)
    }
}

/// `Sliding`: whether a slide's frame 0 moves the view: left or right it does; down or up it is a
/// still frame.
const fn first_frame_moves(axis: Axis) -> bool {
    match axis {
        Axis::Across => true,
        Axis::DownUp => false,
    }
}

/// `Quieting`, `TheFades`: the first frame of work. The silence is asked for and sounds are held
/// off until play goes on, her sound at the door is asked for, the mini-map is turned on, and the
/// colours head for the dark ones.
fn leave(palette: &mut Palette, report: &mut Report, inputs: &Inputs) -> Phase {
    head_for_dark(palette, &inputs.groups, kept_lit(inputs));
    report.room_sounds = set_of([RoomSound::Silence, RoomSound::Held]);
    report.her_sound = Some(HerSound::AtTheDoor);
    report.minimap_on = true;
    Phase::Quieting { asked: true }
}

/// `TheFades`: the HUD's groups that keep their colours as the room goes dark: the always-lit
/// ones; the icons too unless either room keeps the items dark; and the escape timer's while it
/// runs, unless either room keeps the items dark.
fn kept_lit(inputs: &Inputs) -> Set<ColourGroup> {
    let mut kept = set_of([ColourGroup::Lit]);
    kept[ColourGroup::Icon] = !inputs.items_dark;
    kept[ColourGroup::Timer] = !inputs.items_dark && inputs.timer_running;
    kept
}

/// `Arriving`: the tileset is fetched, then the room's contents; the frame of work that waits
/// for the slide ends with it, and lights her visor at once as she lands.
fn fetch(
    placed: bool,
    fetching: Fetching,
    palette: &mut Palette,
    groups: &[ColourGroup; crate::colours::COLOURS],
    report: &mut Report,
) -> Phase {
    match fetching {
        Fetching::Tiles => {
            report.call = Some(RoomCall::FetchTiles);
            Phase::Arriving {
                placed,
                fetching: Fetching::Contents,
            }
        }
        Fetching::Contents => {
            report.call = Some(RoomCall::FetchContents);
            Phase::Arriving {
                placed,
                fetching: Fetching::Done,
            }
        }
        Fetching::Done => {
            light_visor(palette, groups);
            Phase::Landing(WorkFrames::FIRST)
        }
    }
}

/// `Landing`: the next room's track starts on the first frame of work that finds the music
/// ready; on the next she steps clear (`WhereSheLands`), her liquid is read and the fade in
/// begins.
fn music(started: bool, her: &mut Position, inputs: &Inputs, report: &mut Report) -> Phase {
    if started {
        *her = view::stepped_clear(*her);
        report.call = Some(RoomCall::ReadLiquid);
        Phase::Brightening(FadeTime::START)
    } else if inputs.music_ready {
        report.call = Some(RoomCall::PlayMusic);
        Phase::Music { started: true }
    } else {
        Phase::Music { started: false }
    }
}

/// `TheFades`: a step of the fade in; after its last, play goes on, sounds are let through again
/// and her spin's sound is asked for again.
fn brighten(t: FadeTime, palette: &mut Palette, report: &mut Report, steps: u8) -> Option<Phase> {
    fade(palette, t, steps);
    let next = t.next(steps);
    if next.is_none() {
        report.room_sounds = set_of([RoomSound::LetThrough]);
        report.her_sound = Some(HerSound::SpinAgain);
    }
    next.map(Phase::Brightening)
}

impl Phase {
    /// `FinishedFrames`: from the slide's own first step until she steps clear (`Arriving` once
    /// she is placed, `Landing`, `Music`), every frame is a finished frame, whatever the work,
    /// and restarts the HUD's frame count.
    const fn finishes_anyway(self) -> bool {
        matches!(
            self,
            Self::Arriving { placed: true, .. } | Self::Landing(_) | Self::Music { .. }
        )
    }

    /// `PassageTime`: the cost event of this phase's frame of work, or none for the wait on the
    /// slide's end, of which the platform is asked nothing.
    const fn cost_event(self) -> Option<CostEvent> {
        match self {
            Self::Darkening(_) | Self::Brightening(_) => Some(CostEvent::FadeStep),
            Self::BeforeTheSlide(n) if n.get() == 1 => Some(CostEvent::BuildRoom),
            Self::Arriving {
                fetching: Fetching::Tiles,
                ..
            } => Some(CostEvent::FetchTiles),
            Self::Arriving {
                fetching: Fetching::Contents,
                ..
            } => Some(CostEvent::FetchContents),
            Self::Arriving {
                fetching: Fetching::Done,
                ..
            } => None,
            Self::BeforeTheSlide(_)
            | Self::Landing(_)
            | Self::Music { .. }
            | Self::Quieting { .. }
            | Self::Squaring => Some(CostEvent::Other),
        }
    }

    /// `WhatRuns`: what runs or is drawn on this phase's frame of work. While the room is lit,
    /// as it goes dark, the enemies run and all is drawn, still; as the new room's colours come
    /// up the animated tiles, the enemies and their projectiles run; the tiles run once as she
    /// lands and the blocks once as she arrives; nothing else of play runs.
    fn what_runs(self, inputs: &Inputs, steps: u8) -> Set<Activity> {
        let mut runs = match self {
            Self::Quieting { .. } => set_of([Activity::Drawn, Activity::Enemies]),
            Self::Darkening(t) if !t.is_last(steps) => set_of([Activity::Drawn, Activity::Enemies]),
            Self::Brightening(_) => set_of([
                Activity::Drawn,
                Activity::Enemies,
                Activity::Projectiles,
                Activity::Tiles,
            ]),
            Self::Landing(n) if n.next().is_none() => set_of([Activity::Tiles]),
            Self::Arriving {
                fetching: Fetching::Done,
                ..
            } => set_of([Activity::Blocks]),
            Self::Darkening(_)
            | Self::Landing(_)
            | Self::Arriving { .. }
            | Self::BeforeTheSlide(_)
            | Self::Music { .. }
            | Self::Squaring => Set::default(),
        };
        runs[Activity::Timer] = inputs.timer_running && self.draws_timer(inputs.items_dark);
        runs
    }

    /// `WhatRuns`: whether this phase's frame of work draws the escape's timer while it runs:
    /// every one but the contents', and the first only while neither room keeps the items dark.
    const fn draws_timer(self, items_dark: bool) -> bool {
        match self {
            Self::Quieting { asked: false } => !items_dark,
            Self::Arriving {
                fetching: Fetching::Contents | Fetching::Done,
                ..
            } => false,
            Self::Quieting { asked: true }
            | Self::Darkening(_)
            | Self::Squaring
            | Self::BeforeTheSlide(_)
            | Self::Arriving {
                fetching: Fetching::Tiles,
                ..
            }
            | Self::Landing(_)
            | Self::Music { .. }
            | Self::Brightening(_) => true,
        }
    }
}

impl SlideScreen {
    /// `FinishedFrames`: what is shown below the HUD on a frame that restarts the count: the
    /// front layer and the sprites, or only the sprites when either room keeps the items dark.
    const fn below_the_hud(items_dark: bool) -> Self {
        if items_dark {
            Self::Sprites
        } else {
            Self::FrontAndSprites
        }
    }
}
