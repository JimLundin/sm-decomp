//! Direct bomb traversal inputs and the same owner-holding block definitions.
use super::{Context, Transaction, collision::Room};
use crate::game::bomb_terrain::{self as terrain, Input};
use crate::game::projectile_block_reactions::{self as reactions, Definitions, Table};
use crate::game::projectile_room::{Axis, Scan};
/// Encoded tables stay outside the core's opaque-header contract.
pub fn special_header<O>(mut owner: O, c: &mut impl Definitions<O>) -> (O, u16) {
    let bts = c.bts();
    if bts & 0x80 != 0 {
        let table;
        (owner, table) = c.resolve(owner, 0x949e44);
        let pointer;
        (owner, pointer) = c.word(owner, &table, usize::from(c.area()));
        let table;
        (owner, table) = c.resolve(owner, 0x940000 | u32::from(pointer));
        c.word(owner, &table, usize::from(bts & 0x7f))
    } else {
        let table;
        (owner, table) = c.resolve(owner, 0x949da4);
        c.word(owner, &table, usize::from(bts))
    }
}
impl<'g> terrain::Context<Transaction<'g>> for Context {
    type Header = u16;
    fn input(&self, g: &Transaction<'g>, f: Input) -> u16 {
        use Input::*;
        match f {
            Width => word!(g.room_width_in_blocks),
            Height => word!(g.room_height_in_blocks),
            ScrollWidth => word!(g.room_width_in_scrolls),
            ScrollHeight => word!(g.room_height_in_scrolls),
            Radius => word!(g.power_bomb_explosion_radius),
            OriginX => word!(g.power_bomb_explosion_x_pos),
            OriginY => word!(g.power_bomb_explosion_y_pos),
        }
    }
    fn cursor(&self, g: &Transaction<'g>) -> u16 {
        word!(g.cur_block_index)
    }
    fn select(&mut self, g: &mut Transaction<'g>, block: u16) {
        word!(g.cur_block_index = block);
    }
    fn level(&mut self, g: Transaction<'g>) -> (Transaction<'g>, u16) {
        let value = g.read_le(0x10002 + usize::from(word!(g.cur_block_index)) * 2);
        (g, value)
    }
    fn bts(&mut self, g: Transaction<'g>) -> (Transaction<'g>, u8) {
        let value = g.byte(0x16402 + usize::from(word!(g.cur_block_index)));
        (g, value)
    }
    fn slope(&mut self, g: Transaction<'g>) -> (Transaction<'g>, bool) {
        let value = reactions::surface(
            Axis::X,
            &mut Scan {
                displacement: 0,
                position: 0,
                auxiliary: 0,
                span: 0,
                edge: 0,
                missile: 0,
                remaining: 0,
                unblocked: 0,
            },
            &g,
        );
        (g, value)
    }
    fn mechanism(
        &mut self,
        g: Transaction<'g>,
        bombable: bool,
        solid: bool,
    ) -> (Transaction<'g>, Option<u16>) {
        let mut room = Room(Some(g));
        let header = reactions::mechanism((), bombable, solid, &mut room)
            .1
            .map(|m| m.header);
        (room.finish(), header)
    }

    fn special(&mut self, g: Transaction<'g>) -> (Transaction<'g>, u16) {
        let mut room = Room(Some(g));
        let header = special_header((), &mut room).1;
        (room.finish(), header)
    }

    fn spawn(&mut self, g: Transaction<'g>, header: u16) -> Transaction<'g> {
        g.foreign(|g| unsafe { crate::game::room_mechanisms::spawn_collision(header, g) })
            .0
    }
}
