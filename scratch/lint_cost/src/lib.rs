//! Measures what the K5/K8 lint set costs on arithmetic-heavy code: the same Samus
//! movement and slope landing written twice, checked equal on random inputs.

pub mod plain;
pub mod strict;

#[cfg(test)]
mod tests {
    use super::{plain, strict};

    fn level() -> (plain::Level, strict::Level) {
        let width = 64;
        let mut blocks = vec![0u16; 64 * 32];
        let mut bts = vec![0u8; 64 * 32];
        for column in 0..64usize {
            let index = 1280_usize.wrapping_add(column);
            blocks[index] = 0x1000;
            bts[index] = u8::try_from(column % 5).unwrap() | if column % 3 == 0 { 0x40 } else { 0 };
        }
        (
            plain::Level { width, blocks: blocks.clone(), bts: bts.clone() },
            strict::Level { width: 64, blocks, bts },
        )
    }

    fn agree(a: &plain::Samus, b: &strict::Samus) -> bool {
        let vertical = match b.vertical {
            strict::Vertical::Grounded => a.on_ground,
            strict::Vertical::Rising(speed) => !a.on_ground && a.moving_up && a.y_speed == speed.raw(),
            strict::Vertical::Falling(speed) => !a.on_ground && !a.moving_up && a.y_speed == speed.raw(),
        };
        vertical
            && a.x == b.x.raw()
            && a.y == b.y.raw()
            && a.x_speed == b.x_speed.raw()
            && a.facing_left == (b.facing == strict::Facing::Left)
    }

    #[test]
    fn plain_and_strict_agree_on_random_input() {
        let (plain_level, strict_level) = level();
        let mut seed = 0x1234_5678u32;
        for _ in 0..200 {
            let mut a = plain::Samus { x: 0x0100_0000, y: 0x00F0_0000, ..Default::default() };
            let mut b = strict::Samus {
                x: strict::Fixed::from_raw(a.x),
                y: strict::Fixed::from_raw(a.y),
                vertical: strict::Vertical::Falling(strict::Fixed::default()),
                ..Default::default()
            };
            for _ in 0..600 {
                seed = seed.wrapping_mul(1_103_515_245).wrapping_add(12_345);
                let held = u8::try_from((seed >> 16) & 0xF).unwrap();
                let (left, right) = (held & 1 != 0, held & 2 != 0);
                let direction = match (left, right) {
                    (false, false) => None,
                    (true, false) => Some(strict::Facing::Left),
                    (_, true) => Some(strict::Facing::Right),
                };
                plain::step(&mut a, &plain_level, held);
                strict::step(
                    &mut b,
                    &strict_level,
                    strict::Held { direction, run: held & 4 != 0, jump: held & 8 != 0 },
                );
                assert!(agree(&a, &b), "{a:?} {b:?}");
            }
        }
    }
}
