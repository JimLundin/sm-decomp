//! Boyon, the bouncing blobs (bank $A2:8687-$A2:8911). Lanes: speed scale, speed, bounce
//! height, distance risen, bounce phase; the rest is in its extended words.
#![forbid(unsafe_code)]
use crate::actors::access::{AccessMode::LittleEndian as Le, ActorKey as Key};
use crate::actors::api::{self, Axis};
use crate::actors::combat::current;
use crate::actors::host::ActorLease;
use crate::actors::slot_view::{SlotState, slot_view};
use crate::enemy_script::EnemyScript;
use crate::player_projectiles::content::{offset, table};

slot_view! {
    /// The boyon's extended words: the bounce phase at which it lands
    /// (`limit`), its direction (0 falling, 1 rising), whether it rests, its
    /// saved speed scale, whether its rest animation started, and whether its
    /// bounce is set up.
    Boyon, BoyonEdit {
        record: [],
        extended: [
            limit = var_00, direction = var_01, resting = var_02, saved_scale = var_03,
            rest_animation = var_04, initialized = var_05,
        ],
    }
}

const IDLE: u16 = 0x86a7; // instruction lists
const BOUNCING: u16 = 0x86bf;

/// The current boyon's slot.
fn slot(runtime: &impl ActorLease) -> u16 {
    current(runtime).byte_index()
}

/// Initialisation AI - enemy $CEBF (boyon) ($A2:871C): speed scale and bounce height from the
/// Boyon data ($A2:86DF, $A2:86EF) by the bytes of parameter 1.
pub fn initialize(runtime: &mut impl crate::actors::species::TableLease) {
    let key = current(&*runtime);
    runtime.edit_actor(key, Le, |actor| actor.spritemap = 0x804d);
    runtime.edit_actor(key, Le, |actor| actor.restart_animation(IDLE));
    let k = slot(&*runtime);
    *BoyonEdit::of(runtime, k).direction() = 1;
    let [scale, height] = runtime.resolve_actor(key, Le).parameters[0].to_le_bytes();
    let scale = table(&*runtime, 0xa2_86df, isize::from(scale));
    runtime.edit_actor(key, Le, |actor| actor.variables[0] = scale);
    let height = table(&*runtime, 0xa2_86ef, isize::from(height));
    runtime.edit_actor(key, Le, |actor| {
        (actor.variables[2], actor.variables[3], actor.variables[4]) = (height, 0, 0);
        actor.pre_instruction = 0;
    });
}

/// The speed at `phase`: the sample at $A2:8701 (FFh from 23 on) times the speed scale.
fn speed(runtime: &mut impl ActorLease, key: Key, phase: u16) -> u16 {
    let image = runtime.leased_image().expect("ROM must be loaded");
    let sample = image.read(offset(0xa2_8701) + usize::from(phase));
    let sample = if (phase.wrapping_sub(23) as i16) >= 0 {
        255
    } else {
        sample
    };
    u16::from(sample) * u16::from(runtime.resolve_actor(key, Le).variables[0] as u8)
}

/// Calculate initial bounce speed ($A2:8755): the phase at which the bounce reaches its height.
fn initialize_bounce(runtime: &mut (impl ActorLease + SlotState)) {
    let key = current(runtime);
    loop {
        let phase = runtime.resolve_actor(key, Le).variables[4];
        let speed = speed(runtime, key, phase);
        let complete = runtime.edit_actor(key, Le, |actor| {
            actor.variables[1] = speed;
            actor.variables[3] = actor.variables[3].wrapping_add(speed);
            actor.variables[4] = actor.variables[4].wrapping_add(1);
            (actor.variables[3].wrapping_sub(actor.variables[2]) as i16) >= 0
        });
        if complete {
            break;
        }
    }
    let phase = runtime.resolve_actor(key, Le).variables[4];
    let k = slot(runtime);
    *BoyonEdit::of(runtime, k).limit() = phase;
    *BoyonEdit::of(runtime, k).resting() = 1;
    *BoyonEdit::of(runtime, k).rest_animation() = 1;
}

/// Main AI - enemy $CEBF (boyon) ($A2:879C): bounce while Samus is in proximity.
pub fn main_ai(runtime: &mut impl crate::actors::species::TableLease) {
    let key = current(&*runtime);
    let k = slot(&*runtime);
    if Boyon::of(&*runtime, k).initialized() == 0 {
        initialize_bounce(runtime);
        *BoyonEdit::of(runtime, k).initialized() = 1;
        return;
    }
    let scale = runtime.resolve_actor(key, Le).variables[0];
    *BoyonEdit::of(runtime, k).saved_scale() = scale;
    let selected = current(&*runtime);
    let radius = runtime.resolve_actor(selected, Le).parameters[1];
    if api::player_within(runtime, selected, Axis::X, radius) {
        *BoyonEdit::of(runtime, k).resting() = 0;
        *BoyonEdit::of(runtime, k).rest_animation() = 0;
        if runtime.resolve_actor(key, Le).pre_instruction == 0 {
            runtime.edit_actor(key, Le, |actor| actor.pre_instruction = 1);
            runtime.edit_actor(key, Le, |actor| actor.restart_animation(BOUNCING));
        }
    } else if Boyon::of(&*runtime, k).resting() != 0 {
        if Boyon::of(&*runtime, k).rest_animation() == 0 {
            *BoyonEdit::of(runtime, k).rest_animation() = 1;
            runtime.edit_actor(key, Le, |actor| actor.restart_animation(IDLE));
        }
        return;
    }
    let scale = Boyon::of(&*runtime, k).saved_scale();
    runtime.edit_actor(key, Le, |actor| actor.variables[0] = scale);
    match Boyon::of(&*runtime, k).direction() {
        0 => bounce(runtime, true),
        1 => bounce(runtime, false),
        _ => panic!("invalid Goofball state"),
    }
}

/// Boyon bounce movement - falling and rising ($A2:8801, $A2:8850).
fn bounce(runtime: &mut (impl ActorLease + SlotState), falling: bool) {
    let key = current(runtime);
    let k = slot(runtime);
    if falling {
        runtime.edit_actor(key, Le, |actor| {
            actor.variables[4] = actor.variables[4].wrapping_add(1)
        });
    }
    let phase = runtime.resolve_actor(key, Le).variables[4];
    let speed = speed(runtime, key, phase);
    let phase = runtime.edit_actor(key, Le, |actor| {
        actor.variables[1] = speed;
        let distance = speed >> 8;
        if falling {
            actor.y.whole = actor.y.whole.wrapping_add(distance);
        } else {
            actor.y.whole = actor.y.whole.wrapping_sub(distance);
            actor.variables[4] = actor.variables[4].wrapping_sub(1);
        }
        actor.variables[4]
    });
    if falling && (phase.wrapping_sub(Boyon::of(runtime, k).limit()) as i16) >= 0 {
        *BoyonEdit::of(runtime, k).direction() = 1;
        *BoyonEdit::of(runtime, k).resting() = 1;
        runtime.edit_actor(key, Le, |actor| actor.pre_instruction = 0);
    } else if !falling && (phase & 0x8000) != 0 {
        *BoyonEdit::of(runtime, k).direction() = 0;
    }
}

/// Instruction - start bounce ($A2:88C6).
pub fn start_bounce(
    script: EnemyScript,
    runtime: &mut impl crate::actors::species::TableLease,
) -> EnemyScript {
    let k = slot(&*runtime);
    *BoyonEdit::of(runtime, k).resting() = 0;
    crate::enemy_projectiles::host::Effects::queue_sound(runtime, 2, 6, 0xe);
    script
}
