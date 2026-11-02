//! Botwoon execution on the prepared actor aggregate, including the death
//! handshake, body-history consumer and immediate head callbacks.
#![forbid(unsafe_code)]
use crate::actors::access::{AccessMode::Native, ActorKey as Key};
use crate::actors::api::{DeathHandling, ShotHandling};
use crate::actors::botwoon_protocol::{function, head_function, movement_function};
use crate::actors::owned_common_lifecycle::BossState;
use crate::actors::{combat, queries};
use crate::constants::{bank_86::*, enemy_properties::*};
use crate::enemy_script::EnemyScript;
use sm_data::botwoon_holes::Hole;
use sm_data::botwoon_speeds::BotwoonSpeed;
use sm_data::enemy_speeds::SpeedTable;

pub trait Host: BossState + crate::records::Records + crate::effects::SoundEffects {
    fn exceptional_byte(&self, at: usize) -> u8;
    fn set_exceptional_byte(&mut self, at: usize, value: u8);
    fn botwoon_speed_level(&self, index: isize) -> BotwoonSpeed;
    fn botwoon_hole(&self, index: isize) -> Hole;
    fn native_path_record(&mut self, bank: u8, address: u16) -> sm_data::cartridge::RomRecord;
    fn loaded_botwoon_word(&self, address: u32, index: isize) -> u16;
    fn set_palette_word(&mut self, index: u16, value: u16);
    fn create_sprite(&mut self, x: u16, y: u16, kind: u16, appearance: u16);
    fn queue_music_delayed(&mut self, track: u16);
    fn set_boss_bit(&mut self, mask: u16);
    fn boss_byte(&mut self, area: u16) -> u8;
    fn spawn_plm(&mut self, x: u8, y: u8, header: u16);
    fn open_passage(&mut self);
    fn unreachable(&mut self) -> !;
}

#[derive(Clone, Copy)]
enum Field {
    BodyProjectileIndex,
    PositionHistoryIndex,
    BodyTravelTime,
    Function,
    MovementFunction,
    HeadFunction,
    InitialDelay,
    SpitTimer,
    PreDeathCounter,
    DeathCounter,
    LargeWallExplosionTimer,
    WallSmokeTimer,
    UnreadCounter,
    FallingYSpeedIndex,
    Unread801c,
    HeadHidden,
    PreviousHeadHidden,
    HoleCollisionDisabled,
    TargetHistoryIndex,
    TargetHole,
    Speed,
    UnreadHoleAngle,
    TargetHoleAngle,
    InstructionList,
    GoingThroughHole,
    SpitAngle,
    SpeedTableIndex,
    MovementTableIndex,
    Unused8802,
    MovementDataPointer,
    MovementDirection,
    Unused8816,
    PaletteDataOffset,
    HealthPaletteIndex,
    X1FrameAgo,
    Y1FrameAgo,
    X2FramesAgo,
    Y2FramesAgo,
    X3FramesAgo,
    Y3FramesAgo,
    X4FramesAgo,
    Y4FramesAgo,
    InitialLeaveHoleAction,
    Spitting,
    MaxHealth,
    HalfHealth,
    QuarterHealth,
    XPos,
    YPos,
    YSubpos,
    XWidth,
    YHeight,
    Properties,
    Health,
    CurrentInstruction,
    InstructionTimer,
    PaletteIndex,
    Layer,
    Timer,
}
impl Field {
    fn address(self, enemy: i32) -> usize {
        let base: isize = match self {
            Self::BodyProjectileIndex => 0x0fa8,
            Self::PositionHistoryIndex => 0x0faa,
            Self::BodyTravelTime => 0x0fac,
            Self::Function => 0x0fae,
            Self::MovementFunction => 0x0fb0,
            Self::HeadFunction => 0x0fb2,
            Self::InitialDelay => 0x8000,
            Self::SpitTimer => 0x8002,
            Self::PreDeathCounter => 0x8004,
            Self::DeathCounter => 0x8006,
            Self::LargeWallExplosionTimer => 0x8008,
            Self::WallSmokeTimer => 0x800a,
            Self::UnreadCounter => 0x800c,
            Self::FallingYSpeedIndex => 0x8010,
            Self::Unread801c => 0x801c,
            Self::HeadHidden => 0x8026,
            Self::PreviousHeadHidden => 0x8028,
            Self::HoleCollisionDisabled => 0x802a,
            Self::TargetHistoryIndex => 0x802c,
            Self::TargetHole => 0x802e,
            Self::Speed => 0x8030,
            Self::UnreadHoleAngle => 0x8032,
            Self::TargetHoleAngle => 0x8034,
            Self::InstructionList => 0x8036,
            Self::GoingThroughHole => 0x8038,
            Self::SpitAngle => 0x803a,
            Self::SpeedTableIndex => 0x803e,
            Self::MovementTableIndex => 0x8800,
            Self::Unused8802 => 0x8802,
            Self::MovementDataPointer => 0x8804,
            Self::MovementDirection => 0x8808,
            Self::Unused8816 => 0x8816,
            Self::PaletteDataOffset => 0x881c,
            Self::HealthPaletteIndex => 0x881e,
            Self::X1FrameAgo => 0x8820,
            Self::Y1FrameAgo => 0x8822,
            Self::X2FramesAgo => 0x8824,
            Self::Y2FramesAgo => 0x8826,
            Self::X3FramesAgo => 0x8828,
            Self::Y3FramesAgo => 0x882a,
            Self::X4FramesAgo => 0x882c,
            Self::Y4FramesAgo => 0x882e,
            Self::InitialLeaveHoleAction => 0x8832,
            Self::Spitting => 0x8834,
            Self::MaxHealth => 0x8836,
            Self::HalfHealth => 0x8838,
            Self::QuarterHealth => 0x883a,
            Self::XPos => 0x0f7a,
            Self::YPos => 0x0f7e,
            Self::YSubpos => 0x0f80,
            Self::XWidth => 0x0f82,
            Self::YHeight => 0x0f84,
            Self::Properties => 0x0f86,
            Self::Health => 0x0f8c,
            Self::CurrentInstruction => 0x0f92,
            Self::InstructionTimer => 0x0f94,
            Self::PaletteIndex => 0x0f96,
            Self::Layer => 0x0f9a,
            Self::Timer => 0x0f90,
        };
        usize::try_from(isize::try_from(enemy).expect("actor field offset") + base)
            .expect("actor field outside allocation")
    }
    fn is_actor(self) -> bool {
        matches!(
            self,
            Self::BodyProjectileIndex
                | Self::PositionHistoryIndex
                | Self::BodyTravelTime
                | Self::Function
                | Self::MovementFunction
                | Self::HeadFunction
                | Self::XPos
                | Self::YPos
                | Self::YSubpos
                | Self::XWidth
                | Self::YHeight
                | Self::Properties
                | Self::Health
                | Self::CurrentInstruction
                | Self::InstructionTimer
                | Self::PaletteIndex
                | Self::Layer
                | Self::Timer
        )
    }
}

struct State<'a> {
    actor: &'a crate::actors::state::Actor,
    head: &'a crate::actors::botwoon_state::BotwoonHeadState,
    path: &'a crate::actors::botwoon_state::BotwoonPathState,
}
struct StateMut<'a> {
    actor: &'a mut crate::actors::state::Actor,
    head: &'a mut crate::actors::botwoon_state::BotwoonHeadState,
    path: &'a mut crate::actors::botwoon_state::BotwoonPathState,
}
fn read(h: &impl Host, enemy: i32, field: Field, get: impl FnOnce(State<'_>) -> u16) -> u16 {
    if let Ok(enemy) = usize::try_from(enemy)
        && enemy < 2048
        && enemy.is_multiple_of(64)
    {
        let actors = &h.session().actors;
        let index = enemy / 64;
        return get(State {
            actor: &actors.slots[index],
            head: &actors.extended.botwoon_head[index],
            path: &actors.extended.botwoon_path[index],
        });
    }
    // Full/odd/outspan selector aliases use the physical port one byte at a
    // time. The port resolves every projected byte to its actual typed owner.
    let at = field.address(enemy);
    u16::from_le_bytes([h.exceptional_byte(at), h.exceptional_byte(at + 1)])
}
fn write(
    h: &mut impl Host,
    enemy: i32,
    field: Field,
    value: u16,
    set: impl FnOnce(StateMut<'_>, u16),
) {
    if let Ok(enemy) = usize::try_from(enemy)
        && enemy < 2048
        && enemy.is_multiple_of(64)
    {
        let actors = &mut h.session_mut().actors;
        let index = enemy / 64;
        set(
            StateMut {
                actor: &mut actors.slots[index],
                head: &mut actors.extended.botwoon_head[index],
                path: &mut actors.extended.botwoon_path[index],
            },
            value,
        );
        if field.is_actor() {
            h.refresh_inputs();
        }
        return;
    }
    let at = field.address(enemy);
    let [low, high] = value.to_le_bytes();
    h.set_exceptional_byte(at, low);
    h.set_exceptional_byte(at + 1, high);
}
fn read_byte(
    h: &impl Host,
    enemy: i32,
    field: Field,
    lane: usize,
    get: impl FnOnce(State<'_>) -> u16,
) -> u8 {
    if let Ok(index) = usize::try_from(enemy)
        && index < 2048
        && index.is_multiple_of(64)
    {
        return read(h, enemy, field, get).to_le_bytes()[lane];
    }
    h.exceptional_byte(field.address(enemy) + lane)
}
macro_rules! loaded_word {
    ($h:ident, $address:expr, $index:expr $(,)?) => {
        Host::loaded_botwoon_word($h, $address, $index)
    };
}
fn native_path_record(bank: u8, address: u16, h: &mut impl Host) -> sm_data::cartridge::RomRecord {
    h.native_path_record(bank, address)
}
fn spit_angle(h: &impl Host) -> u8 {
    h.session().enemy_projectiles.spawn.angle
}
fn set_spit_angle(h: &mut impl Host, value: u8) {
    h.session_mut().enemy_projectiles.spawn.angle = value;
    h.refresh_inputs();
}
use crate::actors::botwoon_protocol::Signal;
fn death_signal(h: &impl Host, enemy: u16, signal: Signal) -> u16 {
    if enemy < 2048 && enemy.is_multiple_of(64) {
        let extended = &h.session().actors.extended;
        let index = usize::from(enemy) / 64;
        return match signal {
            Signal::PendingHit => extended.botwoon_head[index].pending_hit,
            Signal::Active => extended.botwoon_head[index].active,
            Signal::TailReady => extended.botwoon_head[index].tail_ready,
            Signal::PreviousHealth => extended.botwoon_path[index].previous_health,
            Signal::BodyLanded => extended.botwoon_path[index].body_landed,
        };
    }
    let at = signal.address(enemy);
    u16::from_le_bytes([h.exceptional_byte(at), h.exceptional_byte(at + 1)])
}
fn set_death_signal(h: &mut impl Host, enemy: u16, signal: Signal, value: u16) {
    if enemy < 2048 && enemy.is_multiple_of(64) {
        let extended = &mut h.session_mut().actors.extended;
        let index = usize::from(enemy) / 64;
        match signal {
            Signal::PendingHit => extended.botwoon_head[index].pending_hit = value,
            Signal::Active => extended.botwoon_head[index].active = value,
            Signal::TailReady => extended.botwoon_head[index].tail_ready = value,
            Signal::PreviousHealth => extended.botwoon_path[index].previous_health = value,
            Signal::BodyLanded => extended.botwoon_path[index].body_landed = value,
        }
        return;
    }
    let at = signal.address(enemy);
    let [low, high] = value.to_le_bytes();
    h.set_exceptional_byte(at, low);
    h.set_exceptional_byte(at + 1, high);
}
fn arm_death(h: &mut impl Host) {
    let actor = h.selected_actor();
    let enemy = actor.byte_index();
    if death_signal(h, enemy, Signal::PendingHit) != 0
        && death_signal(h, enemy, Signal::TailReady) != 0
    {
        set_death_signal(h, enemy, Signal::Active, 1);
        h.edit_actor(actor, Native, |actor| {
            actor.variables[3] = function::DEATH_PRE_DEATH_DELAY
        });
        write(
            h,
            i32::from(enemy),
            Field::PreDeathCounter,
            0xf0,
            |state, value| state.head.pre_death_counter = value,
        );
        set_death_signal(h, enemy, Signal::PendingHit, 0);
        set_death_signal(h, enemy, Signal::TailReady, 0);
    }
}
pub const HIDE: u16 = 0x9389;
const BANK: u32 = 0xb3;
/// Properties bit 10: intangible (the decompiler's `K_ENEMY_PROPS_TANGIBLE`).
const INTANGIBLE: u16 = K_ENEMY_PROPS_TANGIBLE as u16;
const SIGN: i32 = 0x8000;
/// Botwoon trails 24 body projectiles and waits 100h frames before its first move.
const BODY_PROJECTILES: u16 = 24;
const INITIAL_DELAY: u16 = 0x100;
/// $B3:971B: the health-based palettes, 10h bytes each, and $B3:981B their
/// health thresholds; the last of 8 is reached at index 10h.
const HEALTH_PALETTES: u32 = 0xb3_971b;
const HEALTH_THRESHOLDS: u32 = 0xb3_981b;
const LAST_HEALTH_PALETTE: u16 = 16;
const PALETTE_END: u16 = 0x200;
/// Leaving a hole picks one of 8 actions; the first 3 move around, the rest spit.
const LEAVE_HOLE_ACTIONS: u8 = 0xe;
const LAST_MOVING_ACTION: u8 = 4;
const SPIT_DELAY: u16 = 0x30;
/// The pre-death delay lasts 100h frames; the head falls to Y C8h with a dust
/// cloud ($86:E509 parameter 1Dh) and a small explosion (sound 24h).
const PRE_DEATH_FRAMES: i32 = 0x100;
const GROUND_Y: u16 = 200;
const FALL_STEP: u16 = 0xc0;
const DUST_CLOUD_PARAMETER: u16 = 0x1d;
const SMALL_EXPLOSION: u8 = 0x24;
/// Landing makes Botwoon intangible and its graphics drawn last.
const LANDED_PROPERTIES: u16 = 0x500;
/// The wall crumbles from death counter 40h to C0h: large explosions (sprite
/// object 1Dh) every 0Ch frames and two smoke clouds (sprite object 9) every 4,
/// around X E8h and E0h.
const CRUMBLE_START: i32 = 0x40;
const CRUMBLE_END: i32 = 0xc0;
const EXPLOSION_FRAMES: u16 = 12;
const SMOKE_FRAMES: u16 = 4;
const LARGE_EXPLOSION: u16 = 0x1d;
const SMOKE: u16 = 9;
const WALL_SPRITE_PALETTE: u16 = 0xa00;
const EXPLOSION_X: u16 = 0xe8;
const SMOKE_X: u16 = 0xe0;
/// Boss bit 1: Botwoon is dead; the elevator music track plays.
const BOTWOON_DEAD: u16 = 2;
const ELEVATOR_MUSIC: u16 = 3;
/// Bank $84 PLMs: clear Botwoon's wall and crumble it, at block (0Fh, 4).
const CLEAR_WALL: u16 = 0xb797;
const CRUMBLE_WALL: u16 = 0xb79b;
const WALL_BLOCK: (u8, u8) = (15, 4);
/// Maridia's area index.
const MARIDIA: u16 = 4;
/// Hidden, the movement path is one of the second 80h entries.
const HIDDEN_PATHS: i32 = 128;
const PATH_CHOICES: u16 = 0x18;
/// $B3:E150: the movement table, whose entries point at movement data:
/// byte pairs of X and Y steps, ended by 80h.
const MOVEMENT_TABLE: u16 = 0xe150;
const END_OF_DATA: u16 = 0xff80;
/// Offsets to the target hole are clamped to +/-FFh.
const MAX_OFFSET: u16 = 255;
/// Hidden, the head is drawn behind the background (layer 7); shown, layer 2.
const HIDDEN_LAYER: u16 = 7;
const SHOWN_LAYER: u16 = 2;
/// $7E:060B: the spit projectile angle; spits fan out 10h apart.

const SPIT_SPREAD: u8 = 16;
/// $B3:9E77: the spit speeds by speed table index.
const SPIT_SPEEDS: [u16; 3] = [2, 3, 4];
/// Angles are measured from the top, clockwise; 40h less is the maths angle.
const QUARTER_TURN: u8 = 0x40;

fn initialize_living<L: Host>(e: i32, game: &mut L) {
    {
        let value: u16 = BODY_PROJECTILES;
        write(
            game,
            e,
            Field::BodyProjectileIndex,
            value,
            |state, value| state.actor.variables[0] = value,
        );
    };
    loop {
        let k = game.current_enemy();
        game.spawn_actor_eproj(BODY_PROJECTILES, k, ADDR_K_EPROJ_BOTWOONS_BODY as u16);
        {
            let old: u16 = read(game, e, Field::BodyProjectileIndex, |state| {
                state.actor.variables[0]
            });
            {
                let value: u16 = old.wrapping_sub(1);
                write(
                    game,
                    e,
                    Field::BodyProjectileIndex,
                    value,
                    |state, value| state.actor.variables[0] = value,
                );
            }
        };
        let finished = (read(game, e, Field::BodyProjectileIndex, |state| {
            state.actor.variables[0]
        }) as i16)
            .wrapping_sub(1)
            < 0;
        {
            let old: u16 = read(game, e, Field::BodyProjectileIndex, |state| {
                state.actor.variables[0]
            });
            {
                let value: u16 = old.wrapping_sub(1);
                write(
                    game,
                    e,
                    Field::BodyProjectileIndex,
                    value,
                    |state, value| state.actor.variables[0] = value,
                );
            }
        };
        if finished {
            break;
        }
    }
    {
        let value: u16 = HIDE;
        write(game, e, Field::CurrentInstruction, value, |state, value| {
            state.actor.instruction = value
        });
    };
    let initial = game.botwoon_speed_level(0);
    {
        {
            let value: u16 = HIDE;
            write(game, e, Field::InstructionList, value, |state, value| {
                state.head.instruction_list = value
            });
        }
        {
            let value: u16 = function::INITIAL;
            write(game, e, Field::Function, value, |state, value| {
                state.actor.variables[3] = value
            });
        }
        {
            let value: u16 = movement_function::MOVE_TOWARD_TARGET_HOLE;
            write(game, e, Field::MovementFunction, value, |state, value| {
                state.actor.variables[4] = value
            });
        }
        {
            let value: u16 = head_function::MOVING_AROUND;
            write(game, e, Field::HeadFunction, value, |state, value| {
                state.actor.pre_instruction = value
            });
        }
        {
            let value: u16 = INITIAL_DELAY;
            write(game, e, Field::InitialDelay, value, |state, value| {
                state.head.initial_delay = value
            });
        }
        {
            let value: u16 = initial.speed;
            write(game, e, Field::Speed, value, |state, value| {
                state.head.speed = value
            });
        }
        {
            let value: u16 = initial.body_travel_time;
            write(game, e, Field::BodyTravelTime, value, |state, value| {
                state.actor.variables[2] = value
            });
        }
        {
            let value: u16 = 1;
            write(game, e, Field::HeadHidden, value, |state, value| {
                state.head.head_hidden = value
            });
        }
        {
            let value: u16 = 1;
            write(game, e, Field::PreviousHeadHidden, value, |state, value| {
                state.head.previous_head_hidden = value
            });
        }
        {
            let value: u16 = 1;
            write(
                game,
                e,
                Field::InitialLeaveHoleAction,
                value,
                |state, value| state.path.initial_leave_hole_action = value,
            );
        }
        {
            let value: u16 = u16::MAX;
            write(game, e, Field::TargetHistoryIndex, value, |state, value| {
                state.head.target_history_index = value
            });
        }
        {
            let value: u16 = 0;
            write(game, e, Field::TargetHole, value, |state, value| {
                state.head.target_hole = value
            });
        }
    };
    {
        let old: u16 = read(game, e, Field::Properties, |state| state.actor.properties);
        {
            let value: u16 = old | INTANGIBLE;
            write(game, e, Field::Properties, value, |state, value| {
                state.actor.properties = value
            });
        }
    };
    let health = read(game, e, Field::Health, |state| state.actor.health);
    {
        {
            let value: u16 = health;
            write(game, e, Field::MaxHealth, value, |state, value| {
                state.path.max_health = value
            });
        }
        {
            let value: u16 = health >> 1;
            write(game, e, Field::HalfHealth, value, |state, value| {
                state.path.half_health = value
            });
        }
        {
            let value: u16 = (health >> 1) >> 1;
            write(game, e, Field::QuarterHealth, value, |state, value| {
                state.path.quarter_health = value
            });
        }
    };
    let x = read(game, e, Field::XPos, |state| state.actor.x.whole);
    {
        {
            let value: u16 = x;
            write(game, e, Field::X1FrameAgo, value, |state, value| {
                state.path.x_1_frame_ago = value
            });
        }
        {
            let value: u16 = x;
            write(game, e, Field::X2FramesAgo, value, |state, value| {
                state.path.x_2_frames_ago = value
            });
        }
        {
            let value: u16 = x;
            write(game, e, Field::X3FramesAgo, value, |state, value| {
                state.path.x_3_frames_ago = value
            });
        }
        {
            let value: u16 = x;
            write(game, e, Field::X4FramesAgo, value, |state, value| {
                state.path.x_4_frames_ago = value
            });
        }
    };
    let y = read(game, e, Field::YPos, |state| state.actor.y.whole);
    {
        {
            let value: u16 = y;
            write(game, e, Field::Y1FrameAgo, value, |state, value| {
                state.path.y_1_frame_ago = value
            });
        }
        {
            let value: u16 = y;
            write(game, e, Field::Y2FramesAgo, value, |state, value| {
                state.path.y_2_frames_ago = value
            });
        }
        {
            let value: u16 = y;
            write(game, e, Field::Y3FramesAgo, value, |state, value| {
                state.path.y_3_frames_ago = value
            });
        }
        {
            let value: u16 = y;
            write(game, e, Field::Y4FramesAgo, value, |state, value| {
                state.path.y_4_frames_ago = value
            });
        }
        {
            let value: u16 = ((i32::from(read(game, e, Field::PaletteIndex, |state| {
                state.actor.palette
            })) >> 4)
                + 0x100) as u16;
            write(game, e, Field::PaletteDataOffset, value, |state, value| {
                state.path.palette_data_offset = value
            });
        }
    };
}
/// Botwoon health-based palette handling ($B3:982B).
fn health_palette<R: Host>(game: &mut R) {
    let e = i32::from(game.current_enemy());
    if read(game, e, Field::HealthPaletteIndex, |state| {
        state.path.health_palette_index
    }) != LAST_HEALTH_PALETTE
        && (read(game, e, Field::Health, |state| state.actor.health) as i16).wrapping_sub(
            loaded_word!(
                game,
                HEALTH_THRESHOLDS,
                (read(game, e, Field::HealthPaletteIndex, |state| state
                    .path
                    .health_palette_index)
                    >> 1) as isize,
            ) as i16,
        ) < 0
    {
        let mut source = 16u16.wrapping_mul(read(game, e, Field::HealthPaletteIndex, |state| {
            state.path.health_palette_index
        }));
        let mut target = read(game, e, Field::PaletteDataOffset, |state| {
            state.path.palette_data_offset
        });
        loop {
            Host::set_palette_word(
                game,
                target >> 1,
                loaded_word!(game, HEALTH_PALETTES, (source >> 1) as isize),
            );
            source = source.wrapping_add(2);
            target = target.wrapping_add(2);
            if target == PALETTE_END {
                break;
            }
        }
        {
            let old: u16 = read(game, e, Field::HealthPaletteIndex, |state| {
                state.path.health_palette_index
            });
            {
                let value: u16 = old.wrapping_add(2);
                write(game, e, Field::HealthPaletteIndex, value, |state, value| {
                    state.path.health_palette_index = value
                });
            }
        };
    }
}
/// Botwoon function - initial ($B3:9878): wait, then go through a hole. Was
/// `wait_entry`.
fn initial<R: Host>(game: &mut R) {
    let e = i32::from(game.current_enemy());
    let delay = read(game, e, Field::InitialDelay, |state| {
        state.head.initial_delay
    })
    .wrapping_sub(1);
    {
        let value: u16 = delay;
        write(game, e, Field::InitialDelay, value, |state, value| {
            state.head.initial_delay = value
        });
    };
    if delay == 0 {
        {
            let value: u16 = function::GO_THROUGH_HOLE;
            write(game, e, Field::Function, value, |state, value| {
                state.actor.variables[3] = value
            });
        };
    }
}
/// Botwoon function - go through hole ($B3:989D), leaving it: set up moving
/// around ($B3:98EC) or spitting ($B3:9913). Was `select_attack`.
pub fn leave_hole<L: Host>(e: i32, game: &mut L) -> bool {
    if read(game, e, Field::GoingThroughHole, |state| {
        state.head.going_through_hole
    }) == 0
    {
        return false;
    }
    {
        let value: u16 = 0;
        write(game, e, Field::GoingThroughHole, value, |state, value| {
            state.head.going_through_hole = value
        });
    };
    let choice = if read(game, e, Field::HeadHidden, |state| state.head.head_hidden) == 0
        && read(game, e, Field::InitialLeaveHoleAction, |state| {
            state.path.initial_leave_hole_action
        }) == 0
        && read(game, e, Field::SpeedTableIndex, |state| {
            state.head.speed_table_index
        }) == 0
    {
        game.roll_random() as u8 & LEAVE_HOLE_ACTIONS
    } else {
        0
    };
    {
        let value: u16 = 0;
        write(
            game,
            e,
            Field::InitialLeaveHoleAction,
            value,
            |state, value| state.path.initial_leave_hole_action = value,
        );
    };
    // Masking above admits exactly 0,2,4,6,8,10,12,14.
    let current = i32::from(game.current_enemy());
    if choice <= LAST_MOVING_ACTION {
        {
            {
                let value: u16 = function::MOVING_AROUND;
                write(game, current, Field::Function, value, |state, value| {
                    state.actor.variables[3] = value
                });
            }
            {
                let value: u16 = movement_function::START_MOVING_BY_MOVEMENT_DATA;
                write(
                    game,
                    current,
                    Field::MovementFunction,
                    value,
                    |state, value| state.actor.variables[4] = value,
                );
            }
            {
                let value: u16 = 0;
                write(game, current, Field::SpitTimer, value, |state, value| {
                    state.head.spit_timer = value
                });
            }
            {
                let value: u16 = head_function::MOVING_AROUND;
                write(game, current, Field::HeadFunction, value, |state, value| {
                    state.actor.pre_instruction = value
                });
            }
            {
                let value: u16 = 0;
                write(game, current, Field::Unused8816, value, |state, value| {
                    state.path.unused_8816 = value
                });
            }
        };
        choose_movement_path(game.current_enemy(), game);
    } else {
        {
            {
                let value: u16 = function::SPITTING;
                write(game, current, Field::Function, value, |state, value| {
                    state.actor.variables[3] = value
                });
            }
            {
                let value: u16 = head_function::SPITTING_SET_ANGLE_AND_SHOW;
                write(game, current, Field::HeadFunction, value, |state, value| {
                    state.actor.pre_instruction = value
                });
            }
            {
                let value: u16 = SPIT_DELAY;
                write(game, current, Field::SpitTimer, value, |state, value| {
                    state.head.spit_timer = value
                });
            }
        };
        {
            let old: u16 = read(game, current, Field::Properties, |state| {
                state.actor.properties
            });
            {
                let value: u16 = old & !INTANGIBLE;
                write(game, current, Field::Properties, value, |state, value| {
                    state.actor.properties = value
                });
            }
        };
    }
    true
}
/// Botwoon function - moving around ($B3:99A4): once at a hole, go through it.
/// Was `resume_chase`.
pub fn moving_around<L: Host>(e: i32, game: &mut L) -> bool {
    if read(game, e, Field::GoingThroughHole, |state| {
        state.head.going_through_hole
    }) == 0
    {
        return false;
    }
    {
        {
            let value: u16 = 0;
            write(game, e, Field::GoingThroughHole, value, |state, value| {
                state.head.going_through_hole = value
            });
        }
        {
            let value: u16 = function::GO_THROUGH_HOLE;
            write(game, e, Field::Function, value, |state, value| {
                state.actor.variables[3] = value
            });
        }
        {
            let value: u16 = movement_function::MOVE_TOWARD_TARGET_HOLE;
            write(game, e, Field::MovementFunction, value, |state, value| {
                state.actor.variables[4] = value
            });
        }
    };
    if read(game, e, Field::HeadHidden, |state| state.head.head_hidden) != 0 {
        {
            let value: u16 = 0;
            write(
                game,
                e,
                Field::HoleCollisionDisabled,
                value,
                |state, value| state.head.hole_collision_disabled = value,
            );
        };
    }
    true
}
/// Botwoon function - spitting ($B3:99E4): until the spit timer runs out, then
/// move around again. Was `await_spit`.
fn spitting<R: Host>(e: i32, runtime: &mut R) -> bool {
    if read(runtime, e, Field::SpitTimer, |state| state.head.spit_timer) != 0 {
        {
            let value: u16 = 0;
            write(runtime, e, Field::Unread801c, value, |state, value| {
                state.head.unread_801c = value
            });
        };
        return true;
    }
    {
        {
            let value: u16 = 0;
            write(
                runtime,
                e,
                Field::GoingThroughHole,
                value,
                |state, value| state.head.going_through_hole = value,
            );
        }
        {
            let value: u16 = function::MOVING_AROUND;
            write(runtime, e, Field::Function, value, |state, value| {
                state.actor.variables[3] = value
            });
        }
        {
            let value: u16 = movement_function::START_MOVING_BY_MOVEMENT_DATA;
            write(
                runtime,
                e,
                Field::MovementFunction,
                value,
                |state, value| state.actor.variables[4] = value,
            );
        }
        {
            let value: u16 = head_function::MOVING_AROUND;
            write(runtime, e, Field::HeadFunction, value, |state, value| {
                state.actor.pre_instruction = value
            });
        }
    };
    let direct = death_signal(runtime, e as u16, Signal::PendingHit) != 0 || {
        let choice = runtime.roll_random() & 1;
        {
            {
                let value: u16 = choice;
                write(runtime, e, Field::HeadHidden, value, |state, value| {
                    state.head.head_hidden = value
                });
            }
            {
                let value: u16 = choice;
                write(
                    runtime,
                    e,
                    Field::PreviousHeadHidden,
                    value,
                    |state, value| state.head.previous_head_hidden = value,
                );
            }
        };
        read(runtime, e, Field::PreviousHeadHidden, |state| {
            state.head.previous_head_hidden
        }) == 0
    };
    if direct {
        {
            let value: u16 = 0;
            write(runtime, e, Field::Unused8816, value, |state, value| {
                state.path.unused_8816 = value
            });
        };
    } else {
        {
            {
                let value: u16 = 0;
                write(
                    runtime,
                    e,
                    Field::HoleCollisionDisabled,
                    value,
                    |state, value| state.head.hole_collision_disabled = value,
                );
            }
            {
                let value: u16 = u16::MAX;
                write(
                    runtime,
                    e,
                    Field::TargetHistoryIndex,
                    value,
                    |state, value| state.head.target_history_index = value,
                );
            }
        };
    }
    choose_movement_path(runtime.current_enemy(), runtime);
    false
}
/// Botwoon function - death sequence - pre-death delay ($B3:9A46). Was
/// `wait_collapse`.
fn pre_death_delay<R: Host>(game: &mut R) {
    let e = i32::from(game.current_enemy());
    let delay = read(game, e, Field::PreDeathCounter, |state| {
        state.head.pre_death_counter
    })
    .wrapping_add(1);
    {
        let value: u16 = delay;
        write(game, e, Field::PreDeathCounter, value, |state, value| {
            state.head.pre_death_counter = value
        });
    };
    if (i32::from(delay) - PRE_DEATH_FRAMES) & SIGN == 0 {
        {
            let value: u16 = function::DEATH_FALLING_TO_GROUND;
            write(game, e, Field::Function, value, |state, value| {
                state.actor.variables[3] = value
            });
        };
    }
}
/// Botwoon function - death sequence - falling to ground ($B3:9A5E). Was `fall`.
pub fn fall_to_ground<L: Host>(game: &mut L) {
    let e = i32::from(game.current_enemy());
    let index = ((8 * i32::from(read_byte(game, e, Field::FallingYSpeedIndex, 1, |state| {
        state.head.falling_y_speed_index
    }))) >> 1) as isize;
    let high = game.enemy_speed(SpeedTable::Quadratic, index + 1);
    let low = game.enemy_speed(SpeedTable::Quadratic, index);
    {
        let value: u32 = (u32::from(high) << 16) | u32::from(low);
        let low = u32::from(read(game, e, Field::YSubpos, |state| {
            state.actor.y.fraction
        }));
        let high = u32::from(read(game, e, Field::YPos, |state| state.actor.y.whole));
        let result = (low | (high << 16)).wrapping_add(value);
        {
            let value: u16 = u16::from_le_bytes([result.to_le_bytes()[0], result.to_le_bytes()[1]]);
            write(game, e, Field::YSubpos, value, |state, value| {
                state.actor.y.fraction = value
            });
        }
        {
            let value: u16 = u16::try_from(result >> 16).unwrap();
            write(game, e, Field::YPos, value, |state, value| {
                state.actor.y.whole = value
            });
        }
    };
    if (i32::from(read(game, e, Field::YPos, |state| state.actor.y.whole)) - i32::from(GROUND_Y))
        & SIGN
        != 0
    {
        {
            let old: u16 = read(game, e, Field::FallingYSpeedIndex, |state| {
                state.head.falling_y_speed_index
            });
            {
                let value: u16 = old.wrapping_add(FALL_STEP);
                write(game, e, Field::FallingYSpeedIndex, value, |state, value| {
                    state.head.falling_y_speed_index = value
                });
            }
        };
    } else {
        {
            let value: u16 = GROUND_Y;
            write(game, e, Field::YPos, value, |state, value| {
                state.actor.y.whole = value
            });
        };
        {
            let value: u16 = function::DEATH_WAIT_FOR_BODY_TO_FALL;
            write(game, e, Field::Function, value, |state, value| {
                state.actor.variables[3] = value
            });
        };
        let x = read(game, e, Field::XPos, |state| state.actor.x.whole);
        let y = read(game, e, Field::YPos, |state| state.actor.y.whole);
        game.place_projectile_spawn(x, y);
        game.spawn_room_eproj(
            ADDR_K_EPROJ_DUST_CLOUD_EXPLOSION as u16,
            DUST_CLOUD_PARAMETER,
        );
        crate::effects::SoundEffects::queue_sound(game, 1, SMALL_EXPLOSION, 6);
        {
            let old: u16 = read(game, e, Field::Properties, |state| state.actor.properties);
            {
                let value: u16 = old | LANDED_PROPERTIES;
                write(game, e, Field::Properties, value, |state, value| {
                    state.actor.properties = value
                });
            }
        };
    }
}
/// Botwoon function - death sequence - crumbling wall ($B3:9AF9). Was `rubble`.
pub fn crumbling_wall<L: Host>(k: u16, game: &mut L) {
    let e = i32::from(k);
    if (i32::from(read(game, e, Field::DeathCounter, |state| {
        state.head.death_counter
    })) - CRUMBLE_END)
        & SIGN
        != 0
    {
        if (i32::from(read(game, e, Field::DeathCounter, |state| {
            state.head.death_counter
        })) - CRUMBLE_START)
            & SIGN
            == 0
        {
            let explosion = (read(game, e, Field::LargeWallExplosionTimer, |state| {
                state.head.large_wall_explosion_timer
            }) as i16)
                .wrapping_sub(1);
            {
                let value: u16 = explosion as u16;
                write(
                    game,
                    e,
                    Field::LargeWallExplosionTimer,
                    value,
                    |state, value| state.head.large_wall_explosion_timer = value,
                );
            };
            if explosion < 0 {
                {
                    let value: u16 = EXPLOSION_FRAMES;
                    write(
                        game,
                        e,
                        Field::LargeWallExplosionTimer,
                        value,
                        |state, value| state.head.large_wall_explosion_timer = value,
                    );
                };
                let x = (game.roll_random() & 0x1f) + EXPLOSION_X;
                let y = (i32::from(read(game, e, Field::DeathCounter, |state| {
                    state.head.death_counter
                })) + i32::from(game.roll_random() & 0x1f)
                    - 8) as u16;
                game.create_sprite(x, y, LARGE_EXPLOSION, WALL_SPRITE_PALETTE);
                crate::effects::SoundEffects::queue_sound(game, 1, SMALL_EXPLOSION, 6);
            }
            let dust = (read(game, e, Field::WallSmokeTimer, |state| {
                state.head.wall_smoke_timer
            }) as i16)
                .wrapping_sub(1);
            {
                let value: u16 = dust as u16;
                write(game, e, Field::WallSmokeTimer, value, |state, value| {
                    state.head.wall_smoke_timer = value
                });
            };
            if dust < 0 {
                {
                    let value: u16 = SMOKE_FRAMES;
                    write(game, e, Field::WallSmokeTimer, value, |state, value| {
                        state.head.wall_smoke_timer = value
                    });
                };
                for _ in 0..2 {
                    let x = (game.roll_random() & 0x3f) + SMOKE_X;
                    let y = (i32::from(read(game, e, Field::DeathCounter, |state| {
                        state.head.death_counter
                    })) + (i32::from(game.roll_random() & 0x1f) - 8))
                        as u16;
                    game.create_sprite(x, y, SMOKE, WALL_SPRITE_PALETTE);
                }
            }
        }
        {
            let old: u16 = read(game, e, Field::DeathCounter, |state| {
                state.head.death_counter
            });
            {
                let value: u16 = old.wrapping_add(1);
                write(game, e, Field::DeathCounter, value, |state, value| {
                    state.head.death_counter = value
                });
            }
        };
    } else {
        {
            let old: u16 = read(game, e, Field::Properties, |state| state.actor.properties);
            {
                let value: u16 = old | K_ENEMY_PROPS_DELETED as u16;
                write(game, e, Field::Properties, value, |state, value| {
                    state.actor.properties = value
                });
            }
        };
        {
            game.set_boss_bit(BOTWOON_DEAD);
            game.queue_music_delayed(ELEVATOR_MUSIC);
        };
    }
}
fn mark_hit<R: Host>(h: &mut R, actor: Key) {
    if h.resolve_actor(actor, Native).health == 0 {
        set_death_signal(h, actor.byte_index(), Signal::PendingHit, 1);
        h.edit_actor(Key::from_legacy(0), Native, |actor| {
            actor.properties |= INTANGIBLE;
        });
    }
}
/// Enemy shot - Botwoon.
pub fn shot<R: Host>(h: &mut R) {
    let actor = h.selected_actor();
    let health = h.resolve_actor(actor, Native).health;
    set_death_signal(h, actor.byte_index(), Signal::PreviousHealth, health);
    combat::shot_reaction(h, ShotHandling::ReactionOnly);
    mark_hit(h, actor);
}
/// Enemy touch ($B3:9FFF) or, with `power_bomb`, power bomb reaction - Botwoon.
pub fn contact<R: Host>(power_bomb: bool, h: &mut R) {
    if power_bomb {
        combat::power_bomb_reaction(h, DeathHandling::LeavePending);
    } else {
        combat::contact_reaction(h, DeathHandling::LeavePending);
    }
    let actor = h.selected_actor();
    mark_hit(h, actor);
}
/// Choose Botwoon movement path ($B3:9933), setting its speed ($B3:995D) by
/// health.
fn choose_movement_path<L: Host>(k: u16, game: &mut L) {
    let e = i32::from(k);
    if read(game, e, Field::HeadHidden, |state| state.head.head_hidden) == 0 {
        {
            let value: u16 = 0;
            write(game, e, Field::SpeedTableIndex, value, |state, value| {
                state.head.speed_table_index = value
            });
        };
        let health = read(game, e, Field::Health, |state| state.actor.health) as i16;
        if health > 0
            && health.wrapping_sub(
                read(game, e, Field::HalfHealth, |state| state.path.half_health) as i16,
            ) < 0
        {
            let speed = if health.wrapping_sub(read(game, e, Field::QuarterHealth, |state| {
                state.path.quarter_health
            }) as i16)
                < 0
            {
                2
            } else {
                1
            };
            {
                let value: u16 = speed;
                write(game, e, Field::SpeedTableIndex, value, |state, value| {
                    state.head.speed_table_index = value
                });
            };
            let index = (4u16.wrapping_mul(read(game, e, Field::SpeedTableIndex, |state| {
                state.head.speed_table_index
            })) >> 1) as isize;
            let level = game.botwoon_speed_level(index);
            {
                {
                    let value: u16 = level.speed;
                    write(game, e, Field::Speed, value, |state, value| {
                        state.head.speed = value
                    });
                }
                {
                    let value: u16 = level.body_travel_time;
                    write(game, e, Field::BodyTravelTime, value, |state, value| {
                        state.actor.variables[2] = value
                    });
                }
            };
        }
    }
    let bias = if read(game, e, Field::HeadHidden, |state| state.head.head_hidden) != 0 {
        HIDDEN_PATHS
    } else {
        0
    };
    {
        let value: u16 = (i32::from(game.roll_random() & PATH_CHOICES)
            + bias
            + 4 * i32::from(read(game, e, Field::TargetHole, |state| {
                state.head.target_hole
            }))) as u16;
        write(game, e, Field::MovementTableIndex, value, |state, value| {
            state.path.movement_table_index = value
        });
    };
}
fn clamp_delta(value: u16) -> u16 {
    if (i32::from(value) - i32::from(MAX_OFFSET + 1)) & SIGN == 0 {
        MAX_OFFSET
    } else if (i32::from(value) + i32::from(MAX_OFFSET + 1)) & SIGN != 0 {
        MAX_OFFSET.wrapping_neg()
    } else {
        value
    }
}
/// Botwoon movement function - move directly toward target hole ($B3:9BB7),
/// from the offsets to it ($B3:9BF8). Was `seek_hole`.
pub fn move_toward_target_hole<L: Host>(game: &mut L) {
    let e = i32::from(game.current_enemy());
    let index = (read(game, e, Field::TargetHole, |state| state.head.target_hole) >> 1) as isize;
    let target = game.botwoon_hole(index);
    let x = clamp_delta(
        (i32::from(target.left) + 4
            - i32::from(read(game, e, Field::XPos, |state| state.actor.x.whole))) as u16,
    );
    let y = clamp_delta(
        (i32::from(target.top) + 4
            - i32::from(read(game, e, Field::YPos, |state| state.actor.y.whole))) as u16,
    );
    let angle = queries::angle_from_delta(x, y);
    let e = i32::from(game.current_enemy());
    {
        {
            let value: u16 = angle;
            write(game, e, Field::UnreadHoleAngle, value, |state, value| {
                state.head.unread_hole_angle = value
            });
        }
        {
            let value: u16 = u16::from(QUARTER_TURN.wrapping_sub(angle as u8));
            write(game, e, Field::TargetHoleAngle, value, |state, value| {
                state.head.target_hole_angle = value
            });
        }
    };
    if read(game, e, Field::HeadHidden, |state| state.head.head_hidden)
        == read(game, e, Field::PreviousHeadHidden, |state| {
            state.head.previous_head_hidden
        })
    {
        move_by_angle(game);
    } else {
        {
            {
                let value: u16 = read(game, e, Field::HeadHidden, |state| state.head.head_hidden);
                write(game, e, Field::PreviousHeadHidden, value, |state, value| {
                    state.head.previous_head_hidden = value
                });
            }
            {
                let value: u16 = 1;
                write(game, e, Field::GoingThroughHole, value, |state, value| {
                    state.head.going_through_hole = value
                });
            }
        };
    }
}
/// Move Botwoon according to speed and angle to target hole ($B3:9C48).
fn move_by_angle<L: Host>(game: &mut L) {
    let e = i32::from(game.current_enemy());
    let phase = read(game, e, Field::TargetHoleAngle, |state| {
        state.head.target_hole_angle
    });
    let amplitude = read(game, e, Field::Speed, |state| state.head.speed);
    let vector = game.direction_step(phase, amplitude);
    let scratch = read(game, e, Field::TargetHoleAngle, |state| {
        state.head.target_hole_angle
    });
    game.set_step_scratch(scratch);
    let phase = read(game, e, Field::TargetHoleAngle, |state| {
        state.head.target_hole_angle
    });
    game.step_along(phase, vector);
}
/// Botwoon movement function - start moving Botwoon according to movement data
/// ($B3:E250). Was `load_path`.
pub fn start_moving_by_movement_data<L: Host>(game: &mut L) {
    let e = i32::from(game.current_enemy());
    {
        {
            let value: u16 = movement_function::MOVE_BY_MOVEMENT_DATA;
            write(game, e, Field::MovementFunction, value, |state, value| {
                state.actor.variables[4] = value
            });
        }
        {
            let value: u16 = 0;
            write(game, e, Field::GoingThroughHole, value, |state, value| {
                state.head.going_through_hole = value
            });
        }
    };
    let record = native_path_record(
        BANK as u8,
        read(game, e, Field::MovementTableIndex, |state| {
            state.path.movement_table_index
        })
        .wrapping_add(MOVEMENT_TABLE),
        game,
    );
    {
        {
            let value: u16 = record.word(0);
            write(
                game,
                e,
                Field::MovementDataPointer,
                value,
                |state, value| state.path.movement_data_pointer = value,
            );
        }
        {
            let value: u16 = record.word(4);
            write(game, e, Field::TargetHole, value, |state, value| {
                state.head.target_hole = value
            });
        }
    };
    let direction = record.word(2) as i16;
    {
        let value: u16 = direction as u16;
        write(game, e, Field::MovementDirection, value, |state, value| {
            state.path.movement_direction = value
        });
    };
    if direction < 0 {
        {
            let old: u16 = read(game, e, Field::MovementDataPointer, |state| {
                state.path.movement_data_pointer
            });
            {
                let value: u16 = old.wrapping_sub(4);
                write(
                    game,
                    e,
                    Field::MovementDataPointer,
                    value,
                    |state, value| state.path.movement_data_pointer = value,
                );
            }
        };
    }
    move_by_movement_data(game);
}
/// Botwoon movement function - move Botwoon according to movement data
/// ($B3:E28C), one step per pixel of speed, until the data ends. Was
/// `advance_path`.
pub fn move_by_movement_data<L: Host>(game: &mut L) {
    let e = i32::from(game.current_enemy());
    let mut delta = [0u16; 2];
    let mut remaining = i32::from(read(game, e, Field::Speed, |state| state.head.speed));
    let step = if read(game, e, Field::MovementDirection, |state| {
        state.path.movement_direction
    }) as i32
        & SIGN
        == 0
    {
        2
    } else {
        -2
    };
    loop {
        let record = native_path_record(
            BANK as u8,
            read(game, e, Field::MovementDataPointer, |state| {
                state.path.movement_data_pointer
            }),
            game,
        );
        for (axis, total) in delta.iter_mut().enumerate() {
            let amount =
                u16::from_ne_bytes(i16::from(i8::from_ne_bytes([record.byte(axis)])).to_ne_bytes());
            if amount == END_OF_DATA {
                {
                    {
                        let value: u16 = 0;
                        write(game, e, Field::Unused8802, value, |state, value| {
                            state.path.unused_8802 = value
                        });
                    }
                    {
                        let value: u16 = 1;
                        write(game, e, Field::GoingThroughHole, value, |state, value| {
                            state.head.going_through_hole = value
                        });
                    }
                };
                return;
            }
            *total = total.wrapping_add(amount);
        }
        {
            let old: u16 = read(game, e, Field::MovementDataPointer, |state| {
                state.path.movement_data_pointer
            });
            {
                let value: u16 = (i32::from(old) + step) as u16;
                write(
                    game,
                    e,
                    Field::MovementDataPointer,
                    value,
                    |state, value| state.path.movement_data_pointer = value,
                );
            }
        };
        remaining -= 1;
        if remaining == 0 {
            break;
        }
    }
    if read(game, e, Field::MovementDirection, |state| {
        state.path.movement_direction
    }) as i32
        & SIGN
        != 0
    {
        delta = delta.map(u16::wrapping_neg);
    }
    {
        let old: u16 = read(game, e, Field::XPos, |state| state.actor.x.whole);
        {
            let value: u16 = old.wrapping_add(delta[0]);
            write(game, e, Field::XPos, value, |state, value| {
                state.actor.x.whole = value
            });
        }
    };
    {
        let old: u16 = read(game, e, Field::YPos, |state| state.actor.y.whole);
        {
            let value: u16 = old.wrapping_add(delta[1]);
            write(game, e, Field::YPos, value, |state, value| {
                state.actor.y.whole = value
            });
        }
    };
}
/// Start the head's instruction list `instruction`.
fn set_instruction_list<L: Host>(e: i32, instruction: u16, game: &mut L) {
    {
        let value: u16 = instruction;
        write(game, e, Field::CurrentInstruction, value, |state, value| {
            state.actor.instruction = value
        });
    };
    {
        let value: u16 = instruction;
        write(game, e, Field::InstructionList, value, |state, value| {
            state.head.instruction_list = value
        });
    };
    {
        {
            let value: u16 = 1;
            write(game, e, Field::InstructionTimer, value, |state, value| {
                state.actor.instruction_delay = value
            });
        }
        {
            let value: u16 = 0;
            write(game, e, Field::Timer, value, |state, value| {
                state.actor.timer = value
            });
        }
    };
}
/// Botwoon head function - moving around ($B3:9DC0): hide or face along the
/// head's motion over the last 4 frames, then shift the position history. Was
/// `animate_motion`.
pub fn head_moving_around<L: Host>(game: &mut L) {
    let e = i32::from(game.current_enemy());
    let x = read(game, e, Field::XPos, |state| state.actor.x.whole).wrapping_sub(read(
        game,
        e,
        Field::X4FramesAgo,
        |state| state.path.x_4_frames_ago,
    ));
    let y = read(game, e, Field::YPos, |state| state.actor.y.whole).wrapping_sub(read(
        game,
        e,
        Field::Y4FramesAgo,
        |state| state.path.y_4_frames_ago,
    ));
    let angle = queries::angle_from_delta(x, y);
    if x != 0 || y != 0 {
        let instruction = if read(game, e, Field::HeadHidden, |state| state.head.head_hidden) != 0 {
            {
                let value: u16 = HIDDEN_LAYER;
                write(game, e, Field::Layer, value, |state, value| {
                    state.actor.layer = value
                });
            };
            {
                let old: u16 = read(game, e, Field::Properties, |state| state.actor.properties);
                {
                    let value: u16 = old | INTANGIBLE;
                    write(game, e, Field::Properties, value, |state, value| {
                        state.actor.properties = value
                    });
                }
            };
            HIDE
        } else {
            {
                let value: u16 = SHOWN_LAYER;
                write(game, e, Field::Layer, value, |state, value| {
                    state.actor.layer = value
                });
            };
            {
                let old: u16 = read(game, e, Field::Properties, |state| state.actor.properties);
                {
                    let value: u16 = old & !INTANGIBLE;
                    write(game, e, Field::Properties, value, |state, value| {
                        state.actor.properties = value
                    });
                }
            };
            // The original faces the raw angle; snesrev smoothed it.
            crate::actor_tables::word(game, |t| &t.botwoon_head_lists, (angle >> 5) as isize)
        };
        if instruction
            != read(game, e, Field::InstructionList, |state| {
                state.head.instruction_list
            })
        {
            set_instruction_list(e, instruction, game);
        }
    }
    {
        {
            let value: u16 = read(game, e, Field::X3FramesAgo, |state| {
                state.path.x_3_frames_ago
            });
            write(game, e, Field::X4FramesAgo, value, |state, value| {
                state.path.x_4_frames_ago = value
            });
        }
        {
            let value: u16 = read(game, e, Field::Y3FramesAgo, |state| {
                state.path.y_3_frames_ago
            });
            write(game, e, Field::Y4FramesAgo, value, |state, value| {
                state.path.y_4_frames_ago = value
            });
        }
        {
            let value: u16 = read(game, e, Field::X2FramesAgo, |state| {
                state.path.x_2_frames_ago
            });
            write(game, e, Field::X3FramesAgo, value, |state, value| {
                state.path.x_3_frames_ago = value
            });
        }
        {
            let value: u16 = read(game, e, Field::Y2FramesAgo, |state| {
                state.path.y_2_frames_ago
            });
            write(game, e, Field::Y3FramesAgo, value, |state, value| {
                state.path.y_3_frames_ago = value
            });
        }
        {
            let value: u16 = read(game, e, Field::X1FrameAgo, |state| state.path.x_1_frame_ago);
            write(game, e, Field::X2FramesAgo, value, |state, value| {
                state.path.x_2_frames_ago = value
            });
        }
        {
            let value: u16 = read(game, e, Field::Y1FrameAgo, |state| state.path.y_1_frame_ago);
            write(game, e, Field::Y2FramesAgo, value, |state, value| {
                state.path.y_2_frames_ago = value
            });
        }
        {
            let value: u16 = read(game, e, Field::XPos, |state| state.actor.x.whole);
            write(game, e, Field::X1FrameAgo, value, |state, value| {
                state.path.x_1_frame_ago = value
            });
        }
        {
            let value: u16 = read(game, e, Field::YPos, |state| state.actor.y.whole);
            write(game, e, Field::Y1FrameAgo, value, |state, value| {
                state.path.y_1_frame_ago = value
            });
        }
    };
}
/// Botwoon head function - spitting - set angle and show ($B3:9E7D): aim at
/// Samus, then spit 5 or 3; returns that head function. Was `aim_spit`.
pub fn head_set_spit_angle<L: Host>(game: &mut L) -> u32 {
    let e = i32::from(game.current_enemy());
    {
        let value: u16 = SHOWN_LAYER;
        write(game, e, Field::Layer, value, |state, value| {
            state.actor.layer = value
        });
    };
    let actor = game.current_enemy();
    let angle = game.player_angle(actor);
    {
        let value: u16 = angle;
        write(game, e, Field::SpitAngle, value, |state, value| {
            state.head.spit_angle = value
        });
    };
    let instruction = crate::actor_tables::word(
        game,
        |t| &t.botwoon_spit_lists,
        ((angle as u8).wrapping_add(16) >> 5) as isize,
    );
    set_instruction_list(e, instruction, game);
    {
        {
            let value: u16 = u16::from(QUARTER_TURN.wrapping_sub(read(
                game,
                e,
                Field::SpitAngle,
                |state| state.head.spit_angle,
            ) as u8));
            write(game, e, Field::SpitAngle, value, |state, value| {
                state.head.spit_angle = value
            });
        }
        {
            let value: u16 = head_function::SPITTING_SPAWN_5;
            write(game, e, Field::HeadFunction, value, |state, value| {
                state.actor.pre_instruction = value
            });
        }
    };
    if read(game, e, Field::Function, |state| state.actor.variables[3]) != function::SPITTING {
        {
            let value: u16 = head_function::SPITTING_SPAWN_3;
            write(game, e, Field::HeadFunction, value, |state, value| {
                state.actor.pre_instruction = value
            });
        };
    }
    (BANK << 16)
        | u32::from(read(game, e, Field::HeadFunction, |state| {
            state.actor.pre_instruction
        }))
}
/// Botwoon head function - spitting - spawn 5 ($B3:9EE0, `wide`, once the
/// spitting flag is set) or 3 ($B3:9F34) spit projectiles. Was `spit`.
pub fn head_spit<L: Host>(wide: bool, game: &mut L) {
    let e = i32::from(game.current_enemy());
    if wide && read(game, e, Field::Spitting, |state| state.path.spitting) == 0 {
        return;
    }
    set_spit_angle(
        game,
        (read(game, e, Field::SpitAngle, |state| state.head.spit_angle) as u8)
            .wrapping_sub(if wide { 2 * SPIT_SPREAD } else { SPIT_SPREAD }),
    );
    let speed = SPIT_SPEEDS[read(game, e, Field::SpeedTableIndex, |state| {
        state.head.speed_table_index
    }) as usize];
    for _ in 0..if wide { 5 } else { 3 } {
        let k = game.current_enemy();
        game.spawn_actor_eproj(speed, k, ADDR_K_EPROJ_BOTWOONS_SPIT as u16);
        set_spit_angle(game, spit_angle(game).wrapping_add(SPIT_SPREAD));
    }
    let target = if wide {
        {
            let value: u16 = 0;
            write(game, e, Field::Spitting, value, |state, value| {
                state.path.spitting = value
            });
        };
        e
    } else {
        i32::from(game.current_enemy())
    };
    {
        let value: u16 = head_function::SPITTING_COOLDOWN;
        write(game, target, Field::HeadFunction, value, |state, value| {
            state.actor.pre_instruction = value
        });
    };
}
/// Botwoon head function - spitting - cooldown ($B3:9F7A). Was `wait_spit`.
pub fn head_cooldown<L: Host>(k: u16, game: &mut L) {
    let e = i32::from(k);
    let delay =
        (read(game, e, Field::SpitTimer, |state| state.head.spit_timer) as i16).wrapping_sub(1);
    {
        let value: u16 = delay as u16;
        write(game, e, Field::SpitTimer, value, |state, value| {
            state.head.spit_timer = value
        });
    };
    if delay < 0 {
        {
            {
                let value: u16 = 0;
                write(game, e, Field::SpitTimer, value, |state, value| {
                    state.head.spit_timer = value
                });
            }
            {
                let value: u16 = head_function::MOVING_AROUND;
                write(game, e, Field::HeadFunction, value, |state, value| {
                    state.actor.pre_instruction = value
                });
            }
        };
    }
}
/// Botwoon / hole collision detection ($B3:9F93): entering a hole toggles the
/// head between hidden and shown. Was `detect_portal`.
pub fn hole_collision<L: Host>(k: u16, game: &mut L) {
    let e = i32::from(game.current_enemy());
    if read(game, i32::from(k), Field::HoleCollisionDisabled, |state| {
        state.head.hole_collision_disabled
    }) != 0
    {
        return;
    }
    for index in [12, 8, 4, 0] {
        let hole = game.botwoon_hole(index);
        let bounds = [hole.left, hole.right, hole.top, hole.bottom];
        if bounds.iter().enumerate().all(|(offset, &bound)| {
            let coordinate = if offset < 2 {
                read(game, e, Field::XPos, |state| state.actor.x.whole)
            } else {
                read(game, e, Field::YPos, |state| state.actor.y.whole)
            };
            ((coordinate as i16).wrapping_sub(bound as i16) >= 0) == (offset % 2 == 0)
        }) {
            {
                let value: u16 = 1;
                write(
                    game,
                    e,
                    Field::HoleCollisionDisabled,
                    value,
                    |state, value| state.head.hole_collision_disabled = value,
                );
            };
            {
                let old: u16 = read(game, e, Field::HeadHidden, |state| state.head.head_hidden);
                {
                    let value: u16 = old ^ 1;
                    write(game, e, Field::HeadHidden, value, |state, value| {
                        state.head.head_hidden = value
                    });
                }
            };
            {
                let value: u16 = read(game, e, Field::PositionHistoryIndex, |state| {
                    state.actor.variables[1]
                });
                write(game, e, Field::TargetHistoryIndex, value, |state, value| {
                    state.head.target_history_index = value
                });
            };
            {
                let old: u16 = read(game, e, Field::UnreadCounter, |state| {
                    state.head.unread_counter
                });
                {
                    let value: u16 = old.wrapping_sub(1);
                    write(game, e, Field::UnreadCounter, value, |state, value| {
                        state.head.unread_counter = value
                    });
                }
            };
            return;
        }
        {
            let value: u16 = 0;
            write(
                game,
                e,
                Field::HoleCollisionDisabled,
                value,
                |state, value| state.head.hole_collision_disabled = value,
            );
        };
    }
}
/// Instruction - enemy radius = `x` x `y` ($B3:94C7, $B3:94D7, $B3:94E7).
pub fn set_hitbox<L: Host>(x: u16, y: u16, j: EnemyScript, game: &mut L) -> EnemyScript {
    let e = i32::from(game.current_enemy());
    {
        {
            let value: u16 = x;
            write(game, e, Field::XWidth, value, |state, value| {
                state.actor.radius.x = value
            });
        }
        {
            let value: u16 = y;
            write(game, e, Field::YHeight, value, |state, value| {
                state.actor.radius.y = value
            });
        }
    };
    j
}
/// Instruction - set Botwoon spitting flag ($B3:9567).
pub fn set_spitting<L: Host>(j: EnemyScript, game: &mut L) -> EnemyScript {
    let e = i32::from(game.current_enemy());
    {
        let value: u16 = 1;
        write(game, e, Field::Spitting, value, |state, value| {
            state.path.spitting = value
        });
    };
    j
}

/// Initialisation AI - Botwoon ($B3:9583): once dead, clear its wall.
pub fn initialize<L: Host>(game: &mut L) {
    let e = i32::from(game.current_enemy());
    if u16::from(game.boss_byte(MARIDIA)) & BOTWOON_DEAD != 0 {
        {
            game.spawn_plm(WALL_BLOCK.0, WALL_BLOCK.1, CLEAR_WALL);
            game.open_passage();
        };
        {
            let value: u16 = HIDE;
            write(game, e, Field::CurrentInstruction, value, |state, value| {
                state.actor.instruction = value
            });
        };
        {
            let old: u16 = read(game, e, Field::Properties, |state| state.actor.properties);
            {
                let value: u16 = old | K_ENEMY_PROPS_DELETED as u16;
                write(game, e, Field::Properties, value, |state, value| {
                    state.actor.properties = value
                });
            }
        };
    } else {
        initialize_living(e, game);
    }
}
/// Main AI - Botwoon ($B3:9668): run its function, then its palette. Was `update`.
pub fn main_ai<R: Host>(runtime: &mut R) {
    arm_death(runtime);
    let actor = runtime.selected_actor();
    let state = runtime.resolve_actor(actor, Native).variables[3];
    let k = runtime.selected_actor().byte_index();
    match state {
        function::INITIAL => initial(runtime),
        function::GO_THROUGH_HOLE | function::MOVING_AROUND => {
            let game = &mut *runtime;
            let e = i32::from(game.current_enemy());
            let changed = if state == function::GO_THROUGH_HOLE {
                leave_hole(e, game)
            } else {
                moving_around(e, game)
            };
            if !changed {
                step_body(e, game);
                if state == function::GO_THROUGH_HOLE {
                    hole_collision(game.current_enemy(), game);
                }
            }
        }
        function::SPITTING => {
            let e = i32::from(runtime.selected_actor().byte_index());
            if spitting(e, runtime) {
                let address = (BANK << 16)
                    | u32::from(read(runtime, e, Field::HeadFunction, |state| {
                        state.actor.pre_instruction
                    }));
                pre_instruction(address, runtime.current_enemy(), runtime);
            }
        }
        function::DEATH_PRE_DEATH_DELAY => pre_death_delay(runtime),
        function::DEATH_FALLING_TO_GROUND => fall_to_ground(runtime),
        function::DEATH_WAIT_FOR_BODY_TO_FALL => {
            let actor = runtime.selected_actor();
            if death_signal(runtime, actor.byte_index(), Signal::BodyLanded) != 0 {
                runtime.edit_actor(actor, Native, |actor| {
                    actor.variables[3] = function::DEATH_CRUMBLING_WALL
                });
                let k = runtime.selected_actor().byte_index();
                {
                    // Spawn Botwoon item drops and start crumbling wall ($B3:9ADD).
                    runtime.spawn_plm(WALL_BLOCK.0, WALL_BLOCK.1, CRUMBLE_WALL);
                    crate::enemy_projectiles::drops::scatter(
                        crate::enemy_projectiles::drops::DropSource::Botwoon,
                        k,
                        runtime,
                    );
                };
                {
                    {
                        let value: u16 = 0;
                        write(
                            runtime,
                            i32::from(k),
                            Field::DeathCounter,
                            value,
                            |state, value| state.head.death_counter = value,
                        );
                    }
                    {
                        let value: u16 = 0;
                        write(
                            runtime,
                            i32::from(k),
                            Field::LargeWallExplosionTimer,
                            value,
                            |state, value| state.head.large_wall_explosion_timer = value,
                        );
                    }
                    {
                        let value: u16 = 0;
                        write(
                            runtime,
                            i32::from(k),
                            Field::WallSmokeTimer,
                            value,
                            |state, value| state.head.wall_smoke_timer = value,
                        );
                    }
                };
            }
        }
        function::DEATH_CRUMBLING_WALL => crumbling_wall(k, runtime),
        _ => {
            Host::unreachable(runtime);
        }
    }
    health_palette(runtime);
}
/// Run the movement function, then the head's, carrying the body along the
/// position history.
fn step_body<L: Host>(e: i32, game: &mut L) {
    match read(game, e, Field::MovementFunction, |state| {
        state.actor.variables[4]
    }) {
        movement_function::MOVE_TOWARD_TARGET_HOLE => move_toward_target_hole(game),
        movement_function::MOVE_BY_MOVEMENT_DATA => move_by_movement_data(game),
        movement_function::START_MOVING_BY_MOVEMENT_DATA => start_moving_by_movement_data(game),
        _ => {
            Host::unreachable(game);
        }
    }
    {
        crate::enemy_projectiles::attachments_fire_arc_botwoon::record_botwoon_head(game);
        crate::enemy_projectiles::attachments_fire_arc_botwoon::follow_botwoon_history(game);
    };
    let address = (BANK << 16)
        | u32::from(read(game, e, Field::HeadFunction, |state| {
            state.actor.pre_instruction
        }));
    pre_instruction(address, game.current_enemy(), game);
    {
        crate::enemy_projectiles::attachments_fire_arc_botwoon::orient_botwoon_body(game);
        crate::enemy_projectiles::attachments_fire_arc_botwoon::advance_botwoon_history(game);
    };
}

/// Run the head callback, including the immediate spawn after aiming.
pub fn pre_instruction<L: Host>(address: u32, k: u16, game: &mut L) {
    match address {
        11763835 => (),
        11767039 => {
            let e = i32::from(game.current_enemy());
            let counter = read(game, e, Field::BodyTravelTime, |state| {
                state.actor.variables[2]
            })
            .wrapping_add(1);
            write(game, e, Field::BodyTravelTime, counter, |state, value| {
                state.actor.variables[2] = value
            });
            write(
                game,
                e,
                Field::BodyProjectileIndex,
                0x8e5a,
                |state, value| state.actor.variables[0] = value,
            );
        }
        11771328 => head_moving_around(game),
        11771517 => {
            let address = head_set_spit_angle(game);
            pre_instruction(address, game.current_enemy(), game);
        }
        11771616 => head_spit(true, game),
        11771700 => head_spit(false, game),
        11771770 => head_cooldown(k, game),
        _ => Host::unreachable(game),
    }
}
