//! Direct room traversal and one owner-holding binding for legacy-shaped reaction traits.
use super::{Context, Transaction};
use crate::game::projectile_block_reactions::{
    self as reactions, Definitions, Material, Mechanism, Progress, SlopeInputs,
};
use crate::game::projectile_flight_control::Flight;
use crate::game::projectile_room::{self as room, Axis, Field, Kind, Scan};
use crate::rom::RomRecord;
impl SlopeInputs for Transaction<'_> {
    fn block(&self) -> u16 {
        word!(self.cur_block_index)
    }
    fn selected(&self) -> u16 {
        word!(self.projectile_index)
    }
    fn width(&self) -> u16 {
        word!(self.room_width_in_blocks)
    }
    fn bts(&self, block: u16) -> u8 {
        self.byte(0x16402 + usize::from(block))
    }
    fn position(&self, slot: u16, axis: Axis) -> u16 {
        match axis {
            Axis::X => word!(self.projectile_x_pos[slot >> 1]),
            Axis::Y => word!(self.projectile_y_pos[slot >> 1]),
        }
    }
    fn radius(&self, slot: u16, axis: Axis) -> u16 {
        match axis {
            Axis::X => word!(self.projectile_x_radius[slot >> 1]),
            Axis::Y => word!(self.projectile_y_radius[slot >> 1]),
        }
    }
    fn square_position(&self, slot: u16, axis: Axis) -> u16 {
        let at = if axis == Axis::X { 0xb64 } else { 0xb78 } + 2 * usize::from(slot >> 1);
        self.read_native(at)
    }
    fn square_radius(&self, slot: u16, axis: Axis) -> u16 {
        let at = if axis == Axis::X { 0xbb4 } else { 0xbc8 } + 2 * usize::from(slot >> 1);
        self.read_native(at)
    }
    fn square_probe(&self, axis: Axis) {
        crate::verify_probe::hit(if axis == Axis::X {
            "square-slope.shot.horizontal"
        } else {
            "square-slope.shot.vertical"
        });
    }
    fn quadrant(&self, index: usize) -> bool {
        crate::game::sm_94::kTab948E54[index] != 0
    }
    fn height(&self, index: usize) -> u8 {
        crate::game::sm_94::kAlignYPos_Tab0[index]
    }
}
fn position(axis: Axis) -> usize {
    match axis {
        Axis::X => 0xb64,
        Axis::Y => 0xb78,
    }
}
fn fraction(axis: Axis) -> usize {
    match axis {
        Axis::X => 0xb8c,
        Axis::Y => 0xba0,
    }
}
impl room::State for Transaction<'_> {
    fn field(&self, slot: u16, axis: Axis, field: Field) -> u16 {
        let offset = usize::from(slot >> 1) * 2;
        let base = match field {
            Field::Position => position(axis),
            Field::Fraction => fraction(axis),
            Field::Radius => {
                if axis == Axis::X {
                    0xbb4
                } else {
                    0xbc8
                }
            }
            Field::Speed => {
                if axis == Axis::X {
                    0xbdc
                } else {
                    0xbf0
                }
            }
            Field::ScreenHigh => return u16::from(self.byte(position(axis) + offset + 1)),
        };
        self.read_native(base + offset)
    }
    fn move_by(&mut self, slot: u16, axis: Axis, amount: i32) {
        let offset = usize::from(slot >> 1) * 2;
        let low = u32::from(self.read_native(fraction(axis) + offset));
        let high = u32::from(self.read_native(position(axis) + offset));
        let sum = (low | high << 16).wrapping_add(amount as u32);
        self.write_slice_native(fraction(axis) + offset, sum as u16);
        self.write_slice_native(position(axis) + offset, (sum >> 16) as u16);
    }
}
impl<'g> room::Context<Transaction<'g>> for Context {
    fn width(&self, owner: &Transaction<'g>) -> u16 {
        word!(owner.room_width_in_blocks)
    }
    fn scrolls(&self, owner: &Transaction<'g>, axis: Axis) -> u16 {
        match axis {
            Axis::X => word!(owner.room_width_in_scrolls),
            Axis::Y => word!(owner.room_height_in_scrolls),
        }
    }
    fn react(
        &mut self,
        owner: Transaction<'g>,
        axis: Axis,
        block: u16,
        scan: Scan,
    ) -> (Transaction<'g>, Scan, u8) {
        let mut context = Room(Some(owner));
        let (_, scan, hit) = reactions::react((), axis, block, scan, &mut context);
        (context.finish(), scan, hit)
    }
    fn retire(&mut self, owner: Transaction<'g>, slot: u16) -> Transaction<'g> {
        super::lifecycle::retire(slot, owner)
    }
}
pub(super) fn scan(
    slot: u16,
    flight: Flight,
    axis: Axis,
    owner: Transaction<'_>,
) -> (Transaction<'_>, u8) {
    let kind = match flight {
        Flight::NonWave => Kind::Beam,
        Flight::Wave => Kind::Wave,
        Flight::Missile | Flight::SuperMissile => Kind::Missile,
    };
    room::collide(owner, slot, axis, kind, &mut Context::default())
}

// Reaction traits have owner-less reads and cursor writes. Hold the SAME owner
// here for that call, rather than caching inputs or keeping a second RAM view.
pub(super) struct Room<'g>(pub(super) Option<Transaction<'g>>);
impl<'g> Room<'g> {
    fn owner(&self) -> &Transaction<'g> {
        self.0.as_ref().unwrap()
    }
    pub(super) fn finish(mut self) -> Transaction<'g> {
        self.0.take().unwrap()
    }
}
impl reactions::Table for RomRecord {
    fn word(&self, index: usize) -> u16 {
        self.image.table_word(self.base, index as isize)
    }
}
impl Definitions<()> for Room<'_> {
    type Table = RomRecord;
    fn resolve(&mut self, _: (), address: u32) -> ((), RomRecord) {
        let owner = self.0.take().unwrap();
        let (owner, record) = Context::record(owner, (address >> 16) as u8, address as u16);
        self.0 = Some(owner);
        ((), record)
    }
    fn bts(&self) -> u8 {
        let owner = self.owner();
        owner.byte(0x16402 + usize::from(word!(owner.cur_block_index)))
    }
    fn area(&self) -> u16 {
        let owner = self.owner();
        word!(owner.area_index)
    }
}
impl reactions::Context<()> for Room<'_> {
    type Header = u16;
    fn byte_limit(&self) -> u16 {
        let owner = self.owner();
        word!(owner.room_size_in_blocks)
    }
    fn cursor(&self) -> u16 {
        let owner = self.owner();
        word!(owner.cur_block_index)
    }
    fn select(&mut self, cell: u16) {
        let owner = self.0.as_mut().unwrap();
        word!(owner.cur_block_index = cell);
    }
    fn material(&self) -> Material {
        let owner = self.owner();
        reactions::material(owner.read_le(0x10002 + usize::from(word!(owner.cur_block_index)) * 2))
    }
    fn extension(&self) -> i16 {
        i16::from(self.bts() as i8)
    }
    fn width(&self) -> u16 {
        let owner = self.owner();
        word!(owner.room_width_in_blocks)
    }
    fn slope(&self, _: &(), axis: Axis, scan: &mut impl Progress) -> bool {
        reactions::slope(axis, scan, self.owner())
    }
    fn mechanism(&mut self, _: (), bombable: bool, solid: bool) -> ((), Option<Mechanism<u16>>) {
        reactions::mechanism((), bombable, solid, self)
    }
    fn spawn(&mut self, _: (), header: u16) {
        let owner = self.0.take().unwrap();
        self.0 = Some(
            owner
                .foreign(|g| unsafe { crate::game::room_mechanisms::spawn_collision(header, g) })
                .0,
        );
    }
}
