//! Install Rio after Ripper; retain existing passive movement wrappers.
use super::{PatchRecord, replace_once};
use std::{fs, path::Path};
pub(crate) fn instrument(root: &Path, patches: &mut Vec<PatchRecord>) -> Result<(), String> {
    for (before, after) in [
        ("case fnRio_Init: Rio_Init(); return;", "case fnRio_Init: sm_native_rio_ai(0); return;"),
        ("case fnRio_Main: Rio_Main(); return;", "case fnRio_Main: sm_native_rio_ai(1); return;"),
        (
            "case fnEnemyInstr_Rio_Instr_1: return EnemyInstr_Rio_Instr_1(k, j);",
            "case fnEnemyInstr_Rio_Instr_1: return sm_native_rio_instruction(k, j);",
        ),
    ] {
        replace_once(root, "src/sm_a0.c", before, after, patches)?;
    }
    replace_once(
        root,
        "restart_observe.h",
        "#endif",
        concat!(
            "unsigned sm_native_rio_active(void);\nvoid sm_native_rio_ai(unsigned entry);\n",
            "const uint16_t *sm_native_rio_instruction(uint16_t k,const uint16_t *jp);\n#endif"
        ),
        patches,
    )?;
    replace_once(
        root,
        "restart_observe.h",
        "#endif",
        concat!(
            "int sm_native_rio_service_begin(unsigned selector,uint16_t k,int32_t amount);\n",
            "void sm_native_rio_service_done(unsigned selector,uint16_t result);\n#endif"
        ),
        patches,
    )?;
    for (selector, name, result, args, call) in [
        (0, "Enemy_MoveRight_IgnoreSlopes", "uint8", "uint16 k, int32 amt", "k, amt"),
        (1, "Enemy_MoveDown", "uint8", "uint16 k, int32 amount32", "k, amount32"),
        (2, "CalculateAngleOfSamusFromEnemy", "uint16", "uint16 k", "k"),
    ] {
        let amount = match selector {
            0 => "amt",
            1 => "amount32",
            _ => "0",
        };
        replace_once(
            root,
            "src/sm_a0.c",
            &format!("{result} {name}({args}) {{"),
            &format!(
                "static {result} NativeRioPrevious_{name}({args});\n\
            {result} {name}({args}) {{\n\
            int observed = sm_native_rio_service_begin({selector}, k, {amount});\n\
            {result} result = NativeRioPrevious_{name}({call});\n\
            if (observed) sm_native_rio_service_done({selector}, result);\n\
            return result;\n}}\nstatic {result} NativeRioPrevious_{name}({args}) {{"
            ),
            patches,
        )?;
    }
    for path in [
        "restart_pilot.c",
        "restart_scroll.c",
        "restart_ceres_door.c",
        "restart_room_transition.c",
        "restart_door_scrolling.c",
        "restart_camera.c",
        "restart_item_instruction.c",
        "restart_persistence.c",
        "restart_player_motion.c",
        "restart_waver.c",
    ] {
        let source = fs::read_to_string(root.join(path)).map_err(|e| e.to_string())?;
        let needle = "sm_native_ripper_active()";
        if !source.contains(needle) {
            return Err(format!("missing post-Ripper fence in {path}"));
        }
        let replacement = source.replace(needle, "(sm_native_ripper_active() || sm_native_rio_active())");
        replace_once(root, path, &source, &replacement, patches)?;
    }
    replace_once(
        root,
        "restart_ripper.c",
        "static bool SiblingActive(void) {\n  return ",
        "static bool SiblingActive(void) {\n  return sm_native_rio_active() || ",
        patches,
    )?;
    for (before, after) in [
        ("effect_kind_counts[68]", "effect_kind_counts[72]"),
        ("if (kind < 68)", "if (kind < 72)"),
        ("kind < 68;", "kind < 72;"),
    ] {
        replace_once(root, "restart_pilot.c", before, after, patches)?;
    }
    fs::write(root.join("restart_rio.c"), include_str!("rio.c")).map_err(|e| e.to_string())?;
    Ok(())
}
