/// Actual corrected Rust capture, one call per Python child. ROM and output stay local.
/// # Safety
/// rom is a valid NUL-terminated path; output is writable for capacity bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sm_verify_player_transitions_rust(
    rom: *const c_char,
    id: u32,
    output: *mut u8,
    capacity: usize,
) -> usize {
    unsafe {
        if rom.is_null()
            || output.is_null()
            || capacity < sm::cpu_infra::player_transitions_rust::CAPACITY
        {
            return 0;
        }
        if STARTED.swap(true, Ordering::SeqCst) {
            eprintln!(
                "defined player transition predecessor: use a separate Python process for each capture"
            );
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
            if sm::cpu_infra::SnesInit(rom, &mut game).is_null() {
                return Err("unable to load oracle ROM".to_owned());
            }
            initialize_verify_audio(&mut game)?;
            match id {
                0..=10 => sm::cpu_infra::player_transitions_rust::capture(
                    id,
                    &original[header..],
                    &mut game,
                ),
                100..=104
                | 300..=304
                | 400..=407
                | 500..=509
                | 600..=605
                | 700..=702
                | 900..=901
                | 1000..=1001
                | 1100..=1106 => {
                    sm::player_transitions_animation::capture(id, &original[header..], &mut game)
                }
                2100..=2102 | 2200..=2202 => {
                    sm::cpu_infra::player_transitions_pose_families::capture(
                        id,
                        &original[header..],
                        &mut game,
                    )
                }
                2759 | 2763 | 3004 | 3036 | 3062 | 3086 => {
                    sm::cpu_infra::player_transitions_selected::capture(
                        id,
                        &original[header..],
                        &mut game,
                    )
                }
                3700..=3702 => sm::cpu_infra::player_transitions_composed::capture(
                    id,
                    &original[header..],
                    &mut game,
                ),
                3800..=3804 => sm::cpu_infra::player_transitions_callers::capture(
                    id,
                    &original[header..],
                    &mut game,
                ),
                3900..=3907 => {
                    sm::cpu_infra::player_settlement::capture(id, &original[header..], &mut game)
                }
                4000..=4082 => {
                    sm::cpu_infra::player_locomotion::capture(id, &original[header..], &mut game)
                }
                4100..=4105 => {
                    sm::cpu_infra::player_update_cases::capture(id, &original[header..], &mut game)
                }
                _ => Err("unknown player transition fixture".into()),
            }
        });
        match result {
            Ok(Ok(bytes)) if bytes.len() <= capacity => {
                std::ptr::copy_nonoverlapping(bytes.as_ptr(), output, bytes.len());
                bytes.len()
            }
            Ok(Err(error)) => {
                eprintln!("defined player transition predecessor: {error}");
                0
            }
            _ => {
                eprintln!("defined player transition predecessor: capture failed");
                0
            }
        }
    }
}
