#![cfg(not(doctest))]
//! The save-files model, built from its behaviour spec alone.
//!
//! The cartridge holds the three player slots, the last file chosen and whether the game has
//! been completed. Boot checks it and reads up to the first valid file (`BootFiles`); the file
//! menu reads all three (`MenuFiles`); choosing a file records the choice and starts or resumes its
//! run (`ChooseFile`); and the run is saved as the trip to Ceres ends and again as its escape
//! fades out (`WriteFile`). Every read accepts either matching check pair or clears the file
//! (`ReadSlot`).
//!
//! The model works on the typed cartridge of `types::save_files`; a supplied `SaveFormat`
//! encodes runs and maps the battery image. A choice and a save each finish in two observable
//! milestones, and the caller reports how far each got.

mod boot;
mod catalogue;
mod choice;
mod files;
mod format;
mod menu;
mod read;
mod selection;
mod write;

pub use boot::BootReport;
pub use catalogue::{Behaviour, BehaviourId, CATALOGUE};
pub use choice::{ChoiceProgress, ChoiceStage, StartData};
pub use files::{Refusal, SaveFiles};
pub use format::SaveFormat;
pub use menu::MenuReport;
pub use read::{
    EMPTY_FILE_AREA, EMPTY_FILE_STATION, FileSummary, ReadOutcome, ReadReceipt, integrity_pair,
};
pub use selection::{initial_cursor, selection_of, slot_number};
pub use write::{SaveCut, WriteStage};
