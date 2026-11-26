//! The whole scene on invented data: from the reach at the elevator to the explosion's dispatch,
//! with interrupted cuts that must complete their turn once.

#[cfg(test)]
mod journey {
    use screens_blackout::{
        MUSIC_STOP_DELAY, Opportunity, Reach, Request, RoomPass, SoundChannel, advance,
    };
    use types::ceres_blackout::{
        BlackoutLevel, BlackoutLight, BlackoutPhase, BlackoutState, BlackoutTimer, FinishedPasses,
        FlightRoute, PreparedPicture, PublishedPicture, ShaftStage, TimerActivity,
        TimerCentiseconds, TimerSprite, WaitPasses, WorkPass,
    };

    /// No request at all.
    const NOTHING: [Request; 0] = [];

    /// Invented timer sprites, drawn on every pass.
    fn picture() -> PreparedPicture {
        PreparedPicture {
            timer_sprites: vec![TimerSprite(7), TimerSprite(3), TimerSprite(9)],
        }
    }

    fn room(reach: Reach) -> Opportunity {
        Opportunity::RoomWork(RoomPass {
            reach,
            picture: picture(),
        })
    }

    fn in_shaft() -> BlackoutState {
        BlackoutState {
            phase: BlackoutPhase::Shaft,
            light: BlackoutLight::Lit(BlackoutLevel::new(5).unwrap()),
            published: PublishedPicture {
                light: BlackoutLight::Lit(BlackoutLevel::new(5).unwrap()),
                timer_sprites: Vec::new(),
            },
            pass: WorkPass::Finished,
            timer: BlackoutTimer {
                time: TimerCentiseconds(5_000),
                activity: TimerActivity::Active,
            },
            shaft: ShaftStage::Rotating,
            finished: FinishedPasses(100),
        }
    }

    /// One whole pass: the room work, then its publication.
    fn whole_pass(state: &mut BlackoutState, reach: Reach) -> Vec<Request> {
        let mut requests = advance(state, room(reach));
        requests.extend(advance(state, Opportunity::Publication));
        requests
    }

    #[test]
    fn the_scene_runs_from_the_reach_to_the_dispatch() {
        let mut state = in_shaft();
        assert_eq!(whole_pass(&mut state, Reach::NotReached), NOTHING);
        let reached = whole_pass(&mut state, Reach::Reached(TimerCentiseconds(4_990)));
        assert_eq!(reached, vec![Request::StandAndLock]);
        assert_eq!(state.phase, BlackoutPhase::Hold(WaitPasses::WHOLE));
        assert_eq!(state.timer.time, TimerCentiseconds(4_990));

        for _ in 0..WaitPasses::WHOLE.get() {
            let requests = whole_pass(&mut state, Reach::Reached(TimerCentiseconds(1)));
            assert_eq!(requests, vec![Request::ContinueRoom]);
        }
        assert_eq!(state.phase, BlackoutPhase::Fade);
        assert_eq!(
            state.light,
            BlackoutLight::Lit(BlackoutLevel::new(5).unwrap())
        );
        assert_eq!(state.timer.time, TimerCentiseconds(4_990));

        for _ in 0..4 {
            whole_pass(&mut state, Reach::NotReached);
        }
        assert_eq!(
            state.light,
            BlackoutLight::Lit(BlackoutLevel::new(1).unwrap())
        );
        let before_blank = state.finished;

        assert_eq!(
            advance(&mut state, room(Reach::NotReached)),
            vec![Request::ContinueRoom]
        );
        assert_eq!(state.phase, BlackoutPhase::AwaitBlank);
        assert_eq!(state.light, BlackoutLight::ForcedBlank);
        for held in [
            Opportunity::Interrupted,
            Opportunity::Publication,
            room(Reach::NotReached),
        ] {
            assert_eq!(advance(&mut state, held), NOTHING);
        }
        assert_eq!(state.finished, before_blank);
        assert_eq!(
            state.published.light,
            BlackoutLight::Lit(BlackoutLevel::new(1).unwrap())
        );

        let route = FlightRoute::ZebesFlight;
        assert_eq!(
            advance(&mut state, Opportunity::VerticalBlankEnded),
            vec![
                Request::DisableShaftInterrupts,
                Request::PrepareSpriteOnly,
                Request::CaptureEscapeSave(route),
            ]
        );
        assert_eq!(advance(&mut state, Opportunity::Interrupted), NOTHING);
        assert_eq!(advance(&mut state, Opportunity::NewWork), NOTHING);
        assert_eq!(
            advance(&mut state, Opportunity::SaveCompleted),
            vec![
                Request::StopMusic(MUSIC_STOP_DELAY),
                Request::Silence(SoundChannel::One),
                Request::Silence(SoundChannel::Two),
                Request::Silence(SoundChannel::Three),
                Request::PrepareStationExplosion(route),
            ]
        );
        assert_eq!(state.timer.activity, TimerActivity::Retired);
        assert_eq!(state.timer.time, TimerCentiseconds(4_990));
        assert_eq!(state.shaft, ShaftStage::Cleared);
        assert_eq!(advance(&mut state, Opportunity::NewWork), NOTHING);

        assert_eq!(advance(&mut state, Opportunity::Publication), NOTHING);
        assert_eq!(state.finished, before_blank.counted_up());
        assert_eq!(state.published.light, BlackoutLight::ForcedBlank);
        assert_eq!(state.published.timer_sprites, picture().timer_sprites);
        assert_eq!(advance(&mut state, Opportunity::Publication), NOTHING);
        assert_eq!(state.finished, before_blank.counted_up());

        assert_eq!(
            advance(&mut state, Opportunity::NewWork),
            vec![Request::DispatchExplosion(route)]
        );
        assert_eq!(advance(&mut state, Opportunity::NewWork), NOTHING);
        assert_eq!(state.phase, BlackoutPhase::Dispatched(route));
    }

    #[test]
    fn a_prepared_wait_pass_survives_a_restore() {
        let mut state = in_shaft();
        state.phase = BlackoutPhase::Hold(WaitPasses::new(2).unwrap());
        advance(&mut state, room(Reach::NotReached));
        let saved = toml::to_string(&state).unwrap();
        let mut restored: BlackoutState = toml::from_str(&saved).unwrap();
        assert_eq!(restored, state);
        advance(&mut restored, Opportunity::Publication);
        assert_eq!(restored.published.timer_sprites, picture().timer_sprites);
        assert_eq!(restored.finished, FinishedPasses(101));
    }

    #[test]
    fn out_of_range_values_are_refused() {
        assert!(WaitPasses::new(0).is_none());
        assert!(WaitPasses::new(61).is_none());
        assert!(BlackoutLevel::new(0).is_none());
        assert!(BlackoutLevel::new(16).is_none());
        assert!(toml::from_str::<BlackoutState>("x").is_err());
    }
}
