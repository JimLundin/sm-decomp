//! Screen fade admission and the cinematic's actual raw brightness boundary.
#![forbid(unsafe_code)]
use super::Runtime;
use crate::game_state::Game;
use sm_game::screen_fade::{CinematicFade, Owner, ScreenFade};

impl Owner for Game {
    fn with_screen_fade<R>(&mut self, action: impl FnOnce(&mut ScreenFade) -> R) -> R {
        if self.session_lease_admissible() {
            return Runtime::new(self).with_screen_fade(action);
        }
        // Failed/stale/suspended Session admission must not poison another
        // owner or acquire a rejected lease. The selected projection owns this
        // compatibility interval; import its actual words and publish changes.
        let before = ScreenFade {
            delay: sm_game::projection::word(self, 0x723),
            counter: sm_game::projection::word(self, 0x725),
        };
        let mut screen = before;
        let result = action(&mut screen);
        for (at, old, new) in [
            (0x723, before.delay, screen.delay),
            (0x725, before.counter, screen.counter),
        ] {
            if new != old {
                for (offset, (old, new)) in old
                    .to_le_bytes()
                    .into_iter()
                    .zip(new.to_le_bytes())
                    .enumerate()
                {
                    if new != old {
                        sm_game::projection::set_byte(self, at + offset, new);
                    }
                }
            }
        }
        result
    }

    fn tick_screen_fade(&mut self) -> bool {
        if self.session_lease_admissible() {
            return Runtime::new(self).tick_screen_fade();
        }
        let (counter, due) =
            sm_game::screen_fade::countdown(sm_game::projection::word(self, 0x725), || {
                sm_game::projection::word(self, 0x723)
            });
        sm_game::projection::set_word(self, 0x725, counter);
        due
    }

    fn cinematic_screen_fade(&mut self, fade: CinematicFade) -> bool {
        if self.session_lease_admissible() {
            return Runtime::new(self).advance_owned_screen_fade(fade);
        }
        // This rejected interval belongs to the selected byte image. Keep
        // every original low-byte store, including redundant final zeroes.
        let slow = matches!(fade, CinematicFade::SlowIn | CinematicFade::SlowOut);
        if slow {
            let (delay, ready) =
                sm_game::screen_fade::cinematic_countdown(sm_game::projection::word(self, 0x723));
            sm_game::projection::set_byte(self, 0x723, delay.to_le_bytes()[0]);
            if !ready {
                return false;
            }
        }
        let brightness = self.ram().read_byte(0x51);
        if matches!(fade, CinematicFade::FastOut | CinematicFade::SlowOut) && brightness & 15 == 0 {
            return true;
        }
        let step = if slow {
            1
        } else {
            sm_game::projection::word(self, 0x725).to_le_bytes()[0]
        };
        let (next, complete) = sm_game::screen_fade::cinematic_brightness(fade, brightness, step)
            .expect("nonzero outgoing brightness");
        self.ram().write_byte(0x51, next);
        if slow {
            let delay = if complete {
                0
            } else {
                sm_game::projection::word(self, 0x725).to_le_bytes()[0]
            };
            sm_game::projection::set_byte(self, 0x723, delay);
            if complete {
                sm_game::projection::set_byte(self, 0x725, 0);
            }
        }
        complete
    }

    fn set_screen_fade(&mut self, delay: u16, counter: u16) {
        if self.session_lease_admissible() {
            Runtime::new(self).set_screen_fade(delay, counter);
        } else {
            // Preserve both unconditional word stores and their byte order.
            sm_game::projection::set_word(self, 0x723, delay);
            sm_game::projection::set_word(self, 0x725, counter);
        }
    }

    fn set_screen_fade_counter(&mut self, counter: u16) {
        if self.session_lease_admissible() {
            Runtime::new(self).set_screen_fade_counter(counter);
        } else {
            sm_game::projection::set_word(self, 0x725, counter);
        }
    }
}

impl Owner for Runtime<'_> {
    fn with_screen_fade<R>(&mut self, action: impl FnOnce(&mut ScreenFade) -> R) -> R {
        if !self.persistent_is_current() {
            return self.game.with_screen_fade(action);
        }
        action(&mut self.session_mut().presentation.visual_mut().screen_fade)
    }

    fn tick_screen_fade(&mut self) -> bool {
        if !self.persistent_is_current() {
            return self.game.tick_screen_fade();
        }
        self.with_screen_fade(|screen| screen.tick())
    }

    fn set_screen_fade(&mut self, delay: u16, counter: u16) {
        if !self.persistent_is_current() {
            self.game.set_screen_fade(delay, counter);
            return;
        }
        self.with_screen_fade(|screen| *screen = ScreenFade { delay, counter });
    }

    fn set_screen_fade_counter(&mut self, counter: u16) {
        if !self.persistent_is_current() {
            self.game.set_screen_fade_counter(counter);
            return;
        }
        self.with_screen_fade(|screen| screen.counter = counter);
    }

    fn cinematic_screen_fade(&mut self, fade: CinematicFade) -> bool {
        self.advance_owned_screen_fade(fade)
    }
}

impl Runtime<'_> {
    pub(crate) fn advance_owned_screen_fade(&mut self, fade: CinematicFade) -> bool {
        if !self.persistent_is_current() {
            return self.game.cinematic_screen_fade(fade);
        }
        if !self.with_screen_fade(|screen| screen.cinematic_ready(fade)) {
            return false;
        }
        // Keep this lazy RAW read: an owned display's brightness projection is
        // a different value. No read happens while the signed delay is positive.
        let brightness = self.game.ram().read_byte(0x51);
        let Some((next, complete)) =
            self.with_screen_fade(|screen| screen.cinematic_brightness(fade, brightness))
        else {
            return true;
        };
        // This physical writer can only replace $51; it reads no Session
        // field. Preserve its RAW write before the fade's final low-byte
        // stores, then adopt that actual byte into the owned display if present.
        // No arbitrary callback or private Display reconstruction is involved.
        self.game.ram().write_byte(0x51, next);
        let state = self.state.as_mut().expect("aggregate unavailable");
        crate::game::session_encoding::write_projected_byte(
            state,
            0x51,
            self.game.ram().read_byte(0x51),
        );
        if self.persistent_epoch.is_some() {
            self.persistent_epoch = Some(self.game.compatibility_write_epoch());
        }
        self.with_screen_fade(|screen| screen.cinematic_finish(fade, complete));
        complete
    }
}
