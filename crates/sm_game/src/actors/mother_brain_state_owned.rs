//! Mother Brain body, brain and tube fields on their existing retained owners.
//! Exceptional full/odd selectors query only the requested bytes.
#![forbid(unsafe_code)]
use crate::player_position::PlayerPosition;
use crate::session_access::SessionAccess;
pub const BODY: i32 = 0;
pub const BRAIN: i32 = 0x40;
pub trait State: SessionAccess + PlayerPosition {
    fn exceptional_byte(&self, at: usize) -> u8;
    fn set_exceptional_byte(&mut self, at: usize, value: u8);
}
#[derive(Clone, Copy)]
pub(crate) enum Field {
    BaseXPos = 1,
    BaseYPos = 3,
    BaseYSubpos = 4,
    BaseXWidth = 5,
    BaseYHeight = 6,
    BaseProperties = 7,
    MotherBrainBodyForm = 31,
    MotherBrainBodyPose = 33,
    MotherBrainBodyAttacksDisabled = 37,
    MotherBrainBodyNeckBaseX = 41,
    MotherBrainBodyNeckBaseY = 42,
    MotherBrainBodyBrainInstructionTimer = 63,
    MotherBrainBodyBrainInstructions = 64,
    MotherBrainBrainShakeTimer = 103,
    MotherBrainBrainNeckFunction = 127,
    MotherBrainBrainNeckTimer = 128,
    MotherBrainBrainLowerAngle = 135,
    MotherBrainBrainUpperAngle = 136,
    MotherBrainBrainSegment1X = 137,
    MotherBrainBrainSegment1Y = 138,
    MotherBrainBrainSegment1Length = 139,
    MotherBrainBrainSegment2X = 140,
    MotherBrainBrainSegment2Y = 141,
    MotherBrainBrainSegment2Length = 142,
    MotherBrainBrainSegment3X = 143,
    MotherBrainBrainSegment3Y = 144,
    MotherBrainBrainSegment3Length = 145,
    MotherBrainBrainSegment4X = 146,
    MotherBrainBrainSegment4Y = 147,
    MotherBrainBrainSegment4Length = 148,
    MotherBrainBrainSegment5X = 149,
    MotherBrainBrainSegment5Y = 150,
    MotherBrainBrainSegment5Length = 151,
    MotherBrainBrainNeckMoving = 152,
    MotherBrainBrainLowerMotion = 153,
    MotherBrainBrainUpperMotion = 154,
    MotherBrainBrainNeckSpeed = 155,
    TubeFunction = 156,
    TubeBurstY = 157,
    TubeFallSpeed = 158,
    TubeSmokeTimer = 159,
    TubeSmokeIndex = 160,
    TubeParameter1 = 162,
    TubeFallDelay = 163,
}
impl Field {
    fn offset(self) -> isize {
        match self {
            Self::BaseXPos => 2,
            Self::BaseYPos => 6,
            Self::BaseYSubpos => 8,
            Self::BaseXWidth => 10,
            Self::BaseYHeight => 12,
            Self::BaseProperties => 14,
            Self::MotherBrainBodyForm => 26760,
            Self::MotherBrainBodyPose => 26764,
            Self::MotherBrainBodyAttacksDisabled => 26772,
            Self::MotherBrainBodyNeckBaseX => 26780,
            Self::MotherBrainBodyNeckBaseY => 26782,
            Self::MotherBrainBodyBrainInstructionTimer => 28808,
            Self::MotherBrainBodyBrainInstructions => 28810,
            Self::MotherBrainBrainShakeTimer => 26760,
            Self::MotherBrainBrainNeckFunction => 26808,
            Self::MotherBrainBrainNeckTimer => 26810,
            Self::MotherBrainBrainLowerAngle => 28808,
            Self::MotherBrainBrainUpperAngle => 28810,
            Self::MotherBrainBrainSegment1X => 28812,
            Self::MotherBrainBrainSegment1Y => 28814,
            Self::MotherBrainBrainSegment1Length => 28816,
            Self::MotherBrainBrainSegment2X => 28818,
            Self::MotherBrainBrainSegment2Y => 28820,
            Self::MotherBrainBrainSegment2Length => 28822,
            Self::MotherBrainBrainSegment3X => 28824,
            Self::MotherBrainBrainSegment3Y => 28826,
            Self::MotherBrainBrainSegment3Length => 28828,
            Self::MotherBrainBrainSegment4X => 28830,
            Self::MotherBrainBrainSegment4Y => 28832,
            Self::MotherBrainBrainSegment4Length => 28834,
            Self::MotherBrainBrainSegment5X => 28836,
            Self::MotherBrainBrainSegment5Y => 28838,
            Self::MotherBrainBrainSegment5Length => 28840,
            Self::MotherBrainBrainNeckMoving => 28842,
            Self::MotherBrainBrainLowerMotion => 28844,
            Self::MotherBrainBrainUpperMotion => 28846,
            Self::MotherBrainBrainNeckSpeed => 28848,
            Self::TubeFunction => 48,
            Self::TubeBurstY => 50,
            Self::TubeFallSpeed => 52,
            Self::TubeSmokeTimer => 54,
            Self::TubeSmokeIndex => 56,
            Self::TubeParameter1 => 60,
            Self::TubeFallDelay => 62,
        }
    }
    fn actor_value(self, a: &crate::actors::state::Actor) -> Option<u16> {
        match self {
            Self::BaseXPos => Some(a.x.whole),
            Self::BaseYPos => Some(a.y.whole),
            Self::BaseYSubpos => Some(a.y.fraction),
            Self::BaseXWidth => Some(a.radius.x),
            Self::BaseYHeight => Some(a.radius.y),
            Self::BaseProperties => Some(a.properties),
            Self::TubeFunction => Some(a.variables[0]),
            Self::TubeBurstY => Some(a.variables[1]),
            Self::TubeFallSpeed => Some(a.variables[2]),
            Self::TubeSmokeTimer => Some(a.variables[3]),
            Self::TubeSmokeIndex => Some(a.variables[4]),
            Self::TubeParameter1 => Some(a.parameters[0]),
            Self::TubeFallDelay => Some(a.parameters[1]),
            _ => None,
        }
    }
    fn set_actor(self, a: &mut crate::actors::state::Actor, value: u16) {
        match self {
            Self::BaseXPos => a.x.whole = value,
            Self::BaseYPos => a.y.whole = value,
            Self::BaseYSubpos => a.y.fraction = value,
            Self::BaseXWidth => a.radius.x = value,
            Self::BaseYHeight => a.radius.y = value,
            Self::BaseProperties => a.properties = value,
            Self::TubeFunction => a.variables[0] = value,
            Self::TubeBurstY => a.variables[1] = value,
            Self::TubeFallSpeed => a.variables[2] = value,
            Self::TubeSmokeTimer => a.variables[3] = value,
            Self::TubeSmokeIndex => a.variables[4] = value,
            Self::TubeParameter1 => a.parameters[0] = value,
            Self::TubeFallDelay => a.parameters[1] = value,
            _ => unreachable!(),
        }
    }
}

fn address(slot: i32, field: Field) -> usize {
    usize::try_from(isize::try_from(slot).expect("actor field offset") + 0xf78 + field.offset())
        .expect("Mother Brain field outside allocation")
}
fn actor_index(slot: i32, field: Field) -> Option<usize> {
    let slot = usize::try_from(slot).ok()?;
    (field.offset() < 64 && slot < 2048 && slot.is_multiple_of(64)).then_some(slot / 64)
}
fn extended<S: State + ?Sized>(s: &S, at: usize) -> Option<u16> {
    let s = s.session();
    match at {
        0x801e => Some(s.actors.extended.botwoon_head[0].pending_hit),
        0x8020 => Some(s.actors.extended.botwoon_head[0].active),
        0x803c => Some(s.actors.extended.botwoon_head[0].tail_ready),
        0x8818 => Some(s.actors.extended.botwoon_path[0].previous_health),
        0x883e => Some(s.actors.extended.botwoon_path[0].body_landed),
        0x7800..0x9000 if at.is_multiple_of(2) => {
            let rel = at - 0x7800;
            Some(*s.actors.extended.word(rel / 0x800, (rel % 0x800) / 2))
        }
        _ => None,
    }
}
fn store_extended<S: State + ?Sized>(s: &mut S, at: usize, value: u16) -> bool {
    let s = s.session_mut();
    match at {
        0x801e => s.actors.extended.botwoon_head[0].pending_hit = value,
        0x8020 => s.actors.extended.botwoon_head[0].active = value,
        0x803c => s.actors.extended.botwoon_head[0].tail_ready = value,
        0x8818 => s.actors.extended.botwoon_path[0].previous_health = value,
        0x883e => s.actors.extended.botwoon_path[0].body_landed = value,
        0x7800..0x9000 if at.is_multiple_of(2) => {
            let rel = at - 0x7800;
            *s.actors.extended.word_mut(rel / 0x800, (rel % 0x800) / 2) = value;
        }
        _ => return false,
    }
    true
}
pub(crate) fn part_get<S: State + ?Sized>(s: &S, slot: i32, field: Field) -> u16 {
    if let Some(index) = actor_index(slot, field) {
        return field
            .actor_value(&s.session().actors.slots[index])
            .expect("actor field");
    }
    let at = address(slot, field);
    if let Some(value) = extended(s, at) {
        return value;
    }
    u16::from_le_bytes([s.exceptional_byte(at), s.exceptional_byte(at + 1)])
}
pub(crate) fn part_set<S: State + ?Sized>(s: &mut S, slot: i32, field: Field, value: u16) {
    if let Some(index) = actor_index(slot, field) {
        field.set_actor(&mut s.session_mut().actors.slots[index], value);
        return;
    }
    let at = address(slot, field);
    if store_extended(s, at, value) {
        return;
    }
    let [low, high] = value.to_le_bytes();
    s.set_exceptional_byte(at, low);
    s.set_exceptional_byte(at + 1, high);
}
pub(crate) fn part_byte<S: State + ?Sized>(s: &S, slot: i32, field: Field, lane: usize) -> u8 {
    if lane < 2 {
        if let Some(index) = actor_index(slot, field) {
            return field
                .actor_value(&s.session().actors.slots[index])
                .expect("actor field")
                .to_le_bytes()[lane];
        }
        if let Some(value) = extended(s, address(slot, field)) {
            return value.to_le_bytes()[lane];
        }
    }
    s.exceptional_byte(address(slot, field) + lane)
}
pub(crate) fn set_part_byte<S: State + ?Sized>(
    s: &mut S,
    slot: i32,
    field: Field,
    lane: usize,
    value: u8,
) {
    if lane < 2 {
        if let Some(index) = actor_index(slot, field) {
            let a = &mut s.session_mut().actors.slots[index];
            let mut bytes = field.actor_value(a).expect("actor field").to_le_bytes();
            bytes[lane] = value;
            field.set_actor(a, u16::from_le_bytes(bytes));
            return;
        }
        let at = address(slot, field);
        if let Some(old) = extended(s, at) {
            let mut bytes = old.to_le_bytes();
            bytes[lane] = value;
            store_extended(s, at, u16::from_le_bytes(bytes));
            return;
        }
    }
    s.set_exceptional_byte(address(slot, field) + lane, value);
}
macro_rules! part {
    (byte $g:ident, $layout:ident[$slot:expr] $(.$field:ident)+ [$lane:expr] = $value:expr) => {{
        let value: u8 = $value;
        $crate::actors::mother_brain_state_owned::set_part_byte($g, $slot, part!(@field $layout, $($field).+), $lane as usize, value)
    }};
    (byte $g:ident, $layout:ident[$slot:expr] $(.$field:ident)+ [$lane:expr]) => {
        $crate::actors::mother_brain_state_owned::part_byte(&*$g, $slot, part!(@field $layout, $($field).+), $lane as usize)
    };
    ($g:ident, $layout:ident[$slot:expr] $(.$prefix:ident)* ($hi:ident, $lo:ident) += $value:expr) => {{
        let value: u32 = $value;
        let low = u32::from(part!($g, $layout[$slot] $(.$prefix)* .$lo));
        let high = u32::from(part!($g, $layout[$slot] $(.$prefix)* .$hi));
        let sum = (low | (high << 16)).wrapping_add(value);
        part!($g, $layout[$slot] $(.$prefix)* .$lo = sum as u16);
        part!($g, $layout[$slot] $(.$prefix)* .$hi = (sum >> 16) as u16);
    }};
    ($g:ident, $layout:ident[$slot:expr] $(.$prefix:ident)*; $($field:ident = $value:expr),+ $(,)?) => {
        $crate::actors::mother_brain_state_owned::part!(@set_fields $g, $layout[$slot] [$(.$prefix)*]; $($field = $value),+);
    };
    (@set_fields $g:ident, $layout:ident[$slot:expr] [$($prefix:tt)*]; $field:ident = $value:expr, $($rest:tt)+) => {
        $crate::actors::mother_brain_state_owned::part!($g, $layout[$slot] $($prefix)* .$field = $value);
        $crate::actors::mother_brain_state_owned::part!(@set_fields $g, $layout[$slot] [$($prefix)*]; $($rest)+);
    };
    (@set_fields $g:ident, $layout:ident[$slot:expr] [$($prefix:tt)*]; $field:ident = $value:expr) => {
        $crate::actors::mother_brain_state_owned::part!($g, $layout[$slot] $($prefix)* .$field = $value);
    };
    ($g:ident, $layout:ident[$slot:expr] $(.$field:ident)+, |$old:ident| $value:expr) => {{
        let slot: i32 = $slot;
        let field = part!(@field $layout, $($field).+);
        let $old = $crate::actors::mother_brain_state_owned::part_get(&*$g, slot, field);
        let value: u16 = $value;
        $crate::actors::mother_brain_state_owned::part_set($g, slot, field, value)
    }};
    ($g:ident, $layout:ident[$slot:expr] $(.$field:ident)+ = $value:expr) => {{
        let value: u16 = $value;
        $crate::actors::mother_brain_state_owned::part_set($g, $slot, part!(@field $layout, $($field).+), value)
    }};
    ($g:ident, $layout:ident[$slot:expr] $(.$field:ident)+) => {
        $crate::actors::mother_brain_state_owned::part_get(&*$g, $slot, part!(@field $layout, $($field).+))
    };
    (@field $($tokens:tt)*) => { $crate::actors::mother_brain_state_owned::part_field!($($tokens)*) };
}
macro_rules! part_field {
    ($layout:ident, base.x_pos) => {
        $crate::actors::mother_brain_state_owned::Field::BaseXPos
    };
    ($layout:ident, base.y_pos) => {
        $crate::actors::mother_brain_state_owned::Field::BaseYPos
    };
    ($layout:ident, base.y_subpos) => {
        $crate::actors::mother_brain_state_owned::Field::BaseYSubpos
    };
    ($layout:ident, base.x_width) => {
        $crate::actors::mother_brain_state_owned::Field::BaseXWidth
    };
    ($layout:ident, base.y_height) => {
        $crate::actors::mother_brain_state_owned::Field::BaseYHeight
    };
    ($layout:ident, base.properties) => {
        $crate::actors::mother_brain_state_owned::Field::BaseProperties
    };
    (MotherBrainBody, form) => {
        $crate::actors::mother_brain_state_owned::Field::MotherBrainBodyForm
    };
    (Body, form) => {
        $crate::actors::mother_brain_state_owned::Field::MotherBrainBodyForm
    };
    (MotherBrainBody, pose) => {
        $crate::actors::mother_brain_state_owned::Field::MotherBrainBodyPose
    };
    (Body, pose) => {
        $crate::actors::mother_brain_state_owned::Field::MotherBrainBodyPose
    };
    (MotherBrainBody, attacks_disabled) => {
        $crate::actors::mother_brain_state_owned::Field::MotherBrainBodyAttacksDisabled
    };
    (Body, attacks_disabled) => {
        $crate::actors::mother_brain_state_owned::Field::MotherBrainBodyAttacksDisabled
    };
    (MotherBrainBody, neck_base_x) => {
        $crate::actors::mother_brain_state_owned::Field::MotherBrainBodyNeckBaseX
    };
    (Body, neck_base_x) => {
        $crate::actors::mother_brain_state_owned::Field::MotherBrainBodyNeckBaseX
    };
    (MotherBrainBody, neck_base_y) => {
        $crate::actors::mother_brain_state_owned::Field::MotherBrainBodyNeckBaseY
    };
    (Body, neck_base_y) => {
        $crate::actors::mother_brain_state_owned::Field::MotherBrainBodyNeckBaseY
    };
    (MotherBrainBody, brain_instruction_timer) => {
        $crate::actors::mother_brain_state_owned::Field::MotherBrainBodyBrainInstructionTimer
    };
    (Body, brain_instruction_timer) => {
        $crate::actors::mother_brain_state_owned::Field::MotherBrainBodyBrainInstructionTimer
    };
    (MotherBrainBody, brain_instructions) => {
        $crate::actors::mother_brain_state_owned::Field::MotherBrainBodyBrainInstructions
    };
    (Body, brain_instructions) => {
        $crate::actors::mother_brain_state_owned::Field::MotherBrainBodyBrainInstructions
    };
    (MotherBrainBrain, shake_timer) => {
        $crate::actors::mother_brain_state_owned::Field::MotherBrainBrainShakeTimer
    };
    (Brain, shake_timer) => {
        $crate::actors::mother_brain_state_owned::Field::MotherBrainBrainShakeTimer
    };
    (MotherBrainBrain, neck_function) => {
        $crate::actors::mother_brain_state_owned::Field::MotherBrainBrainNeckFunction
    };
    (Brain, neck_function) => {
        $crate::actors::mother_brain_state_owned::Field::MotherBrainBrainNeckFunction
    };
    (MotherBrainBrain, neck_timer) => {
        $crate::actors::mother_brain_state_owned::Field::MotherBrainBrainNeckTimer
    };
    (Brain, neck_timer) => {
        $crate::actors::mother_brain_state_owned::Field::MotherBrainBrainNeckTimer
    };
    (MotherBrainBrain, lower_angle) => {
        $crate::actors::mother_brain_state_owned::Field::MotherBrainBrainLowerAngle
    };
    (Brain, lower_angle) => {
        $crate::actors::mother_brain_state_owned::Field::MotherBrainBrainLowerAngle
    };
    (MotherBrainBrain, upper_angle) => {
        $crate::actors::mother_brain_state_owned::Field::MotherBrainBrainUpperAngle
    };
    (Brain, upper_angle) => {
        $crate::actors::mother_brain_state_owned::Field::MotherBrainBrainUpperAngle
    };
    (MotherBrainBrain, segment_1_x) => {
        $crate::actors::mother_brain_state_owned::Field::MotherBrainBrainSegment1X
    };
    (Brain, segment_1_x) => {
        $crate::actors::mother_brain_state_owned::Field::MotherBrainBrainSegment1X
    };
    (MotherBrainBrain, segment_1_y) => {
        $crate::actors::mother_brain_state_owned::Field::MotherBrainBrainSegment1Y
    };
    (Brain, segment_1_y) => {
        $crate::actors::mother_brain_state_owned::Field::MotherBrainBrainSegment1Y
    };
    (MotherBrainBrain, segment_1_length) => {
        $crate::actors::mother_brain_state_owned::Field::MotherBrainBrainSegment1Length
    };
    (Brain, segment_1_length) => {
        $crate::actors::mother_brain_state_owned::Field::MotherBrainBrainSegment1Length
    };
    (MotherBrainBrain, segment_2_x) => {
        $crate::actors::mother_brain_state_owned::Field::MotherBrainBrainSegment2X
    };
    (Brain, segment_2_x) => {
        $crate::actors::mother_brain_state_owned::Field::MotherBrainBrainSegment2X
    };
    (MotherBrainBrain, segment_2_y) => {
        $crate::actors::mother_brain_state_owned::Field::MotherBrainBrainSegment2Y
    };
    (Brain, segment_2_y) => {
        $crate::actors::mother_brain_state_owned::Field::MotherBrainBrainSegment2Y
    };
    (MotherBrainBrain, segment_2_length) => {
        $crate::actors::mother_brain_state_owned::Field::MotherBrainBrainSegment2Length
    };
    (Brain, segment_2_length) => {
        $crate::actors::mother_brain_state_owned::Field::MotherBrainBrainSegment2Length
    };
    (MotherBrainBrain, segment_3_x) => {
        $crate::actors::mother_brain_state_owned::Field::MotherBrainBrainSegment3X
    };
    (Brain, segment_3_x) => {
        $crate::actors::mother_brain_state_owned::Field::MotherBrainBrainSegment3X
    };
    (MotherBrainBrain, segment_3_y) => {
        $crate::actors::mother_brain_state_owned::Field::MotherBrainBrainSegment3Y
    };
    (Brain, segment_3_y) => {
        $crate::actors::mother_brain_state_owned::Field::MotherBrainBrainSegment3Y
    };
    (MotherBrainBrain, segment_3_length) => {
        $crate::actors::mother_brain_state_owned::Field::MotherBrainBrainSegment3Length
    };
    (Brain, segment_3_length) => {
        $crate::actors::mother_brain_state_owned::Field::MotherBrainBrainSegment3Length
    };
    (MotherBrainBrain, segment_4_x) => {
        $crate::actors::mother_brain_state_owned::Field::MotherBrainBrainSegment4X
    };
    (Brain, segment_4_x) => {
        $crate::actors::mother_brain_state_owned::Field::MotherBrainBrainSegment4X
    };
    (MotherBrainBrain, segment_4_y) => {
        $crate::actors::mother_brain_state_owned::Field::MotherBrainBrainSegment4Y
    };
    (Brain, segment_4_y) => {
        $crate::actors::mother_brain_state_owned::Field::MotherBrainBrainSegment4Y
    };
    (MotherBrainBrain, segment_4_length) => {
        $crate::actors::mother_brain_state_owned::Field::MotherBrainBrainSegment4Length
    };
    (Brain, segment_4_length) => {
        $crate::actors::mother_brain_state_owned::Field::MotherBrainBrainSegment4Length
    };
    (MotherBrainBrain, segment_5_x) => {
        $crate::actors::mother_brain_state_owned::Field::MotherBrainBrainSegment5X
    };
    (Brain, segment_5_x) => {
        $crate::actors::mother_brain_state_owned::Field::MotherBrainBrainSegment5X
    };
    (MotherBrainBrain, segment_5_y) => {
        $crate::actors::mother_brain_state_owned::Field::MotherBrainBrainSegment5Y
    };
    (Brain, segment_5_y) => {
        $crate::actors::mother_brain_state_owned::Field::MotherBrainBrainSegment5Y
    };
    (MotherBrainBrain, segment_5_length) => {
        $crate::actors::mother_brain_state_owned::Field::MotherBrainBrainSegment5Length
    };
    (Brain, segment_5_length) => {
        $crate::actors::mother_brain_state_owned::Field::MotherBrainBrainSegment5Length
    };
    (MotherBrainBrain, neck_moving) => {
        $crate::actors::mother_brain_state_owned::Field::MotherBrainBrainNeckMoving
    };
    (Brain, neck_moving) => {
        $crate::actors::mother_brain_state_owned::Field::MotherBrainBrainNeckMoving
    };
    (MotherBrainBrain, lower_motion) => {
        $crate::actors::mother_brain_state_owned::Field::MotherBrainBrainLowerMotion
    };
    (Brain, lower_motion) => {
        $crate::actors::mother_brain_state_owned::Field::MotherBrainBrainLowerMotion
    };
    (MotherBrainBrain, upper_motion) => {
        $crate::actors::mother_brain_state_owned::Field::MotherBrainBrainUpperMotion
    };
    (Brain, upper_motion) => {
        $crate::actors::mother_brain_state_owned::Field::MotherBrainBrainUpperMotion
    };
    (MotherBrainBrain, neck_speed) => {
        $crate::actors::mother_brain_state_owned::Field::MotherBrainBrainNeckSpeed
    };
    (Brain, neck_speed) => {
        $crate::actors::mother_brain_state_owned::Field::MotherBrainBrainNeckSpeed
    };
    (Tube, function) => {
        $crate::actors::mother_brain_state_owned::Field::TubeFunction
    };
    (Tube, burst_y) => {
        $crate::actors::mother_brain_state_owned::Field::TubeBurstY
    };
    (Tube, fall_speed) => {
        $crate::actors::mother_brain_state_owned::Field::TubeFallSpeed
    };
    (Tube, smoke_timer) => {
        $crate::actors::mother_brain_state_owned::Field::TubeSmokeTimer
    };
    (Tube, smoke_index) => {
        $crate::actors::mother_brain_state_owned::Field::TubeSmokeIndex
    };
    (Tube, parameter_1) => {
        $crate::actors::mother_brain_state_owned::Field::TubeParameter1
    };
    (Tube, fall_delay) => {
        $crate::actors::mother_brain_state_owned::Field::TubeFallDelay
    };
}
pub(crate) use {part, part_field};
