use super::hardware::Hardware;
use super::{Event, EventKind, Fault, SampleValue};
use data::timing::{ChannelUpdate, DeviceWrite, Publication, Sample};

impl Hardware {
    pub(super) fn publish(
        &mut self,
        write: &DeviceWrite,
        events: &mut Vec<Event>,
    ) -> Result<(), Fault> {
        match &write.effect {
            Publication::Multiplicand(value) => self.math.multiplicand = *value,
            Publication::Multiplier(value) => {
                if self.math.remaining != 0 {
                    return Err(Fault::MathBusy);
                }
                self.math.multiplier = *value;
                self.math.product = 0;
                self.math.remaining = 8;
                self.math.operand =
                    u16::from(*value).wrapping_shl(8) | u16::from(self.math.multiplicand);
                self.math.shift = u32::from(*value);
            }
            Publication::DividendLow(value) => {
                let [_, high] = self.math.dividend.to_le_bytes();
                self.math.dividend = u16::from_le_bytes([*value, high]);
            }
            Publication::InterruptEnable(enable) => {
                let previous = self.interrupts.enable.nmi;
                self.interrupts.enable = *enable;
                if !previous && enable.nmi && self.interrupts.nmi.physical.line {
                    self.interrupts.nmi.transition = true;
                }
                if enable.irq.vertical()
                    && !enable.irq.horizontal()
                    && self.interrupts.irq.physical.line
                {
                    self.interrupts.irq.transition = true;
                }
                if !enable.irq.vertical() && !enable.irq.horizontal() {
                    self.interrupts.irq.physical.line = false;
                    self.interrupts.irq.transition = false;
                }
                self.interrupts.locked = true;
            }
            Publication::HorizontalLow(value) => {
                self.interrupts.horizontal_target =
                    (self.interrupts.horizontal_target & 0x100) | u16::from(*value);
            }
            Publication::HorizontalHigh(value) => {
                self.interrupts.horizontal_target =
                    (self.interrupts.horizontal_target & 0xff) | u16::from(*value).wrapping_shl(8);
            }
            Publication::VerticalLow(value) => {
                self.interrupts.vertical_target =
                    (self.interrupts.vertical_target & 0x100) | u16::from(*value);
            }
            Publication::VerticalHigh(value) => {
                self.interrupts.vertical_target =
                    (self.interrupts.vertical_target & 0xff) | u16::from(*value).wrapping_shl(8);
            }
            Publication::ChannelUpdate(slot, update) => {
                self.transfers
                    .channels
                    .get_mut(slot.ordinal())
                    .ok_or(Fault::InvalidState)?
                    .update(*update);
            }
            Publication::BulkEnable(enabled) => {
                for (channel, enabled) in self.transfers.channels.iter_mut().zip(enabled) {
                    channel.bulk_enabled = *enabled;
                }
                if enabled.iter().any(|enabled| *enabled) {
                    self.transfers.pending_bulk = true;
                }
            }
            Publication::LineEnable(enabled) => {
                for (channel, enabled) in self.transfers.channels.iter_mut().zip(enabled) {
                    channel.line.enabled = *enabled;
                }
            }
            Publication::Downstream => {}
        }
        events.push(Event {
            at: self.now,
            kind: EventKind::Publication(write.clone()),
        });
        Ok(())
    }
    pub(super) fn sample_device(
        &mut self,
        sample: Sample,
        events: &mut Vec<Event>,
    ) -> Result<(), Fault> {
        let value = match sample {
            Sample::ProductLow | Sample::ProductHigh => {
                if self.math.remaining != 0 {
                    return Err(Fault::MathBusy);
                }
                let [low, high] = self.math.product.to_le_bytes();
                SampleValue::Byte(if sample == Sample::ProductLow {
                    low
                } else {
                    high
                })
            }
            Sample::ControllerLow | Sample::ControllerHigh => {
                let [word, _, _, _] = self.controller.words;
                let [low, high] = word.to_le_bytes();
                SampleValue::Byte(if sample == Sample::ControllerLow {
                    low
                } else {
                    high
                })
            }
            Sample::AcknowledgeIrq => {
                let line = self.interrupts.irq.physical.line;
                if !self.interrupts.irq.physical.hold {
                    self.interrupts.irq.physical.line = false;
                    self.interrupts.irq.transition = false;
                }
                SampleValue::Line(line)
            }
            Sample::AcknowledgeNmi => {
                let line = self.interrupts.nmi.physical.line;
                if !self.interrupts.nmi.physical.hold {
                    self.interrupts.nmi.physical.line = false;
                }
                SampleValue::Line(line)
            }
            Sample::Blanking => SampleValue::Blanking {
                vertical: self.beam.position.line >= 225,
                horizontal: self.beam.position.horizontal <= 2
                    || self.beam.position.horizontal >= 1096,
                controller: self.controller.active,
            },
        };
        events.push(Event {
            at: self.now,
            kind: EventKind::Sample(sample, value),
        });
        Ok(())
    }
}

impl super::hardware::Channel {
    const fn update(&mut self, update: ChannelUpdate) {
        match update {
            ChannelUpdate::Control {
                indirect,
                fixed,
                reverse,
                pattern,
            } => {
                self.config.indirect = indirect;
                self.config.fixed = fixed;
                self.config.reverse = reverse;
                self.config.pattern = pattern;
            }
            ChannelUpdate::Target(target) => self.config.target = target,
            ChannelUpdate::SourceBank(bank) => self.config.source_bank = bank,
            ChannelUpdate::SourceLow(value) => {
                let [_, high] = self.source_offset.to_le_bytes();
                self.source_offset = u16::from_le_bytes([value, high]);
            }
            ChannelUpdate::SourceHigh(value) => {
                let [low, _] = self.source_offset.to_le_bytes();
                self.source_offset = u16::from_le_bytes([low, value]);
            }
            ChannelUpdate::SizeLow(value) => {
                let [_, high] = self.pointer.to_le_bytes();
                self.pointer = u16::from_le_bytes([value, high]);
            }
            ChannelUpdate::SizeHigh(value) => {
                let [low, _] = self.pointer.to_le_bytes();
                self.pointer = u16::from_le_bytes([low, value]);
            }
        }
    }
}
