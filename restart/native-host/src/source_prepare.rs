//! Exact dependency-free source preparation shared by production and diagnostics.
//! Caller authenticates and privately copies the recovered source first. This
//! module applies source patches only; it grants no library or gameplay verdict.
use super::{PatchRecord, replace_once};
use std::{fs, path::Path};
const SHIM: &str = include_str!("shim.c");
const OBSERVE: &str = include_str!("observe.h");
const PILOT: &str = include_str!("pilot.c");
const SCROLL: &str = include_str!("scroll.c");
const CERES_DOOR: &str = include_str!("ceres_door.c");
const ROOM_TRANSITION: &str = include_str!("room_transition.c");
const DOOR_SCROLLING: &str = include_str!("door_scrolling.c");
const CAMERA: &str = include_str!("camera.c");
const ITEM_INSTRUCTION: &str = include_str!("item_instruction.c");
#[path = "camera_prepare.rs"]
mod camera_prepare;
#[path = "door_scrolling_prepare.rs"]
mod door_scrolling_prepare;
#[path = "item_instruction_prepare.rs"]
mod item_instruction_prepare;
#[path = "persistence_prepare.rs"]
mod persistence_prepare;
#[path = "player_motion_prepare.rs"]
mod player_motion_prepare;
#[path = "rio_prepare.rs"]
pub(super) mod rio_prepare;
#[path = "ripper_prepare.rs"]
pub(super) mod ripper_prepare;
#[path = "room_transition_prepare.rs"]
mod room_transition_prepare;
#[path = "waver_prepare.rs"]
pub(super) mod waver_prepare;

fn instrument(root: &Path) -> Result<Vec<PatchRecord>, String> {
    let mut patches = Vec::new();
    for (path, before, after) in [
        (
            "src/sm_80.c",
            "void ReadJoypadInputs(void) {  // 0x809459",
            "void ReadJoypadInputs(void) {  // 0x809459\n  sm_native_observe(0);",
        ),
        (
            "src/sm_82.c",
            "  CoroutineRet ret = RunOneFrameOfGameInner();",
            "  sm_native_observe(4);\n  CoroutineRet ret = RunOneFrameOfGameInner();\n  if (ret == kCoroutineNone) sm_native_observe(1);",
        ),
        (
            "src/sm_80.c",
            "void Vector_NMI(void) {  // 0x809583",
            "void Vector_NMI(void) {  // 0x809583\n  sm_native_observe(2);",
        ),
        (
            "src/sm_80.c",
            "  waiting_for_nmi = 1;\n  coroutine_completion_flags = 1;",
            "  sm_native_observe(3);\n  waiting_for_nmi = 1;\n  coroutine_completion_flags = 1;",
        ),
        (
            "src/sm_80.c",
            "CoroutineRet WaitForNMI_NoUpdate_Async(void) {\n  // Return 0 from this routine as soon as the coroutine has finished\n  if (coroutine_completion_flags) {\n    coroutine_completion_flags = 0;\n    return 0;\n  }\n  coroutine_completion_flags = 1;",
            "CoroutineRet WaitForNMI_NoUpdate_Async(void) {\n  // Return 0 from this routine as soon as the coroutine has finished\n  if (coroutine_completion_flags) {\n    coroutine_completion_flags = 0;\n    return 0;\n  }\n  sm_native_observe(3);\n  coroutine_completion_flags = 1;",
        ),
        (
            "src/sm_cpu_infra.c",
            "int RunAsmCode(uint32 pc, uint16 a, uint16 x, uint16 y, int flags) {",
            "int RunAsmCode(uint32 pc, uint16 a, uint16 x, uint16 y, int flags) {\n  sm_native_observe(5);",
        ),
        (
            "src/snes/cpu.c",
            "  uint8_t opcode = cpu_readOpcode(cpu);",
            "  sm_native_cpu_instruction();\n  uint8_t opcode = cpu_readOpcode(cpu);",
        ),
    ] {
        replace_once(root, path, before, after, &mut patches)?;
    }
    for (entry, path, name) in [
        (0, "src/sm_90.c", "FireUnchargedBeam"),
        (1, "src/sm_90.c", "HandleProjectile"),
        (2, "src/sm_93.c", "DrawPlayerExplosions2"),
        (3, "src/sm_93.c", "DrawBombAndProjectileExplosions"),
    ] {
        let before = format!("void {name}(void) {{");
        let after = format!(
            "static void NativeOriginal_{name}(void);\nvoid {name}(void) {{\n  if (!sm_native_dispatch({entry})) NativeOriginal_{name}();\n  sm_native_entry_done({entry});\n}}\nstatic void NativeOriginal_{name}(void) {{"
        );
        replace_once(root, path, &before, &after, &mut patches)?;
    }
    for (entry, name) in [
        (0, "DisplayViewablePartOfRoom"),
        (1, "UpdateScrollVarsUpdateMap"),
        (2, "CalculateLayer2PosAndScrollsWhenScrolling"),
    ] {
        let before = format!("void {name}(void) {{");
        let after = format!(
            "static void NativeOriginal_{name}(void);\nvoid {name}(void) {{\n  if (!sm_native_scroll_dispatch({entry})) NativeOriginal_{name}();\n  sm_native_scroll_entry_done({entry});\n}}\nstatic void NativeOriginal_{name}(void) {{"
        );
        replace_once(root, "src/sm_80.c", &before, &after, &mut patches)?;
    }
    for (entry, name) in [(0, "CeresDoor_Init"), (1, "CeresDoor_Main")] {
        replace_once(
            root,
            "src/sm_a0.c",
            &format!("case fn{name}: {name}(); return;"),
            &format!("case fn{name}: sm_native_ceres_door_ai({entry}, cur_enemy_index); return;"),
            &mut patches,
        )?;
    }
    for (entry, name) in [
        (2, "CeresDoor_Instr_1"),
        (3, "CeresDoor_Instr_2"),
        (4, "CeresDoor_Instr_3"),
        (5, "CeresDoor_Instr_4"),
        (6, "CeresDoor_Instr_5"),
        (7, "CeresDoor_Instr_6"),
        (8, "CeresDoor_Instr_7"),
        (9, "CeresDoor_Instr_8"),
        (10, "CeresSteam_Instr_4"),
        (11, "CeresSteam_Instr_5"),
    ] {
        replace_once(
            root,
            "src/sm_a0.c",
            &format!("case fn{name}: return {name}(k, j);"),
            &format!("case fn{name}: return sm_native_ceres_door_instr({entry}, k, j);"),
            &mut patches,
        )?;
    }
    replace_once(
        root,
        "src/sm_84.c",
        "case fnsub_84B9F1: return sub_84B9F1(j);",
        "case fnsub_84B9F1: return sm_native_ceres_door_solid_setup(j);",
        &mut patches,
    )?;
    // Instrument only this family's admitted foreign call, leaving its private
    // helpers and other actors' projectile-spawn scheduling unchanged.
    replace_once(
        root,
        "src/sm_a6.c",
        "      SpawnEprojWithRoomGfx(0xE509, v6);",
        "      sm_native_ceres_door_spawn(0xE509, v6);",
        &mut patches,
    )?;
    replace_once(
        root,
        "src/sm_80.c",
        "void NMI_ProcessVramWriteQueue(void) {",
        "static void NativeOriginal_NMI_ProcessVramWriteQueue(void);\nvoid NMI_ProcessVramWriteQueue(void) {\n  sm_native_vram_flush_begin();\n  NativeOriginal_NMI_ProcessVramWriteQueue();\n  sm_native_vram_flush_done();\n}\nstatic void NativeOriginal_NMI_ProcessVramWriteQueue(void) {",
        &mut patches,
    )?;
    replace_once(
        root,
        "src/snes/dma.c",
        "    snes_writeBBus(dma->snes, bAdr, snes_read(dma->snes, (aBank << 16) | aAdr));",
        "    snes_writeBBus(dma->snes, bAdr, snes_read(dma->snes, (aBank << 16) | aAdr));\n    sm_native_vram_dma_byte(bAdr);",
        &mut patches,
    )?;
    replace_once(
        root,
        "src/sm_84.c",
        "uint8 SpawnPLM(uint16 a) {",
        "static uint8 NativeOriginal_SpawnPLM(uint16 a);\nuint8 SpawnPLM(uint16 a) {\n  sm_native_effect(3, a, cur_block_index, 0);\n  uint8 result = NativeOriginal_SpawnPLM(a);\n  sm_native_effect(4, result, 0, 0);\n  return result;\n}\nstatic uint8 NativeOriginal_SpawnPLM(uint16 a) {",
        &mut patches,
    )?;
    replace_once(
        root,
        "src/sm_rtl.c",
        "void WriteReg(uint16 reg, uint8 value) {",
        "void WriteReg(uint16 reg, uint8 value) {\n  sm_native_effect(1, reg, value, 0);",
        &mut patches,
    )?;
    replace_once(
        root,
        "src/sm_rtl.c",
        "void RtlApuWrite(uint32 adr, uint8 val) {",
        "void RtlApuWrite(uint32 adr, uint8 val) {\n  sm_native_effect(2, adr, val, 0);",
        &mut patches,
    )?;
    replace_once(
        root,
        "src/sm_rtl.c",
        "uint8 ReadReg(uint16 reg) {\n  return snes_read(g_snes, reg);\n}",
        "uint8 ReadReg(uint16 reg) {\n  sm_native_effect(7, reg, 0, 0);\n  uint8 result = snes_read(g_snes, reg);\n  sm_native_effect(8, reg, result, 0);\n  return result;\n}",
        &mut patches,
    )?;
    replace_once(
        root,
        "src/sm_cpu_infra.c",
        "void RtlUpdateSnesPatchForBugfix() {",
        "void RtlUpdateSnesPatchForBugfix() {\n  sm_native_rom_changed();",
        &mut patches,
    )?;
    replace_once(
        root,
        "src/sm_rtl.c",
        "static uint8 g_apu_write_ent_pos, g_apu_queue_size, g_apu_time_since_empty;",
        "static uint8 g_apu_write_ent_pos, g_apu_queue_size, g_apu_time_since_empty;\nvoid sm_native_audio_queue(unsigned char *out) {\n  memcpy(out, g_apu_write_ents, 64);\n  memcpy(out + 64, &g_apu_write, 4);\n  out[68] = g_apu_write_ent_pos;\n  out[69] = g_apu_queue_size;\n  out[70] = g_apu_time_since_empty;\n}",
        &mut patches,
    )?;
    room_transition_prepare::instrument(root, &mut patches)?;
    door_scrolling_prepare::instrument(root, &mut patches)?;
    camera_prepare::instrument(root, &mut patches)?;
    item_instruction_prepare::instrument(root, &mut patches)?;
    Ok(patches)
}

pub(super) fn prepare_sources(root: &Path) -> Result<Vec<PatchRecord>, String> {
    let mut patches = instrument(root)?;
    fs::write(root.join("restart_shim.c"), SHIM).map_err(|e| e.to_string())?;
    fs::write(root.join("restart_observe.h"), OBSERVE).map_err(|e| e.to_string())?;
    fs::write(root.join("restart_pilot.c"), PILOT).map_err(|e| e.to_string())?;
    fs::write(root.join("restart_scroll.c"), SCROLL).map_err(|e| e.to_string())?;
    fs::write(root.join("restart_ceres_door.c"), CERES_DOOR).map_err(|e| e.to_string())?;
    fs::write(root.join("restart_room_transition.c"), ROOM_TRANSITION).map_err(|e| e.to_string())?;
    fs::write(root.join("restart_door_scrolling.c"), DOOR_SCROLLING).map_err(|e| e.to_string())?;
    fs::write(root.join("restart_camera.c"), CAMERA).map_err(|e| e.to_string())?;
    fs::write(root.join("restart_item_instruction.c"), ITEM_INSTRUCTION).map_err(|e| e.to_string())?;
    persistence_prepare::instrument(root, &mut patches)?;
    player_motion_prepare::instrument(root, &mut patches)?;
    waver_prepare::instrument(root, &mut patches)?;
    ripper_prepare::instrument(root, &mut patches)?;
    rio_prepare::instrument(root, &mut patches)?;
    Ok(patches)
}
