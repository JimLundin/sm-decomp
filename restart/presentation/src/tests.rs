use super::*;

#[test]
fn rejects_invalid_dimensions_before_loading_sdl() {
    for (width, height, scale) in [(0, 224, 3), (256, 0, 3), (256, 224, 0), (u32::MAX, 2, 2)] {
        assert!(Window::new(width, height, scale, Some(Path::new("missing"))).is_err());
    }
}

#[test]
fn failed_library_load_releases_single_window_reservation() {
    // Integration smoke is opt-in in a separate process; ordinary tests never
    // initialize SDL or modify global driver environment variables.
    for _ in 0..2 {
        let error = Window::new(256, 224, 3, Some(Path::new("/missing/sdl2.so"))).err().unwrap().to_string();
        assert!(error.contains("load SDL2"), "{error}");
    }
}

#[test]
#[ignore = "requires SDL2 and SDL_VIDEODRIVER=dummy SDL_AUDIODRIVER=dummy"]
fn real_sdl_pixels_events_audio_and_cleanup() {
    let path = std::env::var_os("SM_RESTART_SDL2").expect("set SM_RESTART_SDL2");
    let path = Path::new(&path);
    let mut window = Window::new(2, 2, 1, Some(path)).unwrap();
    assert!(Window::new(2, 2, 1, Some(path)).is_err());
    window.poll().unwrap(); // Drain creation events before injecting controls.

    // SAFETY: the test loads the same trusted SDL2 runtime and uses published
    // signatures to inject real events, inspect renderer output, pause its
    // audio device, and observe whether SDL subsystems were actually released.
    unsafe {
        let library = libloading::Library::new(path).unwrap();
        let push = library.get::<unsafe extern "C" fn(*mut sdl::Event) -> i32>(b"SDL_PushEvent\0").unwrap();
        let read = library
            .get::<unsafe extern "C" fn(*mut c_void, *const c_void, u32, *mut c_void, i32) -> i32>(
                b"SDL_RenderReadPixels\0",
            )
            .unwrap();
        let was_init = library.get::<unsafe extern "C" fn(u32) -> u32>(b"SDL_WasInit\0").unwrap();

        let key_event = |kind, scancode| sdl::Event {
            key: sdl::KeyboardEvent {
                kind,
                timestamp: 0,
                window_id: 0,
                state: u8::from(kind == sdl::KEY_DOWN),
                repeat: 0,
                padding: [0; 2],
                keysym: sdl::Keysym { scancode, symbol: 0, modifiers: 0, unused: 0 },
            },
        };
        for (scancode, bit) in
            [(6, 0), (7, 1), (225, 2), (40, 3), (82, 4), (81, 5), (80, 6), (79, 7), (29, 8), (27, 9), (4, 10), (22, 11)]
        {
            assert_eq!(push(&mut key_event(sdl::KEY_DOWN, scancode)), 1);
            assert_eq!(window.poll().unwrap(), Some(1 << bit));
            assert_eq!(window.poll().unwrap(), Some(1 << bit), "held keys persist");
            assert_eq!(push(&mut key_event(sdl::KEY_UP, scancode)), 1);
            assert_eq!(window.poll().unwrap(), Some(0));
        }
        push(&mut key_event(sdl::KEY_DOWN, 225));
        push(&mut key_event(sdl::KEY_DOWN, 229));
        push(&mut key_event(sdl::KEY_UP, 225));
        assert_eq!(window.poll().unwrap(), Some(1 << 2), "other Shift remains held");
        let mut focus = sdl::Event {
            window: sdl::WindowEvent {
                kind: sdl::WINDOW_EVENT,
                timestamp: 0,
                window_id: 0,
                event: sdl::FOCUS_LOST,
                padding: [0; 3],
                data1: 0,
                data2: 0,
            },
        };
        assert_eq!(push(&mut focus), 1);
        assert_eq!(window.poll().unwrap(), Some(0));

        let colors = [0x00ff0000, 0x0000ff00, 0x000000ff, 0x00123456];
        (window.api.pause_audio)(window.audio, 1);
        window.present(&colors, &[123, -123, 456, -456]).unwrap();
        let mut rendered = [0u32; 4];
        assert_eq!(read(window.renderer, ptr::null(), sdl::ARGB8888, rendered.as_mut_ptr().cast(), 8), 0);
        assert_eq!(rendered.map(|pixel| pixel & 0xffffff), colors, "RGB channels and zero alpha survive upload");
        assert_eq!(window.queued_audio_bytes(), 8);
        window.present(&colors, &vec![0; MAX_QUEUED_AUDIO_BYTES as usize / 2]).unwrap();
        assert_eq!(window.queued_audio_bytes(), MAX_QUEUED_AUDIO_BYTES);
        assert_eq!(window.audio_resets(), 1);
        let queued = window.queued_audio_bytes();
        assert!(window.present(&colors[..3], &[]).is_err());
        assert!(window.present(&colors, &[0]).is_err());
        assert!(window.present(&colors, &vec![0; MAX_QUEUED_AUDIO_BYTES as usize]).is_err());
        assert_eq!(window.queued_audio_bytes(), queued, "invalid input has no audio effect");

        let mut close = sdl::Event { window: sdl::WindowEvent { event: sdl::WINDOW_CLOSE, ..focus.window } };
        assert_eq!(push(&mut close), 1);
        assert_eq!(window.poll().unwrap(), None);
        assert_eq!(window.poll().unwrap(), None);
        drop(window);
        assert_eq!(was_init(sdl::INIT), 0, "audio and video released");
        let mut reopened = Window::new(2, 2, 1, Some(path)).unwrap();
        reopened.poll().unwrap();
        assert_eq!(push(&mut key_event(sdl::KEY_DOWN, 41)), 1);
        assert_eq!(reopened.poll().unwrap(), None, "Escape quits");
    }
}
