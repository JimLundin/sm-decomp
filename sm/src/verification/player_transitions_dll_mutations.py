"""Two successful-capture semantic faults in the existing hosted DLL runner."""
import ctypes
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import runpy
import struct
import sys

import actor_dll_mutations as actor


HERE = Path(__file__).resolve().parent
RECIPES = json.loads((HERE / "player_transitions_dll_mutation_recipes.json").read_text(encoding="utf-8-sig"))
EXPECTED = {
    "transitions_second_metadata_word_byte_order": ("PT1", "transition:5", 4),
    "transitions_opcode12_mask_word_byte_order": ("PT2", "transition:7", 7),
}
ORIGINAL_SHA1 = "da957f0d63d14cb441d215462904c4fa8519c613"
CAPACITY = 2 * 1024 * 1024


def require(condition, message):
    if not condition:
        raise RuntimeError(message)


def hosted_only():
    require(os.environ.get("GITHUB_ACTIONS") == "true" and os.name == "nt",
            "transition DLL mutation builds/captures require Windows GitHub CI")


def original_identity(rom):
    data = Path(rom).read_bytes()
    require(len(data) == 0x300000 and hashlib.sha1(data).hexdigest() == ORIGINAL_SHA1,
            "transition checks require the supplied original unheadered 3 MiB ROM")
    return ORIGINAL_SHA1


def comparator(root):
    # Register the module for dataclass annotations; do not run its CLI.
    name = "player_transitions_semantic_comparator"
    spec = importlib.util.spec_from_file_location(name, root / "tools/player_transitions_compare.py")
    require(spec is not None and spec.loader is not None, "missing transition comparator")
    module = importlib.util.module_from_spec(spec)
    sys.modules[name] = module
    spec.loader.exec_module(module)
    return module


def capture_child(root, kind, case, rom, destination):
    """Exactly one actual initialized export invocation in this fresh process."""
    hosted_only()
    require(kind in ("cpu", "rust") and 0 <= case < 11, "unknown transition capture selection")
    rom_sha1 = original_identity(rom)
    os.environ["RUST_BACKTRACE"] = "0"
    verifier = runpy.run_path(str(root / "tools/verify_dll.py"))
    dll = root / "target/verify-dll/sm_verify.dll"
    host = verifier["load_host"](dll)
    source = verifier["source_hash"](root)
    verifier["check_source"](host, source)
    verifier["check_library"](host, root)
    name = "sm_verify_player_transitions_oracle" if kind == "cpu" else "sm_verify_player_transitions_rust"
    function = getattr(host, name)
    function.argtypes = [ctypes.c_char_p, ctypes.c_uint32, ctypes.c_void_p, ctypes.c_size_t]
    function.restype = ctypes.c_size_t
    output = ctypes.create_string_buffer(CAPACITY)
    size = function(os.fsencode(rom), case, output, len(output))
    require(0 < size <= len(output), "transition export failed or exceeded capacity; not a kill")
    data = output.raw[:size]
    require(data.startswith(b"PTOR1" if kind == "cpu" else b"PTRS1"), "wrong transition capture magic")
    # Caller selects a fresh file in its temporary directory. Never overwrite one.
    with destination.open("xb") as handle:
        handle.write(data)
    print(json.dumps({"kind": kind, "case": case, "bytes": size,
                      "sha256": hashlib.sha256(data).hexdigest(), "source_sha256": source,
                      "dll_sha256": hashlib.sha256(dll.read_bytes()).hexdigest(),
                      "original_sha1": rom_sha1}), flush=True)


def capture(copied, kind, case, rom, destination, run):
    command = [sys.executable, "-B", str(copied / "sm/src/verification/room_mutations.py"),
               "--transition-capture", str(copied), kind, str(case), str(rom), str(destination)]
    child = run(command, copied, 30)
    require(child.returncode == 0 and not child.stderr,
            f"transition {kind}/{case}: child failure or diagnostic is not a kill\n{child.stdout}\n{child.stderr}")
    try:
        receipt = json.loads(child.stdout)
    except json.JSONDecodeError as error:
        raise RuntimeError("transition capture did not return one JSON receipt") from error
    data = destination.read_bytes()
    require(0 < len(data) <= CAPACITY, "transition capture empty or oversized")
    verifier = runpy.run_path(str(copied / "tools/verify_dll.py"))
    expected = {"kind": kind, "case": case, "bytes": len(data),
                "sha256": hashlib.sha256(data).hexdigest(),
                "source_sha256": verifier["source_hash"](copied),
                "dll_sha256": hashlib.sha256((copied / "target/verify-dll/sm_verify.dll").read_bytes()).hexdigest(),
                "original_sha1": original_identity(rom)}
    require(receipt == expected, "transition receipt does not bind these exact bytes/source/DLL/original ROM")
    return data, receipt


def altered_expected(compare, reference, recipe):
    case = int(recipe["selected"].split(":")[1])
    expected, _ = compare.expected_stream(reference, case)
    altered = list(expected)
    touched = set()
    for change in recipe["expected_differences"]:
        candidates = [index for index, item in enumerate(expected)
                      if item.tag == change["tag"]
                      and len(item.payload) >= 2
                      and struct.unpack_from("<H", item.payload)[0] == change["semantic"]]
        require(len(candidates) == 1, "mutation discrepancy must select exactly one semantic event")
        index = candidates[0]
        item = altered[index]
        tag, field = change["tag"], change["field"]
        if tag == 2 and field == "ram_word":
            address = change["address"]
            require(0 <= address < compare.RAM_SIZE - 1, "mutation RAM address out of bounds")
            require(all(address + 2 <= start or address >= end
                        for start, end, _ in compare.exclusions(case)),
                    "mutation must not modify excluded CPU scratch expectations")
            offset = 2 + address
        elif tag == 3 and field == "after":
            require(struct.unpack_from("<I", item.payload, 2)[0] == change["address"],
                    "mutation selected wrong publication address")
            offset = 8
        elif tag == 4 and field == "value":
            offset = 2
        else:
            raise RuntimeError("unsupported mutation discrepancy target")
        require((index, offset) not in touched, "duplicate mutation discrepancy")
        touched.add((index, offset))
        require(struct.unpack_from("<H", item.payload, offset)[0] == change["baseline"],
                "mutation discrepancy baseline is stale")
        require(0 <= change["mutant"] <= 0xffff and change["mutant"] != change["baseline"],
                "mutation discrepancy must change a word")
        payload = bytearray(item.payload)
        struct.pack_into("<H", payload, offset, change["mutant"])
        altered[index] = compare.Record(tag, bytes(payload))
    return case, expected, altered


def qualify(compare, reference, actual_bytes, recipe):
    """Every field outside the four/seven approved discrepancies must agree."""
    case, expected, altered = altered_expected(compare, reference, recipe)
    actual = compare.metadata(compare.records(actual_bytes, b"PTRS1"), case, True)
    require(len(actual) == len(altered), "mutation omitted or added semantic events")
    seed = expected[0].payload[2:-1]
    for index, (want, got) in enumerate(zip(altered, actual)):
        require(got.tag == want.tag and len(got.payload) == len(want.payload),
                f"mutation event {index}: malformed or reordered record")
        if got.tag == 2:
            require(got.payload[:2] == want.payload[:2] and got.payload[-1] == 0,
                    f"mutation event {index}: wrong snapshot identity or g_fail")
            compare.qualified_equal(want.payload[2:-1], got.payload[2:-1], case,
                                    f"mutation event {index}")
            compare.excluded_unchanged(seed, got.payload[2:-1], case, f"mutation event {index}")
        else:
            require(got.payload == want.payload, f"mutation event {index}: unrelated semantic discrepancy")
    try:
        compare.compare_case(reference, actual_bytes, case)
    except compare.Mismatch as error:
        # Full altered-stream equality above already ruled out unrelated faults.
        return str(error)
    raise RuntimeError("mutation survived the unchanged-reference comparison")


def run_checks(root, work, paths, run, copy_tracked, rom):
    hosted_only()
    original_identity(rom)
    require(len(RECIPES) == 2 and {r["name"] for r in RECIPES} == set(EXPECTED),
            "both distinct PT1/PT2 recipes must remain selected")
    for recipe in RECIPES:
        fault, selected, differences = EXPECTED[recipe["name"]]
        require(recipe["fault"] == fault and recipe["selected"] == selected
                and recipe["qualification"] == "semantic_trace"
                and len(recipe["expected_differences"]) == differences,
                "wrong transition fault selection/classifier/discrepancy count")
    required = {"tools/player_transitions_compare.py", "sm/src/verification/room_mutations.py",
                "sm/src/verification/actor_dll_mutations.py",
                "sm/src/verification/player_transitions_dll_mutations.py",
                "sm/src/verification/player_transitions_dll_mutation_recipes.json",
                "sm/src/verification/player_transitions_rust.rs",
                "sm/src/verification/player_transitions_rust_dll.rs",
                "sm/src/verification/player_transitions_rust_observer.rs",
                "sm/src/verification/player_transitions_oracle.rs",
                "sm/src/verification/player_transitions_oracle_dll.rs"}
    required.update(edit["path"] for recipe in RECIPES for edit in recipe["edits"])
    require(required.issubset(paths), "transition sources, comparator and recipes must all be tracked")
    compare = comparator(root)
    unchanged = work / "baseline"
    results = work / "transition-captures"
    results.mkdir()
    references, baselines = {}, {}
    for case in range(11):
        reference, cpu_receipt = capture(unchanged, "cpu", case, rom, results / f"cpu-{case}.bin", run)
        actual, rust_receipt = capture(unchanged, "rust", case, rom, results / f"rust-{case}.bin", run)
        comparison = compare.compare_case(reference, actual, case)
        references[case], baselines[case] = reference, actual
        print(json.dumps({"transition_baseline": case, "comparison": comparison,
                          "cpu_receipt": cpu_receipt, "rust_receipt": rust_receipt}), flush=True)
    print("transition baselines: 11/11 original-CPU/Rust semantic comparisons PASS", flush=True)
    for recipe in RECIPES:
        case = int(recipe["selected"].split(":")[1])
        again, _ = capture(unchanged, "rust", case, rom, results / f"recheck-{case}.bin", run)
        compare.compare_case(references[case], again, case)
        require(again == baselines[case], "selected unchanged Rust baseline changed between runs")
        copied = actor.build(root, work, paths, recipe["name"], run, copy_tracked, recipe["edits"])
        actual, receipt = capture(copied, "rust", case, rom, results / f"mutant-{case}.bin", run)
        first_difference = qualify(compare, references[case], actual, recipe)
        print(json.dumps({"transition_mutant": recipe["name"], "receipt": receipt,
                          "exact_intended_differences": recipe["expected_differences"],
                          "first_reference_mismatch": first_difference}), flush=True)
        print(f"{recipe['name']}: KILLED by {recipe['required_assertion']}", flush=True)
    print("transition DLL mutations: 2/2 unique faults killed; 11 unchanged baselines passed", flush=True)
