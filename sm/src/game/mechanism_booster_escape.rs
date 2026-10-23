//! Speed Booster escape: arm the lavaquake, start motion, then follow the player.
#![forbid(unsafe_code)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stage {
    BoosterCollected,
    PlayerFarLeft,
    AdvanceLava,
}
pub trait Context {
    fn completed(&mut self) -> bool;
    fn complete(&mut self);
    fn booster_collected(&self) -> bool;
    fn player_x(&self) -> u16;
    fn target(&self) -> u16;
    fn set_target(&mut self, value: u16);
    fn height(&self) -> u16;
    fn set_height(&mut self, value: u16);
    fn set_velocity(&mut self, value: u16);
    fn set_motion_delay(&mut self, value: u16);
    fn stop_quake(&mut self);
    fn release(&mut self, slot: usize);
    fn set_instruction_timer(&mut self, slot: usize, value: u16);
    fn advance_program_one(&mut self, slot: usize);
    fn progress(&self, slot: usize) -> u16;
    fn set_progress(&mut self, slot: usize, value: u16);
}
pub fn begin(c: &mut impl Context, slot: usize) -> u8 {
    if c.completed() {
        c.release(slot)
    }
    0
}
fn wake(c: &mut impl Context, slot: usize) {
    c.set_instruction_timer(slot, 1);
    c.advance_program_one(slot);
    c.advance_program_one(slot);
    c.set_progress(slot, 0);
}
pub fn poll(c: &mut impl Context, stage: Stage, slot: usize) {
    match stage {
        Stage::BoosterCollected => {
            if c.booster_collected() {
                if c.target() & 0x8000 != 0 {
                    c.release(slot)
                } else {
                    c.set_velocity(0xff80);
                    wake(c, slot)
                }
            } else {
                c.set_target(0xffff);
                c.set_velocity(0);
                c.set_motion_delay(0);
                c.stop_quake();
                c.release(slot)
            }
        }
        Stage::PlayerFarLeft => {
            if c.player_x() <= 0xae0 {
                c.set_motion_delay(1);
                wake(c, slot)
            }
        }
        Stage::AdvanceLava => {
            // The encoded progress also admits odd and non-row-aligned table windows.
            const STEPS: [u16; 10] = [
                0x72b, 0x1bf, 0xff50, 0x50a, 0x167, 0xff20, 0x244, 0x100, 0xff20, 0x8000,
            ];
            let progress = c.progress(slot);
            let index = usize::from(progress >> 1);
            let threshold = STEPS[index];
            if threshold & 0x8000 != 0 {
                c.complete()
            } else if threshold >= c.player_x() {
                if STEPS[index + 1] < c.height() {
                    c.set_height(STEPS[index + 1])
                }
                c.set_velocity(STEPS[index + 2]);
                c.set_progress(slot, progress.wrapping_add(6))
            }
        }
    }
}
