//! Ridley pogo and tail-list compatibility entries.
pub fn set_pogo_speeds(game: &mut (impl crate::runtime_access::RuntimeAccess + ?Sized)) {
    owned::set_pogo_speeds(&mut game.runtime());
}

/// Set Ridley tail segments variable from list ($A6:D3B4): copy the seven
/// words of the tail data list at `source` to the tail segments' variable at
/// `destination`. Was `Ridley_Func_93`.
pub fn set_tail_segments_from_list(
    destination: u16,
    source: u16,
    game: &mut (impl crate::runtime_access::RuntimeAccess + ?Sized),
) {
    owned::set_tail_segments_from_list(destination, source, &mut game.runtime());
}

#[path = "ridley_tables_owned.rs"]
pub mod owned;
