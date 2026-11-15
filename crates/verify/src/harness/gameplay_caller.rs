//! Source capabilities for the ordinary gameplay caller, independent of output receipts.

use std::collections::BTreeSet;

use data::rom::{Rom, RomAddress};
use oracle_record::{Cut, FrameRecord, NmiKind};

/// A missing caller capability or malformed source, independent of any domain.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Error {
    ShortWord,
    NotMainWait,
    ShortReturnBank,
    Return,
    Interval,
    RomImport,
    ContentAddress,
    ShortCapability,
    DebugRoute,
    FxCapability,
    XrayState,
    InstructionCapability,
    PreinstructionCapability,
    FarCallCapability,
    ReverseDmaCapability,
    HdmaChannelCapability,
}

impl Error {
    pub(super) const fn message(self) -> &'static str {
        match self {
            Self::ShortWord => "gameplay caller short word",
            Self::NotMainWait => "gameplay caller is not the ordinary main wait",
            Self::ShortReturnBank => "gameplay caller short return bank",
            Self::Return => "gameplay caller main wait return differs",
            Self::Interval => "gameplay caller completed interval counter differs",
            Self::RomImport => "gameplay caller ROM import",
            Self::ContentAddress => "gameplay caller content address",
            Self::ShortCapability => "gameplay caller short capability",
            Self::DebugRoute => "gameplay caller normal debug route unsupported",
            Self::FxCapability => "gameplay caller FX capability unsupported",
            Self::XrayState => "gameplay caller x-ray state unsupported",
            Self::InstructionCapability => {
                "gameplay caller HDMA instruction capability unsupported"
            }
            Self::PreinstructionCapability => {
                "gameplay caller HDMA preinstruction capability unsupported"
            }
            Self::ReverseDmaCapability => {
                "gameplay caller reverse DMA write capability unsupported"
            }
            Self::HdmaChannelCapability => "gameplay caller HDMA hardware channel unsupported",
            Self::FarCallCapability => "gameplay caller HDMA far-call capability unsupported",
        }
    }
}

fn word(bytes: &[u8], at: usize) -> Result<u16, Error> {
    let value = bytes
        .get(at..at.saturating_add(2))
        .ok_or(Error::ShortWord)?;
    Ok(u16::from_le_bytes(
        value.try_into().map_err(|_| Error::ShortWord)?,
    ))
}

fn byte(bytes: &[u8], at: usize) -> Result<u8, Error> {
    bytes.get(at).copied().ok_or(Error::ShortWord)
}

/// Fixed return position beneath the wait's PHP/PHB, never a stack search.
pub(super) fn main_wait(record: &FrameRecord) -> Result<(), Error> {
    if !matches!(record.cut, Cut::NmiReturn(nmi) if nmi.kind == NmiKind::Requested)
        || record.cpu.k != 0x80
        || !matches!(record.cpu.pc, 0x8343 | 0x8346)
    {
        return Err(Error::NotMainWait);
    }
    let stack = record.cpu.s.wrapping_add(3);
    let low = word(&record.wram, usize::from(stack))?.wrapping_add(1);
    let bank = *record
        .wram
        .get(usize::from(record.cpu.s.wrapping_add(5)))
        .ok_or(Error::ShortReturnBank)?;
    if ((u32::from(bank) << 16) | u32::from(low)) != 0x82_897e {
        return Err(Error::Return);
    }
    Ok(())
}

/// The end proves completion, independently of which dispatch the start selected.
pub(super) fn completed(from: &FrameRecord, next: &FrameRecord) -> Result<(), Error> {
    main_wait(next)?;
    if next.index != from.index.wrapping_add(1)
        || word(&next.wram, 0x05b6)?.wrapping_sub(word(&from.wram, 0x05b6)?) != 1
    {
        return Err(Error::Interval);
    }
    Ok(())
}

/// Imported source content used only to validate finite callback capabilities.
pub(super) struct Caller {
    rom: Rom,
}

impl Caller {
    pub(super) fn load(image: &[u8]) -> Result<Self, Error> {
        Ok(Self {
            rom: Rom::new(image.to_vec()).map_err(|_| Error::RomImport)?,
        })
    }

    fn bytes(&self, address: RomAddress, count: usize) -> Result<&[u8], Error> {
        self.rom
            .lorom()
            .read(address, count)
            .map_err(|_| Error::ContentAddress)
    }

    fn short(&self, address: RomAddress) -> Result<u16, Error> {
        word(self.bytes(address, 2)?, 0)
    }

    fn long(&self, address: RomAddress) -> Result<RomAddress, Error> {
        let bytes = self.bytes(address, 3)?;
        Ok(RomAddress::from_long(
            bytes.try_into().map_err(|_| Error::ShortCapability)?,
        ))
    }

    /// Prove the start route from the preceding cut alone. Upstream finite
    /// capabilities preserve these debug controls until their consumption.
    pub(super) fn start(&self, from: &FrameRecord) -> Result<(), Error> {
        main_wait(from)?;
        let wram = &from.wram;
        if word(wram, 0x0998)? != 8
            || word(wram, 0x185c)? & 255 != 0
            || word(wram, 0x0091)? != 0
            || word(wram, 0x05c5)? != 0
        {
            return Err(Error::DebugRoute);
        }
        self.graph(Self::roots(wram)?, false)
    }

    /// Establish requested-NMI writes without admitting a reverse-DMA producer.
    pub(super) fn receipt_start(&self, from: &FrameRecord) -> Result<(), Error> {
        self.start(from)?;
        let wram = &from.wram;
        if byte(wram, 0x0360)? != 0 {
            return Err(Error::ReverseDmaCapability);
        }
        for slot in (0..12).step_by(2) {
            if word(wram, 0x18b4_usize.saturating_add(slot))?.to_le_bytes()[0] == 0 {
                continue;
            }
            let channel = byte(wram, 0x18c0_usize.saturating_add(slot))?;
            if !matches!(
                channel,
                0x00 | 0x10 | 0x20 | 0x30 | 0x40 | 0x50 | 0x60 | 0x70
            ) {
                return Err(Error::HdmaChannelCapability);
            }
            let pre = RomAddress::new(
                byte(wram, 0x18fc_usize.saturating_add(slot))?,
                word(wram, 0x18f0_usize.saturating_add(slot))?,
            );
            if pre.long() == 0x88_86ef {
                return Err(Error::ReverseDmaCapability);
            }
        }
        self.graph(Self::roots(wram)?, true)
    }

    fn roots(wram: &[u8]) -> Result<Vec<RomAddress>, Error> {
        let mut roots = Vec::new();
        if word(wram, 0x18b0)? & 0x8000 == 0 {
            return Ok(roots);
        }
        for slot in (0..12).step_by(2) {
            if word(wram, 0x18b4_usize.saturating_add(slot))?.to_le_bytes()[0] == 0 {
                continue;
            }
            let pre = RomAddress::new(
                byte(wram, 0x18fc_usize.saturating_add(slot))?,
                word(wram, 0x18f0_usize.saturating_add(slot))?,
            );
            validate_pre(pre)?;
            if pre.long() == 0x88_86ef && word(wram, 0x0a7a)? > 5 {
                return Err(Error::XrayState);
            }
            if pre.long() == 0x88_b3b0 && !matches!(word(wram, 0x196c)?, 0xb343 | 0xb367 | 0xb382)
                || pre.long() == 0x88_c48e
                    && !matches!(word(wram, 0x196c)?, 0xc428 | 0xc44c | 0xc458)
            {
                return Err(Error::FxCapability);
            }
            let root = RomAddress::new(
                byte(wram, 0x18c1_usize.saturating_add(slot))?,
                word(wram, 0x18cc_usize.saturating_add(slot))?,
            );
            roots.push(root);
            if wakes(pre) {
                roots.push(RomAddress::new(root.bank, root.offset.wrapping_add(2)));
            }
        }
        Ok(roots)
    }

    fn graph(&self, mut pending: Vec<RomAddress>, exclude_reverse_dma: bool) -> Result<(), Error> {
        let mut visited = BTreeSet::new();
        while let Some(address) = pending.pop() {
            if !visited.insert(address) {
                continue;
            }
            let opcode = self.short(address)?;
            let arg = RomAddress::new(address.bank, address.offset.wrapping_add(2));
            let next = |count: u16| RomAddress::new(arg.bank, arg.offset.wrapping_add(count));
            match opcode {
                0..=0x7fff | 0x864c | 0x867f => {
                    self.short(arg)?;
                    pending.push(next(2));
                }
                0x8569 | 0x8682 => {}
                0x85ec => pending.push(RomAddress::new(arg.bank, self.short(arg)?)),
                0x8570 => {
                    let pre = self.long(arg)?;
                    validate_pre(pre)?;
                    if exclude_reverse_dma && pre.long() == 0x88_86ef {
                        return Err(Error::ReverseDmaCapability);
                    }
                    pending.push(next(3));
                }
                0x85b4 => {
                    let call = self.long(arg)?;
                    validate_call(call)?;
                    if exclude_reverse_dma
                        && matches!(call.long(), 0x91_CB1C | 0x91_CB57 | 0x91_CB8E | 0x91_D0D3)
                    {
                        return Err(Error::ReverseDmaCapability);
                    }
                    pending.push(next(3));
                }
                0x8655 | 0x866a | 0x8616 | 0x8622 | 0x8637 => {
                    self.bytes(arg, 1)?;
                    pending.push(next(1));
                }
                0xe4bd | 0xe917 | 0x8584 | 0xb3a9 | 0xdb2e | 0xc467 | 0xd916 | 0xd949 | 0xb4ce
                | 0xc582 => pending.push(arg),
                0x8607 => {
                    pending.push(RomAddress::new(arg.bank, self.short(arg)?));
                    pending.push(next(2));
                }
                _ => return Err(Error::InstructionCapability),
            }
        }
        Ok(())
    }
}

fn validate_pre(address: RomAddress) -> Result<(), Error> {
    if matches!(
        address.long(),
        0x88_84b8
            | 0x88_de74
            | 0x88_de96
            | 0x88_adb2
            | 0x88_b3b0
            | 0x88_b4d5
            | 0x88_c48e
            | 0x88_e449
            | 0x88_c589
            | 0x88_d9a1
            | 0x88_b0bc
            | 0x88_da47
            | 0x88_e567
            | 0x88_db36
            | 0x88_dd43
            | 0x88_8b8f
            | 0x88_8de9
            | 0x88_8b98
            | 0x88_86ef
            | 0x91_d27f
            | 0x88_8eb2
            | 0x88_de2d
            | 0x88_de10
            | 0x88_de15
            | 0x88_ea3c
            | 0x88_eacb
            | 0x88_90df
            | 0x88_91a8
            | 0x88_e9e6
            | 0x88_858a
    ) {
        Ok(())
    } else {
        Err(Error::PreinstructionCapability)
    }
}

fn wakes(address: RomAddress) -> bool {
    matches!(
        address.long(),
        0x88_e449
            | 0x88_e567
            | 0x88_8b98
            | 0x88_8de9
            | 0x88_8eb2
            | 0x88_90df
            | 0x88_91a8
            | 0x88_e9e6
            | 0x88_ea3c
            | 0x88_eacb
    )
}

fn validate_call(address: RomAddress) -> Result<(), Error> {
    if matches!(
        address.long(),
        0x88_8b14
            | 0x88_8b32
            | 0x88_8b39
            | 0x88_8b47
            | 0x88_8b4e
            | 0x91_caf9
            | 0x91_cb1c
            | 0x91_cb57
            | 0x91_cb8e
            | 0x91_d0d3
            | 0x91_d173
            | 0x91_d1a0
            | 0x91_d2bc
    ) {
        Ok(())
    } else {
        Err(Error::FarCallCapability)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preceding_permissions_and_completed_endpoint_are_independent() {
        let Some((cache, rom)) = crate::harness::tests::environment() else {
            return;
        };
        let caller = Caller::load(&std::fs::read(rom).unwrap()).unwrap();
        let lock = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../scenarios/locks/lsnes.toml");
        let cached = crate::harness::open_cached(&lock, &cache, "chain-any-0003-DFD7").unwrap();
        let records = cached
            .records_without_pictures()
            .map(|record| record.unwrap().1)
            .collect::<Vec<_>>();
        let pair = records
            .windows(2)
            .find(|pair| caller.start(&pair[0]).is_ok() && completed(&pair[0], &pair[1]).is_ok())
            .unwrap();
        let from = &pair[0];
        let put = |record: &mut FrameRecord, at: usize, value: u16| {
            record.wram[at..at.checked_add(2).unwrap()].copy_from_slice(&value.to_le_bytes());
        };
        assert!(caller.receipt_start(from).is_ok());
        let mut dma = from.clone();
        put(&mut dma, 0x0360, 0x0100);
        assert!(caller.receipt_start(&dma).is_ok());
        put(&mut dma, 0x0360, 1);
        assert_eq!(caller.receipt_start(&dma), Err(Error::ReverseDmaCapability));
        put(&mut dma, 0x0360, 0);
        put(&mut dma, 0x18b0, 0);
        put(&mut dma, 0x18b4, 1);
        put(&mut dma, 0x18f0, 0x84b8);
        dma.wram[0x18fc] = 0x88;
        dma.wram[0x18c0] = 0x1d;
        assert_eq!(
            caller.receipt_start(&dma),
            Err(Error::HdmaChannelCapability)
        );
        dma.wram[0x18c0] = 0x10;
        assert!(caller.receipt_start(&dma).is_ok());
        put(&mut dma, 0x18f0, 0x86ef);
        assert_eq!(caller.receipt_start(&dma), Err(Error::ReverseDmaCapability));
        for at in [0x185c, 0x0091, 0x05c5] {
            let mut changed = from.clone();
            put(&mut changed, at, 1);
            assert!(caller.start(&changed).is_err());
        }
        let mut changed = from.clone();
        let stack = usize::from(changed.cpu.s.wrapping_add(3));
        put(&mut changed, stack, 0);
        assert!(main_wait(&changed).is_err());
        let mut changed = from.clone();
        put(&mut changed, 0x18b0, 0x8000);
        put(&mut changed, 0x18b4, 1);
        put(&mut changed, 0x18f0, 0);
        assert!(caller.start(&changed).is_err());
        put(&mut changed, 0x18f0, 0xb3b0);
        changed.wram[0x18fc] = 0x88;
        put(&mut changed, 0x196c, 0);
        assert_eq!(caller.start(&changed), Err(Error::FxCapability));
        put(&mut changed, 0x18b0, 0);
        assert!(caller.start(&changed).is_ok());
        let mut next = pair[1].clone();
        put(&mut next, 0x0998, 9);
        put(&mut next, 0x0091, 1);
        assert!(completed(from, &next).is_ok());
        put(&mut next, 0x05b6, word(&from.wram, 0x05b6).unwrap());
        assert!(completed(from, &next).is_err());
    }
}
