//! The session lease: owned calls that keep the Session leased, and the
//! compatibility effects that leave it, export the Session and resume.
#![forbid(unsafe_code)]
use crate::session_access::SessionAccess;
use crate::session_owners::AudioHost;
use crate::visual_state::Upload;

pub trait Lease: SessionAccess + AudioHost {
    /// Run `action` on this lease with the former foreign entry's preflight,
    /// unwind publication and refreshed input observations.
    fn owned_call<R>(&mut self, action: impl FnOnce(&mut Self) -> R) -> R;
    /// Re-read the external observations from the lease's RAM view.
    fn refresh_player_observations(&mut self);
    /// An owned call bracketed by player-observation refreshes.
    fn owned_player_call<R>(&mut self, action: impl FnOnce(&mut Self) -> R) -> R {
        self.owned_call(|lease| {
            lease.refresh_player_observations();
            let result = action(lease);
            lease.refresh_player_observations();
            result
        })
    }
    /// Whether the persistent Display owns presentation on this lease.
    fn has_owned_display(&self) -> bool;
    /// Draw Samus sprite `index` through the owned Display's body producer.
    fn draw_owned_samus(&mut self, index: u16, x: u16, y: u16);
    /// The compatibility host's native Samus sprite, export to import.
    fn native_samus_sprite(&mut self, index: u16, x: u16, y: u16);
    /// The compatibility host's delayed music request.
    fn native_music_delayed8(&mut self, track: u16);
    /// The compatibility host's Samus pose command.
    fn native_player_command(&mut self, command: u16);
    /// The compatibility host's animated-tile upload.
    fn native_upload(&mut self, request: Upload);
    /// The original unreachable-state failure, and the animated tiles' form of it.
    fn native_unreachable(&mut self);
    fn native_animation_unreachable(&mut self) -> u16;
}

/// Samus sprite `index` at (`x`, `y`): the owned Display's producer when it
/// owns presentation, otherwise the complete native interval, including null maps.
pub fn draw_samus_sprite<H: Lease>(h: &mut H, index: u16, x: u16, y: u16) {
    if h.has_owned_display() {
        h.draw_owned_samus(index, x, y);
    } else {
        h.native_samus_sprite(index, x, y);
    }
}

/// Music `track` after eight frames: an owned enqueue when the Display is owned.
pub fn music_delayed8<H: Lease>(h: &mut H, track: u16) {
    if h.has_owned_display() {
        h.owned_call(|lease| {
            crate::music_requests::enqueue(track, 8, true, lease).expect("audio enqueue");
        });
    } else {
        h.native_music_delayed8(track);
    }
}
