//! Instrument exact original codec bodies and fence the generated native seam.
//!
//! Root integration: declare this module in prepare.rs, include this file plus
//! persistence.c/persistence_hooks.rs/persistence_types.rs in recipe_hash(),
//! make `patches` mutable, then call instrument AFTER writing all restart_*.c
//! and restart_observe.h, BEFORE retained_sources and compiler argv are frozen.
//! Add restart_persistence.c to compiler argv. This function writes that file.
use super::{PatchRecord, replace_once, room_transition_prepare::structural_end};
use std::{fs, path::Path};

pub(super) fn instrument(root: &Path, patches: &mut Vec<PatchRecord>) -> Result<(), String> {
    for (entry, name, result, params, argument, call) in [
        (0, "SaveToSram", "void", "uint16 a", "a", "a"),
        (1, "LoadFromSram", "uint8", "uint16 a", "a", "a"),
        (2, "PackMapToSave", "void", "void", "0", ""),
        (3, "UnpackMapFromSave", "void", "void", "0", ""),
    ] {
        let path = "src/sm_81.c";
        let source = fs::read_to_string(root.join(path)).map_err(|e| e.to_string())?;
        let declaration = format!("{result} {name}({params}) {{");
        if source.matches(&declaration).count() != 1 {
            return Err(format!("missing or ambiguous persistence body {name}"));
        }
        let start = source.find(&declaration).unwrap();
        let open = start + declaration.len() - 1;
        let end = structural_end(&source, open, b'{', b'}')?;
        let body = &source[open + 1..end - 1];
        let local = if entry >= 2 {
            format!("if (sm_native_persistence_local({entry})) {{ NativePersistenceOriginal_{name}(); return; }}\n")
        } else {
            String::new()
        };
        let original = if entry == 1 {
            format!("output = 0x10000 | NativePersistenceOriginal_{name}({call});")
        } else {
            format!("NativePersistenceOriginal_{name}({call});")
        };
        let returned = if entry == 1 { "return (uint8)output;" } else { "" };
        let replacement = format!(
            "static {result} NativePersistenceOriginal_{name}({params});\n\
             {result} {name}({params}) {{\n{local}uint32 output = 0;\n\
             if (!sm_native_persistence_dispatch({entry}, {argument}, &output)) {{ {original} }}\n\
             sm_native_persistence_done({entry}, {argument}, output);\n{returned}\n}}\n\
             static {result} NativePersistenceOriginal_{name}({params}) {{{body}}}"
        );
        replace_once(root, path, &source[start..end], &replacement, patches)?;
    }
    replace_once(
        root,
        "src/sm_rtl.c",
        "void RtlWriteSram(void) {",
        "static void NativePersistenceOriginal_RtlWriteSram(void);\nvoid RtlWriteSram(void) {\n  int observed = sm_native_persistence_disk_begin();\n  NativePersistenceOriginal_RtlWriteSram();\n  if (observed) sm_native_persistence_disk_done();\n}\nstatic void NativePersistenceOriginal_RtlWriteSram(void) {",
        patches,
    )?;
    replace_once(
        root,
        "restart_observe.h",
        "#endif",
        concat!(
            "unsigned sm_native_persistence_active(void);\n",
            "int sm_native_persistence_dispatch(unsigned entry, uint16_t argument, uint32_t *output);\n",
            "void sm_native_persistence_done(unsigned entry, uint16_t argument, uint32_t output);\n",
            "int sm_native_persistence_local(unsigned entry);\n",
            "int sm_native_persistence_disk_begin(void);\n",
            "void sm_native_persistence_disk_done(void);\n#endif"
        ),
        patches,
    )?;
    // These are generated adapter copies, not mutations of the shared authored
    // source. Include persistence in every existing sibling admission guard.
    for path in [
        "restart_pilot.c",
        "restart_scroll.c",
        "restart_ceres_door.c",
        "restart_room_transition.c",
        "restart_door_scrolling.c",
        "restart_camera.c",
    ] {
        let source = fs::read_to_string(root.join(path)).map_err(|e| e.to_string())?;
        let needle = "sm_native_item_instruction_active()";
        if !source.contains(needle) {
            return Err(format!("missing sibling fence in {path}"));
        }
        let replacement =
            source.replace(needle, "(sm_native_item_instruction_active() || sm_native_persistence_active())");
        replace_once(root, path, &source, &replacement, patches)?;
    }
    replace_once(
        root,
        "restart_item_instruction.c",
        "static bool SiblingActive(void) {\n  return ",
        "static bool SiblingActive(void) {\n  return sm_native_persistence_active() || ",
        patches,
    )?;
    for (before, after) in [
        ("static uint64_t effect_kind_counts[52];", "static uint64_t effect_kind_counts[56];"),
        ("if (kind < 52) effect_kind_counts[kind]++;", "if (kind < 56) effect_kind_counts[kind]++;"),
        ("for (unsigned kind = 1; kind < 52; kind++)", "for (unsigned kind = 1; kind < 56; kind++)"),
        (
            "void sm_native_rom_changed(void) { rom_generation++; }",
            "void sm_native_rom_changed(void) { if (sm_native_persistence_active()) Die(\"ROM change during persistence\"); rom_generation++; }",
        ),
        (
            "void sm_native_commit_wram(const uint8_t *ram) { memcpy",
            "void sm_native_commit_wram(const uint8_t *ram) { if (sm_native_persistence_active()) Die(\"WRAM-only commit during persistence\"); memcpy",
        ),
        (
            "void sm_native_begin_step(void) {",
            "void sm_native_begin_step(void) {\n  if (sm_native_persistence_active()) Die(\"native step during persistence\");",
        ),
    ] {
        replace_once(root, "restart_pilot.c", before, after, patches)?;
    }
    fs::write(root.join("restart_persistence.c"), include_str!("persistence.c")).map_err(|e| e.to_string())?;
    Ok(())
}
