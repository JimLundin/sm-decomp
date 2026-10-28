/// Actual public GameOwner prepared-world route; one initialized child per case.
/// # Safety
/// rom is a valid NUL-terminated path; output is writable for capacity bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sm_verify_room_world_case(
    rom: *const c_char,
    id: u32,
    output: *mut u8,
    capacity: usize,
) -> usize {
    // Match the library's room_world_cases::CASES before consuming STARTED.
    #[cfg(sm_verify_dll)]
    const CASES: u32 = 105;
    #[cfg(not(sm_verify_dll))]
    const CASES: u32 = 11;
    unsafe {
        if rom.is_null() || output.is_null() || id >= CASES || capacity < 524_288 {
            return 0;
        }
        if STARTED.swap(true, Ordering::SeqCst) {
            eprintln!("Room world: use a fresh child per case");
            return 0;
        }
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            sm::host::install_host_hooks(sm::host::HostHooks {
                die: host_die,
                warning: host_warning,
                apu_lock: host_apu_lock,
                apu_unlock: host_apu_unlock,
            });
            stderr = __acrt_iob_func(2);
            let owner = sm::verify_room_world_owner(id);
            owner.with(|game| -> Result<(), String> {
                game.VERIFY_ACTIVE = true;
                if sm::cpu_infra::SnesInit(rom, game).is_null() {
                    return Err("unable to initialize local original ROM".into());
                }
                initialize_verify_audio(game)?;
                // Match the actual headless host: reset preserves these renderer targets.
                let snes = game.g_snes;
                sm::snes::ppu::PpuBeginDrawing(
                    (*snes).snes_ppu.cast(),
                    (&raw mut g_pixels).cast(),
                    1024,
                    0,
                );
                sm::snes::ppu::PpuBeginDrawing(
                    (*snes).my_ppu.cast(),
                    (&raw mut g_my_pixels).cast(),
                    1024,
                    0,
                );
                Ok(())
            })?;
            sm::verify_room_world_case(id, &owner)
                .ok_or_else(|| "room world assertions failed".to_owned())
        }));
        match result {
            Ok(Ok(bytes)) if bytes.len() <= capacity => {
                std::ptr::copy_nonoverlapping(bytes.as_ptr(), output, bytes.len());
                bytes.len()
            }
            Ok(Err(error)) => {
                eprintln!("Room world: {error}");
                0
            }
            _ => {
                eprintln!("Room world capture failed");
                0
            }
        }
    }
}
