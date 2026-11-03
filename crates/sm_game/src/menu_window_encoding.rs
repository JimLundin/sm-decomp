//! Compatibility and checkpoint order for the map-menu motion owner.
#![forbid(unsafe_code)]
use crate::menu_window_state::{Field, WindowState};
use crate::session_cohort::Cohort;
pub fn address(field: Field) -> usize {
    match field {
        Field::RightMargin => 0x9e20,
        Field::LeftMargin => 0x9e22,
        Field::Fraction(i) => 0x9e30 + 4 * i,
        Field::Whole(i) => 0x9e32 + 4 * i,
        Field::VelocityFraction(i) => 0x9e40 + 4 * i,
        Field::VelocityWhole(i) => 0x9e42 + 4 * i,
        Field::Timer => 0x9e50,
    }
}
pub fn decode(bytes: &impl Fn(usize) -> u8) -> WindowState {
    let mut value = WindowState::initialized(0, 0, Default::default(), Default::default(), 0);
    for field in Field::ALL {
        let at = address(field);
        *field.get_mut(&mut value) = u16::from_le_bytes([bytes(at), bytes(at + 1)]);
    }
    value
}
impl Cohort for WindowState {
    fn words(&mut self, mut visit: impl FnMut(usize, &mut u16)) {
        for field in Field::ALL {
            visit(address(field), field.get_mut(self));
        }
    }
    fn session_words(
        session: &mut crate::session::Session,
        mut visit: impl FnMut(usize, &mut u16),
    ) {
        for field in Field::ALL {
            let at = address(field);
            let mut word = crate::fx_encoding::control_word(session, at);
            let before = word;
            visit(at, &mut word);
            if word != before {
                crate::fx_encoding::write_control_word(session, at, word);
            }
        }
    }
    fn session_copy(session: &crate::session::Session) -> Self {
        decode(&|at| crate::fx_encoding::control_byte(session, at))
    }
}

/// Every window field's bytes lie in this range.
const LANES: core::ops::Range<usize> = 0x9e20..0x9e52;
/// Alias byte lanes of the single canonical window owner, including odd words.
pub fn lane(at: usize) -> Option<(Field, usize)> {
    if !LANES.contains(&at) {
        return None;
    }
    Field::ALL.into_iter().find_map(|field| {
        let first = address(field);
        if (first..first + 2).contains(&at) {
            Some((field, at - first))
        } else {
            None
        }
    })
}
pub fn projected_byte(window: &WindowState, at: usize) -> Option<u8> {
    let (field, lane) = lane(at)?;
    Some(field.get(window).to_le_bytes()[lane])
}
pub fn write_byte(window: &mut WindowState, at: usize, value: u8) -> bool {
    let Some((field, lane)) = lane(at) else {
        return false;
    };
    let word = field.get_mut(window);
    let mut bytes = word.to_le_bytes();
    bytes[lane] = value;
    *word = u16::from_le_bytes(bytes);
    true
}

#[cfg(test)]
mod tests {
    #[test]
    fn every_window_lane_lies_in_the_lane_range() {
        for field in super::Field::ALL {
            let first = super::address(field);
            assert!(super::LANES.contains(&first) && super::LANES.contains(&(first + 1)));
        }
    }
}
