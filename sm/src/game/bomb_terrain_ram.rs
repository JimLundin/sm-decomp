//! Checked room observations and encoded content for bomb traversal.
use crate::game::bomb_terrain::{self as terrain, Context, Input};
use crate::game::bomb_lifecycle_ram::Runtime;
use crate::game::projectile_block_reactions::{self as reactions, Definitions, Table};
use crate::game::projectile_block_reactions_ram::{Room, Native};
use crate::game::projectile_room::{Axis, Scan};
use crate::game_state::Game;
use crate::ram_map::checked_word as word;
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
impl<'g> Context<&'g mut Game> for Runtime {
    type Header = u16;
    fn input(&self, g: &&'g mut Game, f: Input) -> u16 {
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
    fn cursor(&self, g: &&'g mut Game) -> u16 {
        g.cur_block_index()
    }
    fn select(&mut self, g: &mut &'g mut Game, block: u16) {
        g.set_cur_block_index(block);
    }
    fn level(&mut self, g: &'g mut Game) -> (&'g mut Game, u16) {
        let value = g.current_level_block();
        (g, value)
    }
    fn bts(&mut self, g: &'g mut Game) -> (&'g mut Game, u8) {
        let value = g.current_block_bts();
        (g, value)
    }
    fn slope(&mut self, g: &'g mut Game) -> (&'g mut Game, bool) {
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
            &*g,
        );
        (g, value)
    }
    fn mechanism(
        &mut self,
        g: &'g mut Game,
        bombable: bool,
        solid: bool,
    ) -> (&'g mut Game, Option<u16>) {
        let header = reactions::mechanism(
            (),
            bombable,
            solid,
            &mut Room {
                game: g,
                content: Native,
            },
        )
        .1
        .map(|m| m.header);
        (g, header)
    }
    fn special(&mut self, g: &'g mut Game) -> (&'g mut Game, u16) {
        let header = special_header(
            (),
            &mut Room {
                game: g,
                content: Native,
            },
        )
        .1;
        (g, header)
    }
    fn spawn(&mut self, g: &'g mut Game, header: u16) -> &'g mut Game {
        unsafe { crate::game::plm_spawning::SpawnPLM(header, g) };
        g
    }
}
/// Setup effects require initialized runtime state.
pub unsafe fn explode(slot: u16, g: &mut Game) {
    terrain::explode(g, slot, &mut Runtime);
}
/// Setup effects require initialized runtime state.
pub unsafe fn spread(slot: u16, g: &mut Game) -> u8 {
    terrain::spread(g, slot, &mut Runtime).1
}
