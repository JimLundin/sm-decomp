use super::Obstacle;
use crate::state::room::{EnemyCommon, EnemyProperty};
use crate::state::slots::Frames;

#[test]
fn the_obstacle_reads_the_slots_flags() {
    let mut enemy = EnemyCommon::default();
    assert!(!Obstacle::from(&enemy).solid);
    enemy.properties.insert(EnemyProperty::SolidToSamus);
    enemy.frozen = Frames(3);
    let obstacle = Obstacle::from(&enemy);
    assert!(obstacle.solid && obstacle.frozen);
}
