//! Descending live-slot scheduler and shared message continuation.
use super::{
    fields::{Fields, Global},
    port::Execution,
    state::Column,
};
use crate::game::enemy_script::{EnemyScript as Script, bank_base};
use crate::game::item_pickups::MessageHandoff;
use crate::game::message_boxes;
impl Execution<'_> {
    /// Drawing and instruction effects retain the initialized native-runtime contract.
    pub(super) unsafe fn run(&mut self) -> u8 {
        let base = bank_base(0x84);
        crate::verify_probe::hit("coroutine.plm");
        let resume = self.port().byte(0x613);
        if let Some(mut handoff) = MessageHandoff::resume(resume) {
            let mut image = crate::rom::RomImage::default();
            let mut script = Script::Stop;
            if resume == 0 {
                if self.port().word(0x1c23) & 0x8000 == 0 {
                    return 0;
                }
                self.port_mut().put_word(0x1c25, 0);
                self.set_global(Global::Selected, 78);
            }
            loop {
                if !handoff.displaying() {
                    if self.global(Global::Selected) & 0x8000 != 0 {
                        self.set_global(Global::Selected, 0);
                        break;
                    }
                    let slot = usize::from(self.global(Global::Selected) >> 1);
                    if self.port().native_word(0x1c37 + slot * 2) == 0 {
                        self.set_global(
                            Global::Selected,
                            self.global(Global::Selected).wrapping_sub(2),
                        );
                        continue;
                    }
                    let pre =
                        u32::from(self.port().native_word(
                            0x1cd7 + usize::from(self.global(Global::Selected) >> 1) * 2,
                        )) | 0x840000;
                    self.pre(pre, self.global(Global::Selected));
                    let slot = usize::from(self.global(Global::Selected) >> 1);
                    let remaining = self.slot(Column::InstructionDelay, slot).wrapping_sub(1);
                    self.set_slot(Column::InstructionDelay, slot, remaining);
                    if remaining != 0 {
                        self.set_global(
                            Global::Selected,
                            self.global(Global::Selected).wrapping_sub(2),
                        );
                        continue;
                    }
                    image = self.port().rom_image();
                    script = Script::at(
                        image.clone(),
                        (base
                            + self.port().native_word(
                                0x1d27 + usize::from(self.global(Global::Selected) >> 1) * 2,
                            ) as isize) as usize,
                    );
                }
                loop {
                    if handoff.displaying() {
                        crate::verify_probe::hit("coroutine.plm.message");
                        let queued = self
                            .port()
                            .word(crate::ram_map::words::queued_message_box_index);
                        let yielded =
                            self.foreign(|g| unsafe { message_boxes::display(queued, g) });
                        if let Err(yielded) = handoff.after_display(yielded) {
                            self.port_mut().put_byte(0x613, handoff.continuation());
                            return yielded;
                        }
                        crate::verify_probe::hit("coroutine.plm.message.completed");
                        self.port_mut()
                            .put_word(crate::ram_map::words::queued_message_box_index, 0);
                        image = self.port().rom_image();
                        script = Script::at(
                            image.clone(),
                            (base
                                + self.port().native_word(
                                    0x1d27 + usize::from(self.global(Global::Selected) >> 1) * 2,
                                ) as isize) as usize,
                        );
                    }
                    if script.word(0) & 0x8000 == 0 {
                        let slot = usize::from(self.global(Global::Selected) >> 1);
                        self.set_slot(Column::InstructionDelay, slot, script.word(0));
                        self.set_slot(Column::DrawProgram, slot, script.word(2));
                        self.set_slot(
                            Column::Program,
                            slot,
                            script.advance(4).offset_from(base) as u16,
                        );
                        self.process_draw(self.global(Global::Selected));
                        self.coordinates(usize::from(self.global(Global::Selected) >> 1));
                        unsafe { self.draw(self.global(Global::Selected)) };
                        break;
                    }
                    let next = unsafe {
                        self.command(
                            u32::from(script.word(0)) | 0x840000,
                            script.advance(2),
                            self.global(Global::Selected),
                        )
                    };
                    let Some(next) = next.resolve(&image, base) else {
                        break;
                    };
                    script = next;
                    if handoff.after_instruction(
                        self.port()
                            .word(crate::ram_map::words::queued_message_box_index),
                    ) {
                        let address = 0x1d27 + usize::from(self.global(Global::Selected) >> 1) * 2;
                        self.port_mut()
                            .put_native_word(address, script.offset_from(base) as u16);
                    }
                }
                self.set_global(
                    Global::Selected,
                    self.global(Global::Selected).wrapping_sub(2),
                );
            }
        }
        self.port_mut().put_byte(0x613, 0);
        0
    }
}
