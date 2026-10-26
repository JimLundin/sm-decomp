/// Actual device-backed synthetic room installation scenarios.
/// # Safety
/// Output must be writable for capacity bytes. One call per child process.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sm_verify_room_background_case(
    id: u32,
    output: *mut u8,
    capacity: usize,
) -> usize {
    unsafe { copy_case(id, output, capacity, sm::verify_room_background_case) }
}
