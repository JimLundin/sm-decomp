from dataclasses import dataclass
from pathlib import PurePosixPath
lazy from collections.abc import Mapping, Sequence

from ordered import FrozenList, FrozenOrderedDict, FrozenOrderedSet

from sm_tools.faults.core.check_mutants import (
    Manifest,
    require_unique_anchor,
    source_path,
)

SELECTOR_VERSION = "0.8.0"
SELECTOR = f"cargo-gamma-rt v{SELECTOR_VERSION} []"
SELECTOR_DEPENDENCY = (
    f'\n[dependencies.cargo-gamma-rt]\nversion = "={SELECTOR_VERSION}"\n'
)


@dataclass(frozen=True, slots=True)
class Span:
    start: int
    end: int


@dataclass(frozen=True, slots=True)
class RustFunction:
    span: Span
    body: Span | None
    name: str
    const: Span | None


@dataclass(frozen=True, slots=True)
class RustSyntax:
    has_error: bool
    functions: FrozenList[RustFunction]


@dataclass(frozen=True, slots=True)
class Replacement:
    ordinal: int
    start: int
    before: bytes
    after: bytes


@dataclass(frozen=True, slots=True)
class Edit:
    start: int
    end: int
    text: bytes


def shared_prefix(left: bytes, right: bytes) -> int:
    return next(
        (
            index
            for index, (first, second) in enumerate(
                zip(left, right, strict=False)
            )
            if first != second
        ),
        min(len(left), len(right)),
    )


def innermost(syntax: RustSyntax, left: int, right: int) -> RustFunction | None:
    return max(
        (
            function
            for function in syntax.functions
            if function.span.start <= left < function.span.end
            and right <= function.span.end
        ),
        key=lambda function: function.span.start,
        default=None,
    )


def variant(
    source: bytes, syntax: RustSyntax, edit: Replacement
) -> tuple[RustFunction, Span, bytes]:
    prefix = shared_prefix(edit.before, edit.after)
    before, after = edit.before[prefix:], edit.after[prefix:]
    suffix = shared_prefix(before[::-1], after[::-1])
    left, right = edit.start + prefix, edit.start + len(edit.before) - suffix
    function = innermost(syntax, left, right)
    if function is None:
        message = f"fault {edit.ordinal} must change a Rust function body"
        raise ValueError(message)
    body = function.body
    if body is None or not body.start < left <= right < body.end:
        message = f"fault {edit.ordinal} changes the function signature"
        raise ValueError(message)
    modified = (
        source[body.start : left]
        + after[: len(after) - suffix]
        + source[right : body.end]
    )
    return function, body, modified


def guarded(original: bytes, alternatives: Sequence[bytes]) -> bytes:
    return (
        b"{ match ::gamma_rt::active() { "
        + b"\n".join(alternatives)
        + b" _ => "
        + original
        + b" } }"
    )


def instrument_file(
    source: bytes,
    syntax: RustSyntax,
    replacements: Sequence[Replacement],
    callers: Sequence[str],
) -> bytes:
    if syntax.has_error:
        message = "could not parse the original Rust source"
        raise ValueError(message)
    variants = FrozenList(
        variant(source, syntax, edit) for edit in replacements
    )
    changed = FrozenOrderedDict(
        (function, body) for function, body, _ in variants
    )
    qualifiers = FrozenList(
        function
        for function in syntax.functions
        if function.const is not None
        and (function.name in callers or function in changed)
    )
    found = FrozenOrderedSet(function.name for function in qualifiers)
    remaining = [caller for caller in callers if caller not in found]
    if remaining:
        message = f"missing const caller(s): {remaining}"
        raise ValueError(message)
    edits = FrozenList((
        *(
            Edit(function.const.start, function.const.end, b"")
            for function in qualifiers
            if function.const is not None
        ),
        *(
            Edit(
                body.start,
                body.end,
                guarded(
                    source[body.start : body.end],
                    FrozenList(
                        f"{edit.ordinal} => ".encode() + modified + b","
                        for edit, (owner, _, modified) in zip(
                            replacements, variants, strict=True
                        )
                        if owner == function
                    ),
                ),
            )
            for function, body in changed.items()
        ),
    ))
    output = source
    boundary = len(source)
    for edit in sorted(
        edits, key=lambda edit: (edit.start, edit.end, edit.text), reverse=True
    ):
        if edit.end > boundary:
            message = "overlapping Rust function bodies or const callers"
            raise ValueError(message)
        boundary = edit.start
        output = output[: edit.start] + edit.text + output[edit.end :]
    return output


def instrumented_files(loaded: Manifest) -> FrozenList[str]:
    return FrozenList(
        FrozenOrderedSet((
            *(entry.file for entry in loaded.faults),
            *loaded.callers,
        ))
    )


def selectable_sources(
    files: Mapping[PurePosixPath, bytes],
    syntax: Mapping[str, RustSyntax],
    loaded: Manifest,
) -> FrozenOrderedDict[PurePosixPath, bytes]:
    changes = FrozenOrderedDict(
        (
            file,
            FrozenList(
                Replacement(
                    ordinal,
                    require_unique_anchor(entry, files[source_path(file)]),
                    entry.before.encode(),
                    entry.after.encode(),
                )
                for ordinal, entry in enumerate(loaded.faults, 1)
                if entry.file == file
            ),
        )
        for file in instrumented_files(loaded)
    )
    instrumented = FrozenOrderedDict(
        (
            source_path(file),
            instrument_file(
                files[source_path(file)],
                syntax[file],
                replacements,
                loaded.callers.get(file, FrozenList[str]()),
            ),
        )
        for file, replacements in changes.items()
    )
    crates = sorted(
        FrozenOrderedSet(
            PurePosixPath(entry.file).parts[1] for entry in loaded.faults
        )
    )
    manifests = FrozenOrderedDict(
        (path, files[path] + SELECTOR_DEPENDENCY.encode())
        for path in (
            PurePosixPath("crates", crate, "Cargo.toml") for crate in crates
        )
    )
    return FrozenOrderedDict(files) | instrumented | manifests


def dependencies(output: str) -> FrozenOrderedSet[str]:
    return FrozenOrderedSet(
        line.removesuffix(" (*)") for line in output.splitlines()
    )


def check_selector_dependencies(
    root: str,
    tree: str,
    normal: FrozenOrderedSet[str],
    selectable: FrozenOrderedSet[str],
) -> None:
    copied = FrozenOrderedSet(
        package.replace(f"({root}/", f"({tree}/") for package in normal
    )
    if any(package.startswith("cargo-gamma-rt v") for package in copied):
        message = "normal replay already links the fault selector"
        raise ValueError(message)
    if selectable != copied | FrozenOrderedSet((SELECTOR,)):
        message = "the selectable build changed another dependency or feature"
        raise ValueError(message)
