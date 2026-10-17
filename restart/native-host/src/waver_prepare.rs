//! Instrument after player_motion_prepare, before hashing retained sources.
use super::{PatchRecord, replace_once};
use std::{fs, path::Path};
pub(crate) fn instrument(root: &Path, patches: &mut Vec<PatchRecord>) -> Result<(), String> {
    for (before, after) in [
        ("case fnWaver_Init: Waver_Init(); return;", "case fnWaver_Init: sm_native_waver_ai(0); return;"),
        ("case fnWaver_Main: Waver_Main(); return;", "case fnWaver_Main: sm_native_waver_ai(1); return;"),
        (
            "case fnWaver_Instr_1: return Waver_Instr_1(k, j);",
            "case fnWaver_Instr_1: return sm_native_waver_instruction(k, j);",
        ),
    ] {
        replace_once(root, "src/sm_a0.c", before, after, patches)?;
    }
    for (selector, name, result_type, arguments, call, observe) in [
        (0, "Enemy_MoveRight_IgnoreSlopes", "uint8", "uint16 k, int32 amt", "k, amt", "k, amt, 0"),
        (1, "SineMult8bit", "uint16", "uint16 a, uint16 varE32", "a, varE32", "0, a, varE32"),
        (2, "Enemy_MoveDown", "uint8", "uint16 k, int32 amount32", "k, amount32", "k, amount32, 0"),
    ] {
        replace_once(
            root,
            "src/sm_a0.c",
            &format!("{result_type} {name}({arguments}) {{"),
            &format!(
                "static {result_type} NativeWaverOriginal_{name}({arguments});\n\
                {result_type} {name}({arguments}) {{\n\
                int observed = sm_native_waver_service_begin({selector}, {observe});\n\
                {result_type} result = NativeWaverOriginal_{name}({call});\n\
                if (observed) sm_native_waver_service_done({selector}, result);\n\
                return result;\n}}\nstatic {result_type} NativeWaverOriginal_{name}({arguments}) {{"
            ),
            patches,
        )?;
    }
    replace_once(
        root,
        "restart_observe.h",
        "#endif",
        concat!(
            "unsigned sm_native_waver_active(void);\n",
            "void sm_native_waver_ai(unsigned entry);\n",
            "const uint16_t *sm_native_waver_instruction(uint16_t k, const uint16_t *jp);\n",
            "int sm_native_waver_service_begin(unsigned selector, uint16_t k, int32_t argument, uint16_t auxiliary);\n",
            "void sm_native_waver_service_done(unsigned selector, uint16_t result);\n#endif"
        ),
        patches,
    )?;
    // Extend all existing guard sites, including generic WRAM commit, setters,
    // services and observers. This preserves each sibling's admitted children.
    for path in [
        "restart_pilot.c",
        "restart_scroll.c",
        "restart_ceres_door.c",
        "restart_room_transition.c",
        "restart_door_scrolling.c",
        "restart_camera.c",
        "restart_item_instruction.c",
        "restart_persistence.c",
    ] {
        let source = fs::read_to_string(root.join(path)).map_err(|e| e.to_string())?;
        let needle = "sm_native_player_motion_active()";
        if !source.contains(needle) {
            return Err(format!("missing post-player-motion fence in {path}"));
        }
        let replacement = source.replace(needle, "(sm_native_player_motion_active() || sm_native_waver_active())");
        replace_once(root, path, &source, &replacement, patches)?;
    }
    replace_once(
        root,
        "restart_player_motion.c",
        "static bool SiblingActive(void) {\n  return ",
        "static bool SiblingActive(void) {\n  return sm_native_waver_active() || ",
        patches,
    )?;
    for (before, after) in [
        ("static uint64_t effect_kind_counts[60];", "static uint64_t effect_kind_counts[64];"),
        ("if (kind < 60) effect_kind_counts[kind]++;", "if (kind < 64) effect_kind_counts[kind]++;"),
        ("for (unsigned kind = 1; kind < 60; kind++)", "for (unsigned kind = 1; kind < 64; kind++)"),
    ] {
        replace_once(root, "restart_pilot.c", before, after, patches)?;
    }
    fs::write(root.join("restart_waver.c"), include_str!("waver.c")).map_err(|e| e.to_string())?;
    Ok(())
}
