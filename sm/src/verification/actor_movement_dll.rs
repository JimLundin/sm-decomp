/// Forty actual predecessor fixtures; unchanged existing copy_case convention.
/// # Safety
/// Output must be writable for capacity bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sm_verify_actor_movement_case(
    id: u32,
    output: *mut u8,
    capacity: usize,
) -> usize {
    unsafe { copy_case(id, output, capacity, sm::verify_actor_movement_case) }
}
