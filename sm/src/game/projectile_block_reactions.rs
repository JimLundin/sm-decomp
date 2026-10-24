//! Synchronous terrain responses over the full projectile owner.
#![forbid(unsafe_code)]
use crate::game::projectile_room::{Axis, Scan};

/// Writes remain visible to a synchronous mechanism callback, including failure.
pub trait Progress {
    fn span(&self) -> u16;
    fn edge(&self) -> u16;
    fn missile(&self) -> u16;
    fn remaining(&self) -> u16;
    fn unblocked(&self) -> u16;
    fn set_remaining(&mut self, value: u16);
    fn set_unblocked(&mut self, value: u16);
}
impl Progress for Scan {
    fn span(&self) -> u16 {
        self.span
    }
    fn edge(&self) -> u16 {
        self.edge
    }
    fn missile(&self) -> u16 {
        self.missile
    }
    fn remaining(&self) -> u16 {
        self.remaining
    }
    fn unblocked(&self) -> u16 {
        self.unblocked
    }
    fn set_remaining(&mut self, value: u16) {
        self.remaining = value;
    }
    fn set_unblocked(&mut self, value: u16) {
        self.unblocked = value;
    }
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Material {
    Air,
    Solid,
    Slope,
    Shootable { solid: bool },
    Bombable { solid: bool },
    Extension(Axis),
}
pub struct Mechanism {
    pub header: u16,
    pub end_scan: bool,
}
pub fn material(level: u16) -> Material {
    match level >> 12 {
        0 | 2 | 3 | 6 => Material::Air,
        1 => Material::Slope,
        kind @ (4 | 12) => Material::Shootable { solid: kind == 12 },
        kind @ (7 | 15) => Material::Bombable { solid: kind == 15 },
        5 => Material::Extension(Axis::X),
        13 => Material::Extension(Axis::Y),
        _ => Material::Solid,
    }
}
pub fn redirect(
    axis: Axis,
    offset: i16,
    width: impl FnOnce() -> u16,
    cursor: impl FnOnce() -> u16,
) -> Option<u16> {
    if offset == 0 {
        return None;
    }
    let delta = if axis == Axis::X {
        offset as u16
    } else {
        (offset as u16).wrapping_mul(width())
    };
    Some(cursor().wrapping_add(delta))
}
pub trait Table {
    fn word(&self, index: usize) -> u16;
}
/// Resolve and spawn mechanism definitions without requiring projectile geometry.
pub trait Definitions {
    type Table: Table;
    fn resolve(&mut self, address: u32) -> Self::Table;
    fn bts(&self) -> u8;
    fn area(&self) -> u16;
    fn spawn(&mut self, header: u16);
}
pub fn mechanism(bombable: bool, solid: bool, content: &mut impl Definitions) -> Option<Mechanism> {
    let header;
    if bombable {
        let table = content.resolve(0x94a012);
        let bts = content.bts();
        if bts & 128 != 0 {
            return None;
        }
        header = table.word(usize::from(bts));
    } else {
        let bts = content.bts();
        if bts & 128 != 0 {
            if !solid {
                return None;
            }
            let table = content.resolve(0x949fc6);
            let cursor = table.word(usize::from(content.area()));
            let table = content.resolve(0x940000 | u32::from(cursor));
            let header = table.word(usize::from(bts & 127));
            return Some(Mechanism {
                header,
                end_scan: false,
            });
        }
        let table = content.resolve(0x949ea6);
        header = table.word(usize::from(bts));
    }
    Some(Mechanism {
        header,
        end_scan: !bombable && header == 0xb974,
    })
}
pub fn trigger(
    bombable: bool,
    solid: bool,
    progress: &mut impl Progress,
    context: &mut impl Definitions,
) {
    if let Some(mechanism) = mechanism(bombable, solid, context) {
        if mechanism.end_scan {
            progress.set_remaining(0);
            progress.set_unblocked(0xffff);
        }
        context.spawn(mechanism.header);
    }
}
pub trait Context: Definitions {
    fn byte_limit(&self) -> u16;
    fn cursor(&self) -> u16;
    fn select(&mut self, cell: u16);
    fn material(&self) -> Material;
    fn extension(&self) -> i16;
    fn width(&self) -> u16;
    fn slope(&self, axis: Axis, scan: &mut impl Progress) -> bool;
}
pub fn react(
    axis: Axis,
    byte_offset: u16,
    mut scan: Scan,
    context: &mut impl Context,
) -> (Scan, u8) {
    let result = react_progress(axis, byte_offset, &mut scan, context);
    (scan, result)
}
pub fn react_progress(
    axis: Axis,
    byte_offset: u16,
    scan: &mut impl Progress,
    context: &mut impl Context,
) -> u8 {
    if byte_offset >= context.byte_limit() {
        return 1;
    }
    context.select(byte_offset >> 1);
    let blocked = loop {
        match context.material() {
            Material::Air => break false,
            Material::Solid => break true,
            Material::Slope => break context.slope(axis, scan),
            Material::Extension(axis) => {
                let extension = context.extension();
                let Some(cell) = redirect(axis, extension, || context.width(), || context.cursor())
                else {
                    break false;
                };
                context.select(cell);
            }
            material @ (Material::Shootable { solid } | Material::Bombable { solid }) => {
                trigger(
                    matches!(material, Material::Bombable { .. }),
                    solid,
                    scan,
                    context,
                );
                break solid;
            }
        }
    };
    if blocked {
        scan.set_unblocked(scan.unblocked().wrapping_sub(1));
    }
    u8::from(blocked)
}
/// Samples only fields reached by the original branch, including broad selectors.
pub trait SlopeInputs {
    fn block(&self) -> u16;
    fn selected(&self) -> u16;
    fn width(&self) -> u16;
    fn bts(&self, block: u16) -> u8;
    fn position(&self, slot: u16, axis: Axis) -> u16;
    fn radius(&self, slot: u16, axis: Axis) -> u16;
    fn square_position(&self, slot: u16, axis: Axis) -> u16 {
        self.position(slot, axis)
    }
    fn square_radius(&self, slot: u16, axis: Axis) -> u16 {
        self.radius(slot, axis)
    }
    fn square_probe(&self, _axis: Axis) {}
    fn quadrant(&self, index: usize) -> bool;
    fn height(&self, index: usize) -> u8;
}
fn square(
    index: usize,
    other: usize,
    position: u16,
    radius: impl FnOnce() -> u16,
    scan: &impl Progress,
    occupied: impl Fn(usize) -> bool,
) -> bool {
    if scan.missile() != 0 {
        return occupied(index ^ if position & 8 != 0 { other } else { 0 });
    }
    let radius = radius();
    let near = position.wrapping_sub(radius) & 8;
    let far = radius.wrapping_add(position).wrapping_sub(1) & 8;
    if scan.span() == 0 {
        return (near == 0 && occupied(index)) || (far != 0 && occupied(index ^ other));
    }
    if scan.remaining() == 0 && far == 0 {
        return occupied(index);
    }
    if scan.remaining() != 0 && scan.remaining() == scan.span() && near != 0 {
        return occupied(index ^ other);
    }
    occupied(index) || occupied(index ^ other)
}
pub fn slope(axis: Axis, scan: &mut impl Progress, input: &impl SlopeInputs) -> bool {
    let slope = input.bts(input.block()) & 31;
    if slope >= 5 {
        return surface(axis, scan, input);
    }
    let slope = input.bts(input.block()) & 31;
    let block = input.block();
    let orientation = usize::from(input.bts(block) >> 6);
    let (shift, other) = if axis == Axis::X { (3, 2) } else { (2, 1) };
    let index = 4 * usize::from(slope) + (orientation ^ usize::from((scan.edge() & 8) >> shift));
    input.square_probe(axis.other());
    let slot = input.selected();
    square(
        index,
        other,
        input.square_position(slot, axis.other()),
        || input.square_radius(slot, axis.other()),
        scan,
        |i| input.quadrant(i),
    )
}
pub fn surface(axis: Axis, scan: &mut impl Progress, input: &impl SlopeInputs) -> bool {
    let block = input.block();
    let slot = input.selected();
    let dividend = input.block();
    let width = u16::from(input.width() as u8);
    let cell = if axis == Axis::X {
        if width == 0 { 0xffff } else { dividend / width }
    } else if width == 0 {
        dividend
    } else {
        dividend % width
    };
    if input.position(slot, axis.other()) >> 4 != cell {
        return false;
    }
    let x = (input.position(slot, Axis::X) & 15) ^ if input.bts(block) & 64 != 0 { 15 } else { 0 };
    let y = (input.position(slot, Axis::Y) & 15) ^ if input.bts(block) & 128 != 0 { 15 } else { 0 };
    let height = input.height(16 * usize::from(input.bts(block) & 31) + usize::from(x)) & 31;
    surface_hit(height, y, scan)
}
fn surface_hit(height: u8, y: u16, scan: &mut impl Progress) -> bool {
    let blocked = u16::from(height & 31) <= y;
    if blocked {
        scan.set_remaining(0);
        scan.set_unblocked(0);
    }
    blocked
}
