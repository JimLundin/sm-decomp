/// # Safety
/// Output is writable for capacity bytes, using the maintained copy_case protocol.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sm_verify_sprite_effect_case(
    id: u32,
    output: *mut u8,
    capacity: usize,
) -> usize {
    unsafe { copy_case(id, output, capacity, sm::verify_sprite_effect_case) }
}
