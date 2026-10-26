//! Definition lookup over the existing aggregate, with Loaded provenance.
#![forbid(unsafe_code)]
use crate::game::{
    player_projectiles::{access::room_bts, host::Host},
    projectile_block_reactions::Definitions,
    room_mechanisms::host::{Content, ContentAddress},
    session_access::SessionAccess,
    session_runtime::Runtime,
};
use crate::rom::RomRecord;
impl Definitions for Runtime<'_> {
    type Table = RomRecord;
    fn resolve(&mut self, address: u32) -> RomRecord {
        Content::loaded_record(
            self,
            ContentAddress {
                bank: (address >> 16) as u8,
                offset: address as u16,
            },
        )
    }
    fn bts(&self) -> u8 {
        room_bts(self, self.session().room.collision_cursor)
    }
    fn area(&self) -> u16 {
        self.session().room.area
    }
    fn spawn(&mut self, header: u16) {
        self.spawn_mechanism(header);
    }
}
