"""Five room-entry DLL faults, called by the adopted room_mutations runner."""

import hashlib
import json
import os
from pathlib import Path
import re
import sys


SELECTION = "sm/src/game/room_entry/selection.rs"
RUNTIME = "sm/src/game/session_runtime.rs"
COMPATIBILITY = "sm/src/game/room_entry/compatibility.rs"
BACKGROUND_SOURCE = "room_background_observer.rs"
CONTACT_SOURCE = "room_contact_start_cases.rs"
GUARD_SOURCE = "room_runtime_checks.rs"
CONTENT_SOURCE = "content_runtime.rs"
RUNTIME_SOURCE = "session_runtime.rs"
EMPTY_HASH = hashlib.sha256(b"").hexdigest()

# Frozen original/candidate-equal observations, reviewed before introducing faults.
CONTACT_BASELINES = {
    8: (262183, "9ec8df05d437b9f8f9a97befb8714a455c929cad2ba7f9796a5c401f1a9132f5"),
    13: (262160, "1c66ed0182cd2b5184b2ee4fbd69baaaadd0188468b0170cebe9724b8a7f4eab"),
    16: (262183, "daf0120fbe58b42431e99f6b434faa98650120ebb7b3966f78622e6c9cf44e13"),
}
BACKGROUND_BASELINES = {
    37: (397572, "cd60a90aa76fc02924834b334480c816bf9839b6f80dc6db9649713f8df0cc0c"),
}
GUARD_PANICS = (
    (GUARD_SOURCE, "intentional room foreign unwind"),
    (RUNTIME_SOURCE, "aggregate unavailable during external effect"),
    (RUNTIME_SOURCE, "Session belongs to a different world binding"),
    ("actor_initializer_checks.rs", "actor initializer fixture deliberate nested dispatch unwind"),
    (RUNTIME_SOURCE, "Game already owns actor identity"),
    (RUNTIME_SOURCE, "Session belongs to a different world binding"),
    ("actor_runtime_checks.rs", "actor runtime nested fixture unwind"),
)
# Owned Session now preflights at Runtime::foreign before taking state and again
# at export_session. Inject the same write-before-binding-check fault into that
# shared preflight so the real mismatched foreign entry remains discriminating.
EXPORT = """fn preflight_export(state: &Session, game: &Game) {
    assert!("""
EARLY_EXPORT = """fn preflight_export(state: &Session, game: &Game) {
    encoding::encode(state, |at, bytes| game.ram().copy_in(at, bytes));
    assert!("""

# Runtime::foreign's entry preflight; owned_call repeats the same call.
FOREIGN_PREFLIGHT = """    pub fn foreign<R>(&mut self, action: impl FnOnce(&mut Game) -> R) -> R {
        assert!(self.persistent_is_current(), "Session lease is dirty");
        preflight_export(self.session(), self.game);"""

# Name, selected actual export/case, equality label, exact-once production edits.
MUTATIONS = (
    (
        "room_contact_marker_after_content", "contact:8",
        "room contact/start 8 full expected RAM",
        (
            (SELECTION,
             "    h.session_mut().room.entry.transition_selector = 0xe17d;\n    let exit = match site {",
             "    let exit = match site {"),
            (SELECTION,
             "    if !h.contact_has_destination(&exit) {",
             "    h.session_mut().room.entry.transition_selector = 0xe17d;\n    if !h.contact_has_destination(&exit) {"),
        ),
    ),
    (
        "room_station_lockout_after_content", "contact:16",
        "room contact/start 16 full expected RAM",
        ((SELECTION,
          "    h.session_mut().mechanisms.save_locked = 1;\n    let station = h.capture_station(site);",
          "    let station = h.capture_station(site);\n    h.session_mut().mechanisms.save_locked = 1;"),),
    ),
    (
        "room_station_overwrites_area_high", "contact:13",
        "room contact/start 13 full expected RAM",
        ((SELECTION,
          "    h.session_mut().room.area = (h.session().room.area & 0xff00) | u16::from(area);",
          "    h.session_mut().room.area = u16::from(area);"),),
    ),
    (
        "room_export_before_binding_check", "guard",
        "room.guard.mismatch-before-writes",
        ((RUNTIME, EXPORT, EARLY_EXPORT),
         (RUNTIME,
          FOREIGN_PREFLIGHT,
          FOREIGN_PREFLIGHT.replace(
              "preflight_export(self.session(), self.game);",
              'preflight_export(self.state.as_ref().expect("aggregate unavailable during external effect"), self.game);'))),
    ),
    (
        "room_cre_vram_word_as_byte_offset", "background:37",
        "background 37 write 7",
        ((COMPATIBILITY,
          "            VramDecodeTarget::Cre => 0x5000,",
          "            VramDecodeTarget::Cre => 0x2800,"),),
    ),
)
PANIC = re.compile(r"(?:^|\n)thread '[^\n]+'(?: \(\d+\))? panicked at ([^\n]+):\n([^\n]*)\n")
NOTE = "note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace"


def stderr_matches(stderr, expected, footer=None):
    """Require only the reviewed mixed-source panic sequence and optional carrier line."""
    text = stderr.replace("\r\n", "\n")
    panics = list(PANIC.finditer(text))
    if len(panics) != len(expected):
        return False
    if not panics:
        return footer is None and not text.strip()
    if text[:panics[0].start()].strip():
        return False
    for index, (panic, (source, wanted)) in enumerate(zip(panics, expected)):
        location, message = panic.groups()
        if (not re.search(r"(?:^|/)" + re.escape(source) + r":\d+:\d+$", location.replace("\\", "/"))
                or message != wanted):
            return False
        end = panics[index + 1].start() if index + 1 < len(panics) else len(text)
        lines = [line.strip() for line in text[panic.end():end].splitlines()
                 if line.strip() and line.strip() != NOTE]
        if index + 1 == len(panics) and footer is not None:
            if not lines or lines.pop() != footer:
                return False
        if wanted.startswith("assertion `left == right` failed:"):
            if len(lines) != 2 or not lines[0].startswith("left: ") or not lines[1].startswith("right: "):
                return False
        elif lines:
            return False
    return True


def baseline_panics(selected):
    if selected == "guard":
        return GUARD_PANICS
    return () if selected in {"contact:13", "background:37"} else ((CONTENT_SOURCE, "ROM must be loaded"),)


def capture_child(root, selected):
    # Called only through room_mutations.py's hosted-only argument dispatch.
    import ctypes
    import runpy

    if selected not in {m[1] for m in MUTATIONS}:
        raise RuntimeError("unknown room-entry mutation baseline")
    os.environ["RUST_BACKTRACE"] = "0"
    verifier = runpy.run_path(str(root / "tools/verify_dll.py"))
    host = verifier["load_host"](root / "target/verify-dll/sm_verify.dll")
    verifier["check_source"](host, verifier["source_hash"](root))
    verifier["check_library"](host, root)
    if selected == "guard":
        function = host.sm_verify_room_runtime_guard
        function.argtypes = []
        function.restype = ctypes.c_bool
        observation = {"selected": selected, "passed": bool(function())}
    else:
        case = int(selected.split(":")[1])
        function = (host.sm_verify_room_background_case if selected.startswith("background:")
                    else host.sm_verify_room_contact_start_case)
        function.argtypes = [ctypes.c_uint32, ctypes.c_void_p, ctypes.c_size_t]
        function.restype = ctypes.c_size_t
        output = ctypes.create_string_buffer(1024 * 1024)
        size = function(case, output, len(output))
        if size > len(output):
            raise RuntimeError("room-entry capture exceeds its output allocation")
        data = output.raw[:size]
        observation = {"selected": selected, "bytes": size, "sha256": hashlib.sha256(data).hexdigest()}
    print(json.dumps(observation), flush=True)


def check_capture(copied, selected, run, mutation=None):
    result = run(
        [sys.executable, "-B", str(Path(__file__).with_name("room_mutations.py").resolve()),
         "--entry-capture", str(copied), selected], copied, 30,
    )
    if result.returncode != 0:
        raise RuntimeError(f"{selected}: capture exit {result.returncode}; not a killed mutant\n{result.stdout}\n{result.stderr}")
    try:
        observation = json.loads(result.stdout)
    except json.JSONDecodeError as error:
        raise RuntimeError(f"{selected}: invalid capture output\n{result.stdout}\n{result.stderr}") from error
    expected_panics = list(baseline_panics(selected))
    if mutation is not None and selected == "guard":
        expected_panics = expected_panics[:3]  # Binding assertion stops before actor checks.
    footer = None
    if selected == "guard":
        expected_result = {"selected": selected, "passed": mutation is None}
    else:
        case = int(selected.split(":")[1])
        baselines = BACKGROUND_BASELINES if selected.startswith("background:") else CONTACT_BASELINES
        size, digest = baselines[case] if mutation is None else (0, EMPTY_HASH)
        expected_result = {"selected": selected, "bytes": size, "sha256": digest}
    if mutation is not None:
        source = (BACKGROUND_SOURCE if selected.startswith("background:")
                  else GUARD_SOURCE if selected == "guard" else CONTACT_SOURCE)
        expected_panics.append((source, "assertion `left == right` failed: " + mutation[2]))
        if selected.startswith("background:"):
            # The effect panic is caught, then finish consumes the observer and
            # rejects the interrupted 20-write trace. Both are required.
            expected_panics.append((BACKGROUND_SOURCE, "assertion `left == right` failed: ordered write count"))
            footer = "background offset case37 failed"
        elif selected != "guard":
            footer = f"room contact/start case {case} assertion failed"
    if observation != expected_result or not stderr_matches(result.stderr, expected_panics, footer):
        raise RuntimeError(
            f"{selected}: expected exact observation and intended assertion sequence; "
            f"survival/unrelated panic is not a kill\n{observation}\n{result.stderr}"
        )


def run_checks(root, work, paths, run, copy_tracked):
    required = {
        SELECTION, RUNTIME, COMPATIBILITY, "sm/src/verification/room_contact_start_cases.rs",
        "sm/src/verification/room_background_observer.rs",
        "sm/src/verification/room_background_offset_case.rs",
        "sm/src/verification/room_background_dll.rs",
        "sm/src/verification/room_runtime_checks.rs", "sm/src/verification/room_world_fixture.rs",
        "sm/src/verification/room_entry_mutations.py",
    }
    if not required.issubset(paths):
        raise RuntimeError("room-entry mutation sources must all be tracked")
    # The old nine-check flow already compiled this exact untouched tree.
    baseline = work / "baseline"
    for _, selected, _, _ in MUTATIONS:
        check_capture(baseline, selected, run)
        print(f"room-entry baseline/{selected}: PASS", flush=True)
    for mutation in MUTATIONS:
        name, selected, label, edits = mutation
        copied = work / name
        copy_tracked(root, copied, paths)
        for relative, anchor, replacement in edits:
            target = copied / relative
            original = target.read_text(encoding="utf-8")
            if original.count(anchor) != 1:
                raise RuntimeError(f"{name}: {relative} anchor must occur exactly once")
            target.write_text(original.replace(anchor, replacement, 1), encoding="utf-8")
        compiled = run([sys.executable, "-B", "tools/verify_dll.py", "--build-only", "--direct"], copied, 600)
        if compiled.returncode != 0:
            raise RuntimeError(f"{name}: compile failed; not a killed mutant\n{compiled.stdout}\n{compiled.stderr}")
        print(f"{name}: compile PASS", flush=True)
        check_capture(copied, selected, run, mutation)
        print(f"{name}: KILLED by {label} ({selected})", flush=True)
    print("room-entry mutation checks: 5/5 killed; all five selected baselines passed", flush=True)
