//! Shared runtime state and explicit callbacks supplied by the frontend.
//! Install callbacks before calling game code or starting an audio thread.
use std::ffi::c_char;
use std::sync::OnceLock;

#[derive(Copy, Clone)]
pub struct HostHooks {
    pub die: unsafe fn(*const c_char),
    pub warning: unsafe fn(*const c_char),
    pub apu_lock: unsafe fn(),
    pub apu_unlock: unsafe fn(),
}

static HOOKS: OnceLock<HostHooks> = OnceLock::new();

/// Call once, before library use. The callback table stays fixed while audio runs.
pub fn install_host_hooks(hooks: HostHooks) {
    assert!(HOOKS.set(hooks).is_ok(), "host callbacks already installed");
}

fn hooks() -> &'static HostHooks {
    HOOKS
        .get()
        .expect("host callbacks must be installed before library use")
}

pub unsafe fn Die(error: *const c_char) {
    unsafe {
        (hooks().die)(error);
    }
}

pub unsafe fn Warning(error: *const c_char) {
    unsafe {
        (hooks().warning)(error);
    }
}

pub unsafe fn RtlApuLock() {
    unsafe {
        (hooks().apu_lock)();
    }
}

pub unsafe fn RtlApuUnlock() {
    unsafe {
        (hooks().apu_unlock)();
    }
}
