/// Synthetic room predecessor cases using the existing copy_case protocol.
/// # Safety
/// Output must be writable for capacity bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sm_verify_room_reference_case(
    id: u32,
    output: *mut u8,
    capacity: usize,
) -> usize {
    unsafe { copy_case(id, output, capacity, sm::verify_room_reference_case) }
}
