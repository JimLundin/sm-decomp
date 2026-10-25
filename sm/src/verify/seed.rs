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

pub(super) unsafe fn load(path: &Path, game: &mut Game) -> Result<(), String> {
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
    if r.get("seed_room")? != u64::from(u16::from_le_bytes([ram[0x79b], ram[0x79c]]))
        || r.get("seed_nmi")? != u64::from(u16::from_le_bytes([ram[0x5b6], ram[0x5b7]]))
        || r.get("seed_movie_frame")? == 0
    {
        return Err("seed register metadata does not match gameplay WRAM".into());
    }
    unsafe {
        game.g_ram.copy_from_slice(ram.as_slice());
        game.g_sram.write_bytes(0, sram.as_slice());
        let snes = &mut *game.g_snes;
        for channel in 0..8 {
            for offset in 0..11 {
                let value = r.get(&format!("dma_{channel}_{offset}"))?;
                let value = u8::try_from(value).map_err(|_| "invalid captured DMA byte")?;
                sm::snes::dma::dma_write(snes.dma, 0x4300 + channel * 16 + offset, value);
            }
        }
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
        for _ in 0..100_000 {
            let pc = u32::from((*game.g_cpu).k) << 16 | u32::from((*game.g_cpu).pc);
            if pc == 0x808348 {
                sm::rtl::FinishLoadSnesState(game);
            }
            if pc == 0x80834a || pc == 0x858142 {
                eprintln!("Imported room seed; completed recorded CPU phase at ${pc:06X}");
                return Ok(());
            }
            sm::snes::cpu::cpu_runOpcode(game.g_cpu, game);
        }
    }
    Err("seed CPU did not reach the verifier frame boundary".into())
}
