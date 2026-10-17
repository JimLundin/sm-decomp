//! Shared runtime state and explicit callbacks supplied by the frontend.
//! Install callbacks before calling game code or starting an audio thread.
use std::ffi::{c_char, c_int};
use std::sync::OnceLock;

use crate::spc_player::SpcPlayer;

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
    (hooks().die)(error);
}

pub unsafe fn Warning(error: *const c_char) {
    (hooks().warning)(error);
}

pub unsafe fn RtlApuLock() {
    (hooks().apu_lock)();
}

pub unsafe fn RtlApuUnlock() {
    (hooks().apu_unlock)();
}
