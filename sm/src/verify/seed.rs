//! Import an original-emulator room observation, distinct from an exact checkpoint.
use sm::{game_state::Game, types::Ppu};
use std::{collections::BTreeMap, path::Path};

fn bytes<const N: usize>(path: &Path) -> Result<Box<[u8; N]>, String> {
    std::fs::read(path)
        .map_err(|e| format!("{}: {e}", path.display()))?
        .into_boxed_slice()
        .try_into()
        .map_err(|_| format!("{}: expected {N} bytes", path.display()))
}

fn words(output: &mut [u16], input: &[u8]) {
    for (word, pair) in output.iter_mut().zip(input.chunks_exact(2)) {
        *word = u16::from_le_bytes([pair[0], pair[1]]);
    }
}

// Original mid-frame seeds precede the patched scheduler's return stack.
// Restore its original wait JSL for one opcode; never leave the patch removed.
struct OriginalWaitCall {
    image: sm::rom::RomImage,
    offset: usize,
    patched: [u8; 4],
}

impl OriginalWaitCall {
    fn new(image: sm::rom::RomImage, original: &[u8]) -> Result<Self, String> {
        let offset = sm::rom::lorom_offset(0x82897a) as usize;
        if original.get(offset..offset + 4) != Some(&[0x22, 0x38, 0x83, 0x80])
            || image.len() < offset + 4
        {
            return Err("original seed wait-call ROM provenance mismatch".into());
        }
        let patched = core::array::from_fn(|i| image.read(offset + i));
        if patched != [0x28, 0x60, 0x83, 0x80] {
            return Err("seed wait-call scheduler patch mismatch".into());
        }
        let guard = Self {
            image,
            offset,
            patched,
        };
        for (i, byte) in original[offset..offset + 4].iter().copied().enumerate() {
            guard.image.write(offset + i, byte);
        }
        Ok(guard)
    }
}

impl Drop for OriginalWaitCall {
    fn drop(&mut self) {
        for (i, byte) in self.patched.iter().copied().enumerate() {
            self.image.write(self.offset + i, byte);
        }
    }
}

const INPUT_WORDS: [usize; 5] = [0x8b, 0x8f, 0x93, 0x97, 0xa3];

struct ControllerRead {
    before: [u16; 5],
    after: [u16; 5],
}

fn controller_read(
    r: &Registers,
    ram: &[u8],
    first_input: u16,
) -> Result<Option<ControllerRead>, String> {
    let fields =
        r.0.keys()
            .filter(|name| name.starts_with("seed_input_"))
            .count();
    if fields == 0 {
        return Ok(None);
    }
    let word = |at| u16::from_le_bytes([ram[at], ram[at + 1]]);
    let prior = |at| -> Result<u16, String> {
        u16::try_from(r.get(&format!("seed_input_{at:04X}"))?)
            .map_err(|_| "invalid captured controller word".into())
    };
    if fields != 15
        || r.get("seed_input_version")? != 1
        || r.get("seed_input_pc")? != 0x809459
        || r.get("seed_boundary_pc")? != 0x808348
        || r.get("seed_input_dp")? != 0
        || r.get("d")? != 0
        || r.get("seed_input_debug")? != 0
        || word(0x5d1) != 0
        || r.get("seed_input_frame")? > r.get("seed_boundary_frame")?
        || r.get("seed_input_event")? == 0
        || r.get("seed_input_event")? > r.get("seed_boundary_event")?
        || r.get("seed_input_room")? != r.get("seed_room")?
        || r.get("seed_input_nmi")? > u64::from(u16::MAX)
        || (r.get("seed_input_nmi")? as u16).wrapping_add(1) != word(0x5b6)
        || prior(0x87)? != word(0x87)
        || prior(0x89)? != word(0x89)
    {
        return Err("unsupported seed controller phase".into());
    }
    let mut before = [0; 5];
    for (value, at) in before.iter_mut().zip(INPUT_WORDS) {
        *value = prior(at)?;
    }
    let after = INPUT_WORDS.map(word);
    let held = after[0];
    if first_input != held {
        return Err("first input does not match the captured controller latch".into());
    }
    // Replay the original non-debug producer symbolically before restoring its
    // observed inputs. This validates new/repeat bits and both timer branches.
    let pressed = held & !before[3];
    let mut repeated = pressed;
    let timer = if held != 0 && held == before[3] {
        let next = before[4].wrapping_sub(1);
        if next == 0 {
            repeated = held;
            prior(0x89)?
        } else {
            next
        }
    } else {
        prior(0x87)?
    };
    if after != [held, pressed, repeated, held, timer] {
        return Err("captured controller read does not reproduce the seed".into());
    }
    Ok(Some(ControllerRead { before, after }))
}

struct Registers(BTreeMap<String, u64>);
impl Registers {
    fn get(&self, name: &str) -> Result<u64, String> {
        self.0
            .get(name)
            .copied()
            .ok_or_else(|| format!("seed register missing: {name}"))
    }
}

fn ppu_registers(ppu: &mut Ppu, r: &Registers) -> Result<(), String> {
    macro_rules! fields {
        ($($field:ident = $name:literal),* $(,)?) => {
            $(ppu.$field = r.get(concat!("ppu_", $name))? as _;)*
        };
    }
    macro_rules! flags {
        ($($field:ident = $name:literal),* $(,)?) => {
            $(ppu.$field = r.get(concat!("ppu_", $name))? != 0;)*
        };
    }
    fields!(
        brightness = "display_brightness",
        mode = "bg_mode",
        vramIncrement = "vram_incsize",
        vramRemapMode = "vram_mapping",
        vramReadBuffer = "vram_readbuffer",
        m7prev = "m7_latch",
        window1left = "window1_left",
        window1right = "window1_right",
        window2left = "window2_left",
        window2right = "window2_right",
        fixedColorR = "color_r",
        fixedColorG = "color_g",
        fixedColorB = "color_b",
        ppu1openBus = "ppu1_mdr",
        ppu2openBus = "ppu2_mdr"
    );
    fields!(
        vramPointer = "vram_addr",
        objSize = "oam_basesize",
        cgramBuffer = "cgram_latchdata",
        oamBuffer = "oam_latchdata",
        clipMode = "color_mask",
        preventMathMode = "colorsub_mask",
        hCount = "hcounter",
        vCount = "vcounter"
    );
    flags!(
        forcedBlank = "display_disabled",
        bg3priority = "bg3_priority",
        vramIncrementOnHigh = "vram_incmode",
        directColor = "direct_color",
        halfColor = "color_halve",
        m7xFlip = "mode7_hflip",
        m7yFlip = "mode7_vflip",
        m7extBg = "mode7_extbg",
        timeOver = "time_over",
        rangeOver = "range_over"
    );
    flags!(
        objPriority = "oam_priority",
        addSubscreen = "addsub_mode",
        subtractColor = "color_mode",
        hCountSecond = "latch_hcounter",
        vCountSecond = "latch_vcounter"
    );
    flags!(
        interlace = "interlace",
        overscan = "overscan",
        pseudoHires = "pseudo_hires",
        objInterlace = "oam_interlace",
        countersLatched = "counters_latched"
    );
    ppu.objTileAdr1 = (r.get("ppu_oam_tdaddr")? >> 1) as u16;
    ppu.objTileAdr2 = ppu
        .objTileAdr1
        .wrapping_add(((r.get("ppu_oam_nameselect")? + 1) << 12) as u16)
        & 0x7fff;
    let cgram = r.get("ppu_cgram_addr")?;
    ppu.cgramPointer = (cgram >> 1) as u8;
    ppu.cgramSecondWrite = cgram & 1 != 0;
    let oam = r.get("ppu_oam_addr")?;
    ppu.oamAdr = (oam >> 1) as u8;
    ppu.oamInHigh = oam & 0x200 != 0;
    ppu.oamSecondWrite = oam & 1 != 0;
    let oam = r.get("ppu_oam_baseaddr")?;
    ppu.oamAdrWritten = oam as u8;
    ppu.oamInHighWritten = oam & 0x100 != 0;
    ppu.mosaicSize = r.get("ppu_mosaic_size")? as u8 + 1;
    let repeat = r.get("ppu_mode7_repeat")?;
    ppu.m7largeField = repeat & 2 != 0;
    ppu.m7charFill = repeat & 1 != 0;
    for (index, name) in [
        "m7a", "m7b", "m7c", "m7d", "m7x", "m7y", "m7_hofs", "m7_vofs",
    ]
    .iter()
    .enumerate()
    {
        ppu.m7matrix[index] = r.get(&format!("ppu_{name}"))? as i16;
    }
    for (index, layer) in ppu.bgLayer.iter_mut().enumerate() {
        layer.hScroll = r.get(&format!("ppu_bg_hofs[{index}]"))? as u16;
        layer.vScroll = r.get(&format!("ppu_bg_vofs[{index}]"))? as u16;
        layer.bigTiles = r.get(&format!("ppu_bg_tilesize[{index}]"))? != 0;
        layer.mosaicEnabled = r.get(&format!("ppu_mosaic_enabled[{index}]"))? != 0;
        layer.tilemapAdr = (r.get(&format!("ppu_bg_scaddr[{index}]"))? >> 1) as u16;
        layer.tileAdr = (r.get(&format!("ppu_bg_tdaddr[{index}]"))? >> 1) as u16;
        let size = r.get(&format!("ppu_bg_scsize[{index}]"))?;
        layer.tilemapWider = size & 1 != 0;
        layer.tilemapHigher = size & 2 != 0;
    }
    for (index, layer) in ppu.layer.iter_mut().enumerate() {
        layer.mainScreenEnabled = r.get(&format!("ppu_bg_enabled[{index}]"))? != 0;
        layer.subScreenEnabled = r.get(&format!("ppu_bgsub_enabled[{index}]"))? != 0;
        layer.mainScreenWindowed = r.get(&format!("ppu_window_enabled[{index}]"))? != 0;
        layer.subScreenWindowed = r.get(&format!("ppu_sub_window_enabled[{index}]"))? != 0;
    }
    for (index, layer) in ppu.windowLayer.iter_mut().enumerate() {
        layer.window1enabled = r.get(&format!("ppu_window1_enabled[{index}]"))? != 0;
        layer.window2enabled = r.get(&format!("ppu_window2_enabled[{index}]"))? != 0;
        layer.window1inversed = r.get(&format!("ppu_window1_invert[{index}]"))? != 0;
        layer.window2inversed = r.get(&format!("ppu_window2_invert[{index}]"))? != 0;
        layer.maskLogic = r.get(&format!("ppu_window_mask[{index}]"))? as u8;
        ppu.mathEnabled[index] = r.get(&format!("ppu_color_enabled[{index}]"))? != 0;
    }
    sm::snes::ppu::PpuRestoreRenderRouting(ppu);
    Ok(())
}

pub(super) unsafe fn load(
    path: &Path,
    game: &mut Game,
    first_input: u16,
    original_rom: &[u8],
) -> Result<(), String> {
    let filename = path
        .file_name()
        .and_then(|s| s.to_str())
        .ok_or("invalid seed path")?;
    let prefix = filename
        .strip_suffix("-wram.bin")
        .ok_or("seed path must name a -wram.bin file")?;
    let region = |name: &str| path.with_file_name(format!("{prefix}-{name}"));
    let ram = bytes::<131072>(path)?;
    if u16::from_le_bytes([ram[0x998], ram[0x999]]) != 8 {
        return Err("room seed requires gameplay state 8".into());
    }
    let sram = bytes::<8192>(&region("sram.bin"))?;
    let vram = bytes::<65536>(&region("vram.bin"))?;
    let cgram = bytes::<512>(&region("cgram.bin"))?;
    let oam = bytes::<544>(&region("oam.bin"))?;
    let apuram = bytes::<65536>(&region("apuram.bin"))?;
    let text = std::fs::read_to_string(region("registers.csv")).map_err(|e| e.to_string())?;
    let mut registers = BTreeMap::new();
    if text.lines().next() != Some("name,value") {
        return Err("invalid seed registers header".into());
    }
    for line in text.lines().skip(1) {
        let (name, value) = line.split_once(',').ok_or("invalid seed register row")?;
        let value = value
            .parse::<u64>()
            .map_err(|_| "invalid seed register value")?;
        if registers.insert(name.to_owned(), value).is_some() {
            return Err("duplicate seed register".into());
        }
    }
    let r = Registers(registers);
    let controller = controller_read(&r, ram.as_slice(), first_input)?;
    let boundary_pc = r.0.get("seed_boundary_pc").copied();
    if let Some(pc) = boundary_pc {
        if pc != 0x808348
            || r.get("pbpc")? != pc + 1
            || r.get("e")? != 0
            || r.get("seed_boundary_frame")? < r.get("seed_movie_frame")?
        {
            return Err("unsupported seed opcode-fetch boundary".into());
        }
    }
    if r.get("seed_room")? != u64::from(u16::from_le_bytes([ram[0x79b], ram[0x79c]]))
        || r.get("seed_nmi")? != u64::from(u16::from_le_bytes([ram[0x5b6], ram[0x5b7]]))
        || r.get("seed_movie_frame")? == 0
    {
        return Err("seed register metadata does not match gameplay WRAM".into());
    }
    // Hold typed authority revoked through every temporary ROM guard drop.
    game.with_reference_import(|game| {
        unsafe {
            game.ram().copy_in(0, ram.as_slice());
            game.g_sram.write_bytes(0, sram.as_slice());
            let snes = game.g_snes;
            for channel in 0..8 {
                for offset in 0..11 {
                    let value = r.get(&format!("dma_{channel}_{offset}"))?;
                    let value = u8::try_from(value).map_err(|_| "invalid captured DMA byte")?;
                    game.write_dma_register(0x4300 + channel * 16 + offset, value);
                }
            }
            let snes = &mut *snes;
            for pointer in [snes.snes_ppu, snes.my_ppu] {
                let ppu = &mut *pointer;
                words(&mut ppu.vram, vram.as_slice());
                words(&mut ppu.cgram, cgram.as_slice());
                words(&mut ppu.oam, &oam[..512]);
                ppu.highOam.copy_from_slice(&oam[512..]);
                ppu_registers(ppu, &r)?;
            }
            (*snes.apu).ram.copy_from(&apuram);
            (*game.g_spc_player).ram.copy_from(&apuram);
            sm::spc_player::SpcPlayer_CopyVariablesFromRam(game.g_spc_player);
            let cpu = &mut *snes.cpu;
            cpu.a = r.get("a")? as u16;
            cpu.x = r.get("x")? as u16;
            cpu.y = r.get("y")? as u16;
            cpu.sp = r.get("s")? as u16;
            cpu.dp = r.get("d")? as u16;
            cpu.db = r.get("db")? as u8;
            cpu.pc = r.get("pc")? as u16;
            cpu.k = r.get("pb")? as u8;
            if let Some(pc) = boundary_pc {
                // The read-only exec hook runs during opcode fetch, after bsnes
                // increments PC but before PLB changes the saved DB/P stack.
                // Restart at the observed hook address, not the half-fetched PC.
                cpu.pc = pc as u16;
                cpu.k = (pc >> 16) as u8;
            }
            cpu.e = r.get("e")? != 0;
            let p = r.get("p")? as u8;
            cpu.c = p & 1 != 0;
            cpu.z = p & 2 != 0;
            cpu.i = p & 4 != 0;
            cpu.d = p & 8 != 0;
            cpu.xf = p & 16 != 0;
            cpu.mf = p & 32 != 0;
            cpu.v = p & 64 != 0;
            cpu.n = p & 128 != 0;
            cpu.irqWanted = r.get("irq")? != 0;
            cpu.waiting = r.get("wai")? != 0;
            cpu.nmiWanted = false;
            snes.hPos = r.get("hcounter")? as u16;
            snes.vPos = r.get("vcounter")? as u16;
            let nmitimen = r.get("io_4200")?;
            snes.autoJoyRead = nmitimen & 1 != 0;
            snes.hIrqEnabled = nmitimen & 0x10 != 0;
            snes.vIrqEnabled = nmitimen & 0x20 != 0;
            snes.nmiEnabled = nmitimen & 0x80 != 0;
            snes.fastMem = r.get("io_420D")? & 1 != 0;
            snes.ppuLatch = r.get("io_4201")? & 0x80 != 0;
            snes.hTimer = (r.get("io_4207")? | (r.get("io_4208")? & 1) << 8) as u16;
            snes.vTimer = (r.get("io_4209")? | (r.get("io_420A")? & 1) << 8) as u16;
            snes.openBus = r.get("mdr")? as u8;
            // Resume the observed in-flight NMI to the existing verifier boundary.
            // This imports a room observation, not an exact emulator checkpoint:
            // SPC/DSP execution phase and PPU write latches are not fully represented.
            // Route expectations must qualify each seed with normal comparisons on.
            let mut original_wait_entered = false;
            for _ in 0..100_000 {
                let pc = u32::from((*game.g_cpu).k) << 16 | u32::from((*game.g_cpu).pc);
                if original_wait_entered && pc == 0x808343 {
                    // Match RunOneFrameOfGame_Emulated's existing wait/NMI boundary.
                    (*game.g_cpu).nmiWanted = true;
                    original_wait_entered = false;
                }
                if pc == 0x808348 {
                    sm::rtl::FinishLoadSnesState(game);
                }
                if pc == 0x80834a || pc == 0x858142 {
                    if let Some(input) = &controller {
                        if pc != 0x80834a
                            || INPUT_WORDS.map(|at| {
                                u16::from_le_bytes([
                                    game.ram().read_byte(at),
                                    game.ram().read_byte(at + 1),
                                ])
                            }) != input.after
                        {
                            return Err("controller state changed during seed normalization".into());
                        }
                        for (at, value) in INPUT_WORDS.into_iter().zip(input.before) {
                            game.ram().copy_in(at, &value.to_le_bytes());
                        }
                    }
                    eprintln!("Imported room seed; completed recorded CPU phase at ${pc:06X}");
                    return Ok(());
                }
                let original_wait = if pc == 0x82897a {
                    original_wait_entered = true;
                    // The capture importer proves this normalization read equals
                    // the first replay word; preserve all 16 hardware input bits.
                    game.input1_mut().currentState = first_input.reverse_bits();
                    Some(OriginalWaitCall::new(
                        (*(*game.g_snes).cart).rom.clone(),
                        original_rom,
                    )?)
                } else {
                    None
                };
                game.step_reference_opcode();
                drop(original_wait);
            }
        }
        unsafe {
            let cpu = &*game.g_cpu;
            Err(format!(
                "seed CPU did not reach the verifier frame boundary: PC=${:02X}{:04X} SP=${:04X} wait={} NMI={}",
                cpu.k, cpu.pc, cpu.sp, cpu.waiting, cpu.nmiWanted
            ))
        }
    })
}
