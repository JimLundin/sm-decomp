use data::pose::animation::{AnimationDelays, DelayList};
use data::pose::transition::PoseTransitions;
use data::pose::{MovementType, PoseFacing, Poses};

use super::*;
use crate::pose;
use crate::state::samus::{Animation, PoseRecord, Speed};

/// The turn's pose: turning on the ground, its list ending in an auto-jump transition.
const TURN: PoseId = PoseId(0x25);
/// The pose the turn ends in: standing facing left.
const STANDING_LEFT: PoseId = PoseId(2);
/// A pose whose list loops.
const LOOPING: PoseId = PoseId(1);

/// Pose definition bytes: facing, movement type, next pose, shot, then four more.
fn definition(facing: u8, movement: u8) -> [u8; 8] {
    [facing, movement, 0xFF, 2, 0, 0, 0x15, 0]
}

/// Every pose up to the turn standing facing right but 2 (standing facing left) and the turn
/// (turning on the ground facing left); pose 1 loops on 3 and 4, pose 2 shows 5 frames, the
/// turn's list is the original's: 2, 2, 2, then the transition to pose 2.
fn content() -> Content {
    let bytes: Vec<u8> = (0..=TURN.0)
        .map(|pose| match PoseId(pose) {
            STANDING_LEFT => definition(4, 0),
            TURN => definition(4, 0x0E),
            _ => definition(8, 0),
        })
        .collect::<Vec<_>>()
        .concat();
    let list = |bytes: &[u8]| DelayList::decode(bytes).unwrap();
    let lists = (0..=TURN.0)
        .map(|pose| match PoseId(pose) {
            LOOPING => list(&[0x03, 0x04, 0xFF]),
            STANDING_LEFT => list(&[0x05, 0xFF]),
            TURN => list(&[0x02, 0x02, 0x02, 0xF8, 0x02]),
            _ => list(&[0x08, 0xFF]),
        })
        .collect();
    Content {
        poses: Poses::decode(&bytes).unwrap(),
        transitions: PoseTransitions::new(Vec::new()),
        delays: AnimationDelays::new(lists, list(&[0x01, 0xFF])),
        slopes: data::level::slope::Slopes::default(),
        enemy_programs: data::enemy::program::EnemyPrograms::default(),
        eye_lists: data::enemy::eye::EyeLists::default(),
        face_block_lists: data::enemy::face_block::FaceBlockLists::default(),
        samus_spritemaps: data::pose::spritemap::SamusSpritemaps::default(),
        samus_assets: data::assets::samus::SamusAssets::default(),
        samus_tiles: data::pose::tiles::SamusTiles::default(),
        graphics: data::pose::tiles::Graphics::default(),
        visor_colours: data::palette::VisorColours::default(),
        face_block_glow: data::enemy::face_block::GlowColours::default(),
        room_plms: crate::plm::RoomPlms::default(),
        room_doors: crate::collision::RoomDoors::default(),
        blocks: crate::blocks_world::BlockContent::default(),
        hud: crate::hud_world::HudContent::default(),
        tile_tables: data::tileset::TileTables::default(),
        message_boxes: data::message::MessageBoxes::default(),
    }
}

fn samus(pose: PoseId, movement: MovementType, frame: u16, timer: u16) -> Samus {
    Samus {
        pose,
        previous: PoseRecord {
            pose,
            facing: PoseFacing::Left,
            movement,
        },
        movement,
        animation: Animation {
            frame: AnimationFrame(frame),
            timer: AnimationTimer(timer),
            ..Animation::default()
        },
        ..Samus::default()
    }
}

/// One frame's animation and pose update, with no request from input.
fn frame(samus: &mut Samus, content: &Content) {
    let requests = PoseRequests::default();
    animate(samus, &Controller::default(), requests, content);
    pose::change(samus, requests.ordinary, content);
}

#[test]
fn the_turns_list_ends_it_in_standing_on_its_last_frame() {
    let content = content();
    let mut samus = samus(TURN, MovementType::TurningAroundOnGround, 0, 2);
    // Frames 0 to 2 show two frames each: the turn holds for five more frames.
    for _ in 0..5 {
        frame(&mut samus, &content);
        assert_eq!(samus.pose, TURN);
        assert_eq!(samus.super_special, SuperSpecial::default());
    }
    assert_eq!(samus.animation.frame, AnimationFrame(2));
    assert_eq!(samus.animation.timer, AnimationTimer(1));
    // The sixth reaches the transition: standing, from the new pose's first frame.
    frame(&mut samus, &content);
    assert_eq!(
        samus.super_special,
        SuperSpecial {
            pose: Some(STANDING_LEFT),
            command: SuperSpecialCommand::TransitionAnimationFinished,
        }
    );
    assert_eq!(samus.pose, STANDING_LEFT);
    assert_eq!(samus.movement, MovementType::Standing);
    assert_eq!(samus.animation.frame, AnimationFrame(0));
    assert_eq!(samus.animation.timer, AnimationTimer(5));
    assert_eq!(samus.previous.pose, STANDING_LEFT);
    assert_eq!(samus.last_different.pose, TURN);
}

#[test]
fn a_looping_list_goes_back_to_its_start() {
    let content = content();
    let mut samus = samus(LOOPING, MovementType::Standing, 0, 1);
    let mut seen = Vec::new();
    for _ in 0..8 {
        frame(&mut samus, &content);
        seen.push((samus.animation.frame.0, samus.animation.timer.0));
    }
    assert_eq!(
        seen,
        [
            (1, 4),
            (1, 3),
            (1, 2),
            (1, 1),
            (0, 3),
            (0, 2),
            (0, 1),
            (1, 4)
        ]
    );
    assert_eq!(samus.pose, LOOPING);
}

#[test]
fn a_timer_counted_past_zero_still_advances_the_frame() {
    let content = content();
    let mut samus = samus(LOOPING, MovementType::Standing, 0, 0);
    frame(&mut samus, &content);
    assert_eq!(samus.animation.frame, AnimationFrame(1));
    assert_eq!(samus.animation.timer, AnimationTimer(4));
}

/// `$91:B491`, the wall jump's list: its start-up, the instruction at frame 2, then the
/// spin's, the space jump's and the screw attack's sequences.
fn wall_jump_list() -> DelayList {
    let mut bytes = vec![0x05, 0x05, 0xFB];
    bytes.extend([0x03, 0x02, 0x03, 0x02, 0x03, 0x02, 0x03, 0x02, 0xFE, 0x08]);
    bytes.extend([0x02, 0x01, 0x02, 0x01, 0x02, 0x01, 0x02, 0x01, 0xFE, 0x08]);
    bytes.extend([0x01; 0x18]);
    bytes.extend([0xFE, 0x18]);
    DelayList::decode(&bytes).unwrap()
}

#[test]
fn the_wall_jumps_instruction_picks_its_sequence_by_her_items() {
    let list = wall_jump_list();
    let cases = [
        (&[][..], 3, 3),
        (&[Item::SpaceJump][..], 0x0D, 2),
        (&[Item::ScrewAttack][..], 0x17, 1),
        (&[Item::SpaceJump, Item::ScrewAttack][..], 0x17, 1),
        // Out of liquids the gravity suit changes nothing.
        (&[Item::GravitySuit][..], 3, 3),
    ];
    for (items, frame, timer) in cases {
        let mut samus = samus(PoseId(0x84), MovementType::WallJumping, 2, 0);
        for &item in items {
            samus.equipment.equipped.insert(item);
        }
        assert_eq!(
            list.at(samus.animation.frame),
            Some(DelayItem::Instruction(Instruction::WallJump))
        );
        assert!(run(
            Instruction::WallJump,
            &mut samus,
            PoseRequests::default()
        ));
        set_timer(&mut samus, &list, 0);
        assert_eq!(samus.animation.frame, AnimationFrame(frame));
        assert_eq!(samus.animation.timer, AnimationTimer(timer));
    }
}

#[test]
fn a_spin_jump_from_a_spin_or_wall_jump_starts_past_the_jump_from_the_floor() {
    const SPIN_RIGHT: PoseId = PoseId(1);
    const SPIN_LEFT: PoseId = PoseId(2);
    const WALL_JUMP: PoseId = PoseId(3);
    const STANDING: PoseId = PoseId(4);
    let bytes = [
        definition(8, 0),
        definition(8, 0x03),
        definition(4, 0x03),
        definition(8, 0x14),
        definition(8, 0),
    ]
    .concat();
    let list = |bytes: &[u8]| DelayList::decode(bytes).unwrap();
    let spin = list(&[0x04, 0x03, 0xFF]);
    let content = Content {
        poses: Poses::decode(&bytes).unwrap(),
        delays: AnimationDelays::new(
            vec![
                list(&[0x08, 0xFF]),
                spin.clone(),
                spin,
                list(&[0x08, 0xFF]),
                list(&[0x08, 0xFF]),
            ],
            list(&[0x01, 0xFF]),
        ),
        ..content()
    };
    for (from, movement, frame, timer) in [
        (SPIN_RIGHT, MovementType::SpinJumping, 1, 3),
        (WALL_JUMP, MovementType::WallJumping, 1, 3),
        (STANDING, MovementType::Standing, 0, 4),
    ] {
        let mut samus = samus(from, movement, 0, 8);
        pose::change(&mut samus, Some(SPIN_LEFT), &content);
        assert_eq!(samus.pose, SPIN_LEFT);
        assert_eq!(samus.animation.frame, AnimationFrame(frame));
        assert_eq!(samus.animation.timer, AnimationTimer(timer));
    }
}

#[test]
fn the_morphing_transition_picks_its_pose_by_items_then_y_speed() {
    // `$90:839A`: with the morph ball (bit $0004 of the items word) the pair `with` is read,
    // else `without`; each pair's `still` is read when her Y speed is zero.
    let pair = |still, moving| BySpeed {
        still: PoseId(still),
        moving: PoseId(moving),
    };
    let instruction = Instruction::ByItemsAndSpeed {
        items: ItemBits(0x0004),
        without: pair(0x10, 0x11),
        with: pair(0x1D, 0x31),
    };
    let chosen = |item: Option<Item>, speed: Speed| {
        let mut samus = Samus::default();
        if let Some(item) = item {
            samus.equipment.equipped.insert(item);
        }
        samus.movement = MovementType::CrouchingTransition;
        samus.motion.vertical_speed = speed.into();
        samus.motion.vertical = crate::state::samus::VerticalMotion::Down;
        run(instruction, &mut samus, PoseRequests::default());
        samus.super_special
    };
    let finished = |pose| SuperSpecial {
        pose: Some(PoseId(pose)),
        command: SuperSpecialCommand::TransitionAnimationFinished,
    };
    assert_eq!(chosen(Some(Item::MorphBall), Speed::ZERO), finished(0x1D));
    assert_eq!(
        chosen(Some(Item::MorphBall), Speed::new(0, 1)),
        finished(0x31)
    );
    assert_eq!(chosen(Some(Item::Bombs), Speed::ZERO), finished(0x10));
    assert_eq!(chosen(None, Speed::new(1, 0)), finished(0x11));
}

#[test]
fn the_end_is_read_on_the_frame_animate_ends_the_turn() {
    let content = content();
    let mut turn = samus(TURN, MovementType::TurningAroundOnGround, 0, 2);
    let controller = Controller::default();
    let none = PoseRequests::default();
    for _ in 0..5 {
        assert!(!ends_this_frame(&turn, &controller, none, &content));
        frame(&mut turn, &content);
    }
    assert!(ends_this_frame(&turn, &controller, none, &content));
    // A jump asked for holds the auto-jump transition back.
    let jump = PoseRequests {
        ordinary: Some(PoseId(0x4B)),
    };
    assert!(!ends_this_frame(&turn, &controller, jump, &content));
    // A looping list never ends.
    let looping = samus(LOOPING, MovementType::Standing, 1, 1);
    assert!(!ends_this_frame(&looping, &controller, none, &content));
}
