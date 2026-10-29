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
RECIPES = runpy.run_path(str(HERE / "player_transitions_dll_mutation_recipes.py"))["RECIPES"]
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
    require((kind in ("cpu", "rust") and 0 <= case < 11)
            or (kind == "rust" and case in (3700, 3702, 3900)), "unknown transition capture selection")
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
                "sm/src/verification/player_transitions_dll_mutation_recipes.py",
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
    run_owned_checks(root, work, paths, run, copy_tracked, rom)
    run_settlement_checks(root, work, paths, run, copy_tracked, rom)


def adoption_comparator(root):
    name = "player_transitions_adoption_completion"
    path = root / "sm/src/verification/player_transitions_adoption_completion.py"
    spec = importlib.util.spec_from_file_location(name, path)
    require(spec is not None and spec.loader is not None, "missing adoption completion parser")
    module = importlib.util.module_from_spec(spec)
    sys.modules[name] = module
    spec.loader.exec_module(module)
    return module


def composed_baseline(complete, compare, data, case):
    # The existing adoption parser delegates selected6 only when its ID matches.
    class NoSelectedCases:
        CASES = ()
    return complete.check_capture(data, case, compare, NoSelectedCases)


def composed_expected(compare, baseline, case, fault):
    """Exact reviewed trace changes; every other byte/event stays identical."""
    records = compare.records(baseline, b"PTRS1")
    metadata, body = records[:4], records[4:]
    def phase(record):
        return struct.unpack_from("<H", record.payload)[0] if record.tag == 2 else None
    def snapshot(number):
        matches = [record for record in body if phase(record) == number]
        require(len(matches) == 1, "composed baseline snapshot is missing/duplicated")
        return matches[0]
    def words(record, changes):
        require(record.tag == 2 and len(record.payload) == compare.RAM_SIZE + 3
                and record.payload[-1] == 0, "invalid composed snapshot")
        data = bytearray(record.payload)
        for address, old, new in changes:
            require(struct.unpack_from("<H", data, 2 + address)[0] == old,
                    "composed mutation expected baseline word is stale")
            struct.pack_into("<H", data, 2 + address, new)
        return compare.Record(2, bytes(data))
    marker = compare.Record(4, struct.pack("<HH", 3702, 1))
    require(body[-2] == marker and body[-1] == snapshot(11), "missing strict completion boundary")
    failed_marker = compare.Record(4, struct.pack("<HH", 3702, 0))
    if fault == "COMPOSE_HISTORY":
        require(case == 3700, "history fault case")
        final = words(snapshot(11), [(0xa24, 3, 0x5678), (0xa26, 4, 0x9abc)])
        return metadata + body[:-2] + [failed_marker, final]
    if fault == "COMPOSE_STALE_REQUEST":
        require(case == 3702, "stale request fault case")
        until = body.index(snapshot(220)) + 1
        final = words(snapshot(11), [(0xa1c, 3, 2), (0xa22, 8, 4),
            (0xa24, 3, 0x5678), (0xa26, 4, 0x9abc), (0xa2a, 0x37, 0xffff),
            (0xa2e, 7, 0), (0xa34, 0, 0x1111), (0xa36, 0, 0x2222), (0xa9a, 0, 2)])
        return metadata + body[:until] + [failed_marker, final]
    require(fault == "COMPOSE_ORDER" and case == 3700, "unknown composed fault")
    final = words(snapshot(11), [(0xa94, 11, 25)])
    changed = []
    for record in body[:-2]:
        number = phase(record)
        if number == 220:
            continue
        if number in (2, 3, 20, 4, 10):
            record = words(record, [(0xa94, 1, 2), (0xa9c, 2, 17)])
        changed.append(record)
    late_clock = compare.Record(2, struct.pack("<H", 220) + final.payload[2:])
    return metadata + changed + [late_clock, failed_marker, final]


def qualify_composed(compare, complete, baseline, actual, case, fault):
    expected = composed_expected(compare, baseline, case, fault)
    observed = compare.records(actual, b"PTRS1")
    require(observed == expected, "composed mutant has missing/extra events or unrelated bytes")
    require(actual != baseline, "composed mutation survived exact baseline comparison")
    try:
        composed_baseline(complete, compare, actual, case)
    except compare.Mismatch as error:
        return str(error)
    raise RuntimeError("composed mutation survived strict completion parser")


def run_owned_checks(root, work, paths, run, copy_tracked, rom):
    """Three additional hosted faults; old PT1/PT2 classifications are unchanged."""
    hosted_only()
    recipe_path = root / "sm/src/verification/player_transitions_composed_mutation_recipes.py"
    recipes = runpy.run_path(str(recipe_path))["RECIPES"]
    expected = {"COMPOSE_ORDER": 3700, "COMPOSE_STALE_REQUEST": 3702, "COMPOSE_HISTORY": 3700}
    require(len(recipes) == 3 and {r["fault"] for r in recipes} == set(expected),
            "exactly three composed faults are required")
    needed = {"sm/src/verification/player_transitions_composed.rs",
              "sm/src/verification/player_transitions_adoption_completion.py",
              "sm/src/verification/player_transitions_composed_mutation_recipes.py"}
    needed.update(edit["path"] for recipe in recipes for edit in recipe["edits"])
    require(needed.issubset(paths), "composed sources/parser/recipes must be tracked")
    compare, complete = comparator(root), adoption_comparator(root)
    results = work / "composed-transition-captures"
    results.mkdir()
    baselines = {}
    for case in (3700, 3702):
        data, receipt = capture(work / "baseline", "rust", case, rom, results / f"baseline-{case}.bin", run)
        detail = composed_baseline(complete, compare, data, case)
        baselines[case] = data
        print(json.dumps({"composed_baseline": case, "receipt": receipt, "completion": detail}), flush=True)
    for recipe in recipes:
        case = expected[recipe["fault"]]
        require(recipe["selected"] == f"transition:{case}" and recipe["qualification"] == "exact_completed_trace",
                "composed recipe selection/classifier changed")
        require(len(recipe["edits"]) == 3 and recipe["edits"][0]["path"].startswith("sm/src/game/player_transitions/")
                and all(edit["path"] == "sm/src/verification/player_transitions_composed.rs" for edit in recipe["edits"][1:]),
                "one gameplay fault plus two bounded observation-mode edits required")
        again, _ = capture(work / "baseline", "rust", case, rom, results / f"recheck-{recipe['fault']}.bin", run)
        require(again == baselines[case], "unchanged composed baseline changed between runs")
        composed_baseline(complete, compare, again, case)
        copied = actor.build(root, work, paths, recipe["name"], run, copy_tracked, recipe["edits"])
        actual, receipt = capture(copied, "rust", case, rom, results / f"mutant-{recipe['fault']}.bin", run)
        mismatch = qualify_composed(compare, complete, baselines[case], actual, case, recipe["fault"])
        print(json.dumps({"composed_mutant": recipe["fault"], "receipt": receipt,
                          "exact_completed_trace": True, "strict_completion_mismatch": mismatch}), flush=True)
        print(f"{recipe['name']}: KILLED by exact normally returned composed contract difference", flush=True)
    print("owned composed mutations: 3/3 unique faults killed; 2 unchanged baselines passed", flush=True)


def settlement_comparator(root):
    name = "player_settlement_completion"
    path = root / "sm/src/verification/player_settlement_completion.py"
    spec = importlib.util.spec_from_file_location(name, path)
    require(spec is not None and spec.loader is not None, "missing settlement completion parser")
    module = importlib.util.module_from_spec(spec)
    sys.modules[name] = module
    spec.loader.exec_module(module)
    return module


def settlement_expected(compare, baseline, fault):
    """Transform only exact 3900 semantic discrepancies, preserving all records."""
    require(fault in ("SETTLEMENT_EARLY_CONSUME", "SETTLEMENT_LOST_FRACTION"),
            "unknown settlement fault")
    records = compare.records(baseline, b"PTRS1")
    altered = []
    seen = set()
    markers = 0
    for record in records:
        if record.tag == 2:
            require(len(record.payload) == compare.RAM_SIZE + 3 and record.payload[-1] == 0,
                    "invalid settlement baseline snapshot")
            phase = struct.unpack_from("<H", record.payload)[0]
            require(phase not in seen, "duplicate settlement phase")
            seen.add(phase)
            changes = []
            if fault == "SETTLEMENT_EARLY_CONSUME":
                if phase in (220, 28):
                    changes = [(0xa4c, 0x2000, 0x1fff), (0xa4e, 0x8001, 0x8000)]
                elif phase == 11:
                    changes = [(0x9c2, 97, 99), (0xa4c, 0x9fff, 0x1fff),
                               (0xa4e, 0, 0x8000), (0xa50, 0, 1)]
            elif phase == 11:
                changes = [(0xa4c, 0x9fff, 0x2000)]
            payload = bytearray(record.payload)
            for at, old, new in changes:
                require(struct.unpack_from("<H", payload, 2 + at)[0] == old,
                        "settlement baseline discrepancy word is stale")
                struct.pack_into("<H", payload, 2 + at, new)
            record = compare.Record(2, bytes(payload))
        elif record == compare.Record(4, struct.pack("<HH", 3903, 1)):
            markers += 1
            record = compare.Record(4, struct.pack("<HH", 3903, 0))
        altered.append(record)
    require(seen == {1, 220, 28, 11} and markers == 1, "settlement exact boundaries missing")
    return altered


def qualify_settlement(compare, complete, baseline, actual, fault):
    require(compare.records(actual, b"PTRS1") == settlement_expected(compare, baseline, fault),
            "settlement mutant differs outside exact reviewed full trace")
    require(actual != baseline, "settlement mutation survived exact baseline comparison")
    try:
        complete.check_capture(actual, 3900, compare)
    except compare.Mismatch as error:
        return str(error)
    raise RuntimeError("settlement mutation survived strict completion parser")


def run_settlement_checks(root, work, paths, run, copy_tracked, rom):
    hosted_only()
    recipe_path = root / "sm/src/verification/player_settlement_mutation_recipes.py"
    recipes = runpy.run_path(str(recipe_path))["RECIPES"]
    faults = {"SETTLEMENT_EARLY_CONSUME": "sm/src/game/player_update/mod.rs",
              "SETTLEMENT_LOST_FRACTION": "sm/src/game/samus_periodic_damage.rs"}
    require(len(recipes) == 2 and {r["fault"] for r in recipes} == set(faults),
            "exactly two distinct settlement faults required")
    required = {"sm/src/verification/player_settlement.rs",
                "sm/src/verification/player_settlement_completion.py",
                "sm/src/verification/player_settlement_mutation_recipes.py"}
    required.update(edit["path"] for recipe in recipes for edit in recipe["edits"])
    require(required.issubset(paths), "settlement sources/parser/recipes must be tracked")
    compare, complete = comparator(root), settlement_comparator(root)
    results = work / "settlement-transition-captures"
    results.mkdir()
    baseline, receipt = capture(work / "baseline", "rust", 3900, rom, results / "baseline-3900.bin", run)
    detail = complete.check_capture(baseline, 3900, compare)
    print(json.dumps({"settlement_baseline": 3900, "receipt": receipt, "completion": detail}), flush=True)
    for recipe in recipes:
        require(recipe["selected"] == "transition:3900" and recipe["qualification"] == "exact_completed_trace",
                "settlement selection/classifier changed")
        require(len(recipe["edits"]) == 3 and recipe["edits"][0]["path"] == faults[recipe["fault"]]
                and all(edit["path"] == "sm/src/verification/player_settlement.rs" for edit in recipe["edits"][1:]),
                "one settlement gameplay fault plus two bounded observation edits required")
        again, _ = capture(work / "baseline", "rust", 3900, rom,
                           results / f"recheck-{recipe['fault']}.bin", run)
        require(again == baseline, "settlement unchanged baseline changed between runs")
        complete.check_capture(again, 3900, compare)
        copied = actor.build(root, work, paths, recipe["name"], run, copy_tracked, recipe["edits"])
        actual, receipt = capture(copied, "rust", 3900, rom,
                                  results / f"mutant-{recipe['fault']}.bin", run)
        mismatch = qualify_settlement(compare, complete, baseline, actual, recipe["fault"])
        print(json.dumps({"settlement_mutant": recipe["fault"], "receipt": receipt,
                          "exact_completed_trace": True, "strict_completion_mismatch": mismatch}), flush=True)
        print(f"{recipe['name']}: KILLED by exact normally returned settlement contract difference", flush=True)
    print("player settlement mutations: 2/2 unique faults killed; unchanged 3900 baseline passed", flush=True)
