//! Synchronous ABI: the caller owns both disjoint allocations and excludes concurrent access.
use crate::{RomView, WRAM_SIZE, WramView, decompress_to_mem};
use std::panic::{AssertUnwindSafe, catch_unwind};

#[repr(C)]
#[derive(Default)]
pub struct Report {
    pub written: usize,
    pub final_source: u32,
    pub modes: u32,
}

/// Execute one call without retaining pointers or invoking callbacks.
///
/// # Safety
/// Non-null pointers must refer to live allocations of the supplied sizes. ROM is
/// immutable and WRAM exclusively borrowed until this function returns. Report
/// must be writable and disjoint from both allocations. Invalid addresses cannot
/// be made safe by a C ABI; extents, integer overflow and overlap are checked here.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sm_decompress_v1(
    rom: *const u8,
    rom_len: usize,
    wram: *mut u8,
    wram_len: usize,
    source: u32,
    destination: usize,
    report: *mut Report,
) -> i32 {
    let Some(rom_end) = (rom as usize).checked_add(rom_len) else { return 1 };
    let Some(wram_end) = (wram as usize).checked_add(wram_len) else { return 1 };
    let Some(report_end) = (report as usize).checked_add(std::mem::size_of::<Report>()) else { return 1 };
    if rom.is_null()
        || wram.is_null()
        || report.is_null()
        || wram_len != WRAM_SIZE
        || rom_len == 0
        || rom_len > 0x400000
        || !(report as usize).is_multiple_of(std::mem::align_of::<Report>())
        || ((rom as usize) < wram_end && (wram as usize) < rom_end)
        || ((report as usize) < rom_end && (rom as usize) < report_end)
        || ((report as usize) < wram_end && (wram as usize) < report_end)
    {
        return 1;
    }
    match catch_unwind(AssertUnwindSafe(|| {
        let rom = RomView::new(unsafe { std::slice::from_raw_parts(rom, rom_len) })?;
        let mut wram = WramView::new(unsafe { std::slice::from_raw_parts_mut(wram, wram_len) })?;
        let result = decompress_to_mem(&rom, &mut wram, source, destination)?;
        let names = [
            "literal",
            "byte-fill",
            "word-fill",
            "increment-fill",
            "absolute-copy",
            "absolute-xor",
            "relative-copy",
            "relative-xor",
        ];
        let modes = names
            .iter()
            .enumerate()
            .fold(0, |bits, (i, name)| bits | if result.cases.contains(name) { 1 << i } else { 0 });
        unsafe { report.write(Report { written: result.written, final_source: result.final_source, modes }) };
        Ok::<(), crate::DecompressError>(())
    })) {
        Ok(Ok(())) => 0,
        Ok(Err(_)) => 2,
        Err(_) => 3,
    }
}
