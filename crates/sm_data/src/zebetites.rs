//! The zebetites' tables in bank $A6.
use crate::table::{LengthError, Rows};

/// $A6:FC03: placement per zebetites destroyed count, six lists of four words:
/// multi-part flag, height, instruction list, X, then Y for enemy parameter 1
/// zero and non-zero.
pub const PLACEMENT_ADDRESS: u32 = 0xa6_fc03;
pub const PLACEMENTS: usize = 4;
pub const PLACEMENT_BYTES: usize = PLACEMENTS * 12;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Placement {
    pub multi_part: u16,
    pub height: u16,
    pub instruction_list: u16,
    pub x_position: u16,
    pub y_position: u16,
    pub y_position_alternate: u16,
}

pub type Placements = Rows<Placement, PLACEMENTS>;

pub fn import_placements(bytes: &[u8]) -> Result<Placements, LengthError> {
    Rows::import_columns(
        bytes,
        |[
            multi_part,
            height,
            instruction_list,
            x_position,
            y_position,
            y_position_alternate,
        ]| {
            Placement {
                multi_part,
                height,
                instruction_list,
                x_position,
                y_position,
                y_position_alternate,
            }
        },
    )
}

/// $A6:FD4A: instruction lists per health stage for the big zebetite, then
/// the small pair ($FD54).
pub const HEALTH_LIST_ADDRESS: u32 = 0xa6_fd4a;
pub const HEALTH_STAGES: usize = 5;
pub const HEALTH_LIST_BYTES: usize = HEALTH_STAGES * 4;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HealthLists {
    pub big: u16,
    pub small_pair: u16,
}

pub type HealthStages = Rows<HealthLists, HEALTH_STAGES>;

pub fn import_health_lists(bytes: &[u8]) -> Result<HealthStages, LengthError> {
    Rows::import_columns(bytes, |[big, small_pair]| HealthLists { big, small_pair })
}
