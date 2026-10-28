//! Samus NMI upload order shared by native registers and the owned tile sampler.
#![forbid(unsafe_code)]

use super::samus_body::Half;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Part {
    First,
    Second,
}
/// Descriptor/source tokens retain the host's actual captured identity. Owned
/// sources are supplied authored transfers; native sources retain bank words.
pub(crate) trait Host {
    type Error;
    type Descriptor: Copy;
    type Source;
    fn vmain(&mut self) -> Result<(), Self::Error>;
    fn enabled(&mut self, half: Half) -> Result<u8, Self::Error>;
    fn descriptor(&mut self, half: Half) -> Result<Self::Descriptor, Self::Error>;
    fn destination(&mut self, descriptor: Self::Descriptor, part: Part) -> Result<(), Self::Error>;
    fn configure_dma(&mut self) -> Result<(), Self::Error>;
    fn source(&mut self, descriptor: Self::Descriptor) -> Result<Self::Source, Self::Error>;
    fn advance_source(
        &mut self,
        descriptor: Self::Descriptor,
        source: Self::Source,
        count: u16,
    ) -> Result<Self::Source, Self::Error>;
    fn write_source(&mut self, source: &Self::Source) -> Result<(), Self::Error>;
    fn bank(&mut self, descriptor: Self::Descriptor) -> Result<(), Self::Error>;
    fn count(&mut self, descriptor: Self::Descriptor, part: Part) -> Result<u16, Self::Error>;
    fn write_count(&mut self, count: u16) -> Result<(), Self::Error>;
    fn dma(
        &mut self,
        descriptor: Self::Descriptor,
        part: Part,
        count: u16,
        source: Self::Source,
    ) -> Result<(), Self::Error>;
}
pub(crate) fn half<H: Host>(host: &mut H, descriptor: H::Descriptor) -> Result<(), H::Error> {
    host.destination(descriptor, Part::First)?;
    host.configure_dma()?;
    let source = host.source(descriptor)?;
    host.write_source(&source)?;
    host.bank(descriptor)?;
    let count = host.count(descriptor, Part::First)?;
    host.write_count(count)?;
    host.dma(descriptor, Part::First, count, source)?;

    // Destination and source are prepared even when the second count is zero.
    host.destination(descriptor, Part::Second)?;
    let source = host.source(descriptor)?;
    let first_count = host.count(descriptor, Part::First)?;
    let source = host.advance_source(descriptor, source, first_count)?;
    host.write_source(&source)?;
    if host.count(descriptor, Part::Second)? != 0 {
        let count = host.count(descriptor, Part::Second)?;
        host.write_count(count)?;
        host.dma(descriptor, Part::Second, count, source)?;
    }
    Ok(())
}
pub(crate) fn upload<H: Host>(host: &mut H) -> Result<(), H::Error> {
    host.vmain()?;
    for part in [Half::Top, Half::Bottom] {
        if host.enabled(part)? != 0 {
            let descriptor = host.descriptor(part)?;
            half(host, descriptor)?;
        }
    }
    Ok(())
}
