import hashlib
import re
from dataclasses import dataclass, field
from typing import Literal
lazy from collections.abc import Mapping, Sequence
lazy from pathlib import PurePath

from ordered import FrozenList, FrozenOrderedDict, FrozenOrderedSet

USAGE = """usage: gate

The local source gate: Quality's formatting, lint, dependency, crate-rule and doc-link checks, the
mechanical review findings of prereview, and each of these only when the branch, committed since
its base or in the working tree, changes its subject: the lane and clean-room tools' self-tests,
verify's and the tools' tests, the planted faults' manifest, the adjusted vectors' rule owners and
each changed domain's adjusted vectors.

A pass with nothing uncommitted records its key: the tree, the base's tree, and what the checks
ran with. A later gate of the same key says whose pass it reuses and runs only the review findings,
which read the commit messages the key leaves out.
SM_GATE_FRESH=1 runs the checks anyway.

Environment: SM_GATE_BASE (default origin/main; a stacked branch names the branch it builds on),
SM_RUST_BIN (the toolchain's bin), SM_GATE_PASSED (where passes are recorded)."""

LANE = "tools/src/sm_tools/lane"
LANE_TESTS = "tools/tests/lane"
CLEAN = "tools/cleanroom"
CHECK_MUTANTS = "tools/check_mutants.py"
ADJUSTED_VECTORS_RUN = f"{CLEAN}/adjusted_vectors_run.py"
PREREVIEW = "prereview"
PYTEST = FrozenList(("-m", "pytest", "-p", "no:cacheprovider", "-q"))
PROCESS = "tools/src/sm_tools/shell/process.py"
TOOLS_PROJECT = FrozenList((
    "tools/pyproject.toml",
    "tools/uv.lock",
    "tools/conftest.py",
))
SCRATCH = f"{LANE}/shell/scratch.py"


@dataclass(frozen=True, slots=True)
class Suite:
    subjects: FrozenList[str]
    runner: FrozenList[str] = field(default_factory=FrozenList[str])


SUITES = FrozenOrderedDict({
    f"{LANE_TESTS}/test_worktree.py": Suite(
        FrozenList((
            f"{LANE}/core/worktree.py",
            f"{LANE}/shell/worktree.py",
            PROCESS,
            SCRATCH,
        )),
        PYTEST,
    ),
    f"{LANE_TESTS}/test_gate.py": Suite(
        FrozenList((
            f"{LANE}/core/gate.py",
            f"{LANE}/shell/gate.py",
            f"{LANE}/core/changes.py",
            f"{LANE}/shell/changes.py",
            PROCESS,
            SCRATCH,
        )),
        PYTEST,
    ),
    f"{CLEAN}/test_adjusted_vectors_run.py": Suite(
        FrozenList((ADJUSTED_VECTORS_RUN, f"{CLEAN}/adjusted_vectors_fixtures"))
    ),
})
OWN_TESTS = FrozenList((
    *(
        f"{LANE}/{layer}/{tool}.py"
        for layer in ("core", "shell")
        for tool in ("played", "land", "prereview")
    ),
    f"{LANE_TESTS}/test_land.py",
    f"{LANE_TESTS}/test_prereview.py",
    f"{LANE}/__init__.py",
    f"{LANE}/core/__init__.py",
    f"{LANE}/shell/__init__.py",
    f"{LANE_TESTS}/__init__.py",
    f"{CLEAN}/__init__.py",
    f"{CLEAN}/adjusted_probe_tests.toml",
    f"{CLEAN}/adjusted_rule_owners",
    f"{CLEAN}/box_relay.py",
    f"{CLEAN}/test_box_relay.py",
    f"{CLEAN}/scan_lexicon",
    f"{CLEAN}/scan_allow.txt",
    f"{CLEAN}/allowed_deps.txt",
    f"{CLEAN}/scan_fixtures",
    f"{CLEAN}/rule_fixtures",
))
VERIFY_LIB = FrozenList((
    "crates",
    "scenarios",
    "docs/behaviours",
    "docs/architecture/domains.toml",
    "README.md",
    "Cargo.toml",
    "Cargo.lock",
    "rust-toolchain.toml",
))
FAULTS = FrozenList((
    "tools/mutants.toml",
    CHECK_MUTANTS,
    "crates/game/src/production",
    "crates/game/src/production.rs",
    "crates/verify/src/replay/played.rs",
    "scenarios/replay/index.toml",
    "scenarios/replay/baseline.toml",
))
OWNERS = FrozenList((
    f"{CLEAN}/adjusted_rule_owners",
    f"{CLEAN}/adjusted_probe_tests.toml",
    ADJUSTED_VECTORS_RUN,
    "scenarios/vectors",
    "docs/behaviours",
    "crates/clean",
))
DOMAIN_ROOTS = FrozenList(("docs/behaviours", "scenarios/vectors"))
UNINHERITED = FrozenList(("SM_VECTOR_FILE",))
INCLUDE = re.compile(
    r"""^\s*(\[\[)?\s*["']?include["']?\s*(=|\]\])""", re.MULTILINE
)
DOMAIN = re.compile(
    r"^docs/behaviours/([^/.]+)\.md$|^scenarios/vectors/([^/]+)\.toml$"
)
ENVIRONMENT = re.compile(
    r"^(RUST|CARGO|CLEANROOM_|CLIPPY_CONF_DIR$|SM_RUST_BIN$)"
)
UNTRUSTED_INDEX = re.compile(r"(?m)^[a-zS]")
TEST_FAILED = re.compile(r"^test .* FAILED$")
README_FAILED = re.compile(r"committed_readme_is_current .* FAILED")
README_STALE = (
    "README STALE: run cargo run -p tools --bin tools -- report --write"
)
NO_DENY = (
    "DENY SKIPPED: no cargo-deny here; CI's Quality runs cargo deny check bans"
)
CONFIG_NAMES = FrozenList(("config", "config.toml"))
LIMIT = 20
NOT_RUN = 127
TAIL = 5
FOLLOWING_ERROR = 7
FOLLOWING_PANIC = 2
type Kept = Literal["log", "head", "last", "after-errors", "verify-failures"]


@dataclass(frozen=True, slots=True)
class Matching:
    pattern: str
    limit: int = LIMIT


@dataclass(frozen=True, slots=True)
class Check:
    name: str
    failure: str
    commands: FrozenList[FrozenList[str]]
    kept: Kept | Matching = "log"
    environment: FrozenOrderedDict[str, str] = field(
        default_factory=FrozenOrderedDict[str, str]
    )
    without: FrozenList[str] = field(default_factory=FrozenList[str])


@dataclass(frozen=True, slots=True)
class Ran:
    check: Check
    status: int
    log: FrozenList[str]


@dataclass(frozen=True, slots=True)
class Tools:
    rust: PurePath
    deny: PurePath | None
    python: str
    path: str


@dataclass(frozen=True, slots=True)
class Changed:
    suites: FrozenList[str]
    verify_lib: bool
    faults: bool
    owners: bool
    domains: FrozenList[str]


@dataclass(frozen=True, slots=True)
class RanWith:
    rustc: str
    deny: str | None
    python: str
    version: str
    environment: FrozenOrderedDict[str, str]
    configurations: FrozenList[str]


@dataclass(frozen=True, slots=True)
class Homes:
    home: PurePath
    cargo: PurePath
    xdg: PurePath
    temp: PurePath


def faults_command(python: str) -> FrozenList[str]:
    return FrozenList((python, "-I", CHECK_MUTANTS, "validate"))


def adjusted_command(python: str, *arguments: str) -> FrozenList[str]:
    return FrozenList((python, "-I", ADJUSTED_VECTORS_RUN, *arguments))


def tool_command(python: str, *command: str) -> FrozenList[str]:
    return FrozenList((str(PurePath(python).parent / command[0]), *command[1:]))


def suite_command(suite: str, python: str) -> FrozenList[str]:
    return FrozenList((python, *SUITES[suite].runner, suite))


def unknown_tool_files() -> FrozenList[str]:
    known = (
        *(
            path
            for suite, run in SUITES.items()
            for path in (suite, *run.subjects)
        ),
        *OWN_TESTS,
    )
    return FrozenList((
        LANE,
        LANE_TESTS,
        CLEAN,
        *TOOLS_PROJECT,
        *(f":(exclude){path}" for path in known),
    ))


def suite_paths(suite: str) -> FrozenList[str]:
    return FrozenList((suite, *SUITES[suite].subjects))


def named_domains(paths: Sequence[str]) -> FrozenList[str]:
    named = FrozenOrderedSet(
        next(name for name in match.groups() if name)
        for path in paths
        if (match := DOMAIN.match(path))
    )
    return FrozenList(sorted(named))


def domain_files(name: str) -> FrozenList[str]:
    return FrozenList((
        f"docs/behaviours/{name}.md",
        f"scenarios/vectors/{name}.toml",
    ))


def configurations(top: PurePath, homes: Homes) -> FrozenList[PurePath]:
    home, xdg = homes.home, homes.xdg
    return FrozenList((
        homes.cargo / "config",
        homes.cargo / "config.toml",
        home / "rustfmt.toml",
        home / ".rustfmt.toml",
        xdg / "rustfmt/rustfmt.toml",
        xdg / "rustfmt/.rustfmt.toml",
        *(
            directory / name
            for start in (top, homes.temp)
            for directory in (start, *start.parents)
            for name in (
                ".cargo/config",
                ".cargo/config.toml",
                "rustfmt.toml",
                ".rustfmt.toml",
            )
        ),
    ))


def is_cargo_configuration(path: PurePath) -> bool:
    return path.name in CONFIG_NAMES


def includes_another(text: str) -> bool:
    return bool(INCLUDE.search(text))


def untracked_state(status: str, index: str, others: str) -> bool:
    return bool(status or UNTRUSTED_INDEX.search(index) or others.strip())


def ran_with_digest(ran: RanWith) -> str:
    lines = (
        ran.rustc,
        ran.deny or "no cargo-deny",
        ran.python,
        ran.version,
        *sorted(
            f"{name}={value}"
            for name, value in ran.environment.items()
            if ENVIRONMENT.match(name)
        ),
        *ran.configurations,
    )
    return hashlib.sha256("\n".join(lines).encode()).hexdigest()


def pass_key(tree: str, base_tree: str, ran_with: str) -> str:
    return f"{tree}-{base_tree}-{ran_with}"


def key_tree(key: str) -> str:
    return key.split("-", maxsplit=1)[0]


def check_environment(
    check: Check, environment: Mapping[str, str]
) -> FrozenOrderedDict[str, str]:
    dropped = (*check.without, *UNINHERITED)
    return FrozenOrderedDict(
        (name, value)
        for name, value in (dict(environment) | dict(check.environment)).items()
        if name not in dropped
    )


def after_errors(log: Sequence[str]) -> FrozenList[str]:
    return FrozenList(
        line
        for index, start in enumerate(log)
        if start.startswith("error")
        for line in log[index : index + FOLLOWING_ERROR]
    )[:LIMIT]


def verify_failures(log: Sequence[str]) -> FrozenList[str]:
    failed = (line for line in log if TEST_FAILED.match(line))
    panics = (
        line
        for index, start in enumerate(log)
        if "panicked at" in start
        for line in log[index : index + FOLLOWING_PANIC]
    )
    return FrozenList((*failed, *panics))[:LIMIT]


def kept(reason: Kept | Matching, log: Sequence[str]) -> FrozenList[str]:
    match reason:
        case Matching(pattern, limit):
            return FrozenList(line for line in log if re.search(pattern, line))[
                :limit
            ]
        case "head":
            return FrozenList(log[:LIMIT])
        case "last":
            return FrozenList(log[-1:])
        case "after-errors":
            return after_errors(log)
        case "verify-failures":
            return verify_failures(log)
        case "log":
            return FrozenList(log)


def report(ran: Ran) -> FrozenList[str]:
    shown = FrozenList(
        line for line in kept(ran.check.kept, ran.log) if line.strip()
    )
    return FrozenList((ran.check.failure, *(shown or ran.log[-TAIL:])))


def prereview_check(base: str, python: str) -> Check:
    return Check(
        PREREVIEW,
        "PREREVIEW FAIL",
        commands(tool_command(python, PREREVIEW, base)),
    )


def commands(*command: Sequence[str]) -> FrozenList[FrozenList[str]]:
    return FrozenList(FrozenList(line) for line in command)


def always(tools: Tools, changed: Changed) -> FrozenList[Check]:
    cargo = str(tools.rust / "cargo")
    on_path = FrozenOrderedDict({"PATH": tools.path})
    deny = tools.deny
    return FrozenList((
        Check(
            "fmt",
            "FMT FAIL",
            commands((cargo, "fmt", "--all", "--check")),
            Matching(r"^Diff in"),
        ),
        Check(
            "clippy",
            "CLIPPY FAIL",
            commands((
                cargo,
                "clippy",
                "--locked",
                "--workspace",
                "--all-targets",
                "--no-deps",
                "--",
                "-D",
                "warnings",
            )),
            Matching(r"^(error|warning)|-->", 10),
            without=FrozenList(("CLIPPY_CONF_DIR",)),
        ),
        *(
            (
                Check(
                    "deny",
                    "DENY FAIL",
                    commands((str(deny), "--locked", "check", "bans")),
                    "after-errors",
                    environment=on_path,
                ),
            )
            if deny
            else ()
        ),
        Check(
            "crate-rules",
            "CRATE RULES FAIL",
            commands((cargo, "checks", "crate-rules")),
        ),
        Check(
            "clean-room",
            "CLEAN ROOM FAIL",
            commands((cargo, "cleanroom", "check")),
        ),
        *(
            Check(
                suite,
                f"SELF TEST FAIL {suite}",
                commands(suite_command(suite, tools.python)),
                environment=FrozenOrderedDict({"CARGO": cargo}),
            )
            for suite in changed.suites
        ),
        Check(
            "doc-links",
            "DOC LINKS FAIL",
            commands((cargo, "checks", "doc-links")),
            "head",
        ),
    ))


def selected(tools: Tools, changed: Changed) -> FrozenList[Check]:
    cargo = str(tools.rust / "cargo")
    python = tools.python
    on_path = FrozenOrderedDict({"PATH": tools.path})
    verify = Check(
        "verify-lib",
        "VERIFY LIB FAIL",
        commands(
            (cargo, "test", "-p", "verify", "--lib"),
            (cargo, "test", "-p", "tools"),
        ),
        "verify-failures",
        environment=on_path,
    )
    faults = Check(
        "faults",
        f"FAULTS FAIL: {CHECK_MUTANTS} validate",
        commands(faults_command(python)),
        "last",
    )
    owners = Check(
        "owners",
        f"OWNERS FAIL: {ADJUSTED_VECTORS_RUN} --owners",
        commands(adjusted_command(python, "--owners")),
        Matching(r"^adjusted_vectors_run: "),
    )
    adjusted = Check(
        "adjusted",
        f"ADJUSTED FAIL: {ADJUSTED_VECTORS_RUN} --missing",
        commands(adjusted_command(python, "--missing", *changed.domains)),
        Matching(r" has no adjusted vector$"),
    )
    return FrozenList(
        check
        for check, wanted in (
            (verify, changed.verify_lib),
            (faults, changed.faults),
            (owners, changed.owners),
            (adjusted, bool(changed.domains)),
        )
        if wanted
    )


def checks(tools: Tools, changed: Changed) -> FrozenList[Check]:
    return FrozenList((*always(tools, changed), *selected(tools, changed)))


def skipped(changed: Changed) -> FrozenList[str]:
    return FrozenList(
        name
        for name, ran in (
            ("SELF-TESTS", bool(changed.suites)),
            ("VERIFY-LIB", changed.verify_lib),
            ("FAULTS", changed.faults),
            ("OWNERS", changed.owners),
            ("ADJUSTED", bool(changed.domains)),
        )
        if not ran
    )


def in_order(found: Sequence[Ran], prereview: Ran) -> FrozenList[Ran]:
    after_links = (
        next(
            index
            for index, ran in enumerate(found)
            if ran.check.name == "doc-links"
        )
        + 1
    )
    return FrozenList((*found[:after_links], prereview, *found[after_links:]))


def ran_lines(ran: Ran, *, deny: bool) -> FrozenList[str]:
    return FrozenList((
        *((NO_DENY,) if ran.check.name == "crate-rules" and not deny else ()),
        *(report(ran) if ran.status else ()),
        *(
            (README_STALE,)
            if ran.check.name == "verify-lib"
            and any(README_FAILED.search(line) for line in ran.log)
            else ()
        ),
    ))


def results(
    ran: Sequence[Ran], changed: Changed, base: str, *, deny: bool
) -> FrozenList[str]:
    return FrozenList((
        *(line for each in ran for line in ran_lines(each, deny=deny)),
        *(
            f"{name} SKIPPED: no change to its subject since {base}"
            for name in skipped(changed)
        ),
    ))


def failed(ran: Sequence[Ran]) -> bool:
    return any(each.status != 0 for each in ran)


def reused(record: str) -> str:
    return (
        f"GATE REUSED: {record.strip()} passed this tree over the same base, "
        "with the same tools"
    )


def unresolved(base: str) -> str:
    return (
        f"GATE: the base {base} is not a commit here; set SM_GATE_BASE to the"
        " one the branch builds on"
    )
