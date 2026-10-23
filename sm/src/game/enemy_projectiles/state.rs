//! Ordinary owned enemy-projectile state; no Game or encoded-memory interface.
#![forbid(unsafe_code)]

#[derive(Clone, Copy, Default)]
pub(super) struct Position {
    pub whole: u16,
    pub fraction: u16,
}

#[derive(Clone, Copy, Default)]
pub(super) struct Point {
    pub x: u16,
    pub y: u16,
}

#[derive(Clone, Copy, Default)]
pub(super) struct Rectangle {
    pub origin: Point,
    pub extent: Point,
}

#[derive(Clone, Copy, Default)]
pub(super) struct DropMetadata {
    pub flags: u16,
    pub enemy_definition: u16,
    pub killed_enemy: u16,
}

#[derive(Clone, Copy, Default)]
pub(super) struct Projectile {
    pub definition: u16,
    pub graphics: u16,
    pub counter: u16,
    pub pre_action: u16,
    pub x: Position,
    pub y: Position,
    pub velocity: Point,
    // These words have different meanings for different projectile behaviors.
    // Keep their full stored values until each behavior's valid-state closure is known.
    pub behavior_a: u16,
    pub behavior_b: u16,
    pub program: u16,
    pub spritemap: u16,
    pub delay: u16,
    pub radius: u16,
    pub properties: u16,
    pub behavior_c: u16,
    pub angle: u16,
    pub drop: DropMetadata,
}

#[derive(Clone, Copy, Default)]
pub(super) struct SpawnContext {
    pub primary: u16,
    pub secondary: u16,
    pub angle: u8,
    pub rectangle: Rectangle,
    pub variant: u16,
    pub point: Point,
    pub drop_definition: u16,
}

#[derive(Default)]
pub(super) struct EnemyProjectiles {
    pub slots: [Projectile; 18],
    pub flags: u16,
    pub selected: u16,
    pub spawn: SpawnContext,
}

/// Semantic selectors used only by the private compatibility fallback.
#[derive(Clone, Copy)]
#[repr(usize)]
pub(super) enum SlotField {
    Definition,
    Graphics,
    Counter,
    PreAction,
    XFraction,
    X,
    YFraction,
    Y,
    VelocityX,
    VelocityY,
    BehaviorA,
    BehaviorB,
    Program,
    Spritemap,
    Delay,
    Radius,
    Properties,
    BehaviorC,
    Angle,
    DropFlags,
    DropDefinition,
    KilledEnemy,
}
