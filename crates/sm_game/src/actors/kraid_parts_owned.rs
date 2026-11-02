//! Kraid's small part callbacks use the finite actor and interaction owners.
#![forbid(unsafe_code)]
pub trait Host:
    crate::actors::owned_common_lifecycle::BossState
    + crate::actors::boss_fields_owned::State
    + crate::actors::boss_interaction::Access
    + crate::player_position::PlayerPosition
{
    fn native_kraid_part(&mut self, bank: u8, address: u16) -> sm_data::cartridge::RomRecord;
}
pub(crate) fn native_record<H: Host + ?Sized>(
    bank: u8,
    address: u16,
    host: &mut H,
) -> sm_data::cartridge::RomRecord {
    host.native_kraid_part(bank, address)
}
impl Host for crate::actors::owned_execution::PreparedActorState<'_> {
    fn native_kraid_part(&mut self, bank: u8, address: u16) -> sm_data::cartridge::RomRecord {
        self.native_actor_record(bank, address)
    }
}
