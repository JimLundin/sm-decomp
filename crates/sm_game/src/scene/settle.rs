//! Source beam event order for the late display interval.

/// The retained native adapter owns its backend. The portable G adapter owns
/// mutable display/IRQ/room state; neither adapter is a presentation consumer.
/// Each event commits its original prefix before a later event can fail.
pub trait SettlementHost {
    type Error;

    fn reset_beam(&mut self);
    fn nmi_wanted(&self) -> bool;
    fn advance_at(&mut self, position: u16) -> Result<(), Self::Error>;
    fn vertical_irq_enabled(&self) -> bool;
    fn vertical_position(&self) -> u16;
    fn vertical_timer(&self) -> u16;
    fn interrupt(&mut self) -> Result<(), Self::Error>;
    fn clear_nmi(&mut self);
}

/// No CPU/APU work occurs between the original four events. In particular,
/// the row's raster precedes HDMA, and IRQ follows wrap even on the final row.
/// The host owns row-start/vblank/latch effects at their actual event positions.
/// A preexisting wanted flag takes zero rows after reset; only that selected
/// wanted flag is cleared after success, leaving NMI status intact.
pub fn settle<H: SettlementHost>(host: &mut H) -> Result<(), H::Error> {
    host.reset_beam();
    while !host.nmi_wanted() {
        for position in [0, 512, 1024, 1362] {
            host.advance_at(position)?;
        }
        if host.vertical_irq_enabled()
            && i32::from(host.vertical_position()) - 1 == i32::from(host.vertical_timer())
        {
            host.interrupt()?;
        }
    }
    host.clear_nmi();
    Ok(())
}
