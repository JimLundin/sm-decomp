//! The screen brightness fade ($0723 frames per step, $0725 countdown), owned
//! by `Visuals::screen_fade`, read and stored as a typed [`ScreenFade`] copy.
#![forbid(unsafe_code)]
use crate::ram::CompatibilityRam;
use crate::runtime_access::RuntimeAccess;
use crate::session::Session;
use crate::session_cohort::Cohort;
pub use crate::visual_state::ScreenFade;

const DELAY: usize = 0x723;
const COUNTER: usize = 0x725;

impl Cohort for ScreenFade {
    fn words(&mut self, mut visit: impl FnMut(usize, &mut u16)) {
        visit(DELAY, &mut self.delay);
        visit(COUNTER, &mut self.counter);
    }
    fn session_words(session: &mut Session, visit: impl FnMut(usize, &mut u16)) {
        session.presentation.visual_mut().screen_fade.words(visit);
    }
}

/// Start the fade with `delay` frames per step and the countdown at
/// `counter`, through a typed copy.
pub fn store<H: CompatibilityRam + RuntimeAccess + ?Sized>(game: &mut H, delay: u16, counter: u16) {
    crate::session_cohort::with(game, |_, fade: &mut ScreenFade| {
        *fade = ScreenFade { delay, counter }
    });
}
