from collections import defaultdict
from collections.abc import Sequence
from dataclasses import dataclass
from os.path import commonprefix

import tree_sitter_rust
from tree_sitter import Language, Node, Parser, Query, QueryCursor

RUST = Language(tree_sitter_rust.language())
CONST_FUNCTIONS = Query(
    RUST,
    '(function_item (function_modifiers "const" @qualifier) name: (identifier) @name) @function',
)


@dataclass(frozen=True)
class Replacement:
    ordinal: int
    start: int
    before: bytes
    after: bytes


def instrument_file(
    source: bytes, replacements: Sequence[Replacement], callers: Sequence[str]
) -> bytes:
    tree = Parser(RUST).parse(source)
    if tree.root_node.has_error:
        raise ValueError("could not parse the original Rust source")
    variants: dict[Node, list[tuple[int, bytes]]] = defaultdict(list)
    for edit in replacements:
        prefix = len(commonprefix((edit.before, edit.after)))
        before, after = edit.before[prefix:], edit.after[prefix:]
        suffix = len(commonprefix((before[::-1], after[::-1])))
        left, right = (
            edit.start + prefix,
            edit.start + len(edit.before) - suffix,
        )
        node = tree.root_node.descendant_for_byte_range(left, right)
        while node is not None and node.type != "function_item":
            node = node.parent
        if node is None:
            raise ValueError(
                f"fault {edit.ordinal} must change a Rust function body"
            )
        body = node.child_by_field_name("body")
        if body is None or not body.start_byte < left <= right < body.end_byte:
            raise ValueError(
                f"fault {edit.ordinal} changes the function signature"
            )
        modified = (
            source[body.start_byte : left]
            + after[: len(after) - suffix]
            + source[right : body.end_byte]
        )
        variants[node].append((edit.ordinal, modified))

    edits: list[tuple[int, int, bytes]] = []
    remaining = list(callers)
    for _, capture in QueryCursor(CONST_FUNCTIONS).matches(tree.root_node):
        name = capture["name"][0]
        caller = source[name.start_byte : name.end_byte].decode()
        if caller in callers or capture["function"][0] in variants:
            qualifier = capture["qualifier"][0]
            edits.append((qualifier.start_byte, qualifier.end_byte, b""))
            if caller in callers:
                remaining.remove(caller)
    if remaining:
        raise ValueError(f"missing const caller(s): {remaining}")
    for function, alternatives in variants.items():
        body = function.child_by_field_name("body")
        assert body is not None
        original = source[body.start_byte : body.end_byte]
        branches = [
            f"{ordinal} => ".encode() + modified + b","
            for ordinal, modified in alternatives
        ]
        guarded = (
            b"{ match ::gamma_rt::active() { "
            + b"\n".join(branches)
            + b" _ => "
            + original
            + b" } }"
        )
        edits.append((body.start_byte, body.end_byte, guarded))
    output = source
    boundary = len(source)
    for left, right, replacement in sorted(edits, reverse=True):
        if right > boundary:
            raise ValueError(
                "overlapping Rust function bodies or const callers"
            )
        boundary = left
        output = output[:left] + replacement + output[right:]
    return output
