/// Install only with the reviewed alignment-repaired SaveToSram body and stage hooks.
/// # Safety
/// rom is a valid NUL-terminated path; output is writable for capacity bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sm_verify_flags_save_candidate(
    rom: *const c_char,
    id: u32,
    output: *mut u8,
    capacity: usize,
) -> usize {
    unsafe {
        if rom.is_null() || output.is_null() || capacity < 2_097_152 {
            return 0;
        }
        if STARTED.swap(true, Ordering::SeqCst) {
            eprintln!("Flags repaired save: use a separate Python process for each capture");
            return 0;
        }
        let result = std::panic::catch_unwind(|| {
            let path = CStr::from_ptr(rom)
                .to_str()
                .map_err(|error| error.to_string())?;
            let original = std::fs::read(path).map_err(|error| error.to_string())?;
            let header = match original.len() % 0x8000 {
                0 => 0,
                0x200 => 0x200,
                _ => return Err("unsupported original ROM file length".to_owned()),
            };
            let mut game = Game::new();
            sm::host::install_host_hooks(sm::host::HostHooks {
                die: host_die,
                warning: host_warning,
                apu_lock: host_apu_lock,
                apu_unlock: host_apu_unlock,
            });
            stderr = __acrt_iob_func(2);
            game.VERIFY_ACTIVE = true;
            game.g_runmode = sm::cpu_infra::RM_THEIRS as u8;
            if sm::cpu_infra::snes_init(rom, &mut game).is_null() {
                return Err("unable to load oracle ROM".to_owned());
            }
            initialize_verify_audio(&mut game)?;
            sm::cpu_infra::flags_save_candidate::capture(id, &original[header..], &mut game)
        });
        match result {
            Ok(Ok(bytes)) if bytes.len() <= capacity => {
                std::ptr::copy_nonoverlapping(bytes.as_ptr(), output, bytes.len());
                bytes.len()
            }
            Ok(Err(error)) => {
                eprintln!("Flags repaired save: {error}");
                0
            }
            _ => {
                eprintln!("Flags repaired save: capture failed");
                0
            }
        }
    }
}
