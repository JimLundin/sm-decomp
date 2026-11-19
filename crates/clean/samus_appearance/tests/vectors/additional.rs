//! Additional complete receipts beyond the supplied observation subsequences.
use super::suspended_vectors::opportunity_base;
use super::suspended_vectors::{checkpoint, make_work, restore, sound_names};
use super::{contact_clips, figure, owned_opportunity, read_file, state};
use samus_appearance::{AnimationDecision, Answer, Receipt, WorkStep};

#[test]
fn actual_selection_from_is_not_the_current_playing_figure() {
    let (file, _, _) = read_file();
    let mut work = make_work(&file, "Animation", "C-kept-run-advance");
    while !matches!(
        work.steps.get(work.consumed),
        Some(WorkStep::Animation(AnimationDecision::SelectedClip))
    ) {
        assert!(work.complete());
    }
    assert_eq!(work.state, state(2, 5, 0, 0));
    work.animation.as_mut().expect("owner inputs").selected_from =
        figure("Upright Falling Right Forward");
    let offered = work.offer().expect("initialization offer");
    assert_eq!(offered.answer, Answer::StartFirst);
    assert_eq!(
        offered.inputs.figure,
        figure("Upright Falling Right Forward")
    );
    work = restore(work);
    work.animation.as_mut().expect("owner inputs").selected_from =
        figure("Upright Running Right Forward");
    work.finish();
    assert_eq!(work.state, state(0, 7, 0, 0));
    let initialized = work
        .completed
        .iter()
        .find_map(|r| match r {
            Receipt::Animation(d) if d.rule == AnimationDecision::SelectedClip => Some(d),
            _ => None,
        })
        .expect("completed initialization");
    assert_eq!(
        initialized.inputs.figure,
        figure("Upright Falling Right Forward")
    );
    assert_eq!(initialized.answer, Answer::StartFirst);
}

#[test]
fn contact_control_and_repeated_drawing_survive_every_pause_in_order() {
    let (file, _, _) = read_file();
    let contacts = contact_clips(&file);
    let contact = owned_opportunity(opportunity_base(&file, "C-contact"), &contacts);
    let draw = owned_opportunity(opportunity_base(&file, "C-end-wall"), &contacts);
    let mut work = make_work(&file, "Animation", "C-branch-spin");
    work.steps.insert(0, WorkStep::Appearance(contact));
    work.steps.push(WorkStep::Appearance(draw.clone()));
    work.steps.push(WorkStep::Appearance(draw));
    let mut expected = work.clone();
    expected.finish();
    assert_eq!(expected.state, state(2, 7, 0, 0));
    assert_eq!(
        sound_names(&expected),
        ["Footstep", "PlainSpin", "SpinEnded", "SpinEnded"]
    );
    assert_eq!(expected.graphics.len(), 1);
    loop {
        let _ = work.offer();
        work = checkpoint(work, &expected);
        if !work.complete() {
            break;
        }
        work = checkpoint(work, &expected);
    }
    assert_eq!(work, expected);
}
