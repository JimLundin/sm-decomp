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
    const CASES: u32 = 107;
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

#[cfg(sm_verify_dll)]
static RETURNING_DIE_READ_ACTIVE: AtomicBool = AtomicBool::new(false);
#[cfg(sm_verify_dll)]
static RETURNING_DIE_READ_CALLS: std::sync::atomic::AtomicUsize =
    std::sync::atomic::AtomicUsize::new(0);

#[cfg(sm_verify_dll)]
struct ReturningDieReadScope;
#[cfg(sm_verify_dll)]
impl Drop for ReturningDieReadScope {
    fn drop(&mut self) {
        RETURNING_DIE_READ_ACTIVE.store(false, Ordering::SeqCst);
    }
}

#[cfg(sm_verify_dll)]
unsafe fn returning_die_short_read(error: *const c_char) {
    unsafe {
        if RETURNING_DIE_READ_ACTIVE.load(Ordering::SeqCst)
            && !error.is_null()
            && CStr::from_ptr(error).to_bytes() == b"fread failed\n"
        {
            RETURNING_DIE_READ_CALLS.fetch_add(1, Ordering::SeqCst);
            return;
        }
        host_die(error);
    }
}

/// One fresh process; the integrator supplies an isolated 66-byte fixture path.
/// # Safety
/// The actual initialized backend and native loader retain their existing unsafe contracts.
#[cfg(sm_verify_dll)]
unsafe fn returning_die_short_load_case(rom: &CStr, fixture: &CStr) -> Result<(), String> {
    unsafe {
        if STARTED.swap(true, Ordering::SeqCst) {
            return Err("returning-Die load: use a fresh child".into());
        }
        sm::host::install_host_hooks(sm::host::HostHooks {
            die: returning_die_short_read,
            warning: host_warning,
            apu_lock: host_apu_lock,
            apu_unlock: host_apu_unlock,
        });
        stderr = __acrt_iob_func(2);
        let mut game = Game::new();
        verify::initialize_checkpoint_machine(&mut game, &rom.to_owned())?;
        initialize_verify_audio(&mut game)?;
        verify::begin_verifier_drawing(&mut game);
        let file = sm::c_library::fopen(fixture.as_ptr(), c"w+b".as_ptr());
        if file.is_null() {
            return Err("unable to create returning-Die load fixture".into());
        }
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let header: [u32; 16] = [
                2,
                0x1020_3040,
                3,
                0x2468,
                0x1122_3344,
                0,
                275493,
                0x4567,
                0x5678,
                0x6677_8899,
                0,
                0,
                0,
                0,
                0,
                0,
            ];
            assert_eq!(
                sm::c_library::fwrite(header.as_ptr().cast(), 1, 64, file) as usize,
                64
            );
            let log = [0x31u8, 0x42];
            assert_eq!(
                sm::c_library::fwrite(log.as_ptr().cast(), 1, log.len(), file) as usize,
                2
            );
            assert_eq!(sm::c_library::ftell(file), 66);
            assert_eq!(sm::c_library::fseek(file, 0, 0), 0);
            sm::samus_game_returning_die_load_witness(&mut game, |game| {
                let _scope = ReturningDieReadScope;
                RETURNING_DIE_READ_CALLS.store(0, Ordering::SeqCst);
                assert!(!RETURNING_DIE_READ_ACTIVE.swap(true, Ordering::SeqCst));
                sm::rtl::StateRecorder_Load(file, true, game);
                assert_eq!(RETURNING_DIE_READ_CALLS.load(Ordering::SeqCst), 1);
            });
            assert!(!RETURNING_DIE_READ_ACTIVE.load(Ordering::SeqCst));
            assert_eq!(RETURNING_DIE_READ_CALLS.load(Ordering::SeqCst), 1);
        }));
        let closed = sm::c_library::fclose(file);
        assert!(!RETURNING_DIE_READ_ACTIVE.load(Ordering::SeqCst));
        if result.is_err() {
            return Err("returning-Die short-load assertions failed".into());
        }
        if closed != 0 {
            return Err("unable to close returning-Die load fixture".into());
        }
        Ok(())
    }
}
