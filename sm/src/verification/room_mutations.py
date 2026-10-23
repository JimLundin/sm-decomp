"""Four actual room-mechanism mutation checks; Windows GitHub CI only."""

import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile


SCHEDULER = "sm/src/game/room_mechanisms/scheduler.rs"
COMMANDS = "sm/src/game/room_mechanisms/commands.rs"
RUNTIME = "sm/src/game/room_mechanisms/runtime.rs"
ENTRY = '    h.probe("coroutine.plm");'
DISPLAY_EDGE = "                let queued = h.session().messages.queued;"
RESUMED_ENTRY = "    let mutation_resumed = matches!(pass, Pass::DisplayMessage);\n" + ENTRY
# Inject only after Runtime published/dropped the owner, before the display
# observation callback. No mutable RAM surface is introduced into gameplay.
RUNTIME_DISPLAY_EDGE = """    fn display_message(&mut self, request: u16) -> u8 {
        self.foreign(|game| unsafe {
            #[cfg(sm_verify_dll)]
            {
                crate::game::room_mechanism_checks::observe_message_display(request, game)
"""
RUNTIME_EARLY_PASS = """    fn display_message(&mut self, request: u16) -> u8 {
        self.foreign(|game| unsafe {
            game.g_ram[0x613] = 1;
            #[cfg(sm_verify_dll)]
            {
                crate::game::room_mechanism_checks::observe_message_display(request, game)
"""
# Each edit names its actual production file and must match exactly once.
# The first two faults remember entry continuation, so fresh messages do not
# accidentally exercise the resumed-pass mutation.
MUTATIONS = (
    (
        "repeat_resumed_pre", 1020, "resumed pre-action changed counter",
        (
            (SCHEDULER, ENTRY, RESUMED_ENTRY),
            (SCHEDULER, DISPLAY_EDGE, """                if mutation_resumed {
                    let selected = h.session().mechanisms.selected;
                    let identity = u32::from(mechanism!(h, (selected >> 1) as isize, pre_action)) | 0x840000;
                    pre::run(h, identity, selected);
                }
""" + DISPLAY_EDGE),
        ),
    ),
    (
        "repeat_resumed_delay", 1020, "resumed scheduler changed delay",
        (
            (SCHEDULER, ENTRY, RESUMED_ENTRY),
            (SCHEDULER, DISPLAY_EDGE, """                if mutation_resumed {
                    let slot = (h.session().mechanisms.selected >> 1) as isize;
                    let remaining = mechanism!(h, slot, instruction_delay).wrapping_sub(1);
                    mechanism!(h, slot, instruction_delay = remaining);
                }
""" + DISPLAY_EDGE),
        ),
    ),
    (
        "publish_pass_before_display", 1021, "local Pass leaked into callback publication",
        (
            (RUNTIME, RUNTIME_DISPLAY_EDGE, RUNTIME_EARLY_PASS),
        ),
    ),
    (
        "query_wrong_flag_map", 1033, "persistent query selected wrong flag map",
        (
            (COMMANDS,
             """            let found = if ea == 8685640 {
                h.room_object_used(argument)""",
             """            let found = if ea == 8685640 {
                h.item_collected(argument)"""),
        ),
    ),
)
# Synthetic observations frozen before faults were introduced. They contain no
# ROM/movie inputs. Assertion labels do not alter successful observation bytes.
EXPECTED = {
    1020: (7682778, "2a00ad7662f052c45a810bc61222e249e46faa8274462cd363a74a4d77cc2c4d", [3, 0, 0, 0]),
    1021: (7683014, "7a382ef319cad77105f4e62e1f71564fe0c439dd58bddd00a7a222109eba3a0e", [3, 0, 0, 0]),
    1033: (131076, "437d32b0e20950d0dd6fc5e2a68ba4629078666d83156af92f0075de88706dbe", [1, 52, 146, 0]),
}


def run(command, cwd, timeout):
    return subprocess.run(
        command, cwd=cwd, capture_output=True, text=True, encoding="utf-8",
        errors="replace", timeout=timeout, check=False,
    )


def capture_child(root, case):
    # A fresh child owns the DLL, catches Rust scenario assertions through the
    # existing export, and releases all native allocations before the next build.
    import ctypes
    import runpy

    verifier = runpy.run_path(str(root / "tools/verify_dll.py"))
    host = verifier["load_host"](root / "target/verify-dll/sm_verify.dll")
    verifier["check_source"](host, verifier["source_hash"](root))
    verifier["check_library"](host, root)
    function = host.sm_verify_room_mechanism_case
    function.argtypes = [ctypes.c_uint32, ctypes.c_void_p, ctypes.c_size_t]
    function.restype = ctypes.c_size_t
    output = ctypes.create_string_buffer(16 * 1024 * 1024)
    size = function(case, output, len(output))
    data = output.raw[:size]
    print(json.dumps({
        "case": case, "bytes": size,
        "result": list(data[131072:131076]),
        "sha256": hashlib.sha256(data).hexdigest(),
    }), flush=True)


def copy_tracked(root, destination, paths):
    destination.mkdir()
    for name in paths:
        relative = Path(name)
        if relative.is_absolute() or ".." in relative.parts:
            raise RuntimeError(f"unexpected tracked path: {name}")
        target = destination / relative
        target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(root / relative, target)


def build_and_capture(root, work, paths, name, mutation=None):
    copied = work / name
    copy_tracked(root, copied, paths)
    if mutation is not None:
        for source_path, anchor, replacement in mutation[3]:
            target = copied / source_path
            original = target.read_text(encoding="utf-8")
            if original.count(anchor) != 1:
                raise RuntimeError(
                    f"{name}: {source_path} mutation anchor must occur exactly once"
                )
            target.write_text(original.replace(anchor, replacement, 1), encoding="utf-8")

    # Inherit the CI native archive/SDK/linker environment unchanged. Reuse the
    # maintained direct build and its embedded-source/native-library checks.
    compiled = run(
        [sys.executable, "-B", "tools/verify_dll.py", "--build-only", "--direct"],
        copied, 600,
    )
    if compiled.returncode != 0:
        raise RuntimeError(f"{name}: compile failed; not a killed mutant\n{compiled.stdout}\n{compiled.stderr}")
    print(f"{name}: compile PASS", flush=True)
    cases = tuple(EXPECTED) if mutation is None else (mutation[1],)
    for case in cases:
        result = run(
            [sys.executable, "-B", str(Path(__file__).resolve()), "--capture", str(copied), str(case)],
            copied, 30,
        )
        if result.returncode != 0:
            raise RuntimeError(
                f"{name}/{case}: capture process exit {result.returncode}; not a killed mutant\n"
                f"{result.stdout}\n{result.stderr}"
            )
        try:
            observation = json.loads(result.stdout)
        except json.JSONDecodeError as error:
            raise RuntimeError(f"{name}/{case}: invalid capture output\n{result.stdout}\n{result.stderr}") from error
        if observation.get("case") != case:
            raise RuntimeError(f"{name}: wrong case output: {observation}")
        if mutation is None:
            size, digest, expected_result = EXPECTED[case]
            if (observation.get("bytes") != size
                    or observation.get("sha256") != digest
                    or observation.get("result") != expected_result
                    or result.stderr):
                raise RuntimeError(f"baseline/{case}: frozen observation must pass\n{observation}\n{result.stderr}")
            print(f"baseline/{case}: frozen bytes PASS", flush=True)
        else:
            expected_assertion = "assertion `left == right` failed: " + mutation[2] + "\n"
            case_assertion = f"assertion `left == right` failed: unexpected result for baseline case {case}\n"
            stderr = result.stderr.replace("\r\n", "\n")
            if (observation.get("bytes") != 0
                    or observation.get("result") != []
                    or expected_assertion not in stderr
                    or case_assertion not in stderr):
                raise RuntimeError(
                    f"{name}: expected zero-length capture and both specific assertion failures; "
                    f"survival/crash/unrelated panic is not a kill\n{observation}\n{stderr}"
                )
            print(f"{name}: KILLED by {mutation[2]} (case {case})", flush=True)


def main():
    if os.environ.get("GITHUB_ACTIONS") != "true":
        raise RuntimeError("mutation builds and execution are restricted to GitHub CI")
    if os.name != "nt":
        raise RuntimeError("room scheduler mutation DLL requires Windows")
    if len(sys.argv) == 4 and sys.argv[1] == "--capture":
        case = int(sys.argv[3])
        if case not in EXPECTED:
            raise RuntimeError("only the three room-mechanism observation cases are permitted")
        capture_child(Path(sys.argv[2]).resolve(), case)
        return
    if len(sys.argv) != 1:
        raise RuntimeError("run without arguments after CI installs the native dependencies")
    root = Path(__file__).resolve().parents[3]
    tracked = run(["git", "ls-files", "-z"], root, 30)
    if tracked.returncode != 0:
        raise RuntimeError(f"cannot enumerate tracked source\n{tracked.stderr}")
    paths = tracked.stdout.rstrip("\0").split("\0")
    required = {SCHEDULER, COMMANDS, RUNTIME, "tools/verify_dll.py", "sm/src/verification/room_mechanism_callback_edges.rs"}
    if not required.issubset(paths):
        raise RuntimeError("install this runner with the adopted room verification carrier")
    with tempfile.TemporaryDirectory(prefix="room-scheduler-mutations-") as directory:
        work = Path(directory)
        build_and_capture(root, work, paths, "baseline")
        for mutation in MUTATIONS:
            build_and_capture(root, work, paths, mutation[0], mutation)
    print("room mechanism mutation checks: 4/4 killed; all three frozen baselines passed", flush=True)


if __name__ == "__main__":
    main()
