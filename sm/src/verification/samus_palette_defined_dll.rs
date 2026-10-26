/// Exact shared synthetic fixtures; implementation selection stays in the library.
/// # Safety
/// Output must be writable for capacity bytes, matching the existing copy_case API.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sm_verify_samus_palette_defined_case(
    id: u32,
    output: *mut u8,
    capacity: usize,
) -> usize {
    unsafe { copy_case(id, output, capacity, sm::verify_samus_palette_defined_case) }
}
