//! Actual producer/consumer checks: there is no codec or refresh between calls.
use super::{Event, game::player_projectiles as player, player_checks::prepared, room};

fn missile_content(h: &super::Composed) {
    // Actual projectile table index 1, direction-right launch record.
    for (cursor, value) in [
        (0x83f3, 0x8900),
        (0x8900, 37),
        (0x8906, 0x9200),
        (0x9200, 100),
        (0x9202, 0x9345),
        (0x9204, 0x0202),
    ] {
        h.word(0x93, cursor, value);
    }
}
fn pickup_then_missile() {
    let mut h = prepared();
    missile_content(&h);
    h.word(0x84, 0x9800, 5);
    room::command(&mut h, 8685993, 0x9800, 0);
    assert_eq!(
        (
            h.world.player.inventory.ammunition[0].current,
            h.world.player.inventory.ammunition[0].maximum
        ),
        (5, 5),
        "1040 pickup commits ammunition"
    );
    assert_eq!(h.world.player.messages.queued, 2);
    assert_eq!(h.events, [Event::RoomVisual(2), Event::RoomMusic(0x168)]);
    h.world.player.projectiles.controls.selected_weapon = 1;
    player::fire_missile(&mut h);
    assert_eq!(
        h.world.player.inventory.ammunition[0].current, 4,
        "1040 actual missile spends pickup"
    );
    let shot = &h.world.player.projectiles.shots[0];
    assert_eq!(
        (shot.kind, shot.damage, shot.x, shot.y),
        (0x8100, 37, 41, 33)
    );
}
fn pickup_then_beam() {
    let mut h = prepared();
    // Wave beam picks a distinct definition so a stale equipped-mask cannot pass.
    h.word(0x93, 0x83c3, 0x8a00);
    h.word(0x93, 0x8a00, 43);
    h.word(0x93, 0x8a06, 0x9000);
    h.word(0x84, 0x9800, 1);
    h.byte(0x84, 0x9802, 9);
    h.late_message = Some(12);
    room::command(&mut h, 8685744, 0x9800, 0);
    assert_eq!(h.world.player.inventory.beams.active, 1);
    assert_eq!(
        h.world.player.messages.queued, 12,
        "1041 late message operand observes music callback"
    );
    assert_eq!(h.events, [Event::RoomVisual(1), Event::RoomMusic(0x168)]);
    player::fire(&mut h);
    assert_eq!(
        (
            h.world.player.projectiles.shots[0].kind,
            h.world.player.projectiles.shots[0].damage
        ),
        (0x8001, 43),
        "1041 pickup changes actual beam"
    );
}
fn refill_then_missile() {
    let mut h = prepared();
    missile_content(&h);
    h.world.player.inventory.ammunition[0].maximum = 7;
    room::command(&mut h, 8686800, 0x9800, 0);
    assert_eq!(h.world.player.messages.queued, 0x16);
    assert_eq!(h.events, [Event::RoomControl(1, 0x16, 7)]);
    assert_eq!(
        h.world.player.inventory.ammunition[0].current, 7,
        "1042 actual station refills"
    );
    h.world.player.projectiles.controls.selected_weapon = 1;
    player::fire_missile(&mut h);
    assert_eq!(
        h.world.player.inventory.ammunition[0].current, 6,
        "1042 actual missile spends refill"
    );
    assert_eq!(h.world.player.projectiles.shots[0].damage, 37);
}
fn plant_then_launch() {
    let mut h = prepared();
    // Remove the prepared wall so a moved muzzle cannot collide during creation.
    for cell in h.world.player.room.terrain.cells_mut() {
        cell.level = 0;
    }
    h.world.player.mechanisms.slots[0].work0 = 67;
    h.world.player.mechanisms.slots[0].work1 = 81;
    h.world.player.player.invincibility = 3;
    room::hold(&mut h, 0);
    assert_eq!(
        (
            h.world.player.player.x,
            h.world.player.player.y,
            h.world.player.player.invincibility
        ),
        (67, 81, 0x13)
    );
    player::fire(&mut h);
    let shot = &h.world.player.projectiles.shots[0];
    assert_eq!(
        (shot.x, shot.y, shot.damage),
        (78, 82, 20),
        "1043 actual launch reads plant position"
    );
    assert_eq!(
        h.world.player.player.invincibility, 0x13,
        "contact protection is not projectile protection"
    );
    assert_eq!(h.world.player.projectiles.controls.protection, 10);
}
fn pickup_yield_resume_then_spend() {
    let mut h = prepared();
    missile_content(&h);
    h.word(0x84, 0x9800, 8685993u32 as u16);
    h.word(0x84, 0x9802, 5);
    h.word(0x84, 0x9804, 8685236u32 as u16);
    h.world.player.mechanisms.flags = 0x8000;
    let slot = &mut h.world.player.mechanisms.slots[0];
    slot.header = 1;
    slot.pre_action = 8684649u32 as u16;
    slot.instruction_delay = 1;
    slot.program = 0x9800;
    h.message_yields.extend([1, 0]);
    let mut pass = room::host::Pass::Instructions;
    assert!(
        matches!(room::advance(&mut h, &mut pass), room::host::RunResult::Yield(value) if value.get() == 1)
    );
    assert!(matches!(pass, room::host::Pass::DisplayMessage));
    assert_eq!(h.world.player.mechanisms.slots[0].program, 0x9804);
    assert_eq!(h.world.player.inventory.ammunition[0].current, 5);
    // A real consumer executes while the mechanism scheduler is suspended.
    h.world.player.projectiles.controls.selected_weapon = 1;
    player::fire_missile(&mut h);
    assert_eq!(h.world.player.inventory.ammunition[0].current, 4);
    assert!(matches!(
        room::advance(&mut h, &mut pass),
        room::host::RunResult::Complete
    ));
    assert!(matches!(pass, room::host::Pass::Instructions));
    assert_eq!(h.world.player.messages.queued, 0);
    assert_eq!(
        h.world.player.inventory.ammunition[0].current, 4,
        "1044 resume retains intervening actual missile spend"
    );
    assert!(h.events.contains(&Event::RoomMessage(2, 5)));
    assert!(h.events.contains(&Event::RoomMessage(2, 4)));
    h.world.player.projectiles.controls.cooldown = 0;
    player::fire_missile(&mut h);
    assert_eq!(
        h.world.player.inventory.ammunition[0].current, 3,
        "1044 resumed owner feeds next actual missile"
    );
    assert_eq!(h.world.player.projectiles.shots[1].damage, 37);
}
pub(super) fn run() {
    pickup_then_missile();
    pickup_then_beam();
    refill_then_missile();
    plant_then_launch();
    pickup_yield_resume_then_spend();
}
