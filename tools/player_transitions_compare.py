#!/usr/bin/env python3
"""Compare frozen PTOR1 CPU references with actual Rust semantic observations.

Read-only with respect to captures. This tool does not load a DLL, execute game
code, build a predecessor, or certify capture provenance. Root performs those
steps separately. Rust snapshots include the Session encoding where required;
they do not pretend to contain original-CPU registers or scratch publications.
"""
import argparse
from dataclasses import dataclass
import hashlib
import json
from pathlib import Path
import struct
import sys


RAM_SIZE = 0x20000
CAPACITY = 2 * 1024 * 1024
CPU = struct.Struct("<IB5HB")
# Independently audited oracle-results-2; root's oracle-main-results-1 is identical.
# A changed reference requires a new independent review, never --accept/--bless.
REFERENCE_HASHES = (
    "2a7874be086035f954d779465a5a9c1c024708ec7959724550a9fe5d86fb536e",
    "e6d2d488f6ee5d973259698c4b9968a9ffa78f4b2ee04a8ddce60b558f72b5cf",
    "af65f1f7f77aae4e206f5d97cf5e35df84c41db51d0e7b15d3a5e15668dfc703",
    "c0e58d84b8d1c5cc1a108ac215f5034eeb359ab764640bbee51296e88dfa9131",
    "b621f2f5826efaf3c181c9997f7443183e73340469c93d3b9eb8976e76018468",
    "8865b9a831f439d5c98dc97b41a0316ea00fcd318d4cb225dfaea28e797a0e88",
    "2304a413de28e7aed75a9abf9c653e69611ee85daffce0d16263d74cfb728287",
    "3fdc170557761d5b7977d4e6c22a00c9c3e9f864e51c33d6eaacb38bcecb4bd9",
    "930e3561de04f197b0e782cb5703bc5b118c702f1924966bed8f2745201a8188",
    "95543af887a9f63a8e5e8cb92aee0abd958ae7bbddf0942425f0251cadad98bb",
    "04f39816d6ec97aa85afb54735d3b9647c42d4fb530596d848cf2c03656cec46",
)


class Mismatch(ValueError):
    pass


def require(condition, message):
    if not condition:
        raise Mismatch(message)


@dataclass(frozen=True)
class Record:
    tag: int
    payload: bytes


def records(data, magic):
    require(data.startswith(magic), f"expected {magic!r} magic")
    require(len(data) <= CAPACITY, "capture exceeds 2 MiB capacity")
    offset = len(magic)
    result = []
    while offset < len(data):
        require(offset + 5 <= len(data), "truncated record header")
        tag, size = struct.unpack_from("<BI", data, offset)
        offset += 5
        require(size <= len(data) - offset, "truncated record payload")
        result.append(Record(tag, data[offset:offset + size]))
        offset += size
    require(result, "capture has no records")
    return result


def exclusions(case):
    stack = {1: 13, 2: 13, 4: 9, 5: 3, 6: 3}.get(case, 7)
    result = [(0x2000 - stack, 0x2000, 1)]
    if case >= 7:
        result.extend([(0, 3, 2), (0x12, 0x14, 2)])
    return result


def authored(case):
    stream = 0x91B401 if case == 10 else 0x91B400
    return b"".join(struct.pack("<II", address, len(data)) + data
                    for address, data in [
                        (0x91B012, struct.pack("<H", stream & 0xffff)),
                        (stream, bytes([0x8c, 0x08, 0x20, 0x11, 0x22, 0x5a]))])


def metadata(items, case, has_exclusions):
    require(items[0] == Record(0, struct.pack("<IB", case, int(case >= 7))),
            "missing/wrong case identity or original/authored family")
    offset = 1
    if case >= 7:
        require(offset < len(items) and items[offset] == Record(8, authored(case)),
                "wrong/missing authored pointer and instruction bytes")
        offset += 1
    if has_exclusions:
        payload = b"".join(struct.pack("<IIB", *span) for span in exclusions(case))
        require(offset < len(items) and items[offset] == Record(9, payload),
                "wrong/missing exact CPU exclusions")
        offset += 1
    require(offset < len(items), "metadata without observations")
    return items[offset:]


def cpu_fields(item):
    expected_size = CPU.size + (RAM_SIZE + 1 if item.tag in (1, 2, 5, 7) else 0)
    require(len(item.payload) == expected_size, "wrong CPU observation size")
    return CPU.unpack_from(item.payload)


def cpu_ram(item):
    cpu_fields(item)
    require(item.tag in (1, 2, 5, 7), "CPU observation has no RAM")
    require(item.payload[-1] == 0, "CPU g_fail is set")
    return item.payload[CPU.size:-1]


def word(ram, address):
    return struct.unpack_from("<H", ram, address)[0]


def qualified_equal(reference, actual, case, label):
    require(len(reference) == RAM_SIZE and len(actual) == RAM_SIZE,
            f"{label}: wrong RAM length")
    cursor = 0
    for start, end, _ in sorted(exclusions(case)) + [(RAM_SIZE, RAM_SIZE, 0)]:
        if reference[cursor:start] != actual[cursor:start]:
            address = next(i for i in range(cursor, start) if reference[i] != actual[i])
            raise Mismatch(f"{label}: RAM {address:05x}: CPU={reference[address]:02x}, "
                           f"Rust={actual[address]:02x}")
        cursor = end


def excluded_unchanged(seed, actual, case, label):
    for start, end, _ in exclusions(case):
        require(actual[start:end] == seed[start:end],
                f"{label}: Rust changed excluded bytes [{start:05x},{end:05x})")


def sha256(data):
    return hashlib.sha256(data).hexdigest()


def expected_stream(data, case):
    """Map selected real CPU boundaries to Rust semantics, with explicit reuse.

    No expected RAM is calculated from replacement gameplay. Redundant Rust
    boundaries reuse one CPU observation only where intervening operations do
    not write RAM. CPU-only phases and scratch store 0012 are not Rust events.
    """
    require(sha256(data) == REFERENCE_HASHES[case], "reference hash is not frozen audited PTOR1")
    source = metadata(records(data, b"PTOR1"), case, True)
    points = {}
    branches = {}
    stores = {}
    for item in source:
        require(item.tag in (1, 2, 3, 4, 5, 7), "unknown CPU record tag")
        if item.tag == 4:
            require(len(item.payload) == 12, "wrong CPU publication size")
            pc, address, before, after = struct.unpack("<IIHH", item.payload)
            require(address + 2 <= RAM_SIZE, "CPU publication out of range")
            require(pc not in stores, "duplicate CPU publication PC")
            stores[pc] = (address, before, after)
        else:
            pc = cpu_fields(item)[0]
            if item.tag == 3:
                require(pc not in branches, "duplicate CPU decision PC")
                branches[pc] = item
                continue
            require(pc not in points, "duplicate CPU observation PC")
            points[pc] = item
            if item.tag != 3:
                cpu_ram(item)
    result = []
    used_snapshots = set()
    used_publications = set()

    def snapshot(event, pc):
        require(pc in points, f"missing mapped CPU snapshot {pc:06x}")
        result.append(Record(2, struct.pack("<H", event) + cpu_ram(points[pc]) + b"\0"))
        used_snapshots.add(pc)

    def publication(event, pc):
        require(pc in stores, f"missing mapped CPU publication {pc:06x}")
        result.append(Record(3, struct.pack("<HIHH", event, *stores[pc])))
        used_publications.add(pc)

    def scalar(event, value):
        result.append(Record(4, struct.pack("<HH", event, value)))

    def carry(pc):
        require(pc in points, f"missing CPU callback result {pc:06x}")
        return cpu_fields(points[pc])[1] & 1

    if case <= 3:
        snapshot(1, 0x91f433)
        snapshot(2, 0x91f440)
        publication(1, 0x91f443)
        snapshot(3, 0x91f446)
        snapshot(20, 0x91f4dc)
        scalar(1, carry(0x91f476))
        snapshot(4, 0x91f476)
        snapshot(10, 0x91f4a0)
        if case in (1, 2):
            result.append(Record(5, struct.pack("<BH", 2, 0)))
            snapshot(27, 0x91deba)
            # Original CopyToSamusSuitPalette receives the source in X.
            result.append(Record(5, struct.pack("<BH", 1, cpu_fields(points[0x91dd5b])[3])))
            snapshot(28, 0x91dd5b)
            for index in range(16):
                publication(11, 0x91dd67 + 7 * index)
        snapshot(11, 0x91f467)
    elif case <= 6:
        snapshot(1, 0x91f468)
        if case == 4:
            snapshot(21, 0x91f543)
            publication(4, 0x91f56c)
            snapshot(22, 0x90cffa)
            publication(5, 0x91f592)
            publication(6, 0x91f595)
        else:
            snapshot(23, 0x91f88c)
            publication(7, 0x91f8a1 if case == 5 else 0x91f8a9)
        scalar(1, carry(0x91f476))
        snapshot(4, 0x91f476)
        snapshot(5, 0x91f47f)
        publication(2, 0x91f482)
        snapshot(6, 0x91f485)
        if case in (5, 6):
            snapshot(7, 0x91f485)
            snapshot(24, 0x91f8d3)
            publication(8, 0x91f923)
            # F490 is before the turning call; F49A is after index arithmetic.
            # Neither captures its return carry. Do not invent that comparison.
            snapshot(8, 0x91f49a)
            publication(3, 0x91f49d)
            snapshot(9, 0x91f4a0)
        snapshot(10, 0x91f4a1)
        snapshot(11, 0x91f4a1)
    else:
        snapshot(1, 0x9082dc)
        snapshot(25, 0x90852c)
        snapshot(26, 0x90848b)
        scalar(3, cpu_fields(points[0x90848e])[2])
        snapshot(30, 0x90848e)
        # This is the observed CPU branch target, not an emulated Rust branch.
        selected_match = 0x9084a3 in branches
        require(selected_match != (0x908497 in branches), "missing/ambiguous CPU equipment branch")
        scalar(5, int(selected_match))
        scalar(6, word(cpu_ram(points[0x90848e]), 0xa96) + (4 if selected_match else 3))
        publication(9, 0x9084ab if selected_match else 0x90849e)
        publication(10, 0x9084b1)
        snapshot(31, 0x9084b4)
        scalar(4, carry(0x908313))
        snapshot(32, 0x908313)
        snapshot(11, 0x908323)
    # Every gameplay publication and effect must have an explicit semantic map.
    ignored_publications = {0x90848e} if case >= 7 else set()
    if ignored_publications:
        require(0x90848e in stores and stores[0x90848e][0] == 0x12,
                "ignored CPU publication must be the command12 scratch word")
    require(set(stores) - used_publications == ignored_publications,
            "unmapped CPU gameplay publication")
    require(all(cpu_fields(item)[0] in used_snapshots for item in source if item.tag == 7),
            "unmapped CPU effect entry")
    return result, len([item for item in source if item.tag in (1, 2, 5, 7)])


def compare_case(reference, rust, case):
    expected, cpu_snapshot_count = expected_stream(reference, case)
    actual = metadata(records(rust, b"PTRS1"), case, True)
    require(len(actual) == len(expected),
            f"event count: expected {len(expected)}, actual {len(actual)}")
    seed = expected[0].payload[2:-1]
    counts = {"snapshots": 0, "publications": 0, "actual_scalars": 0,
              "palette_events": 0, "effect_snapshots": 0, "projected_snapshots": 0}
    for index, (want, got) in enumerate(zip(expected, actual)):
        require(got.tag == want.tag, f"event {index}: expected tag {want.tag}, actual {got.tag}")
        require(len(got.payload) == len(want.payload), f"event {index}: payload length differs")
        if got.tag == 2:
            event = struct.unpack_from("<H", want.payload)[0]
            label = f"event {index}, snapshot {event}"
            require(got.payload[:2] == want.payload[:2], f"{label}: wrong semantic ID/order")
            require(got.payload[-1] == 0, f"{label}: Rust g_fail is set")
            observed = got.payload[2:-1]
            qualified_equal(want.payload[2:-1], observed, case, label)
            excluded_unchanged(seed, observed, case, label)
            counts["snapshots"] += 1
            counts["effect_snapshots"] += int(20 <= event <= 28)
            counts["projected_snapshots"] += int(event in (27, 28))
        else:
            require(got.payload == want.payload,
                    f"event {index}, tag {got.tag}: expected {want.payload.hex()}, actual {got.payload.hex()}")
            counts[{3: "publications", 4: "actual_scalars", 5: "palette_events"}[got.tag]] += 1
    return {"case": case, "status": "PASS", "family": "authored_data" if case >= 7 else "original_data",
            "reference_sha256": sha256(reference), "rust_sha256": sha256(rust),
            "events": len(actual), **counts, "cpu_full_observations_available": cpu_snapshot_count,
            "exclusions": exclusions(case), "rust_excluded_bytes_unchanged": True}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--reference-dir", type=Path, required=True, help="frozen PTOR1 0.bin through 10.bin")
    parser.add_argument("--rust-dir", type=Path, required=True, help="actual PTRS1 0.bin through 10.bin")
    args = parser.parse_args()
    cases = []
    for case in range(11):
        try:
            reference = (args.reference_dir / f"{case}.bin").read_bytes()
            rust = (args.rust_dir / f"{case}.bin").read_bytes()
            cases.append(compare_case(reference, rust, case))
        except (OSError, ValueError, KeyError, struct.error) as error:
            cases.append({"case": case, "status": "FAIL", "error": str(error)})
    passed = all(case["status"] == "PASS" for case in cases)
    print(json.dumps({"status": "PASS" if passed else "FAIL", "cases": cases,
                      "scope": "11 actual Rust semantic traces against frozen original-CPU captures",
                      "limits": ["Qualified RAM equality only; exact recorded CPU exclusions retained.",
                                 "Every Rust excluded byte independently unchanged at every snapshot.",
                                 "Palette effect snapshots project owned Session encoding into a copy of RAM.",
                                 "No Rust CPU registers, PCs, stack simulation or 96-phase equivalence claimed.",
                                 "Cases 7..10 use authored data, not natural vanilla reachability.",
                                 "Capture provenance and source eligibility are reviewed separately."]}, indent=2))
    return 0 if passed else 1


if __name__ == "__main__":
    sys.exit(main())
