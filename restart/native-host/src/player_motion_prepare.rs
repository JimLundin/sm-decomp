//! Run after persistence_prepare::instrument and before retained-source hashing.
//! This writes restart_player_motion.c; root adds it to compiler argv and hashes
//! these four player_motion sources in the recipe. All edits have exact receipts.
use super::{PatchRecord, replace_once};
use std::{fs, path::Path};

pub(super) fn instrument(root: &Path, patches: &mut Vec<PatchRecord>) -> Result<(), String> {
    for (entry, name) in [(0, "Samus_MoveUp"), (1, "Samus_MoveDown")] {
        replace_once(
            root,
            "src/sm_90.c",
            &format!("void {name}(int32 amt) {{"),
            &format!(
                "static void NativePlayerMotionOriginal_{name}(int32 amt);\n\
             void {name}(int32 amt) {{\n\
             if (!sm_native_player_motion_dispatch({entry}, amt)) NativePlayerMotionOriginal_{name}(amt);\n\
             sm_native_player_motion_done({entry}, amt);\n}}\n\
             static void NativePlayerMotionOriginal_{name}(int32 amt) {{"
            ),
            patches,
        )?;
    }
    replace_once(
        root,
        "src/sm_94.c",
        "int32 Samus_MoveDown_NoSolidColl(int32 amt) {",
        concat!(
            "static int32 NativePlayerMotionOriginal_Terrain(int32 amt);\n",
            "int32 Samus_MoveDown_NoSolidColl(int32 amt) {\n",
            "  int observed = sm_native_player_motion_terrain_begin(amt);\n",
            "  int32 result = NativePlayerMotionOriginal_Terrain(amt);\n",
            "  if (observed) sm_native_player_motion_terrain_done(result);\n",
            "  return result;\n}\nstatic int32 NativePlayerMotionOriginal_Terrain(int32 amt) {"
        ),
        patches,
    )?;
    replace_once(
        root,
        "restart_observe.h",
        "#endif",
        concat!(
            "unsigned sm_native_player_motion_active(void);\n",
            "int sm_native_player_motion_dispatch(unsigned entry, int32_t amount);\n",
            "void sm_native_player_motion_done(unsigned entry, int32_t amount);\n",
            "int sm_native_player_motion_terrain_begin(int32_t amount);\n",
            "void sm_native_player_motion_terrain_done(int32_t result);\n#endif"
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
    ] {
        let source = fs::read_to_string(root.join(path)).map_err(|e| e.to_string())?;
        let needle = "sm_native_persistence_active()";
        if !source.contains(needle) {
            return Err(format!("missing post-persistence sibling fence in {path}"));
        }
        let replacement =
            source.replace(needle, "(sm_native_persistence_active() || sm_native_player_motion_active())");
        replace_once(root, path, &source, &replacement, patches)?;
    }
    replace_once(
        root,
        "restart_persistence.c",
        "static bool SiblingActive(void) {\n  return ",
        "static bool SiblingActive(void) {\n  return sm_native_player_motion_active() || ",
        patches,
    )?;
    for (before, after) in [
        ("static uint64_t effect_kind_counts[56];", "static uint64_t effect_kind_counts[60];"),
        ("if (kind < 56) effect_kind_counts[kind]++;", "if (kind < 60) effect_kind_counts[kind]++;"),
        ("for (unsigned kind = 1; kind < 56; kind++)", "for (unsigned kind = 1; kind < 60; kind++)"),
    ] {
        replace_once(root, "restart_pilot.c", before, after, patches)?;
    }
    fs::write(root.join("restart_player_motion.c"), include_str!("player_motion.c")).map_err(|e| e.to_string())?;
    Ok(())
}
