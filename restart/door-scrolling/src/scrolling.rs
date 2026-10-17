use crate::{Host, Observation, Result, Routine, memory::*};

pub(crate) struct Scrolling<'a, H> {
    pub ram: Wram<'a>,
    pub host: &'a mut H,
}

impl<H: Host> Scrolling<'_, H> {
    pub fn call(&mut self, routine: Routine) -> Result<u8> {
        self.host.observe(Observation::Enter { routine, counter: self.ram.word(COUNTER) })?;
        let result = match routine {
            Routine::FixDoorsMovingUp => self.fix_up()?,
            Routine::ScrollingSetup => self.setup()?,
            Routine::FollowTransition => self.follow()?,
            Routine::SetupRight => self.setup_right()?,
            Routine::SetupLeft => self.setup_left()?,
            Routine::SetupDown => self.setup_down()?,
            Routine::SetupUp => self.setup_up()?,
            Routine::UpdatePreviousBlocks => self.previous_blocks(),
            Routine::UpdateBgOffsets => self.bg_offsets(),
            Routine::MoveRight => self.move_right()?,
            Routine::MoveLeft => self.move_left()?,
            Routine::MoveDown => self.move_down()?,
            Routine::MoveUp => self.move_up()?,
        };
        self.host.observe(Observation::Return { routine, result, counter: self.ram.word(COUNTER) })?;
        Ok(result)
    }

    fn fix_up(&mut self) -> Result<u8> {
        self.ram.set(COUNTER, 0);
        self.host.calculate_blocks(self.ram.0)?;
        self.call(Routine::UpdatePreviousBlocks)?;
        self.ram.add(PREV_L1_Y_BLOCK, 1);
        self.ram.add(PREV_L2_Y_BLOCK, 1);
        self.call(Routine::MoveUp)?;
        Ok(0)
    }

    fn setup(&mut self) -> Result<u8> {
        self.ram.set(L1_X, self.ram.word(DEST_X));
        self.ram.set(L1_Y, self.ram.word(DEST_Y));
        self.call(match self.ram.word(DIRECTION) & 3 {
            0 => Routine::SetupRight,
            1 => Routine::SetupLeft,
            2 => Routine::SetupDown,
            _ => Routine::SetupUp,
        })?;
        Ok(0)
    }

    fn setup_right(&mut self) -> Result<u8> {
        self.host.calculate_layer2_x(self.ram.0)?;
        self.ram.sub(L2_X, 256);
        self.host.calculate_layer2_y(self.ram.0)?;
        self.ram.sub(L1_X, 256);
        self.call(Routine::UpdateBgOffsets)?;
        self.host.calculate_blocks(self.ram.0)?;
        self.call(Routine::UpdatePreviousBlocks)?;
        self.ram.sub(PREV_L1_X_BLOCK, 1);
        self.ram.sub(PREV_L2_X_BLOCK, 1);
        self.call(Routine::MoveRight)?;
        Ok(0)
    }

    fn setup_left(&mut self) -> Result<u8> {
        self.host.calculate_layer2_x(self.ram.0)?;
        self.ram.add(L2_X, 256);
        self.host.calculate_layer2_y(self.ram.0)?;
        self.ram.add(L1_X, 256);
        self.call(Routine::UpdateBgOffsets)?;
        self.host.calculate_blocks(self.ram.0)?;
        self.call(Routine::UpdatePreviousBlocks)?;
        self.ram.add(PREV_L1_X_BLOCK, 1);
        self.ram.add(PREV_L2_X_BLOCK, 1);
        self.call(Routine::MoveLeft)?;
        Ok(0)
    }

    fn setup_down(&mut self) -> Result<u8> {
        self.host.calculate_layer2_x(self.ram.0)?;
        self.host.calculate_layer2_y(self.ram.0)?;
        self.ram.sub(L2_Y, 224);
        self.ram.sub(L1_Y, 224);
        self.call(Routine::UpdateBgOffsets)?;
        self.host.calculate_blocks(self.ram.0)?;
        self.call(Routine::UpdatePreviousBlocks)?;
        self.ram.sub(PREV_L1_Y_BLOCK, 1);
        self.ram.sub(PREV_L2_Y_BLOCK, 1);
        self.call(Routine::MoveDown)?;
        Ok(0)
    }

    fn setup_up(&mut self) -> Result<u8> {
        self.host.calculate_layer2_x(self.ram.0)?;
        let layer1_y = self.ram.word(L1_Y);
        self.ram.add(L1_Y, 31);
        self.host.calculate_layer2_y(self.ram.0)?;
        self.ram.add(L2_Y, 224);
        self.ram.set(L1_Y, layer1_y.wrapping_add(256));
        self.call(Routine::UpdateBgOffsets)?;
        self.ram.add(DEST_Y, 32);
        self.host.calculate_blocks(self.ram.0)?;
        self.call(Routine::UpdatePreviousBlocks)?;
        self.ram.add(PREV_L1_Y_BLOCK, 1);
        self.ram.add(PREV_L2_Y_BLOCK, 1);
        self.ram.sub(L1_Y, 1);
        self.call(Routine::MoveUp)?;
        Ok(0)
    }

    fn previous_blocks(&mut self) -> u8 {
        self.ram.set(PREV_L1_X_BLOCK, self.ram.word(L1_X_BLOCK));
        self.ram.set(PREV_L2_X_BLOCK, self.ram.word(L2_X_BLOCK));
        self.ram.set(PREV_L1_Y_BLOCK, self.ram.word(L1_Y_BLOCK));
        self.ram.set(PREV_L2_Y_BLOCK, self.ram.word(L2_Y_BLOCK));
        0
    }

    fn bg_offsets(&mut self) -> u8 {
        // Both BG2 offsets use layer 1, as in the recovered source.
        self.ram.set(BG1_X_OFFSET, self.ram.word(BG1_H).wrapping_sub(self.ram.word(L1_X)));
        self.ram.set(BG1_Y_OFFSET, self.ram.word(BG1_V).wrapping_sub(self.ram.word(L1_Y)));
        self.ram.set(BG2_X_OFFSET, self.ram.word(BG2_H).wrapping_sub(self.ram.word(L1_X)));
        self.ram.set(BG2_Y_OFFSET, self.ram.word(BG2_V).wrapping_sub(self.ram.word(L1_Y)));
        0
    }

    fn follow(&mut self) -> Result<u8> {
        let completed = self.call(match self.ram.word(DIRECTION) & 3 {
            0 => Routine::MoveRight,
            1 => Routine::MoveLeft,
            2 => Routine::MoveDown,
            _ => Routine::MoveUp,
        })?;
        if completed != 0 {
            self.ram.set(L1_X, self.ram.word(DEST_X));
            self.ram.set(L1_Y, self.ram.word(DEST_Y));
            self.ram.set(COMPLETION, self.ram.word(COMPLETION) | 0x8000);
        }
        Ok(0)
    }

    fn move_samus(&mut self, position: usize, subposition: usize, previous: usize, subtract: bool) {
        let pair = (u32::from(self.ram.word(position)) << 16) | u32::from(self.ram.word(subposition));
        let speed = (u32::from(self.ram.word(SPEED)) << 16) | u32::from(self.ram.word(SUBSPEED));
        let moved = if subtract { pair.wrapping_sub(speed) } else { pair.wrapping_add(speed) };
        self.ram.set(position, (moved >> 16) as u16);
        self.ram.set(subposition, moved as u16);
        self.ram.set(previous, self.ram.word(position));
    }

    fn move_right(&mut self) -> Result<u8> {
        let counter = self.ram.word(COUNTER);
        self.move_samus(SAMUS_X, SAMUS_SUB_X, SAMUS_PREV_X, false);
        self.ram.add(L1_X, 4);
        self.ram.add(L2_X, 4);
        self.host.update_scroll_vars(self.ram.0)?;
        self.ram.set(COUNTER, counter.wrapping_add(1));
        if counter != 63 {
            return Ok(0);
        }
        self.host.update_scroll_vars(self.ram.0)?;
        Ok(1)
    }

    fn move_left(&mut self) -> Result<u8> {
        let counter = self.ram.word(COUNTER);
        self.move_samus(SAMUS_X, SAMUS_SUB_X, SAMUS_PREV_X, true);
        self.ram.sub(L1_X, 4);
        self.ram.sub(L2_X, 4);
        self.host.update_scroll_vars(self.ram.0)?;
        self.ram.set(COUNTER, counter.wrapping_add(1));
        Ok(u8::from(counter == 63))
    }

    fn move_down(&mut self) -> Result<u8> {
        let counter = self.ram.word(COUNTER);
        if counter != 0 {
            if counter < 0x39 {
                self.move_samus(SAMUS_Y, SAMUS_SUB_Y, SAMUS_PREV_Y, false);
                self.ram.add(L1_Y, 4);
                self.ram.add(L2_Y, 4);
                self.host.update_scroll_vars(self.ram.0)?;
            }
        } else {
            let bg1_y = self.ram.word(BG1_V);
            let bg2_y = self.ram.word(BG2_V);
            let layer1_y = self.ram.word(L1_Y);
            self.ram.sub(L1_Y, 15);
            let layer2_y = self.ram.word(L2_Y);
            self.ram.sub(L2_Y, 15);
            self.host.calculate_blocks(self.ram.0)?;
            self.call(Routine::UpdatePreviousBlocks)?;
            self.ram.sub(PREV_L1_Y_BLOCK, 1);
            self.ram.sub(PREV_L2_Y_BLOCK, 1);
            self.host.update_scroll_vars(self.ram.0)?;
            // Only these four saved scalars are restored after child copyback.
            self.ram.set(L2_Y, layer2_y);
            self.ram.set(L1_Y, layer1_y);
            self.ram.set(BG2_V, bg2_y);
            self.ram.set(BG1_V, bg1_y);
        }
        let next = counter.wrapping_add(1);
        self.ram.set(COUNTER, next);
        if next < 0x39 {
            return Ok(0);
        }
        self.host.update_scroll_vars(self.ram.0)?;
        Ok(1)
    }

    fn move_up(&mut self) -> Result<u8> {
        let counter = self.ram.word(COUNTER);
        if counter != 0 {
            self.move_samus(SAMUS_Y, SAMUS_SUB_Y, SAMUS_PREV_Y, true);
            self.ram.sub(L1_Y, 4);
            self.ram.sub(L2_Y, 4);
            if self.ram.word(COUNTER) >= 5 {
                self.host.update_scroll_vars(self.ram.0)?;
            } else {
                self.ram.set(BG1_H, self.ram.word(BG1_X_OFFSET).wrapping_add(self.ram.word(L1_X)));
                self.ram.set(BG1_V, self.ram.word(BG1_Y_OFFSET).wrapping_add(self.ram.word(L1_Y)));
                self.ram.set(BG2_H, self.ram.word(BG2_X_OFFSET).wrapping_add(self.ram.word(L2_X)));
                self.ram.set(BG2_V, self.ram.word(BG2_Y_OFFSET).wrapping_add(self.ram.word(L2_Y)));
            }
        } else {
            let bg1_y = self.ram.word(BG1_V);
            let bg2_y = self.ram.word(BG2_V);
            let layer1_y = self.ram.word(L1_Y);
            self.ram.sub(L1_Y, 16);
            let layer2_y = self.ram.word(L2_Y);
            self.ram.sub(L2_Y, 16);
            self.host.calculate_blocks(self.ram.0)?;
            self.call(Routine::UpdatePreviousBlocks)?;
            self.ram.add(PREV_L1_Y_BLOCK, 1);
            self.ram.add(PREV_L2_Y_BLOCK, 1);
            self.host.update_scroll_vars(self.ram.0)?;
            self.ram.set(L2_Y, layer2_y);
            self.ram.set(L1_Y, layer1_y);
            self.ram.set(BG2_V, bg2_y);
            self.ram.set(BG1_V, bg1_y);
        }
        self.ram.set(COUNTER, counter.wrapping_add(1));
        Ok(u8::from(counter == 56))
    }
}
