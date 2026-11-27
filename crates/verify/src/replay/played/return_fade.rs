//! Native D1 passage timing and palette publication, from a pending touch.

use super::{Sources, camera, colours, frame, movement, seed, word};
use crate::replay::result::Outcome;
use crate::source;
use data::rom::{Rom, RomAddress};
use game::production::{
    AdvanceInput, PassageState, Production, ProductionContent, ProductionOutputs, ProductionRules,
    ProductionSeed,
};
use interface_hud::{Dropped, NewArea, OutOfPlayInputs};
use types::Parts as _;
use world_doors::{Activity, Doors, FadeTime, Finishing, Inputs, Passage, Phase, Scene};

#[test]
#[ignore = "pinned record cache and player ROM"]
fn native_return_darkening() -> Result<(), Outcome> {
    prove(&super::tests::sources())
}

fn prove(sources: &Sources) -> Result<(), Outcome> {
    let lock = super::read_lock(sources)?;
    let witnesses = super::witnesses::read(super::lock_root(sources)?)?;
    let fade = &witnesses.shaft()?.return_fade;
    let window = &fade.window;
    let mut rows = super::open_rows(sources, &lock, &window.row)?;
    let mut previous = rows.record(window.first.saturating_sub(1))?;
    let (door, mut scene) = origin(&previous, sources)?;
    let mut production = admit(Doors::Passing(Passage::START), &scene)?;
    let mut physical: Vec<_> = previous.cgram.to_vec();
    let mut doors = Doors::Passing(Passage::START);
    let mut clock = frame::seed(&previous.wram)?;
    let mut hud = crate::replay::hud::decode_state(&previous.wram)?;
    let mut timing = super::return_timing::ReturnTiming::default();
    let mut restored: Option<(Doors, Scene, super::return_timing::ReturnTiming)> = None;
    let mut early = 0_usize;
    let mut prepared = 0_usize;
    for cut in window.first..=window.last {
        let current = rows.record(cut)?;
        let publishes = super::finished(&current);
        let ready = timing.ready(
            Some(door.through),
            doors,
            publishes,
            current.nmi().map(|nmi| nmi.interrupted_pc),
        );
        let inputs = Inputs {
            touched: false,
            turn_ready: ready,
            publishes,
            sounds_sent: super::sounds_sent(&previous.wram)?,
            music_ready: super::music_ready(&previous.wram)?,
            timer_running: matches!(hud.countdown, interface_hud::Countdown::Begun(_)),
            items_dark: door.items_dark,
            doorway: Some(door.arrival),
            groups: colours::groups(),
        };
        let exports = advance(&mut production, &inputs, cut)?;
        scene = exports.scene;
        doors = exports.doors;
        hud.out_of_play(&OutOfPlayInputs {
            mini_map_turned_on: exports.report.minimap_on,
            new_area: NewArea::SameArea,
            dropped: Dropped::Nothing,
        });
        if exports.report.finishing == Finishing::Finished {
            clock.count.0 = clock.count.0.wrapping_add(1);
            clock.finished_frames = clock.finished_frames.wrapping_add(1);
            physical = scene
                .palette
                .screen
                .iter()
                .map(|colour| colour.bits())
                .collect();
        }
        if let Some((saved_doors, saved_scene, mut saved_timing)) = restored.take() {
            let ready_after_restore = saved_timing.ready(
                Some(door.through),
                saved_doors,
                publishes,
                current.nmi().map(|nmi| nmi.interrupted_pc),
            );
            let mut restored_production = admit(saved_doors, &saved_scene)?;
            let after_restore = advance(
                &mut restored_production,
                &Inputs {
                    turn_ready: ready_after_restore,
                    ..inputs
                },
                cut,
            )?;
            check(
                cut,
                "doors.restored-publication",
                after_restore == exports && saved_timing == timing,
            )?;
        }
        if ready && !publishes {
            restored = Some((doors, scene, timing));
        }
        let complete = ready || publishes;
        compare(
            &current,
            cut,
            doors,
            &scene,
            clock,
            &hud,
            &physical,
            exports.report,
            complete,
            fade,
        )?;
        prepared = prepared.saturating_add(usize::from(cut >= fade.first_palette && complete));
        early = early.saturating_add(usize::from(ready && !publishes));
        previous = current;
    }
    assert_eq!(early, fade.interrupted);
    assert_eq!(prepared, fade.prepared);
    Ok(())
}

#[expect(
    clippy::too_many_arguments,
    reason = "these independent owners are compared at the same cut"
)]
fn compare(
    current: &oracle_record::FrameRecord,
    cut: u64,
    doors: Doors,
    scene: &Scene,
    clock: game::production::played::frame::State,
    hud: &interface_hud::HudState,
    physical: &[u16],
    report: world_doors::Report,
    complete: bool,
    fade: &super::witnesses::shaft::Fade,
) -> Result<(), Outcome> {
    let expected_phase = match read(&current.wram, 0x099c)? {
        0xe29e => Phase::Quieting { asked: true },
        0xe2db => Phase::Darkening(FadeTime(read(&current.wram, 0xc400)?)),
        0xe2f7 => Phase::Squaring,
        _ => {
            return Err(super::data_failure(
                Some(cut),
                "return door phase observation",
            ));
        }
    };
    check(
        cut,
        "doors.phase",
        matches!(doors, Doors::Passing(passage) if passage.phase == expected_phase),
    )?;
    let expected_clock = frame::seed(&current.wram)?;
    check(
        cut,
        "finished-frames",
        clock.finished_frames == expected_clock.finished_frames,
    )?;
    check(cut, "frame-count", clock.count == expected_clock.count)?;
    check(
        cut,
        "hud.countdown",
        hud.countdown == crate::replay::hud::countdown_origin(&current.wram)?,
    )?;
    check(
        cut,
        "movement.position",
        scene.her == movement::seed(&current.wram)?.position,
    )?;
    if cut < fade.first_palette {
        return check(
            cut,
            "doors.timer-draw",
            report.runs[Activity::Timer] == (cut == fade.timer_draw),
        );
    }
    let expected = colours::palette(&current.wram)?;
    check(
        cut,
        "doors.colours.goal",
        scene.palette.goal == expected.goal,
    )?;
    if complete {
        check(
            cut,
            "doors.colours.screen",
            scene.palette.screen == expected.screen,
        )?;
    }
    check(
        cut,
        "doors.colours.published",
        physical == current.cgram.as_ref(),
    )
}

fn origin(
    previous: &oracle_record::FrameRecord,
    sources: &Sources,
) -> Result<(game::production::played::passage::Doorway, Scene), Outcome> {
    let image = Rom::new(std::fs::read(&sources.rom).unwrap()).unwrap();
    let rooms = seed::rooms(&image)?;
    let content = super::connected::Content {
        movement: movement::Content::load(&sources.rom)?,
        appearance: super::appearance::Content::load(&sources.rom)?,
    };
    let room = seed::room(previous, &content, &rooms.world)?.1;
    let pointer = read(&previous.wram, 0x078d)?;
    let identity = import::world::doorway(RomAddress::new(0x83, pointer)).unwrap();
    let door = *room
        .doorways
        .iter()
        .find(|door| door.through == identity)
        .unwrap();
    let camera = camera::seed(&previous.wram, &camera::Content)?;
    let scene = Scene {
        her: movement::seed(&previous.wram)?.position,
        camera: camera.position(),
        seen: types::Pair {
            x: camera.seen().x.pixel(),
            y: camera.seen().y.pixel(),
        },
        palette: colours::palette(&previous.wram)?,
    };
    Ok((door, scene))
}

fn admit(doors: Doors, scene: &Scene) -> Result<Production, Outcome> {
    Production::admit(
        ProductionSeed::Doors(Box::new(PassageState {
            doors,
            scene: *scene,
        })),
        ProductionContent::Doors,
        ProductionRules::Doors(world_doors::Rules::default()),
    )
    .map_err(|_| super::data_failure(None, "return door admission"))
}

fn advance(
    production: &mut Production,
    inputs: &Inputs,
    cut: u64,
) -> Result<world_doors::Exports, Outcome> {
    match production.advance(AdvanceInput::Doors(inputs)) {
        Ok(ProductionOutputs::Doors(exports)) => Ok(*exports),
        _ => Err(super::data_failure(
            Some(cut),
            "return door production step",
        )),
    }
}

fn read(wram: &[u8], offset: u16) -> Result<u16, Outcome> {
    word(
        wram,
        source::WramWord {
            name: "return fade observation",
            offset,
        },
    )
}

fn check(cut: u64, field: &str, same: bool) -> Result<(), Outcome> {
    if same {
        Ok(())
    } else {
        Err(Outcome::Difference {
            cut,
            field: field.to_owned(),
        })
    }
}
