//! Verification-only finite collision host over the actual Composed Session.
//! Prepared backend callbacks are contract witnesses, not vanilla backend proof.
#![forbid(unsafe_code)]
use crate::{Composed, rom::RomRecord};
use std::collections::BTreeMap;
use {
    crate::game::room_entry::EntryPoll,
    sm_game::player_collision::effects::{
        Content, Effects, ImportedCollisionReads, MechanismHeader,
    },
};

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum CollisionEvent {
    PoseTable,
    WalkedTable,
    Special(u8, bool),
    Bomb(u8),
    Mechanism {
        header: MechanismHeader,
        displacement: i32,
        x: u16,
        y: u16,
        x_radius: u16,
        y_radius: u16,
        cursor: u16,
    },
    Door,
    VerticalBts(u16),
    CancelSpeedBoost,
    BossDefeated,
}
#[derive(Default)]
pub(crate) struct CollisionContract {
    pub(crate) on_cancel: Option<fn(&mut Composed)>,
    pub(crate) on_mechanism: Option<fn(&mut Composed, MechanismHeader, &mut i32) -> u8>,
    pub(crate) on_door: Option<fn(&mut Composed) -> EntryPoll>,
    pub(crate) on_vertical_bts: Option<fn(&mut Composed, u16) -> u16>,
    pub(crate) events: Vec<CollisionEvent>,
    pub(crate) special_headers: BTreeMap<(u8, bool), MechanismHeader>,
    pub(crate) bomb_headers: BTreeMap<u8, MechanismHeader>,
    // Only non-owned bytes may be authored here. Owner projection always wins.
    external: BTreeMap<usize, u8>,
}
impl Composed {
    pub(crate) fn prepare_collision_external(&mut self, at: usize, byte: u8) {
        assert!(at < 0x20000, "collision fixture outside physical RAM");
        assert!(
            crate::game::session_encoding::projected_byte(&self.world.player, at).is_none(),
            "collision fixture must edit actual owner"
        );
        self.collision_contract.external.insert(at, byte);
    }
    fn collision_byte(&self, at: usize) -> u8 {
        assert!(
            at < 0x20000,
            "collision imported read outside physical RAM: {at:x}"
        );
        crate::game::session_encoding::projected_byte(&self.world.player, at).unwrap_or_else(|| {
            *self
                .collision_contract
                .external
                .get(&at)
                .unwrap_or_else(|| panic!("unprepared collision imported byte {at:x}"))
        })
    }
    fn collision_word(&self, at: usize) -> u16 {
        u16::from_le_bytes([self.collision_byte(at), self.collision_byte(at + 1)])
    }
}
impl Content for Composed {
    type Record = RomRecord;
    fn pose_table(&mut self) -> RomRecord {
        self.collision_contract
            .events
            .push(CollisionEvent::PoseTable);
        RomRecord {
            image: self.image.clone(),
            base: 0x11 * 0x8000 + 0x3629,
        }
    }
    fn walked_pose_table(&mut self) -> RomRecord {
        self.collision_contract
            .events
            .push(CollisionEvent::WalkedTable);
        RomRecord {
            image: self.transition_image(sm_game::visual_host::Source::Loaded),
            base: 0x11 * 0x8000 + 0x6b74,
        }
    }
    fn special_header(&mut self, behavior: u8, solid: bool) -> MechanismHeader {
        self.collision_contract
            .events
            .push(CollisionEvent::Special(behavior, solid));
        *self
            .collision_contract
            .special_headers
            .get(&(behavior, solid))
            .expect("unprepared collision special header")
    }
    fn bomb_header(&mut self, behavior: u8) -> MechanismHeader {
        self.collision_contract
            .events
            .push(CollisionEvent::Bomb(behavior));
        *self
            .collision_contract
            .bomb_headers
            .get(&behavior)
            .expect("unprepared collision bomb header")
    }
}
impl ImportedCollisionReads for Composed {
    fn imported_level_word(&self, byte_offset: usize) -> u16 {
        let [low, high] = sm_game::player_collision::geometry::level_word_bytes(byte_offset);
        u16::from_le_bytes([self.collision_byte(low), self.collision_byte(high)])
    }
    fn imported_behavior_byte(&self, index: usize) -> u8 {
        self.collision_byte(
            0x16402usize
                .checked_add(index)
                .expect("behavior offset overflow"),
        )
    }
    fn imported_interactive_word(&self, byte_offset: usize) -> u16 {
        self.collision_word(
            0x17ecusize
                .checked_add(byte_offset)
                .expect("interactive offset overflow"),
        )
    }
}
impl Effects for Composed {
    #[cfg(any(test, sm_verify_dll))]
    fn record_launch(
        &mut self,
        direction: sm_game::launch_motion::MovementDirection,
        displacement: i32,
    ) {
        crate::game::player_collision_owned_checks::record_launch(direction, displacement);
    }
    fn apply_mechanism(&mut self, header: MechanismHeader, displacement: &mut i32) -> u8 {
        let p = &self.world.player.player;
        self.collision_contract
            .events
            .push(CollisionEvent::Mechanism {
                header,
                displacement: *displacement,
                x: p.x,
                y: p.y,
                x_radius: p.contact.x_radius,
                y_radius: p.contact.radius,
                cursor: self.world.player.room.collision_cursor,
            });
        (self
            .collision_contract
            .on_mechanism
            .expect("unprepared collision mechanism effect"))(self, header, displacement)
    }
    fn enter_door(&mut self) -> EntryPoll {
        self.collision_contract.events.push(CollisionEvent::Door);
        (self
            .collision_contract
            .on_door
            .expect("unprepared collision door effect"))(self)
    }
    fn cancel_speed_boost(&mut self) {
        self.collision_contract
            .events
            .push(CollisionEvent::CancelSpeedBoost);
        if let Some(callback) = self.collision_contract.on_cancel {
            callback(self);
        }
        sm_game::samus_palette::cancel_speed_boost(self);
    }
    fn boss_defeated(&mut self) -> bool {
        self.collision_contract
            .events
            .push(CollisionEvent::BossDefeated);
        crate::game::progress_flags::boss_defeated(self, 1)
    }
    fn vertical_slope_bts(&mut self, block: u16) -> u16 {
        self.collision_contract
            .events
            .push(CollisionEvent::VerticalBts(block));
        (self
            .collision_contract
            .on_vertical_bts
            .expect("unprepared physical slope read"))(self, block)
    }
    fn invalid_clearance_flags(&mut self, flags: u16) {
        panic!("invalid collision clearance flags {flags:x}");
    }
}

#[cfg(test)]
mod checks {
    use super::*;
    use {
        crate::game::player_collision_owned_checks as owned,
        sm_game::actors::access::{AccessMode, ActorAccess, ActorKey},
    };
    pub(super) fn empty_room() -> Composed {
        let mut h = Composed::new();
        let s = &mut h.world.player;
        s.room.width = 16;
        s.room.height = 16;
        s.player.x = 100;
        s.player.y = 100;
        s.player.contact.x_radius = 2;
        s.player.contact.radius = 2;
        s.actors.schedule.interactive_write_position = 0;
        s.actors.schedule.interactive.fill(0xffff);
        h
    }
    #[test]
    fn unchecked_launch() {
        owned::unchecked_launch(&mut empty_room());
    }
    #[test]
    fn frame_reads_owned_clock() {
        owned::same_interval_frame(&mut empty_room());
    }
    #[test]
    fn radius_reads_owned_player() {
        let mut h = empty_room();
        let actor = ActorKey::from_legacy(0);
        h.edit_actor(actor, AccessMode::LittleEndian, |a| {
            a.x.whole = 120;
            a.y.whole = 100;
            a.radius.x = 2;
            a.radius.y = 2;
            a.properties = 0x8000;
        });
        h.world.player.actors.schedule.interactive[0] = 0;
        h.world.player.actors.schedule.interactive_write_position = 2;
        owned::same_interval_collision_radius(&mut h, actor);
    }
    #[test]
    fn shared_sequence_real_motion() {
        owned::shared_sequence_move_y(&mut empty_room());
    }
    #[test]
    fn clearance_probe_keeps_position() {
        owned::pose_probe_does_not_commit(&mut empty_room());
    }
    #[test]
    fn control_skip_clears_speed_without_motion() {
        use crate::game::player_collision::operations::{
            self, ControlResult, HorizontalTransition,
        };
        let mut h = empty_room();
        h.world.player.player.contact.base_run.whole = 5;
        h.world.player.player.contact.base_run.fraction = 0x8000;
        h.world.player.player.collision.flags = 0xffff;
        let result = operations::horizontal_control(&mut h, HorizontalTransition::ClearAndSkip);
        assert_eq!(result, ControlResult::Skipped);
        let p = &h.world.player.player;
        assert_eq!((p.x, p.y), (100, 100));
        assert_eq!(
            (
                p.contact.base_run.whole,
                p.contact.base_run.fraction,
                p.collision.flags
            ),
            (0, 0, 0)
        );
    }
    #[test]
    fn grapple_rejects_empty_room_and_publishes_no_actor() {
        use crate::game::player_collision::operations::{self, JumpKind, JumpRequest};
        let mut h = empty_room();
        h.world.player.player.movement_and_facing = 8;
        assert!(
            !operations::jump_admission(
                &mut h,
                JumpRequest {
                    kind: JumpKind::Grapple,
                    amount: 1 << 16
                }
            ),
            "collision.synthetic.grapple_refreshed_inputs"
        );
        assert_eq!(h.world.player.actors.controls.shake_request, 0xffff);
        assert_eq!(
            (h.world.player.player.x, h.world.player.player.y),
            (100, 100)
        );
    }
    #[test]
    fn walking_captures_transition_before_current_pose_lookup() {
        use crate::game::player_collision::operations;
        let mut h = empty_room();
        h.world.player.player.pose = 2;
        h.world.player.player.movement_and_facing = 0x108;
        h.world.player.player.collision.solid_actor_flags = 1;
        h.world.player.player.transitions.ordinary =
            crate::game::player_transitions::state::PoseRequestWord::from_raw(3);
        h.byte(0x91, 0xb629 + 2 * 8 + 3, 5);
        h.byte(0x91, 0xb629 + 3 * 8 + 3, 7);
        h.word(0x91, 0xeb74 + 5 * 2, 0x1234);
        h.word(0x91, 0xeb74 + 7 * 2, 0x5678);
        operations::walked_obstacle(&mut h);
        assert_eq!(h.world.player.player.transitions.ordinary.to_raw(), 0x1234);
        assert_eq!(h.world.player.player.collision.solid_actor_flags, 0);
        assert_eq!(
            h.collision_contract.events,
            [CollisionEvent::WalkedTable, CollisionEvent::PoseTable]
        );
    }
    #[test]
    fn equal_radius_pose_clears_prior_probe_flags() {
        use crate::game::player_collision::operations;
        let mut h = empty_room();
        h.world.player.player.pose = 2;
        h.world.player.player.history.previous.pose = 3;
        h.world.player.player.clearance.actor_flags = 3;
        h.world.player.player.clearance.terrain_flags = 3;
        h.byte(0x91, 0xb629 + 2 * 8 + 6, 2);
        h.byte(0x91, 0xb629 + 3 * 8 + 6, 2);
        operations::fit_pose(&mut h);
        let p = &h.world.player.player;
        assert_eq!((p.clearance.actor_flags, p.clearance.terrain_flags), (0, 0));
        assert_eq!((p.x, p.y, p.contact.radius), (100, 100, 2));
        assert_eq!(
            h.collision_contract.events,
            [CollisionEvent::PoseTable, CollisionEvent::PoseTable]
        );
    }
    fn special_room() -> Composed {
        let mut h = empty_room();
        h.world.player.room.terrain.cells_mut()[102].level = 0x3000;
        h.world.player.room.terrain.cells_mut()[102].behavior = 0x80;
        h.collision_contract
            .special_headers
            .insert((0x80, false), MechanismHeader(0x1234));
        h
    }
    #[test]
    fn synthetic_mechanism_commit_reads_live_owner_and_local_displacement() {
        use crate::game::player_collision::operations::{self, Axis, DisplacementRequest};
        let mut h = special_room();
        h.collision_contract.on_mechanism = Some(|h, header, displacement| {
            assert_eq!(header, MechanismHeader(0x1234));
            assert_eq!(*displacement, 1 << 16);
            let p = &mut h.world.player.player;
            p.x = 200;
            p.x_fraction = 0x8000;
            p.contact.x_radius = 7;
            p.pose = 0x81;
            *displacement = 0x4000;
            0
        });
        operations::displace(
            &mut h,
            DisplacementRequest::TerrainOnly {
                axis: Axis::X,
                amount: 1 << 16,
            },
        );
        let p = &h.world.player.player;
        assert_eq!(
            (p.x, p.x_fraction, p.contact.x_radius, p.pose),
            (200, 0xc000, 7, 0x81),
            "collision.synthetic.live_owner_after_setup"
        );
        assert!(
            matches!(
                h.collision_contract.events.as_slice(),
                [
                    CollisionEvent::Special(0x80, false),
                    CollisionEvent::Mechanism {
                        header: MechanismHeader(0x1234),
                        displacement: 65536,
                        x: 100,
                        y: 100,
                        x_radius: 2,
                        y_radius: 2,
                        cursor: 102
                    }
                ]
            ),
            "collision.synthetic.setup_entry_order"
        );
    }
    #[test]
    fn synthetic_grapple_mechanism_refreshes_jump_inputs() {
        use crate::game::player_collision::operations::{self, JumpKind, JumpRequest};
        let mut h = special_room();
        h.world.player.player.movement_and_facing = 8;
        h.collision_contract.on_mechanism = Some(|h, _, _| {
            h.world.player_inputs.jump_button = 0x80;
            h.world.player_inputs.pressed = 0x80;
            1
        });
        assert!(
            operations::jump_admission(
                &mut h,
                JumpRequest {
                    kind: JumpKind::Grapple,
                    amount: 1 << 16
                }
            ),
            "collision.synthetic.grapple_refreshed_inputs"
        );
        assert!(
            h.collision_contract
                .events
                .iter()
                .any(|e| matches!(e, CollisionEvent::Mechanism { .. }))
        );
    }
    #[test]
    fn synthetic_wall_mechanism_rereads_pose_after_initial_readiness() {
        use crate::game::player_collision::operations::{self, JumpKind, JumpRequest};
        let mut h = special_room();
        h.world
            .player
            .player
            .history
            .last_different
            .movement_and_facing = 3 << 8;
        h.world.player_inputs.held = 0x200;
        h.collision_contract.on_mechanism = Some(|h, _, _| {
            h.world.player.player.pose = 0x81;
            h.world.player.player.animation_clock.frame = 99;
            h.world.player_inputs.jump_button = 0x80;
            h.world.player_inputs.pressed = 0x80;
            1
        });
        assert!(!operations::jump_admission(
            &mut h,
            JumpRequest {
                kind: JumpKind::Wall,
                amount: 1 << 16
            }
        ));
        assert_eq!(
            (
                h.world.player.player.animation_clock.frame,
                h.world.player.player.animation_clock.remaining
            ),
            (26, 1),
            "collision.synthetic.wall_late_pose_initial_readiness"
        );
        assert!(
            h.collision_contract
                .events
                .iter()
                .any(|e| matches!(e, CollisionEvent::Mechanism { .. }))
        );
    }
    #[test]
    fn synthetic_mechanism_unwind_keeps_partial_owner_publication() {
        use crate::game::player_collision::operations::{self, Axis, DisplacementRequest};
        let mut h = special_room();
        h.collision_contract.on_mechanism = Some(|h, _, _| {
            h.world.player.player.pose = 0x81;
            h.world.player.player.x = 222;
            std::panic::panic_any("collision.synthetic.partial_owner");
        });
        let error = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            operations::displace(
                &mut h,
                DisplacementRequest::TerrainOnly {
                    axis: Axis::X,
                    amount: 1 << 16,
                },
            );
        }))
        .expect_err("callback must unwind");
        assert_eq!(
            error.downcast_ref::<&str>(),
            Some(&"collision.synthetic.partial_owner")
        );
        assert_eq!(
            (h.world.player.player.x, h.world.player.player.pose),
            (222, 0x81),
            "collision.synthetic.unwind_partial_owner"
        );
        assert_eq!(h.world.player.room.collision_cursor, 102);
    }
    fn cancellation_room() -> Composed {
        let mut h = empty_room();
        let contact = &mut h.world.player.player.contact;
        contact.base_run.whole = 7;
        contact.extra_run.whole = 5;
        contact.acceleration_mode = 2;
        contact.momentum = 0;
        h
    }
    fn assert_pending_speed(h: &Composed) {
        let c = &h.world.player.player.contact;
        assert_eq!(
            (c.base_run.whole, c.extra_run.whole, c.acceleration_mode),
            (7, 5, 2)
        );
    }
    fn assert_cleared_speed(h: &Composed) {
        let c = &h.world.player.player.contact;
        assert_eq!(
            (
                c.base_run.whole,
                c.base_run.fraction,
                c.extra_run.whole,
                c.extra_run.fraction,
                c.acceleration_mode
            ),
            (0, 0, 0, 0, 0)
        );
    }
    #[test]
    fn actor_cancellation_precedes_position_and_launch() {
        use crate::game::player_collision::operations::{
            self, HorizontalDirection, HorizontalRequest, Obstruction,
        };
        let mut h = cancellation_room();
        let actor = ActorKey::from_legacy(0);
        h.edit_actor(actor, AccessMode::LittleEndian, |a| {
            a.x.whole = 112;
            a.y.whole = 100;
            a.radius.x = 2;
            a.radius.y = 2;
            a.properties = 0x8000;
        });
        h.world.player.actors.schedule.interactive[0] = 0;
        h.world.player.actors.schedule.interactive_write_position = 2;
        h.collision_contract.on_cancel = Some(|h| {
            assert_eq!(
                h.world.player.player.x, 100,
                "collision.order.actor_cancel_before_launch"
            );
            assert_eq!(
                owned::launches_so_far(),
                [],
                "collision.order.actor_cancel_before_launch"
            );
            assert_pending_speed(h);
        });
        let (result, launches) = owned::observed(|| {
            operations::horizontal(
                &mut h,
                HorizontalRequest {
                    base: 7 << 16,
                    direction: HorizontalDirection::Right,
                },
            )
        });
        assert_eq!(result.obstruction, Obstruction::SolidActor(actor));
        assert_eq!(h.world.player.player.x, 108);
        assert_eq!(launches, [(1, 8 << 16)]);
        assert_cleared_speed(&h);
        assert_eq!(
            h.collision_contract
                .events
                .iter()
                .filter(|e| matches!(e, CollisionEvent::CancelSpeedBoost))
                .count(),
            1
        );
    }
    #[test]
    fn terrain_cancellation_follows_position_and_launch() {
        use crate::game::player_collision::operations::{
            self, HorizontalDirection, HorizontalRequest, Obstruction,
        };
        let mut h = cancellation_room();
        h.world.player.room.terrain.cells_mut()[103].level = 0x8000;
        h.collision_contract.on_cancel = Some(|h| {
            assert_eq!(
                (h.world.player.player.x, h.world.player.player.x_fraction),
                (110, 0xffff),
                "collision.order.terrain_cancel_after_launch"
            );
            assert_eq!(
                owned::launches_so_far(),
                [(1, 10 << 16)],
                "collision.order.terrain_cancel_after_launch"
            );
            assert_pending_speed(h);
        });
        let (result, launches) = owned::observed(|| {
            operations::horizontal(
                &mut h,
                HorizontalRequest {
                    base: 7 << 16,
                    direction: HorizontalDirection::Right,
                },
            )
        });
        assert_eq!(result.obstruction, Obstruction::Terrain);
        assert_eq!(
            (h.world.player.player.x, h.world.player.player.x_fraction),
            (110, 0xffff)
        );
        assert_eq!(launches, [(1, 10 << 16)]);
        assert_cleared_speed(&h);
        assert_eq!(
            h.collision_contract
                .events
                .iter()
                .filter(|e| matches!(e, CollisionEvent::CancelSpeedBoost))
                .count(),
            1
        );
    }
}
