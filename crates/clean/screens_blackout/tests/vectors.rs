//! Runs every vector of the spec's vector file through the model, refusing any key it doesn't
//! know, and checks the catalogue against the vectors.

#[cfg(test)]
mod vectors {
    use screens_blackout::catalogue::{BehaviourId, behaviour};
    use screens_blackout::{Opportunity, Reach, Request, RoomPass, SoundChannel, advance};
    use serde::Deserialize;
    use types::Checked;
    use types::ceres_blackout::{
        BlackoutLevel, BlackoutLight, BlackoutPhase, BlackoutState, BlackoutTimer, FinishedPasses,
        FlightRoute, PreparedPicture, PublishedPicture, ShaftStage, TimerActivity,
        TimerCentiseconds, TimerSprite, WaitPasses, WorkPass,
    };

    const VECTORS: &str = include_str!("../../../../scenarios/vectors/screens-blackout.toml");

    /// An invented escape time for fixtures that state none.
    const INVENTED_TIME: u32 = 4_321;
    /// The vectors' level for forced blank.
    const BLANK: u8 = 0;

    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct VectorFile {
        vector: Vec<Vector>,
    }

    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Vector {
        id: String,
        source: String,
        entries: Vec<String>,
        before: Snapshot,
        takes: Vec<Take>,
        input: Option<Input>,
        after: Snapshot,
    }

    #[derive(Deserialize, Default)]
    #[serde(deny_unknown_fields)]
    struct Snapshot {
        phase: Option<String>,
        hold: Option<u8>,
        timer: Option<u32>,
        light: Option<u8>,
        published_light: Option<u8>,
        finished: Option<u16>,
        room_work: Option<usize>,
        target: Option<String>,
        timer_visible: Option<bool>,
        shaft_rotates: Option<bool>,
        final_room_published: Option<bool>,
        pending_timer_pieces: Option<Vec<u16>>,
        displayed_timer_pieces: Option<Vec<u16>>,
        events: Option<Vec<String>>,
    }

    #[derive(Deserialize, Clone, Copy)]
    #[serde(deny_unknown_fields)]
    struct Input {
        reached: bool,
        drawn_timer: u32,
    }

    #[derive(Deserialize, Clone, Copy, Debug)]
    enum Take {
        Room,
        Publish,
        Interrupt,
        VblankEnded,
        SaveFinished,
        NewWork,
        Restore,
    }

    fn light(level: u8) -> BlackoutLight {
        if level == BLANK {
            BlackoutLight::ForcedBlank
        } else {
            BlackoutLight::Lit(BlackoutLevel::new(level).expect("a lit level"))
        }
    }

    const fn level(light: BlackoutLight) -> u8 {
        match light {
            BlackoutLight::ForcedBlank => BLANK,
            BlackoutLight::Lit(level) => level.get(),
        }
    }

    fn route(name: Option<&str>) -> FlightRoute {
        match name {
            None | Some("ZebesFlight") => FlightRoute::ZebesFlight,
            Some(other) => panic!("unknown target {other}"),
        }
    }

    fn phase(before: &Snapshot) -> BlackoutPhase {
        let carried = route(before.target.as_deref());
        match before.phase.as_deref().unwrap_or("Shaft") {
            "Shaft" => BlackoutPhase::Shaft,
            "Hold" => BlackoutPhase::Hold(
                WaitPasses::new(before.hold.expect("a wait")).expect("a wait of 1 to 60"),
            ),
            "Fade" => BlackoutPhase::Fade,
            "AwaitBlank" => BlackoutPhase::AwaitBlank,
            "Saving" => BlackoutPhase::Saving(carried),
            "Explosion" => BlackoutPhase::Explosion(carried),
            "Dispatched" => BlackoutPhase::Dispatched(carried),
            other => panic!("unknown phase {other}"),
        }
    }

    const fn phase_name(phase: BlackoutPhase) -> &'static str {
        match phase {
            BlackoutPhase::Shaft => "Shaft",
            BlackoutPhase::Hold(_) => "Hold",
            BlackoutPhase::Fade => "Fade",
            BlackoutPhase::AwaitBlank => "AwaitBlank",
            BlackoutPhase::Saving(_) => "Saving",
            BlackoutPhase::Explosion(_) => "Explosion",
            BlackoutPhase::Dispatched(_) => "Dispatched",
        }
    }

    fn sprites(pieces: &[u16]) -> Vec<TimerSprite> {
        pieces.iter().copied().map(TimerSprite).collect()
    }

    fn pieces(sprites: &[TimerSprite]) -> Vec<u16> {
        sprites.iter().map(|sprite| sprite.0).collect()
    }

    /// The pass a fixture stands in: the pieces it states, or the suspended phases' pending
    /// picture (every suspension follows room work), or a finished pass.
    fn pass(before: &Snapshot, phase: BlackoutPhase) -> WorkPass {
        let prepared = || {
            WorkPass::Prepared(PreparedPicture {
                timer_sprites: sprites(before.pending_timer_pieces.as_deref().unwrap_or(&[])),
            })
        };
        if before.pending_timer_pieces.is_some() {
            return prepared();
        }
        match phase {
            BlackoutPhase::AwaitBlank | BlackoutPhase::Saving(_) => prepared(),
            BlackoutPhase::Explosion(_) if !before.final_room_published.unwrap_or(false) => {
                prepared()
            }
            BlackoutPhase::Shaft
            | BlackoutPhase::Hold(_)
            | BlackoutPhase::Fade
            | BlackoutPhase::Explosion(_)
            | BlackoutPhase::Dispatched(_) => WorkPass::Finished,
        }
    }

    fn state(before: &Snapshot) -> BlackoutState {
        let phase = phase(before);
        let lit = before.light.unwrap_or(BlackoutLevel::FULL.get());
        BlackoutState {
            phase,
            light: light(lit),
            published: PublishedPicture {
                light: light(before.published_light.unwrap_or(lit)),
                timer_sprites: sprites(before.displayed_timer_pieces.as_deref().unwrap_or(&[])),
            },
            pass: pass(before, phase),
            timer: BlackoutTimer {
                time: TimerCentiseconds(before.timer.unwrap_or(INVENTED_TIME)),
                activity: if before.timer_visible.unwrap_or(true) {
                    TimerActivity::Active
                } else {
                    TimerActivity::Retired
                },
            },
            shaft: if before.shaft_rotates.unwrap_or(true) {
                ShaftStage::Rotating
            } else {
                ShaftStage::Cleared
            },
            finished: FinishedPasses(before.finished.unwrap_or(0)),
        }
    }

    const fn request_name(request: Request) -> &'static str {
        match request {
            Request::StandAndLock => "StandAndLock",
            Request::ContinueRoom => "ContinueRoom",
            Request::DisableShaftInterrupts => "DisableShaftInterrupts",
            Request::PrepareSpriteOnly => "PrepareSpriteOnly",
            Request::CaptureEscapeSave(_) => "CaptureEscapeSave",
            Request::StopMusic(_) => "StopMusic",
            Request::Silence(SoundChannel::One) => "SilenceOne",
            Request::Silence(SoundChannel::Two) => "SilenceTwo",
            Request::Silence(SoundChannel::Three) => "SilenceThree",
            Request::PrepareStationExplosion(_) => "PrepareStationExplosion",
            Request::DispatchExplosion(_) => "DispatchExplosion",
        }
    }

    const fn room_pass(input: Option<Input>) -> RoomPass {
        let reach = match input {
            Some(Input {
                reached: true,
                drawn_timer,
            }) => Reach::Reached(TimerCentiseconds(drawn_timer)),
            _ => Reach::NotReached,
        };
        RoomPass {
            reach,
            picture: PreparedPicture {
                timer_sprites: Vec::new(),
            },
        }
    }

    /// The State saved as text and read back, as a restored game would.
    fn restored(state: &BlackoutState) -> BlackoutState {
        let saved = toml::to_string(state).expect("the State saves");
        toml::from_str(&saved).expect("the State restores")
    }

    fn run(vector: &Vector) -> (BlackoutState, Vec<Request>) {
        let mut state = state(&vector.before);
        let mut requests = Vec::new();
        for take in &vector.takes {
            let opportunity = match take {
                Take::Room => Opportunity::RoomWork(room_pass(vector.input)),
                Take::Publish => Opportunity::Publication,
                Take::Interrupt => Opportunity::Interrupted,
                Take::VblankEnded => Opportunity::VerticalBlankEnded,
                Take::SaveFinished => Opportunity::SaveCompleted,
                Take::NewWork => Opportunity::NewWork,
                Take::Restore => {
                    state = restored(&state);
                    continue;
                }
            };
            requests.extend(advance(&mut state, opportunity));
        }
        (state, requests)
    }

    fn check<T: PartialEq + core::fmt::Debug>(id: &str, field: &str, want: Option<T>, got: &T) {
        if let Some(want) = want {
            assert_eq!(got, &want, "{id}: {field}");
        }
    }

    fn check_vector(vector: &Vector) {
        let (state, requests) = run(vector);
        let after = &vector.after;
        let id = vector.id.as_str();
        let hold = match state.phase {
            BlackoutPhase::Hold(wait) => wait.get(),
            BlackoutPhase::Shaft
            | BlackoutPhase::Fade
            | BlackoutPhase::AwaitBlank
            | BlackoutPhase::Saving(_)
            | BlackoutPhase::Explosion(_)
            | BlackoutPhase::Dispatched(_) => 0,
        };
        let target = match state.phase {
            BlackoutPhase::Saving(route)
            | BlackoutPhase::Explosion(route)
            | BlackoutPhase::Dispatched(route) => Some(format!("{route:?}")),
            BlackoutPhase::Shaft
            | BlackoutPhase::Hold(_)
            | BlackoutPhase::Fade
            | BlackoutPhase::AwaitBlank => None,
        };
        let pending = match &state.pass {
            WorkPass::Prepared(picture) => pieces(&picture.timer_sprites),
            WorkPass::Finished => Vec::new(),
        };
        let room_work = requests
            .iter()
            .filter(|request| **request == Request::ContinueRoom)
            .count();
        let events: Vec<String> = requests
            .iter()
            .map(|request| request_name(*request).to_owned())
            .collect();
        check(
            id,
            "phase",
            after.phase.as_deref(),
            &phase_name(state.phase),
        );
        check(id, "hold", after.hold, &hold);
        check(id, "timer", after.timer, &state.timer.time.0);
        check(id, "light", after.light, &level(state.light));
        check(
            id,
            "published_light",
            after.published_light,
            &level(state.published.light),
        );
        check(id, "finished", after.finished, &state.finished.0);
        check(id, "room_work", after.room_work, &room_work);
        check(id, "target", after.target.clone().map(Some), &target);
        check(
            id,
            "timer_visible",
            after.timer_visible,
            &(state.timer.activity == TimerActivity::Active),
        );
        check(
            id,
            "shaft_rotates",
            after.shaft_rotates,
            &(state.shaft == ShaftStage::Rotating),
        );
        check(
            id,
            "final_room_published",
            after.final_room_published,
            &(state.pass == WorkPass::Finished),
        );
        check(
            id,
            "pending_timer_pieces",
            after.pending_timer_pieces.clone(),
            &pending,
        );
        check(
            id,
            "displayed_timer_pieces",
            after.displayed_timer_pieces.clone(),
            &pieces(&state.published.timer_sprites),
        );
        check(id, "events", after.events.clone(), &events);
    }

    fn vectors() -> Vec<Vector> {
        toml::from_str::<VectorFile>(VECTORS)
            .expect("the vector file reads")
            .vector
    }

    fn entry(name: &str) -> BehaviourId {
        BehaviourId::ALL
            .iter()
            .copied()
            .find(|id| format!("{id:?}") == name)
            .unwrap_or_else(|| panic!("unknown entry {name}"))
    }

    #[test]
    fn every_vector_passes() {
        let vectors = vectors();
        assert!(!vectors.is_empty());
        for vector in &vectors {
            assert!(!vector.source.is_empty(), "{}: source", vector.id);
            check_vector(vector);
        }
    }

    #[test]
    fn every_entry_checked_by_vectors_has_one() {
        let vectors = vectors();
        for &id in BehaviourId::ALL {
            let named = vectors
                .iter()
                .any(|vector| vector.entries.iter().any(|name| entry(name) == id));
            let by_vectors = behaviour(id).checked == Checked::ByVectors;
            assert_eq!(named, by_vectors, "{id:?}");
        }
    }

    #[test]
    fn the_catalogue_names_each_entry_once() {
        for &id in BehaviourId::ALL {
            assert_eq!(behaviour(id).id, id);
        }
        let names: Vec<String> = BehaviourId::ALL
            .iter()
            .map(|id| format!("{id:?}"))
            .collect();
        for name in ["ElevatorStopsSamus", "PlatformWait", "DepartureHandoff"] {
            assert_eq!(names.iter().filter(|n| *n == name).count(), 1, "{name}");
        }
        assert_eq!(names.len(), 3);
    }
}
