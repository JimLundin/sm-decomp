//! Run after Waver instrumentation, preserving its public movement wrapper.
use super::{PatchRecord, replace_once};
use std::{fs, path::Path};
pub(crate) fn instrument(root: &Path, patches: &mut Vec<PatchRecord>) -> Result<(), String> {
    for (entry, name) in
        [(0, "JetPowerRipper_Init"), (1, "JetPowerRipper_Main"), (2, "Ripper_Init"), (3, "Ripper_Main")]
    {
        replace_once(
            root,
            "src/sm_a0.c",
            &format!("case fn{name}: {name}(); return;"),
            &format!("case fn{name}: sm_native_ripper_ai({entry}); return;"),
            patches,
        )?;
    }
    // The retained inner public body is Waver's observation layer. An inactive
    // actor's Begin returns0, so either family executes original movement once.
    replace_once(
        root,
        "src/sm_a0.c",
        "uint8 Enemy_MoveRight_IgnoreSlopes(uint16 k, int32 amt) {",
        concat!(
            "static uint8 NativeRipperPrevious_Horizontal(uint16 k, int32 amt);\n",
            "uint8 Enemy_MoveRight_IgnoreSlopes(uint16 k, int32 amt) {\n",
            "int observed = sm_native_ripper_movement_begin(k, amt);\n",
            "uint8 result = NativeRipperPrevious_Horizontal(k, amt);\n",
            "if (observed) sm_native_ripper_movement_done(result);\n",
            "return result;\n}\nstatic uint8 NativeRipperPrevious_Horizontal(uint16 k, int32 amt) {"
        ),
        patches,
    )?;
    replace_once(
        root,
        "restart_observe.h",
        "#endif",
        concat!(
            "unsigned sm_native_ripper_active(void);\nvoid sm_native_ripper_ai(unsigned entry);\n",
            "int sm_native_ripper_movement_begin(uint16_t slot, int32_t amount);\n",
            "void sm_native_ripper_movement_done(uint8_t result);\n#endif"
        ),
        patches,
    )?;
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
    ] {
        let source = fs::read_to_string(root.join(path)).map_err(|e| e.to_string())?;
        let needle = "sm_native_waver_active()";
        if !source.contains(needle) {
            return Err(format!("missing post-Waver fence in {path}"));
        }
        let replacement = source.replace(needle, "(sm_native_waver_active() || sm_native_ripper_active())");
        replace_once(root, path, &source, &replacement, patches)?;
    }
    replace_once(
        root,
        "restart_waver.c",
        "static bool SiblingActive(void) {\n  return ",
        "static bool SiblingActive(void) {\n  return sm_native_ripper_active() || ",
        patches,
    )?;
    for (before, after) in [
        ("static uint64_t effect_kind_counts[64];", "static uint64_t effect_kind_counts[68];"),
        ("if (kind < 64) effect_kind_counts[kind]++;", "if (kind < 68) effect_kind_counts[kind]++;"),
        ("for (unsigned kind = 1; kind < 64; kind++)", "for (unsigned kind = 1; kind < 68; kind++)"),
    ] {
        replace_once(root, "restart_pilot.c", before, after, patches)?;
    }
    fs::write(root.join("restart_ripper.c"), include_str!("ripper.c")).map_err(|e| e.to_string())?;
    Ok(())
}
