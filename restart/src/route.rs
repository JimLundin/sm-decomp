//! Controller policies use observations only. The supplied pad trace is retained
//! so a successful native route can be replayed without these policies.
use sm_native_host::State;

#[derive(Default)]
pub struct Route {
    pub room_steps: u64,
    room: u16,
    pub entered_ceres: bool,
    first_x: u16,
    pub controller_response: bool,
    pub escape_stage: u8,
    pub arrived: bool,
    pub escaping: bool,
    pub stationary_steps: u64,
    last_position: Option<(u16, u16, u16, u16)>,
    climb_stage: usize,
    jumping: bool,
    jump_age: u64,
    release_steps: u8,
}

impl Route {
    pub fn observe(&mut self, state: &State) {
        let position = (state.game_state, state.room, state.x, state.y);
        if self.last_position == Some(position) {
            self.stationary_steps += 1;
        } else {
            self.stationary_steps = 0;
        }
        self.last_position = Some(position);
        if state.room != self.room {
            self.room = state.room;
            self.room_steps = 0;
            self.climb_stage = 0;
            self.jumping = false;
            self.jump_age = 0;
            self.release_steps = 0;
        } else {
            self.room_steps += 1;
        }
        if state.game_state == 8 && state.area == 6 {
            if !self.entered_ceres {
                self.entered_ceres = true;
                self.first_x = state.x;
            }
            if state.x.abs_diff(self.first_x) > 8 && state.polls > 0 && state.joy1_current != 0 {
                self.controller_response = true;
            }
        }
        if self.entered_ceres {
            match (self.escape_stage, state.game_state) {
                (0, 0x20) => self.escape_stage = 1,
                (1, 0x21) => self.escape_stage = 2,
                (2, 0x22) if state.loading_game_state == 0x22 && state.timer == 0 => self.escape_stage = 3,
                _ => {}
            }
        }
        if self.escape_stage == 3 && state.game_state == 8 && state.area == 0 {
            self.arrived = true;
        }
    }

    pub fn buttons(&mut self, state: &State, ram: &[u8], scenario: &str) -> u16 {
        if scenario == "none" {
            return 0;
        }
        if !self.entered_ceres {
            // Fresh reset/menu presses with explicit released intervals.
            return if state.host_steps % 64 < 16 { 0x008 } else { 0 };
        }
        if state.game_state != 8 {
            return 0;
        }
        if scenario == "boot" {
            return 0x080;
        }
        let word = |offset: usize| u16::from_le_bytes([ram[offset], ram[offset + 1]]);
        let run = word(0x9b6).reverse_bits();
        let jump = word(0x9b4).reverse_bits();
        // Normal gameplay changes the escape timer. No state is modified here.
        if state.timer != 0 {
            self.escaping = true;
        }
        let toward = |x: u16| {
            if state.x + 3 < x {
                0x080
            } else if state.x > x + 3 {
                0x040
            } else {
                0
            }
        };
        if !self.escaping {
            return match state.room {
                0xdf45 => {
                    toward(if state.y < 150 {
                        208
                    } else if state.y < 240 {
                        128
                    } else if state.y < 350 {
                        208
                    } else if state.y < 460 {
                        128
                    } else {
                        248
                    }) | run
                }
                0xdfd7 => {
                    toward(if state.y < 190 {
                        208
                    } else if state.y < 310 {
                        48
                    } else {
                        248
                    }) | run
                }
                0xdf8d => 0x080 | run | if self.room_steps % 90 < 60 { jump } else { 0 },
                // Ceres Ridley leaves when Samus has less than 30 health.
                // Let actual contact damage reach this trigger; never alter HP.
                0xe0b5 => {
                    if word(0x9c2) >= 30 {
                        toward(128)
                    } else {
                        0
                    }
                }
                _ => 0x080 | run,
            };
        }
        if state.room == 0xdf45 {
            return self.shaft_buttons(state, ram, run, jump);
        }
        if state.room == 0xdfd7 {
            if self.climb_stage == 0 && state.y < 260 {
                self.climb_stage = 1;
            }
            if self.climb_stage == 1 && state.y < 120 {
                self.climb_stage = 2;
            }
            return toward(match self.climb_stage {
                0 => 48,
                1 => 208,
                _ => 0,
            }) | run
                | if self.room_steps % 90 < 65 { jump } else { 0 };
        }
        // A full jump can land on the roof of a closed proximity door. Clear
        // the floor step, then release so Samus reaches its actual doorway.
        if state.x < 40 && state.y < 105 {
            return 0x080;
        }
        0x040 | run | if state.x >= 40 && state.y > 125 && self.room_steps % 90 < 60 { jump } else { 0 }
    }

    fn shaft_buttons(&mut self, s: &State, ram: &[u8], run: u16, jump: u16) -> u16 {
        let word = |o| u16::from_le_bytes([ram[o], ram[o + 1]]);
        let platforms = [
            (80i32, 592i32, 0i32, 111i32),
            (128, 496, 96, 159),
            (80, 400, 0, 95),
            (128, 288, 96, 159),
            (80, 192, 0, 95),
            (128, 96, 96, 159),
        ];
        let x = i32::from(s.x);
        let feet = i32::from(s.y) + i32::from(word(0xb00));
        let radius = i32::from(word(0xafe));
        let grounded = word(0xb2e) == 0 && word(0xb2c) == 0 && word(0xb36) == 0;
        let toward = |goal: i32| {
            if x < goal - 2 {
                0x080
            } else if x > goal + 2 {
                0x040
            } else {
                0
            }
        };
        if self.climb_stage >= platforms.len() {
            return toward(128);
        }
        let (target, surface, left, right) = platforms[self.climb_stage];
        if grounded && (feet - surface).abs() <= 2 && x >= left && x <= right {
            self.climb_stage += 1;
            self.jumping = false;
            self.release_steps = 3;
            return 0;
        }
        let outside = if x < target { left - radius - 2 } else { right + radius + 2 };
        if grounded && feet >= surface && x > left - radius - 2 && x < right + radius + 2 {
            self.jumping = false;
            return toward(outside) | run;
        }
        if self.jumping && grounded && self.jump_age > 8 {
            self.jumping = false;
            self.release_steps = 3;
        }
        if self.release_steps > 0 {
            self.release_steps -= 1;
        } else if grounded && !self.jumping {
            self.jumping = true;
            self.jump_age = 0;
        }
        if self.jumping {
            self.jump_age += 1;
        }
        let horizontal = if feet < surface { target } else { outside };
        toward(horizontal) | run | if self.jumping { jump } else { 0 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn state(game_state: u16, area: u16) -> State {
        State { game_state, area, ..State::default() }
    }

    #[test]
    fn loading_flag_or_area_zero_alone_cannot_claim_ceres_escape() {
        let mut route = Route::default();
        route.observe(&State { loading_game_state: 0x22, ..state(0x22, 0) });
        route.observe(&state(8, 0));
        assert!(!route.arrived);
        assert_eq!(route.escape_stage, 0);
    }

    #[test]
    fn escape_requires_reached_ceres_and_ordered_states_then_arrival() {
        let mut route = Route::default();
        route.observe(&state(8, 6));
        route.observe(&state(0x21, 6));
        assert_eq!(route.escape_stage, 0);
        route.observe(&state(0x20, 6));
        route.observe(&state(0x21, 6));
        route.observe(&State { loading_game_state: 0x22, timer: 1, ..state(0x22, 6) });
        assert_eq!(route.escape_stage, 2);
        route.observe(&State { loading_game_state: 0x22, ..state(0x22, 6) });
        assert_eq!(route.escape_stage, 3);
        assert!(!route.arrived);
        route.observe(&state(8, 0));
        assert!(route.arrived);
    }

    #[test]
    fn controller_response_requires_movement_with_observed_input() {
        let mut route = Route::default();
        route.observe(&State { x: 128, ..state(8, 6) });
        route.observe(&State { x: 145, polls: 1, ..state(8, 6) });
        assert!(!route.controller_response);
        route.observe(&State { x: 145, polls: 2, joy1_current: 0x100, ..state(8, 6) });
        assert!(route.controller_response);
    }
}
