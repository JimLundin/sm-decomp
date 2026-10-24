"""Nine actual room/native/restore mutation checks; Windows GitHub CI only."""

import hashlib
import json
import os
import re
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile

import room_entry_mutations


INVENTORY = "sm/src/game/item_pickups.rs"
SESSION_RUNTIME = "sm/src/game/session_runtime.rs"
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
                flags::room_object_used(h, argument)""",
             """            let found = if ea == 8685640 {
                flags::item_collected(h, argument)"""),
        ),
    ),
    (
        "omit_native_return_import", 1440, "native return import: native word 0",
        ((SESSION_RUNTIME,
          "        self.state = Some(import_session(self.game));",
          "        self.state = Some(encoding::decode(&self.game.g_ram, std::sync::Arc::clone(self.game.world())));"),),
    ),
    (
        "retain_native_owner_on_callback_panic", 1441,
        "native foreign panic retained callback: native word 0",
        ((SESSION_RUNTIME,
          """        let state = self
            .state
            .take()
            .expect("projectile state unavailable during external effect");""",
          """        let state = self
            .state
            .clone()
            .expect("projectile state unavailable during external effect");"""),),
    ),
    (
        "omit_native_core_panic_publication", 1442, "native core panic publication: native word 0",
        ((SESSION_RUNTIME,
          """impl Drop for Runtime<'_> {
    fn drop(&mut self) {
        if let Some(state) = self.state.take() {
            export_session(&state, self.game);""",
          """impl Drop for Runtime<'_> {
    fn drop(&mut self) {
        if let Some(state) = self.state.take() {
            encoding::encode(&state, &mut self.game.g_ram);"""),),
    ),
    (
        "skip_health_equality_reserve", 1383, "health restore boundary 1",
        ((INVENTORY,
          "                if (health.wrapping_sub(self.health.maximum) as i16) >= 0 {",
          "                if (health.wrapping_sub(self.health.maximum) as i16) > 0 {"),),
    ),
    (
        "missile_reserve_always_99", 1398, "missile restore boundary 16",
        ((INVENTORY,
          """                    let reserve_limit = if self.ammunition[0].maximum.wrapping_sub(99) & 0x8000 != 0
                    {
                        self.ammunition[0].maximum
                    } else {
                        99
                    };""",
          "                    let reserve_limit = 99;"),),
    ),
)
# Synthetic observations frozen before faults were introduced. They contain no
# ROM/movie inputs. Assertion labels do not alter successful observation bytes.
EXPECTED = {
    1020: (7682778, "2a00ad7662f052c45a810bc61222e249e46faa8274462cd363a74a4d77cc2c4d", [3, 0, 0, 0]),
    1021: (7683014, "7a382ef319cad77105f4e62e1f71564fe0c439dd58bddd00a7a222109eba3a0e", [3, 0, 0, 0]),
    1033: (131076, "437d32b0e20950d0dd6fc5e2a68ba4629078666d83156af92f0075de88706dbe", [1, 52, 146, 0]),
    # Getter observations precede RAM; this four-byte slice is frozen RAM data.
    1037: (131180, "b59dac4de27781371d9d709b96a7b0dbff7819afc5a6bc075fd63276bb38e02f", [155, 118, 81, 44]),
    1383: (131103, "47086ac87c425e6775227da10444a42ae1810c4a68561e76ed892cb8f34ef40e", [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
    1398: (131103, "11d8949850bb734e864866e5082861f781e05348fb63ab5f5d66dc6a6aa2770f", [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
    # Root must replace pending hashes after the first reviewed candidate capture.
    1440: (131187, '9c9ef6d50b0d7bcf89a352f83d4391b8ebc0061314ac689d31ba3396823d4274', [1, 145, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
    1441: (131187, '9a3eee5a30108687f88a8121c73e19bb30ef458ff9c32808a3fb6a9a0c8c6d82', [0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0]),
    1442: (131103, '94675070da80afa06e75128b61205f0dde8902ae3faf9bb207d32ca2289b9def', [0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0]),
}
NATIVE_CASES = (1440, 1441, 1442)
RESTORE_CASES = (1383, 1398)
ENEMY_CASES = NATIVE_CASES + RESTORE_CASES
INTENTIONAL = {
    1441: "intentional native spawn callback unwind",
    1442: "intentional native spawn core unwind",
}
PANIC = re.compile(r"(?:^|\n)thread '[^\n]+'(?: \(\d+\))? panicked at ([^\n]+):\n([^\n]*)\n")
NOTE = "note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace"


def native_stderr_matches(stderr, messages, source="enemy_native_runtime_cases.rs"):
    # Only these native cases intentionally unwind. Require the exact sequence
    # of payloads, the actual verification source, and Rust's expected details.
    # Extra diagnostics, unrelated assertions and crashes cannot pass this check.
    text = stderr.replace("\r\n", "\n")
    panics = list(PANIC.finditer(text))
    if len(panics) != len(messages):
        return False
    if not panics:
        return not text.strip()
    if text[:panics[0].start()].strip():
        return False
    for index, (panic, expected) in enumerate(zip(panics, messages)):
        location, message = panic.groups()
        if (not re.search(r"(?:^|/)" + re.escape(source) + r":\d+:\d+$", location.replace("\\", "/"))
                or message != expected):
            return False
        end = panics[index + 1].start() if index + 1 < len(panics) else len(text)
        lines = [line.strip() for line in text[panic.end():end].splitlines()
                 if line.strip() and line.strip() != NOTE]
        if expected.startswith("assertion `left == right` failed:"):
            if len(lines) != 2 or not lines[0].startswith("left: ") or not lines[1].startswith("right: "):
                return False
        elif lines:
            return False
    return True


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

    if case in ENEMY_CASES:
        # Keep intentional-panic formatting deterministic; native SDK/linker
        # environment remains inherited without alteration.
        os.environ["RUST_BACKTRACE"] = "0"
    verifier = runpy.run_path(str(root / "tools/verify_dll.py"))
    host = verifier["load_host"](root / "target/verify-dll/sm_verify.dll")
    verifier["check_source"](host, verifier["source_hash"](root))
    verifier["check_library"](host, root)
    function = (host.sm_verify_enemy_projectile_case if case in ENEMY_CASES
                else host.sm_verify_room_mechanism_case)
    function.argtypes = [ctypes.c_uint32, ctypes.c_void_p, ctypes.c_size_t]
    function.restype = ctypes.c_size_t
    output = ctypes.create_string_buffer(16 * 1024 * 1024)
    size = function(case, output, len(output))
    data = output.raw[:size]
    result_range = slice(131088, 131099) if case in ENEMY_CASES else slice(131072, 131076)
    print(json.dumps({
        "case": case, "bytes": size,
        "result": list(data[result_range]),
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
            expected_panics = [INTENTIONAL[case]] if case in INTENTIONAL else []
            source = "inventory_restore_cases.rs" if case in RESTORE_CASES else "enemy_native_runtime_cases.rs"
            stderr_ok = (native_stderr_matches(result.stderr, expected_panics, source)
                         if case in ENEMY_CASES else not result.stderr)
            if (observation.get("bytes") != size
                    or observation.get("sha256") != digest
                    or observation.get("result") != expected_result
                    or not stderr_ok):
                raise RuntimeError(f"baseline/{case}: frozen observation must pass\n{observation}\n{result.stderr}")
            print(f"baseline/{case}: frozen bytes PASS", flush=True)
        else:
            expected_assertion = "assertion `left == right` failed: " + mutation[2] + "\n"
            case_assertion = f"assertion `left == right` failed: unexpected result for baseline case {case}\n"
            stderr = result.stderr.replace("\r\n", "\n")
            if case in ENEMY_CASES:
                kind = "restore" if case in RESTORE_CASES else "native Runtime"
                source = "inventory_restore_cases.rs" if case in RESTORE_CASES else "enemy_native_runtime_cases.rs"
                case_assertion = f"assertion `left == right` failed: {kind} case {case} failed\n"
                expected_panics = [INTENTIONAL[case]] if case in INTENTIONAL else []
                expected_panics += [expected_assertion.rstrip("\n"), case_assertion.rstrip("\n")]
                stderr_ok = native_stderr_matches(stderr, expected_panics, source)
            else:
                stderr_ok = expected_assertion in stderr and case_assertion in stderr
            if (observation.get("bytes") != 0
                    or observation.get("result") != []
                    or not stderr_ok):
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
    if len(sys.argv) == 4 and sys.argv[1] == "--entry-capture":
        room_entry_mutations.capture_child(Path(sys.argv[2]).resolve(), sys.argv[3])
        return
    if len(sys.argv) == 4 and sys.argv[1] == "--capture":
        case = int(sys.argv[3])
        if case not in EXPECTED:
            raise RuntimeError("only the nine fixed room/native/restore observation cases are permitted")
        capture_child(Path(sys.argv[2]).resolve(), case)
        return
    if len(sys.argv) != 1:
        raise RuntimeError("run without arguments after CI installs the native dependencies")
    if any(not isinstance(digest, str) or re.fullmatch(r"[0-9a-f]{64}", digest) is None
           for _, digest, _ in EXPECTED.values()):
        raise RuntimeError("native baseline hashes are pending root capture; no mutation builds permitted")
    root = Path(__file__).resolve().parents[3]
    tracked = run(["git", "ls-files", "-z"], root, 30)
    if tracked.returncode != 0:
        raise RuntimeError(f"cannot enumerate tracked source\n{tracked.stderr}")
    paths = tracked.stdout.rstrip("\0").split("\0")
    required = {INVENTORY, "sm/src/verification/inventory_restore_cases.rs", SCHEDULER, COMMANDS, RUNTIME, SESSION_RUNTIME, "sm/src/verification/enemy_native_runtime_cases.rs", "tools/verify_dll.py", "sm/src/verification/room_mechanism_callback_edges.rs"}
    if not required.issubset(paths):
        raise RuntimeError("install this runner with the adopted room verification carrier")
    with tempfile.TemporaryDirectory(prefix="room-scheduler-mutations-") as directory:
        work = Path(directory)
        build_and_capture(root, work, paths, "baseline")
        for mutation in MUTATIONS:
            build_and_capture(root, work, paths, mutation[0], mutation)
        room_entry_mutations.run_checks(root, work, paths, run, copy_tracked)
    print("room/native/restore mutation checks: 9/9 killed; all nine frozen baselines passed", flush=True)


if __name__ == "__main__":
    main()
