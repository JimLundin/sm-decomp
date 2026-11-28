import tree_sitter_rust
from ordered import FrozenList
from tree_sitter import Language, Node, Parser

from sm_tools.faults.core.selectable_faults import (
    RustFunction,
    RustSyntax,
    Span,
)

RUST = Language(tree_sitter_rust.language())


def span(node: Node | None) -> Span | None:
    return None if node is None else Span(node.start_byte, node.end_byte)


def child(node: Node | None, kind: str) -> Node | None:
    if node is None:
        return None
    return next((found for found in node.children if found.type == kind), None)


def function(source: bytes, node: Node) -> RustFunction:
    name = node.child_by_field_name("name")
    return RustFunction(
        Span(node.start_byte, node.end_byte),
        span(node.child_by_field_name("body")),
        ""
        if name is None
        else source[name.start_byte : name.end_byte].decode(),
        span(child(child(node, "function_modifiers"), "const")),
    )


def functions(source: bytes, root: Node) -> FrozenList[RustFunction]:
    found: list[RustFunction] = []
    pending = [root]
    while pending:
        node = pending.pop()
        if node.type == "function_item":
            found.append(function(source, node))
        pending.extend(reversed(node.children))
    return FrozenList(found)


def parse(source: bytes) -> RustSyntax:
    tree = Parser(RUST).parse(source)
    return RustSyntax(
        tree.root_node.has_error, functions(source, tree.root_node)
    )
