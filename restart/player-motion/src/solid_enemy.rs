use crate::memory::*;

/// Read-only result used both for admission and the actual checker effects.
pub(crate) struct Contact {
    pub collision: u16,
    pub amount: i32,
    last: Option<u16>,
    direction: u16,
    clear_fraction: bool,
}

impl Contact {
    pub fn apply(&self, ram: &mut [u8]) {
        if let Some(index) = self.last {
            put(ram, INDEX, index);
            if self.collision != 0 {
                put(ram, COLLIDING + usize::from(self.direction) * 2, index);
            }
        }
        if self.clear_fraction {
            put(ram, YF, 0);
        }
    }
}

pub(crate) fn list_supported(ram: &[u8]) -> bool {
    if word(ram, LIST_GATE) == 0 {
        return true;
    }
    for n in 0..32 {
        let index = word(ram, LIST + n * 2);
        if index == 0xffff {
            return true;
        }
        if index > 0x7c0 || !index.is_multiple_of(0x40) {
            return false;
        }
    }
    false
}

// Preserve the C carry/borrow and conditional extra-pixel rounding, including
// its integer-promoted fraction sum and wrapped-position equality checks.
fn destination(position: u16, subposition: u16, amount: i32, positive: bool) -> u16 {
    let integer = (amount >> 16) as u16;
    let fraction = amount as u16;
    if positive {
        let mut future = position.wrapping_add(integer);
        let sum = u32::from(subposition) + u32::from(fraction);
        let mut zero = sum == 0;
        if sum > 0xffff {
            zero = future == 0xffff;
            future = future.wrapping_add(1);
        }
        if !zero {
            future = future.wrapping_add(1);
        }
        future
    } else {
        let mut future = position.wrapping_sub(integer);
        let mut equal = subposition == fraction;
        if subposition < fraction {
            equal = future == 1;
            future = future.wrapping_sub(1);
        }
        if !equal {
            future = future.wrapping_sub(1);
        }
        future
    }
}

pub(crate) fn inspect(ram: &[u8], amount: i32, direction: u16) -> Contact {
    let direction = direction & 3;
    let mut result = Contact { collision: 0, amount, last: None, direction, clear_fraction: false };
    if word(ram, LIST_GATE) == 0 {
        return result;
    }
    let (axis, fraction) = if direction < 2 { (X, XF) } else { (Y, YF) };
    let future = destination(word(ram, axis), word(ram, fraction), amount, direction & 1 != 0);
    let px = if direction < 2 { future } else { word(ram, X) };
    let py = if direction >= 2 { future } else { word(ram, Y) };
    result.last = Some(0);
    // The enabled list was checked immutably before inspect is called.
    for n in 0..32 {
        let index = word(ram, LIST + 2 * n);
        if index == 0xffff {
            break;
        }
        result.last = Some(index);
        let enemy = ENEMIES + usize::from(index);
        if word(ram, enemy + 0x26) == 0 && word(ram, enemy + 0x0e) & 0x8000 == 0 {
            continue;
        }
        let ex = word(ram, enemy + 2);
        let ey = word(ram, enemy + 6);
        let ew = word(ram, enemy + 0x0a);
        let eh = word(ram, enemy + 0x0c);
        let dx = (ex.wrapping_sub(px) as i16).wrapping_abs() as u16;
        let dy = (ey.wrapping_sub(py) as i16).wrapping_abs() as u16;
        if !(dx < ew || dx.wrapping_sub(ew) < word(ram, XR)) || !(dy < eh || dy.wrapping_sub(eh) < word(ram, YR)) {
            continue;
        }
        let (position, radius, enemy_position, enemy_radius) =
            if direction < 2 { (word(ram, X), word(ram, XR), ex, ew) } else { (word(ram, Y), word(ram, YR), ey, eh) };
        let (delta, exact) = if direction & 1 == 0 {
            let edge = enemy_position.wrapping_add(enemy_radius);
            let promoted = i32::from(position) - i32::from(radius);
            ((promoted - i32::from(edge)) as i16, promoted == i32::from(edge))
        } else {
            let delta = enemy_position.wrapping_sub(enemy_radius).wrapping_sub(radius.wrapping_add(position)) as i16;
            (delta, delta == 0)
        };
        if exact || delta >= 0 {
            result.collision = 0xffff;
            result.amount = i32::from(delta) << 16;
            result.clear_fraction = exact;
            return result;
        }
    }
    result
}
