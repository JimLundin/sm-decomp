//! Grapple controller, swing placement, rope-length changes and swept block contact.
use crate::actors::{self, api as actor_api};
use crate::combat_rules::GrappleOutcome;
use crate::constants::{buttons::*, samus_movement::*, samus_poses::*};
use crate::grapple_aim::{self, ConnectionSet};
use crate::grapple_origins::{self, OriginSet};
use crate::player_collision::operations as player_collision;
use crate::player_transitions as pt;
use crate::player_transitions::state::{
    DrawMode, HurtAction, InterruptionAction, MovementMode, PoseRequestWord,
};

/// Samus's movement handler after a grapple launch ($90:946E).
const GRAPPLE_LAUNCH_MOVEMENT: MovementMode = MovementMode::from_raw(0x946e);
use crate::actors::host::abs16;
use crate::projectile_geometry::SINE_COSINE;
use crate::projectile_room::Axis;
use crate::ram::{word, words as ram_words};
use crate::word_transfers;

/// What the grapple controller needs from the runtime besides named RAM,
/// sound, the cartridge images and the owned-Session view.
/// The grapple runs on a Session lease: its Session-owned player fields are
/// read and written by name, the rest through the lease's RAM projection.
pub trait Host:
    crate::session_access::SessionAccess
    + crate::ram::CompatibilityRam
    + crate::effects::SoundEffects
    + crate::cartridge::NativeImage
    + crate::cartridge::LoadedImage
    + crate::records::Records
    + crate::player_position::PlayerPosition
    + crate::player_position::PlayerFacts
{
    /// The owned-Session view the grapple's player rules run on.
    type Lease<'a>: crate::runtime_access::SessionView
    where
        Self: 'a;
    /// Run `action` on a Session lease: a fresh one per call for the compat
    /// game, or the lease the grapple already runs inside.
    fn with_lease<'s, R>(&'s mut self, action: impl FnOnce(&mut Self::Lease<'s>) -> R) -> R;
    /// The original's unreachable-state failure.
    fn grapple_unreachable(&mut self) -> bool;
    /// Run a Samus pose command, as the original's command call.
    fn grapple_pose_command(&mut self, command: u16) -> u16;
    /// Multiply through the hardware scratch words at $05E9/$05EB.
    fn multiply_with_scratch(&mut self, a: u16, b: u16) -> u32;
    /// Samus's bottom edge under the current pose.
    fn player_bottom(&self) -> u16;
    /// Settle the owned grapple state after the beam connects or releases.
    fn settle_grapple(&mut self);
    /// The native word spanning the block behaviour bytes at `index` and the
    /// next block ($7F:6402), as the original read it.
    fn block_behavior_word(&self, index: usize) -> i16;
    /// The periodic damage accumulator ($0A4E fraction, $0A50 whole), as two
    /// native words read in that order.
    fn periodic_damage(&self) -> (u16, u16);
}

/// The grapple swing's working words, owned by `Session::player.swing`. Names
/// follow the $7E RAM map; the swing itself is bank $9B.
#[derive(Clone, Default, Debug, PartialEq)]
pub struct SwingState {
    /// $0CFC: the end angle's mirror of $0CFA, indexing the swing offsets ($9B:C1C2).
    pub mirrored_angle: u16,
    /// $0D00: change applied to the beam length.
    pub length_delta: u16,
    /// $0D16/$0D18: the beam's start position.
    pub start: crate::geometry::Point,
    /// $0D26: angular velocity, pi/8000h radians; positive is clockwise.
    pub angular_velocity: u16,
    /// $0D28: angular acceleration due to the swing's angle.
    pub angle_acceleration: u16,
    /// $0D2A: angular acceleration due to button input.
    pub input_acceleration: u16,
    /// $0D2C: angular deceleration due to the swing's angle, while Samus rises.
    pub rising_deceleration: u16,
    /// $0D2E: extra angular velocity, outside the 480h speed cap.
    pub extra_angular_velocity: u16,
    /// $0D30: kick cooldown, set after bouncing off a wall or ceiling.
    pub kick_cooldown: u16,
    /// $0D38: failed swing movement counter ($94:ACFE's unused branch).
    pub failed_movement_counter: u16,
    /// $0D3A/$0D3C: set to 2 and cleared when the beam starts; never read.
    pub start_marks: [u16; 2],
    /// $0D90/$0D92: the swing collision probe's position ($94:A957).
    pub probe: crate::geometry::Point,
    /// $0D94/$0D96: the swing collision probe's block ($94:A957).
    pub probe_block: crate::geometry::Point,
    /// $0D3E: the beam start animation timer.
    pub start_animation_timer: u16,
    /// $0D40: the beam start animation's tile pointer (bank $9A).
    pub start_animation_tiles: u16,
}

/// The 8-bit by 8-bit hardware product.
fn multiply_8x8(a: u8, b: u8) -> u16 {
    u16::from(a) * u16::from(b)
}

/// Follow extension blocks until the selected reaction finishes; callbacks can
/// change the live block index before requesting another reaction.
fn react_to_block<H: Host + ?Sized>(game: &mut H, mut react: impl FnMut(u16, &mut H) -> u8) -> u8 {
    loop {
        let offset = 0x10002 + usize::from(word!(game.CUR_BLOCK_INDEX)) * 2;
        let block = u16::from_le_bytes([
            game.compatibility_byte(offset),
            game.compatibility_byte(offset + 1),
        ]);
        let result = react(block >> 12, game);
        if result & 0x80 == 0 {
            return result;
        }
    }
}

fn project_axis(origin: u16, sample: i16, length: u16) -> u16 {
    let distance = if sample == -256 || sample == 256 {
        length
    } else {
        (u16::from(length as u8) * u16::from(sample.unsigned_abs() as u8)) >> 8
    };
    if sample < 0 {
        origin.wrapping_sub(distance)
    } else {
        origin.wrapping_add(distance)
    }
}
fn place_sample<H: Host + ?Sized>(angle: u16, length: u16, game: &mut H) {
    {
        let index = (angle >> 1) as usize;
        if game.session().player.grapple_flags & 0x8000 == 0 {
            let offset = if SINE_COSINE[index + 64] < 0 { 7 } else { 8 };
            game.session_mut().player.grapple_endpoint.x =
                game.session().player.grapple_endpoint.x & 0xfff0 | offset;
            let offset = if SINE_COSINE[index] < 0 { 7 } else { 8 };
            game.session_mut().player.grapple_endpoint.y =
                game.session().player.grapple_endpoint.y & 0xfff0 | offset;
        }
        let x = project_axis(
            game.session().player.grapple_endpoint.x,
            SINE_COSINE[index + 64],
            length,
        );
        game.session_mut().player.swing.probe.x = x;
        game.session_mut().player.swing.probe_block.x = (x >> 4) as u8 as u16;
        let y = project_axis(
            game.session().player.grapple_endpoint.y,
            SINE_COSINE[index],
            length,
        );
        game.session_mut().player.swing.probe.y = y;
        game.session_mut().player.swing.probe_block.y = (y >> 4) as u8 as u16;
    }
}
fn contact<H: Host + ?Sized>(game: &mut H) -> u8 {
    let x = game.session().player.swing.probe_block.x;
    let row = multiply_8x8(
        game.session().player.swing.probe_block.y as u8,
        word!(game.ROOM_WIDTH_IN_BLOCKS) as u8,
    );
    word!(game.CUR_BLOCK_INDEX = row.wrapping_add(x));
    react_to_block(game, |block_type, game| match block_type {
        0 | 3 | 4 | 6 | 7 => 0,
        1 | 8 | 9 | 11 | 12 | 14 | 15 => 1,
        2 => spike_contact(false, game),
        5 => game
            .with_lease(|lease| crate::player_collision::grapple::redirect_block(lease, Axis::X)),
        10 => spike_contact(true, game),
        13 => game
            .with_lease(|lease| crate::player_collision::grapple::redirect_block(lease, Axis::Y)),
        _ => unreachable!(),
    })
}
static AIR_DAMAGE: [u16; 16] = [0, 0, 0x10, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
static SOLID_DAMAGE: [u16; 16] = [60, 16, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
fn spike_contact<H: Host + ?Sized>(solid: bool, game: &mut H) -> u8 {
    if game.session().player.invincibility == 0 {
        // The original reads a word spanning adjacent BTS bytes, not just one byte.
        let kind = game.block_behavior_word(usize::from(word!(game.CUR_BLOCK_INDEX)));
        if kind >= 0 {
            let damage = if solid {
                SOLID_DAMAGE[kind as usize]
            } else {
                AIR_DAMAGE[kind as usize]
            };
            let amount = ((damage as i16 as i32) << 16) as u32;
            if amount != 0 {
                let (low, high) = game.periodic_damage();
                let (low, high) = (low as u32, high as u32);
                let sum = (low | (high << 16)).wrapping_add(amount);
                word_transfers!(game, ascending, 1;
                    native(0xa4e) <- value(sum as u16),
                    native(0xa50) <- value((sum >> 16) as u16));
                game.session_mut().player.invincibility = 60;
                game.session_mut().player.knockback_timer = 10;
            }
        }
    }
    solid as u8
}
fn check_clearance<H: Host + ?Sized>(angle: u16, game: &mut H) -> u8 {
    word!(game.GRAPPLE_COLLISION_DISTANCE = 6);
    let mut length = game.session().player.grapple_length.wrapping_add(8);
    loop {
        place_sample(angle, length, game);
        let blocked = contact(game) & 1;
        if blocked != 0 {
            return blocked;
        }
        length = length.wrapping_add(8);
        if word!(game.GRAPPLE_COLLISION_DISTANCE -= 1, reread) == 0 {
            return 0;
        }
    }
}
fn dispatch_connection<H: Host + ?Sized>(ea: u32, game: &mut H) {
    let (pose, transition_graphics) = match ea {
        10205657 => (K_POSE_B2_FACE_R_GRAPPLE_AIR as u16, 9),
        10205666 => (K_POSE_B3_FACE_L_GRAPPLE_AIR as u16, 9),
        10205674 => (K_POSE_A8_FACE_R_GRAPPLING as u16, 10),
        10205683 => (K_POSE_AA_FACE_R_GRAPPLING_AIM_DR as u16, 10),
        10205692 => (K_POSE_AB_FACE_L_GRAPPLING_AIM_DL as u16, 10),
        10205701 => (K_POSE_A9_FACE_L_GRAPPLING as u16, 10),
        10205710 => (K_POSE_B4_FACE_R_GRAPPLING_CROUCH as u16, 10),
        10205719 => (K_POSE_B6_FACE_R_GRAPPLING_CROUCH_AIM_DR as u16, 10),
        10205728 => (K_POSE_B7_FACE_L_GRAPPLING_CROUCH_AIM_DL as u16, 10),
        10205737 => (K_POSE_B5_FACE_L_GRAPPLING_CROUCH as u16, 10),
        _ => {
            game.grapple_unreachable();
            return;
        }
    };
    begin_connection(pose, transition_graphics, game);
}
pub fn connect_beam<H: Host + ?Sized>(game: &mut H) {
    if game.movement_type() as i32 == K_MOVEMENT_TYPE_1A_GRABBED_BY_DRAYGON as i32 {
        game.session_mut().player.grapple = (0x9bc77ei32) as u16;
        game.session_mut().player.swing.length_delta = 0;
    } else {
        let v1: i32 = 2 * game.session().player.grapple_direction as i32;
        let set = if word!(game.SAMUS_Y_SPEED) != 0 || word!(game.SAMUS_Y_SUBSPEED) != 0 {
            ConnectionSet::Airborne
        } else if game.movement_type() == 5 {
            ConnectionSet::Crouching
        } else {
            ConnectionSet::Standard
        };
        let connection = grapple_aim::connection(game, set, v1 as isize);
        game.session_mut().player.grapple = connection.beam_function;
        dispatch_connection(u32::from(connection.handler) | 0x9b0000, game);
    };
}
// Each field reacquires the registered native image at its original read point.
macro_rules! special_angle_word {
    ($game:ident, $row:ident, $offset:literal) => {
        $game.native_image().table_word(
            sm_data::cartridge::lorom_offset(0x9bc43e) as usize + 10 * usize::from($row) + $offset,
            0,
        )
    };
}
macro_rules! clamp_previous_grapple_position {
    ($game:ident, $current:ident, $previous:ident, $set_previous:ident) => {
        let delta: i16 = ($game.$current() as i16).wrapping_sub($game.$previous() as i16);
        if ($game.$current() as i16).wrapping_sub($game.$previous() as i16) < 0 {
            if delta as i32 + 12 & 0x8000 != 0 {
                let previous = $game.$current().wrapping_add(12);
                $game.$set_previous(previous);
            }
        } else if delta as i32 - 13 & 0x8000 == 0 {
            let previous = $game.$current().wrapping_sub(12);
            $game.$set_previous(previous);
        }
    };
}
pub fn apply_special_angle<H: Host + ?Sized>(game: &mut H) -> u8 {
    let mut row: u16 = 7;
    while game.session().player.grapple_angle != special_angle_word!(game, row, 0) {
        row = row.wrapping_sub(1);
        if row & 0x8000 != 0 {
            return 0;
        }
    }
    game.session_mut().player.transitions.interrupted =
        PoseRequestWord::from_raw(special_angle_word!(game, row, 2));
    game.set_player_x(
        game.session()
            .player
            .grapple_endpoint
            .x
            .wrapping_add(special_angle_word!(game, row, 4)),
    );
    game.set_player_y(
        game.session()
            .player
            .grapple_endpoint
            .y
            .wrapping_add(special_angle_word!(game, row, 6)),
    );
    game.session_mut().player.grapple = special_angle_word!(game, row, 8);
    game.session_mut().player.transitions.interruption = InterruptionAction::from_raw(0);
    game.session_mut().player.grapple_slow_scrolling = 0;
    clamp_previous_grapple_position!(game, player_x, previous_player_x, set_previous_player_x);
    clamp_previous_grapple_position!(game, player_y, previous_player_y, set_previous_player_y);
    1
}

fn begin_connection<H: Host + ?Sized>(pose: u16, transition_graphics: u16, game: &mut H) {
    game.session_mut().player.transitions.interrupted = PoseRequestWord::from_raw(pose);
    game.session_mut().player.transitions.interruption =
        InterruptionAction::from_raw(transition_graphics);
    let angle = u16::swap_bytes(actor_api::angle_from_delta(
        (game.player_x()).wrapping_sub(game.session().player.grapple_endpoint.x),
        (game.player_y()).wrapping_sub(game.session().player.grapple_endpoint.y),
    ));
    game.session_mut().player.grapple_angle = angle;
    game.session_mut().player.swing.mirrored_angle = angle;
    game.session_mut().player.swing.length_delta = 0;
    if (game.session().player.grapple_length as i32 - 64i32) & 0x8000 == 0 {
        game.session_mut().player.grapple_length =
            game.session().player.grapple_length.wrapping_sub(24);
    }
    place_rope_origin(game);
}

pub fn place_rope_origin<H: Host + ?Sized>(game: &mut H) {
    let angle = 2 * game.grapple_angle() as u16;
    let length = game.session().player.grapple_length;
    place_sample(angle, length, game);
    game.session_mut().player.swing.start.x = game.session().player.swing.probe.x;
    game.session_mut().player.swing.start.y = game.session().player.swing.probe.y;
}
pub fn change_length<H: Host + ?Sized>(game: &mut H) -> u8 {
    if game.session().player.swing.length_delta == 0 {
        return 0;
    }
    let (end, shortening) = length_target(&mut game.session_mut().player);
    let angle = 2 * game.grapple_angle() as u16;
    let mut length = game.session().player.grapple_length;
    while length != end {
        let sample_length = length.wrapping_add(if shortening { 7 } else { 57 });
        place_sample(angle, sample_length, game);
        if contact(game) != 0 {
            game.session_mut().player.grapple_length = length;
            return 1;
        }
        length = if shortening {
            length.wrapping_sub(1)
        } else {
            length.wrapping_add(1)
        };
    }
    game.session_mut().player.grapple_length = end;
    0
}
pub fn apply_input<H: Host + ?Sized>(game: &mut H) {
    if word!(game.JOYPAD1_NEWKEYS) & K_BUTTON_UP as u16 != 0 {
        if game.session().player.grapple_length != 0 {
            game.session_mut().player.swing.length_delta = (-2i16) as u16;
        }
    } else if word!(game.JOYPAD1_NEWKEYS) & K_BUTTON_DOWN as u16 != 0 {
        if (i32::from(game.session().player.grapple_length) - 64) & 0x8000 != 0 {
            game.session_mut().player.swing.length_delta = 2;
        } else {
            game.session_mut().player.grapple_length = 64;
        }
    }
    let angle = u16::from(game.grapple_angle()) << 8;
    if (0x4000..0xc000).contains(&angle) {
        let direction = if word!(game.JOYPAD1_LASTKEYS) & 0x200 != 0 {
            1i16
        } else if word!(game.JOYPAD1_LASTKEYS) & 0x100 != 0 {
            -1
        } else {
            0
        };
        if direction != 0 {
            if angle == 0x8000 && game.session().player.swing.angular_velocity == 0 {
                game.session_mut().player.swing.angular_velocity = (direction * 256) as u16;
            }
            let acceleration = if game.session().player.grapple_flags != 0
                && game.session().player.grapple_flags & 1 != 0
            {
                6
            } else {
                12
            };
            game.session_mut().player.swing.input_acceleration = (direction * acceleration) as u16;
            return;
        }
    }
    game.session_mut().player.swing.input_acceleration = 0;
}

/// The length a beam length change heads for, and whether it shortens: the
/// change stops at 8 when shortening and 63 ($3F) when lengthening.
pub fn length_target(player: &mut crate::session::Player) -> (u16, bool) {
    let end = player
        .swing
        .length_delta
        .wrapping_add(player.grapple_length);
    let shortening = player.swing.length_delta & 0x8000 != 0;
    if shortening && end < 8 {
        player.swing.length_delta = 0;
        (8, true)
    } else if !shortening && end >= 63 {
        player.swing.length_delta = 0;
        (63, false)
    } else {
        (end, shortening)
    }
}
/// The swing's angular acceleration and rising deceleration for its angle.
pub fn apply_gravity(player: &mut crate::session::Player) {
    let angle = player.grapple_angle;
    let liquid = player.grapple_flags != 0 && player.grapple_flags & 1 != 0;
    let swing = &mut player.swing;
    if angle & 0xff00 == 0x8000 {
        swing.angle_acceleration = 0;
        swing.rising_deceleration = 0;
        if abs16(swing.angular_velocity).to_ne_bytes()[1] == 0 {
            swing.angular_velocity = 0;
        }
        return;
    }
    let direction = if angle & 0x8000 != 0 { -1i16 } else { 1 };
    let shift = if matches!(angle & 0xc000, 0 | 0xc000) {
        2
    } else {
        0
    };
    swing.rising_deceleration = (direction * (5 >> shift)) as u16;
    swing.angle_acceleration = (direction * (24 >> (shift + u32::from(liquid)))) as u16;
}

/// Accelerate the swing, decelerate it while rising, and cap it at $480.
pub fn accelerate(player: &mut crate::session::Player) {
    let angle = player.grapple_angle;
    let swing = &mut player.swing;
    // As the checked_i32 word update: old load, then the operand, in i32.
    swing.angular_velocity = (swing.angular_velocity as i32
        + (swing.input_acceleration as i32 + swing.angle_acceleration as i32))
        as u16;
    if (swing.angular_velocity ^ angle) & 0x8000 != 0 {
        swing.angular_velocity = swing
            .angular_velocity
            .wrapping_add(swing.rising_deceleration);
    }
    let velocity = swing.angular_velocity as i16;
    if velocity.unsigned_abs() >= 0x480 {
        swing.angular_velocity = velocity.clamp(-0x480, 0x480) as u16;
    }
}

pub fn rebound_jump<H: Host + ?Sized>(game: &mut H) {
    if game.session().player.swing.kick_cooldown == 0
        || word!(game.BUTTON_CONFIG_JUMP_A) & word!(game.JOYPAD1_NEWKEYS) == 0
    {
        return;
    }
    let velocity = game.session().player.swing.angular_velocity;
    let impulse = if velocity == 0 {
        0
    } else {
        let magnitude = if game.session().player.grapple_flags != 0
            && game.session().player.grapple_flags & 1 != 0
        {
            0x180u16
        } else {
            0x300
        };
        if velocity & 0x8000 != 0 {
            magnitude.wrapping_neg()
        } else {
            magnitude
        }
    };
    game.session_mut().player.swing.extra_angular_velocity = impulse;
}

pub fn advance_swing<H: Host + ?Sized>(game: &mut H) -> u8 {
    let scale = if game.session().player.grapple_flags & 1 != 0 {
        160
    } else {
        256
    };
    let velocity = game
        .session()
        .player
        .swing
        .extra_angular_velocity
        .wrapping_add(game.session().player.swing.angular_velocity);
    let backwards = velocity & 0x8000 != 0;
    let magnitude = if backwards {
        velocity.wrapping_neg()
    } else {
        velocity
    };
    let distance = (game.multiply_with_scratch(magnitude, scale) >> 8) as u16;
    if distance == 0 {
        return 0;
    }
    let delta = if backwards {
        distance.wrapping_neg()
    } else {
        distance
    };
    let target = 2 * (game.session().player.grapple_angle.wrapping_add(delta) >> 8);
    let mut angle = 2 * game.grapple_angle() as u16;
    while angle != target {
        let next = (if backwards {
            angle.wrapping_sub(2)
        } else {
            angle.wrapping_add(2)
        }) & 0x1ff;
        if check_clearance(next, game) & 1 != 0 {
            let stopped = ((angle >> 1) << 8) | 0x80;
            game.session_mut().player.grapple_angle = stopped;
            game.session_mut().player.swing.mirrored_angle = stopped;
            if (word!(game.GRAPPLE_COLLISION_DISTANCE) == 6
                || word!(game.GRAPPLE_COLLISION_DISTANCE) == 5)
                && game.session().player.grapple_length == 8
            {
                word!(game.GRAPPLE_SPECIAL_ANGLE_FLAG |= 0x8000);
                game.session_mut().player.swing.angular_velocity = 0;
                game.session_mut().player.swing.extra_angular_velocity = 0;
            } else {
                game.session_mut().player.swing.kick_cooldown = 16;
                game.session_mut().player.swing.angular_velocity =
                    -(game.session().player.swing.angular_velocity as i16 as i32 >> 1) as u16;
                game.session_mut().player.swing.extra_angular_velocity =
                    -(game.session().player.swing.extra_angular_velocity as i16 as i32 >> 1) as u16;
            }
            return 1;
        }
        angle = next;
    }
    game.session_mut().player.grapple_angle =
        game.session().player.grapple_angle.wrapping_add(delta);
    game.session_mut().player.swing.mirrored_angle = game.session().player.grapple_angle;
    word!(game.GRAPPLE_SPECIAL_ANGLE_FLAG &= !0x8000);
    if {
        game.session_mut().player.swing.kick_cooldown =
            game.session().player.swing.kick_cooldown.wrapping_sub(1);
        game.session().player.swing.kick_cooldown
    } & 0x8000
        != 0
    {
        game.session_mut().player.swing.kick_cooldown = 0;
    }
    let impulse = game.session().player.swing.extra_angular_velocity as i16;
    let decayed = if impulse >= 0 {
        impulse.wrapping_sub(6).max(0)
    } else {
        impulse.wrapping_add(6).min(0)
    };
    game.session_mut().player.swing.extra_angular_velocity = decayed as u16;
    0
}

pub fn launch_from_swing<H: Host + ?Sized>(game: &mut H) {
    let backwards = game.session().player.swing.angular_velocity & 0x8000 != 0;
    let magnitude = if backwards {
        (-2i32 * i32::from(game.session().player.swing.angular_velocity)) as u16
    } else {
        2u16.wrapping_mul(game.session().player.swing.angular_velocity)
    };
    let sample = SINE_COSINE[usize::from(game.grapple_angle()) + 64];
    let vertical = game.multiply_with_scratch(sample.unsigned_abs(), magnitude);
    word_transfers!(game, ascending, 1;
        native(ram_words::SAMUS_Y_SUBSPEED) <- value(vertical as u16),
        native(ram_words::SAMUS_Y_SPEED) <- value((vertical >> 16) as u16));
    game.session_mut().player.contact.y_direction = if (sample < 0) == backwards { 2 } else { 1 };
    game.session_mut().player.contact.acceleration_mode = 2;
    let adjustment = 64u16.wrapping_sub((3 * (i32::from(magnitude) >> 9)) as u16);
    let sample = abs16(
        SINE_COSINE[usize::from(game.grapple_angle().wrapping_sub(adjustment as u8)) + 64] as u16,
    );
    let horizontal = game.multiply_with_scratch(sample, magnitude);
    word_transfers!(game, ascending, 1;
        native(ram_words::SAMUS_X_BASE_SUBSPEED) <- value(horizontal as u16),
        native(ram_words::SAMUS_X_BASE_SPEED) <- value((horizontal >> 16) as u16));
}

macro_rules! initialize_grapple_endpoints {
    ($game:ident, $set:ident, $pose_index:ident, $y_adjustment:ident $(,)?) => {
        let origin = grapple_origins::origin($game, OriginSet::$set, $pose_index as isize);
        word!($game.GRAPPLE_BEAM_ORIGIN_X_OFFSET = origin.x);
        word!($game.GRAPPLE_BEAM_ORIGIN_Y_OFFSET = origin.y.wrapping_sub($y_adjustment));
        $game.session_mut().player.grapple_endpoint.x = origin.x.wrapping_add($game.player_x());
        $game.session_mut().player.swing.start.x = $game.session().player.grapple_endpoint.x;
        word!(
            $game.X_POS_OF_START_OF_GRAPPLE_BEAM_PREVFRAME =
                origin.previous_x.wrapping_add($game.player_x())
        );
        $game.session_mut().player.grapple_endpoint.y =
            (origin.y as i32 + $game.player_y() as i32 - $y_adjustment as i32) as u16;
        $game.session_mut().player.swing.start.y = $game.session().player.grapple_endpoint.y;
        word!(
            $game.Y_POS_OF_START_OF_GRAPPLE_BEAM_PREVFRAME =
                (origin.previous_y as i32 + $game.player_y() as i32 - $y_adjustment as i32) as u16
        );
    };
}

fn draygon_direction<H: Host + ?Sized>(pose: u16, game: &mut H) -> u16 {
    let (toward, down, up, straight) = if pose as i32 == K_POSE_BE_FACE_L_DRAYGON_MOVE as i32 {
        (K_BUTTON_LEFT, 6, 8, 7)
    } else {
        (K_BUTTON_RIGHT, 3, 1, 2)
    };
    if word!(game.JOYPAD1_LASTKEYS) as i32 & toward as i32 == 0 {
        return straight;
    }
    if word!(game.JOYPAD1_LASTKEYS) as i32 & K_BUTTON_DOWN as i32 != 0 {
        return down;
    }
    if word!(game.JOYPAD1_LASTKEYS) as i32 & K_BUTTON_UP as i32 != 0 {
        return up;
    }
    straight
}
// Capture the native allocation before evaluating each fresh byte offset.
macro_rules! native_grapple_byte {
    ($game:ident, $address:literal, $offset:expr) => {
        $game
            .native_image()
            .read(sm_data::cartridge::lorom_offset($address) as usize + $offset)
    };
}
macro_rules! native_pose_parameter {
    ($game:ident, $offset:literal) => {
        native_grapple_byte!(
            $game,
            0x91b629,
            8 * usize::from($game.session().player.pose) + $offset
        )
    };
}
macro_rules! place_samus_relative_to_grapple_origin {
    ($game:ident, $offset_table:literal, $offset_index:ident) => {
        $game.set_player_x(($game.session().player.swing.start.x).wrapping_add(
            native_grapple_byte!($game, $offset_table, usize::from($offset_index)) as i8 as u16,
        ));
        $game.set_player_y(($game.session().player.swing.start.y).wrapping_add(
            native_grapple_byte!($game, $offset_table, usize::from($offset_index) + 1) as i8 as u16,
        ));
    };
}

macro_rules! restore_grapple_start_positions {
    ($pose_offset:ident, $game:ident, $set:ident, $y_adjustment:ident $(,)?) => {
        let v1: i32 = $pose_offset as i32 >> 1;
        let origin = grapple_origins::origin($game, OriginSet::$set, v1 as isize);
        $game.session_mut().player.swing.start.x = origin.x.wrapping_add($game.player_x());
        word!(
            $game.X_POS_OF_START_OF_GRAPPLE_BEAM_PREVFRAME =
                origin.previous_x.wrapping_add($game.player_x())
        );
        $game.session_mut().player.swing.start.y =
            (origin.y as i32 + $game.player_y() as i32 - $y_adjustment as i32) as u16;
        word!(
            $game.Y_POS_OF_START_OF_GRAPPLE_BEAM_PREVFRAME =
                (origin.previous_y as i32 + $game.player_y() as i32 - $y_adjustment as i32) as u16
        );
    };
}

pub fn position_swing<H: Host + ?Sized>(game: &mut H) {
    let v0: u16 = abs16(game.session().player.swing.angular_velocity);
    let mut v1: u16 = 0;
    let start_grapple_animation: bool = if (v0 as i32 - 64i32) & 0x8000 == 0 {
        game.session_mut().player.grapple_slow_scrolling = 1;
        true
    } else {
        game.session_mut().player.grapple_slow_scrolling = 0;
        (((game.grapple_angle() as i32) << 8) != 0x8000)
            || ({
                if (game.session().player.animation_clock.frame as i32 - 64i32) & 0x8000 != 0 {
                    game.session_mut().player.animation_clock.remaining = 8;
                    game.session_mut().player.animation_clock.frame = 64;
                }
                v1 = native_grapple_byte!(
                    game,
                    0x9bc1c2,
                    usize::from(game.grapple_mirrored_angle())
                ) as u16;
                false
            })
    };
    if start_grapple_animation {
        game.session_mut().player.animation_clock.remaining = 15;
        v1 =
            native_grapple_byte!(game, 0x9bc1c2, usize::from(game.grapple_mirrored_angle())) as u16;
        game.session_mut().player.animation_clock.frame = v1;
    }
    let v2: u16 = (2u16).wrapping_mul(v1);
    if (abs16(game.session().player.swing.extra_angular_velocity) & 0xff00) == 256 {
        let mut v3: u16 = game.session().player.animation_clock.frame;
        if (game.session().player.animation_clock.frame as i32 - 64i32) & 0x8000 == 0 {
            v3 = 16;
        }
        game.session_mut().player.animation_clock.frame = (v3).wrapping_add(32);
    }
    if game.facing() == 4 {
        place_samus_relative_to_grapple_origin!(game, 0x9bc2c2, v2);
    } else {
        place_samus_relative_to_grapple_origin!(game, 0x9bc302, v2);
    }
    word!(game.X_POS_OF_START_OF_GRAPPLE_BEAM_PREVFRAME = game.session().player.swing.start.x);
    word!(game.Y_POS_OF_START_OF_GRAPPLE_BEAM_PREVFRAME = game.session().player.swing.start.y);
    clamp_swing_position(game);
}

pub fn clamp_swing_position<H: Host + ?Sized>(game: &mut H) {
    let v0: i16 = ((game.player_x()) as i16).wrapping_sub((game.previous_player_x()) as i16);
    game.with_lease(|lease| {
        pt::clamp_previous_position(crate::session_access::SessionAccess::session_mut(lease), v0)
    });
}

pub fn position_anchored<H: Host + ?Sized>(game: &mut H) {
    let v0: i32 = game.session().player.grapple_direction as i32;
    let origin = grapple_origins::origin(game, OriginSet::Standard, v0 as isize);
    game.set_player_x((game.session().player.swing.start.x).wrapping_sub(origin.x));
    word!(
        game.X_POS_OF_START_OF_GRAPPLE_BEAM_PREVFRAME =
            origin.previous_x.wrapping_add(game.player_x())
    );
    game.set_player_y((game.session().player.swing.start.y).wrapping_sub(origin.y));
    word!(
        game.Y_POS_OF_START_OF_GRAPPLE_BEAM_PREVFRAME =
            origin.previous_y.wrapping_add(game.player_y())
    );
}

pub fn restore_start_positions<H: Host + ?Sized>(game: &mut H) {
    let r22: u16 = native_pose_parameter!(game, 4) as u16;
    let v0: u16 = (2u16).wrapping_mul(game.session().player.grapple_direction);
    if game.session().player.pose as i32 == K_POSE_49_FACE_L_MOONWALK as i32
        || game.session().player.pose as i32 == K_POSE_4A_FACE_R_MOONWALK as i32
        || game.movement_type() != 1
    {
        restore_grapple_start_positions!(v0, game, Standard, r22);
    } else {
        restore_grapple_start_positions!(v0, game, Running, r22);
    };
}

static GRAPPLE_BANNED_FOR_MOVEMENT_TYPE: [u8; 28] = [
    0, 0, 0, 1, 1, 0, 0, 1, 1, 1, 1, 0, 0, 1, 1, 1, 0, 1, 1, 1, 1, 0, 0, 1, 1, 1, 0, 1,
];
pub fn cancel_incompatible_pose<H: Host + ?Sized>(game: &mut H) {
    if GRAPPLE_BANNED_FOR_MOVEMENT_TYPE[game.movement_type() as usize] == 0 {
        if game.session().player.grapple as i32 != 0x9bc4f0 & 0xffff
            && (game.session().player.grapple as i32 + 0x3882) & 0x8000 != 0
        {
            let v0: i16 = native_pose_parameter!(game, 3) as i16;
            if (v0 as u16 & 0xf0) == 0 {
                if v0 as i32 == game.session().player.grapple_direction as i32 {
                    return;
                }
                if word!(game.GRAPPLE_AUTO_FIRE_TIMER) != 0 {
                    game.queue_sound(0, 7_u8, 6);
                    game.session_mut().player.grapple = (0x9bc51ei32) as u16;
                    return;
                }
            }
        } else {
            return;
        }
    }
    if game.session().player.grapple as i32 != 0x9bc4f0 & 0xffff {
        game.session_mut().player.grapple = (0x9bc856i32) as u16;
    }
}

pub fn start_firing<H: Host + ?Sized>(game: &mut H) {
    let (r22, v0): (u16, i32) = if game.session().player.pose as i32
        == K_POSE_F0_FACE_R_DRAYGON_MOVE as i32
        || game.session().player.pose as i32 == K_POSE_BE_FACE_L_DRAYGON_MOVE as i32
    {
        (
            6,
            draygon_direction(game.session().player.pose, game) as i32,
        )
    } else {
        let r22 = native_pose_parameter!(game, 4) as u16;
        let v0 = native_pose_parameter!(game, 3) as i32;
        if (v0 & 0xf0) != 0 {
            game.session_mut().player.grapple = (0x9bc856i32) as u16;
            return;
        }
        (r22, v0)
    };
    game.session_mut().player.grapple_direction = v0 as u16;
    let v1: i32 = v0;
    let extension = grapple_aim::extension(game, v1 as isize);
    word!(game.GRAPPLE_BEAM_EXTENSION_X_VELOCITY = extension.x_velocity);
    word!(game.GRAPPLE_BEAM_EXTENSION_Y_VELOCITY = extension.y_velocity);
    game.session_mut().player.grapple_angle = extension.angle;
    game.session_mut().player.swing.mirrored_angle = game.session().player.grapple_angle;
    word!(game.GRAPPLE_AUTO_FIRE_TIMER = 10);
    if game.session().player.pose as i32 == K_POSE_49_FACE_L_MOONWALK as i32
        || game.session().player.pose as i32 == K_POSE_4A_FACE_R_MOONWALK as i32
        || game.movement_type() != 1
    {
        initialize_grapple_endpoints!(game, Standard, v1, r22);
    } else {
        initialize_grapple_endpoints!(game, Running, v1, r22);
    }
    word!(game.GRAPPLE_BEAM_END_X_SUBPOS = 0);
    word!(game.GRAPPLE_BEAM_END_Y_SUBPOS = 0);
    word!(game.GRAPPLE_BEAM_END_X_SUBOFFSET = 0);
    word!(game.GRAPPLE_BEAM_END_X_OFFSET = 0);
    word!(game.GRAPPLE_BEAM_END_Y_SUBOFFSET = 0);
    word!(game.GRAPPLE_BEAM_END_Y_OFFSET = 0);
    game.session_mut().player.grapple_flags = 0;
    game.session_mut().player.swing.length_delta = 12;
    game.session_mut().player.grapple_length = 0;
    game.session_mut().player.swing.angular_velocity = 0;
    game.session_mut().player.swing.angle_acceleration = 0;
    game.session_mut().player.swing.input_acceleration = 0;
    game.session_mut().player.swing.rising_deceleration = 0;
    game.session_mut().player.swing.extra_angular_velocity = 0;
    game.session_mut().player.swing.kick_cooldown = 0;
    word!(game.OFFSCREEN_BEAM_HDMA_INDEX = 0);
    word!(game.GRAPPLE_BEAM_UNK_D20 = 0);
    game.session_mut().player.swing.start_marks[0] = 2;
    game.session_mut().player.swing.start_marks[1] = 0;
    game.session_mut().player.swing.start_animation_timer = 5;
    game.session_mut().player.swing.start_animation_tiles =
        grapple_aim::point_animation(game).first;
    game.session_mut().player.swing.probe.x = 0;
    game.session_mut().player.swing.failed_movement_counter = 0;
    word!(game.GRAPPLE_SPECIAL_ANGLE_FLAG = 0);
    game.session_mut().player.grapple_slow_scrolling = 0;
    start_rope_animation(game);
    game.session_mut().player.transitions.draw_handler = DrawMode::Grapple;
    game.session_mut().player.transitions.grapple_walljump_timer = 0;
    game.with_lease(|lease| crate::samus_palette::write_beam_palette(lease, 2));
    word!(game.PALETTE_BUFFER[223] = 32657);
    game.session_mut().player.grapple = (0x9bc703i32) as u16;
    game.queue_sound(0, 5_u8, 1);
    game.session_mut().projectiles.controls.charge = 1;
    word!(game.PLAY_RESUME_CHARGING_BEAM_SFX = 0);
    if game.session().player.movement_handler == GRAPPLE_LAUNCH_MOVEMENT {
        game.session_mut().player.movement_handler = MovementMode::Normal;
    }
}

pub fn cancel<H: Host + ?Sized>(game: &mut H) {
    game.queue_sound(0, 7_u8, 15);
    if game.movement_type() == K_MOVEMENT_TYPE_16_GRAPPLING as u8 {
        game.with_lease(pt::request_fallback_pose);
    } else {
        game.grapple_pose_command(0x1c);
    }
    finish_release(game);
}

pub fn release_stationary<H: Host + ?Sized>(game: &mut H) {
    game.queue_sound(0, 7_u8, 15);
    let pose = if game.session().player.pose == K_POSE_B2_FACE_R_GRAPPLE_AIR as u16 {
        K_POSE_01_FACE_R_NORMAL as u16
    } else if game.session().player.pose == K_POSE_B3_FACE_L_GRAPPLE_AIR as u16 {
        K_POSE_02_FACE_L_NORMAL as u16
    } else {
        let crouching = (i32::from(game.session().player.contact.radius) - 17) & 0x8000 != 0;
        if native_pose_parameter!(game, 3) & 0xf0 != 0 {
            match (crouching, game.facing() == 4) {
                (true, true) => K_POSE_28_FACE_L_CROUCH as u16,
                (true, false) => K_POSE_27_FACE_R_CROUCH as u16,
                (false, true) => K_POSE_02_FACE_L_NORMAL as u16,
                (false, false) => K_POSE_01_FACE_R_NORMAL as u16,
            }
        } else {
            let table = if crouching { 0x9bc9c4 } else { 0x9bc9ba };
            // Capture the destination image before the fresh native pose-byte read.
            u16::from(game.native_image().read(
                sm_data::cartridge::lorom_offset(table) as usize
                    + usize::from(native_pose_parameter!(game, 3)),
            ))
        }
    };
    game.session_mut().player.transitions.transitional = PoseRequestWord::from_raw(pose);
    game.session_mut().player.transitions.hurt = HurtAction::from_raw(0);
    word!(game.INPUT_TO_POSE_CALC = 1);
    word! { game, 0;
        SAMUS_X_BASE_SPEED, SAMUS_X_BASE_SUBSPEED, SAMUS_Y_SUBSPEED, SAMUS_Y_SPEED,
    }
    finish_release(game);
}

pub fn release_walljump<H: Host + ?Sized>(game: &mut H) {
    game.queue_sound(0, 7_u8, 15);
    if game.facing() == 8 {
        game.session_mut().player.transitions.transitional =
            PoseRequestWord::from_raw(K_POSE_84_FACE_L_WALLJUMP as u16);
    } else {
        game.session_mut().player.transitions.transitional =
            PoseRequestWord::from_raw(K_POSE_83_FACE_R_WALLJUMP as u16);
    }
    game.session_mut().player.transitions.hurt = HurtAction::from_raw(6);
    game.session_mut().player.contact.acceleration_mode = 0;
    word!(game.SAMUS_COLLIDES_WITH_SOLID_ENEMY = 0);
    word!(game.SAMUS_IS_FALLING_FLAG = 0);
    word!(game.UNUSED_WORD_7E0B1A = 0);
    word!(game.UNUSED_WORD_7E0B2A = 0);
    word!(game.SAMUS_Y_SUBSPEED = 0);
    word!(game.SAMUS_Y_SPEED = 0);
    game.session_mut().player.contact.y_direction = 0;
    word!(game.UNUSED_WORD_7E0B38 = 0);
    word!(game.SAMUS_X_BASE_SPEED = 0);
    word!(game.SAMUS_X_BASE_SUBSPEED = 0);
    finish_release(game);
}

pub fn release_swing<H: Host + ?Sized>(game: &mut H) {
    game.queue_sound(0, 7_u8, 15);
    if game.session().player.swing.angular_velocity & 0x8000 == 0 {
        game.session_mut().player.transitions.transitional =
            PoseRequestWord::from_raw(K_POSE_52_FACE_L_JUMP_NO_AIM_MOVE_F as u16);
    } else {
        game.session_mut().player.transitions.transitional =
            PoseRequestWord::from_raw(K_POSE_51_FACE_R_JUMP_NO_AIM_MOVE_F as u16);
    }
    game.session_mut().player.transitions.hurt = HurtAction::from_raw(7);
    finish_release(game);
}

fn finish_release<H: Host + ?Sized>(game: &mut H) {
    word!(game.OFFSCREEN_BEAM_HDMA_INDEX = 0);
    word!(game.GRAPPLE_BEAM_UNK_D20 = 0);
    game.session_mut().player.grapple_direction = 0;
    word!(game.GRAPPLE_SPECIAL_ANGLE_FLAG = 0);
    game.session_mut().player.transitions.grapple_walljump_timer = 0;
    game.session_mut().player.grapple_slow_scrolling = 0;
    word!(game.GRAPPLE_AUTO_FIRE_TIMER = 0);
    game.session_mut().player.grapple_flags = 0;
    game.session_mut().projectiles.controls.charge = 0;
    game.session_mut().projectiles.controls.flare_frames[0] = 0;
    game.session_mut().projectiles.controls.flare_frames[1] = 0;
    game.session_mut().projectiles.controls.flare_frames[2] = 0;
    game.session_mut().projectiles.controls.flare_timers[0] = 0;
    game.session_mut().projectiles.controls.flare_timers[1] = 0;
    game.session_mut().projectiles.controls.flare_timers[2] = 0;
    let beams = game.session().inventory.beams.active & 0xfff;
    game.with_lease(|lease| crate::samus_palette::write_beam_palette(lease, beams));
    game.session_mut().player.grapple = (0x9bc4f0i32) as u16;
    game.session_mut().player.transitions.draw_handler = DrawMode::Normal;
    game.settle_grapple();
    if word!(game.SAMUS_AUTO_CANCEL_HUD_ITEM_INDEX) != 0 {
        word! { game, 0; HUD_ITEM_INDEX, SAMUS_AUTO_CANCEL_HUD_ITEM_INDEX }
    }
}

pub fn react_beam<H: Host + ?Sized>(game: &mut H) -> u8 {
    game.with_lease(crate::player_collision::grapple::react_beam)
}
pub fn extend_beam<H: Host + ?Sized>(game: &mut H) -> u8 {
    game.with_lease(crate::player_collision::grapple::extend_beam)
}
pub fn start_rope_animation<H: Host + ?Sized>(game: &mut H) {
    game.with_lease(crate::grapple_rope::start);
}

fn connected_to_block<H: Host + ?Sized>(game: &mut H) -> bool {
    word! { game, 0;
        GRAPPLE_BEAM_EXTENSION_X_VELOCITY, GRAPPLE_BEAM_EXTENSION_Y_VELOCITY,
    }
    let connected = react_beam(game) & 1 != 0;
    if connected {
        game.session_mut().player.grapple_interaction_flags = 1;
    }
    connected
}

/// The grapple point's tile upload runs on the compat game: it queues native
/// VRAM transfers, so it needs only RAM words, the menu runtime and records.
pub fn update_tiles<
    H: crate::ram::CompatibilityRam
        + crate::menus::host::MenuRuntime
        + crate::player_position::PlayerFacts
        + crate::records::Records
        + ?Sized,
>(
    game: &mut H,
) {
    if (word!(game.GRAPPLE_POINT_ANIM_TIMER -= 1, reread) & 0x8000) != 0 {
        word!(game.GRAPPLE_POINT_ANIM_TIMER = 5);
        word!(game.GRAPPLE_POINT_ANIM_PTR += 512);
        if ((word!(game.GRAPPLE_POINT_ANIM_PTR)) as i16)
            .wrapping_sub(grapple_aim::point_animation(game).end as i16)
            >= 0
        {
            word!(game.GRAPPLE_POINT_ANIM_PTR = grapple_aim::point_animation(game).first);
        }
    }
    let cursor = game.queue_native_transfer(
        word!(game.VRAM_WRITE_QUEUE_TAIL),
        32,
        &|game: &H| word!(game.GRAPPLE_POINT_ANIM_PTR),
        0x9a,
        &|_: &H| 0x6200,
    );
    let index = (game.grapple_angle() >> 1 & 0xfe) >> 1;
    game.queue_native_transfer(
        cursor,
        128,
        &|game: &H| crate::gameplay_tables::grapple_point_tile(game, isize::from(index)),
        0x9a,
        &|_: &H| 0x6210,
    );
    if (word!(game.FLARE_COUNTER) as i32 - 120i32) & 0x8000 != 0 {
        word!(game.FLARE_COUNTER += 1);
    }
}

pub fn update<H: Host + ?Sized>(game: &mut H) {
    let r18: u16;
    if word!(game.GRAPPLE_AUTO_FIRE_TIMER) != 0 {
        word!(game.GRAPPLE_AUTO_FIRE_TIMER -= 1);
    }
    game.session_mut().player.grapple_interaction_flags =
        game.session().player.grapple_interaction_flags & (!(1));
    cancel_incompatible_pose(game);
    dispatch(
        ((game.session().player.grapple) as u32) | (0x9b0000u32),
        game,
    );
    if game.session().player.grapple as i32 != 0x9bc4f0 & 0xffff
        && (game.session().player.grapple as i32 - (0x9bc856 & 0xffff)) & 0x8000 != 0
        && (word!(game.SAMUS_SUIT_PALETTE_INDEX) & 4) == 0
        && word!(game.FX_TYPE) != 0
        && {
            r18 = game.player_bottom();
            (word!(game.FX_Y_POS) & 0x8000) == 0
        }
        && (word!(game.FX_Y_POS) as i32 - r18 as i32) & 0x8000 != 0
    {
        game.session_mut().player.grapple_flags = game.session().player.grapple_flags | (1u16);
    } else {
        game.session_mut().player.grapple_flags = game.session().player.grapple_flags & (!(1));
    };
}

fn shoot_held<H: Host + ?Sized>(game: &H) -> bool {
    word!(game.BUTTON_CONFIG_SHOOT_X) & word!(game.JOYPAD1_LASTKEYS) != 0
}

fn dispatch<H: Host + ?Sized>(ea: u32, game: &mut H) {
    match ea {
        0x9bc4f0 => {
            if (word!(game.BUTTON_CONFIG_SHOOT_X) & word!(game.JOYPAD1_NEWKEYS)) != 0
                || (word!(game.BUTTON_CONFIG_SHOOT_X) & word!(game.JOYPAD1_NEWINPUT_SAMUSFILTER))
                    != 0
            {
                start_firing(game);
            } else if game.session().projectiles.controls.charge != 0 {
                game.session_mut().projectiles.controls.charge = 0;
                game.session_mut().projectiles.controls.flare_frames[0] = 0;
                game.session_mut().projectiles.controls.flare_frames[1] = 0;
                game.session_mut().projectiles.controls.flare_frames[2] = 0;
                game.session_mut().projectiles.controls.flare_timers[0] = 0;
                game.session_mut().projectiles.controls.flare_timers[1] = 0;
                game.session_mut().projectiles.controls.flare_timers[2] = 0;
                game.with_lease(|lease| {
                    crate::samus_palette::load_suit(
                        lease,
                        crate::visual_state::PaletteBuffer::Current,
                    )
                });
            }
        }
        0x9bc703 => {
            let shooting = shoot_held(game);
            if shooting {
                game.session_mut().player.grapple_length = game
                    .session()
                    .player
                    .grapple_length
                    .wrapping_add(game.session().player.swing.length_delta);
            }
            if !shooting || (game.session().player.grapple_length as i32 - 128) & 0x8000 == 0 {
                game.session_mut().player.grapple = 0xc856;
                return;
            }
            let contact = game.with_lease(actors::contacts::grapple_contact);
            let outcome = match contact {
                Some(contact) => {
                    game.with_lease(|lease| actors::combat::resolve_grapple(lease, contact))
                }
                None => GrappleOutcome::Miss,
            };
            let attached = match outcome {
                GrappleOutcome::Attach => true,
                GrappleOutcome::Cancel => false,
                GrappleOutcome::Miss => {
                    let v2: u8 = extend_beam(game);
                    if (v2 & 1) == 0 {
                        return;
                    }
                    v2 & 0x40 != 0
                }
            };
            if !attached {
                game.session_mut().player.grapple = 0xc856;
                return;
            }
            game.queue_sound(0, 6_u8, 6);
            connect_beam(game);
            game.session_mut().player.swing.length_delta = -8i32 as u16;
            game.session_mut().player.grapple_interaction_flags =
                game.session().player.grapple_interaction_flags | (1u16);
        }
        0x9bc77e | 0x9bc814 => {
            let attached = shoot_held(game)
                && ((ea == 0x9bc77e
                    && game
                        .with_lease(actors::contacts::grapple_contact)
                        .is_some_and(|contact| contact.has_reaction()))
                    || connected_to_block(game));
            if !attached {
                if ea == 0x9bc814 {
                    game.session_mut().player.transitions.grapple_walljump_timer = 30;
                    game.session_mut().player.grapple = 0xc832;
                } else {
                    game.session_mut().player.grapple = 0xc856;
                }
            }
        }
        0x9bc79d => {
            if shoot_held(game) {
                apply_input(game);
                if game.session().player.swing.length_delta != 0 {
                    change_length(game);
                }
                apply_gravity(&mut game.session_mut().player);
                accelerate(&mut game.session_mut().player);
                rebound_jump(game);
                advance_swing(game);
                if (word!(game.GRAPPLE_SPECIAL_ANGLE_FLAG) & 0x8000) != 0
                    && apply_special_angle(game) & 1 != 0
                {
                    return;
                }
                let keep_swinging: bool = if game
                    .with_lease(actors::contacts::grapple_contact)
                    .is_some_and(|contact| contact.has_reaction())
                {
                    game.session_mut().player.grapple_flags =
                        game.session().player.grapple_flags | (0x8000u16);
                    false
                } else {
                    !connected_to_block(game)
                };
                if !keep_swinging {
                    place_rope_origin(game);
                    position_swing(game);
                    return;
                }
            }
            if game.session().player.swing.angular_velocity != 0
                || game.session().player.grapple_angle != 0x8000
            {
                launch_from_swing(game);
                game.session_mut().player.grapple = (0x9bcb8bi32) as u16;
                game.session_mut().player.movement_handler = GRAPPLE_LAUNCH_MOVEMENT;
            } else {
                game.session_mut().player.grapple = (0x9bc8c5i32) as u16;
            };
        }
        0x9bc832 => {
            if ({
                game.session_mut().player.transitions.grapple_walljump_timer = game
                    .session()
                    .player
                    .transitions
                    .grapple_walljump_timer
                    .wrapping_sub(1);
                game.session().player.transitions.grapple_walljump_timer
            } & 0x8000)
                == 0
            {
                let request = player_collision::JumpRequest {
                    kind: player_collision::JumpKind::Grapple,
                    amount: (16i16 as i32) << 16,
                };
                if game.with_lease(|lease| player_collision::jump_admission(lease, request)) {
                    game.session_mut().player.grapple = (0x9bc9cei32) as u16;
                }
            } else {
                game.session_mut().player.grapple = (0x9bc8c5i32) as u16;
            };
        }
        0x9bbd95 => position_swing(game),
        0x9bbe98 => clamp_swing_position(game),
        0x9bbeeb => position_anchored(game),
        0x9bbf1b => restore_start_positions(game),
        0x9bc51e => start_firing(game),
        0x9bc856 => cancel(game),
        0x9bc8c5 => release_stationary(game),
        0x9bc9ce => release_walljump(game),
        0x9bcb8b => release_swing(game),
        0x9bbb64 => apply_input(game),
        0x9bbc1f => apply_gravity(&mut game.session_mut().player),
        0x9bbcff => accelerate(&mut game.session_mut().player),
        0x9bbd44 => rebound_jump(game),
        _ => {
            game.grapple_unreachable();
        }
    }
}
