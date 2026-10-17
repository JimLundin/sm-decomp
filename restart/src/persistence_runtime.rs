//! Install the verified bounded SRAM codec for ordinary native Save/Load calls.
//! Hook installation is configuration; native counters report exercised ownership.
use sm_native_host::{Game, PersistenceOutcome, PersistenceOutput, PersistenceServices};
use sm_persistence::{Entry, Output, Services};

struct Disk<'a>(&'a mut PersistenceServices);
impl Services for Disk<'_> {
    fn available(&self) -> bool {
        self.0.available()
    }
    fn write_sram(&mut self, ram: &mut [u8], sram: &mut [u8]) -> Result<(), String> {
        self.0.write_sram(ram, sram)
    }
}

/// Register before the first game step. Safe unowned calls retain native fallback;
/// codec/service errors propagate and must never be retried through native C.
pub fn install(game: &mut Game) -> Result<(), String> {
    game.set_persistence_hook(|entry, argument, ram, sram, rom, services| {
        let entry = Entry::try_from(entry as u32)?;
        Ok(match sm_persistence::execute(entry, argument, ram, sram, rom, &mut Disk(services))? {
            Some(Output::Done) => PersistenceOutcome::Handled(PersistenceOutput::Done),
            Some(Output::Loaded(value)) => PersistenceOutcome::Handled(PersistenceOutput::Loaded(value)),
            None => PersistenceOutcome::Refused,
        })
    })
}
