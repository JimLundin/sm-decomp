//! Qualify the room's exact warning-art publisher against native transfers.

use crate::cache::Cache;
use crate::lock::{Lock, RowId};
use data::assets::ridley_warning::{Art, SheetPatch};
use data::rom::Rom;
use game::exact::ridley_warning;
use oracle_record::FrameRecord;
use std::path::PathBuf;

const ART_BEGIN: u64 = 1270;
const HEADING: u64 = 1284;
const TYPING_DONE: u64 = 1584;

fn env(name: &str) -> PathBuf {
    std::env::var_os(name).unwrap().into()
}

fn cut(cache: &Cache, number: u64) -> FrameRecord {
    super::readers::record(cache, number).unwrap()
}

fn check_batch(drawing: &mut [u16], cache: &Cache, row: &str, number: u64, patch: &SheetPatch) {
    let before = cut(cache, number.wrapping_sub(1));
    let after = cut(cache, number);
    let span = ridley_warning::sheet(drawing, patch).unwrap();
    assert!(
        before.vram[span.clone()] != drawing[span.clone()],
        "{row} {number} held art"
    );
    assert!(
        after.vram[span.clone()] == drawing[span],
        "{row} {number} transferred art"
    );
}

fn check_text(drawing: &mut [u16], cache: &Cache, row: &str, art: &Art) {
    let heading = cut(cache, HEADING);
    let done = cut(cache, TYPING_DONE);
    let span = ridley_warning::line(drawing, &art.heading).unwrap();
    assert!(
        heading.vram[span.clone()] == drawing[span.clone()],
        "{row} heading cells"
    );
    assert!(
        done.vram[span.clone()] == drawing[span],
        "{row} carried heading cells"
    );
    for (index, letter) in art.english.iter().enumerate() {
        let span = ridley_warning::letter(drawing, letter).unwrap();
        assert!(
            done.vram[span.clone()] == drawing[span],
            "{row} English slot {index}"
        );
    }
}

#[test]
#[ignore = "pinned record cache and player ROM"]
fn imported_warning_art_matches_native_transfers() {
    let rom = Rom::new(std::fs::read(env("SM_ROM")).unwrap()).unwrap();
    let art = import::ridley_warning::import(rom.lorom()).unwrap();
    let lock = Lock::parse(&std::fs::read_to_string(env("SM_LOCK")).unwrap()).unwrap();
    for row in ["chain-any-0006-E0B5", "samus-ceres-push"] {
        let cache =
            Cache::open(&env("SM_CACHE"), lock.row(&RowId(row.to_owned())).unwrap()).unwrap();
        let mut drawing = cut(&cache, ART_BEGIN).vram;
        // Both schedules advance on the overlap cut without replacing the room drawing.
        for (number, batch) in ((ART_BEGIN + 1)..).zip(&art.sprites) {
            check_batch(&mut drawing, &cache, row, number, batch);
        }
        for (number, batch) in ((HEADING - 7)..).zip(&art.foreground) {
            check_batch(&mut drawing, &cache, row, number, &batch.patch);
        }
        check_text(&mut drawing, &cache, row, &art);
    }
}
