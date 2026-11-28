from pathlib import Path

import pytest
from ordered import FrozenList, FrozenOrderedSet

from sm_tools.ci.core.run_tests import written

OPTION = "--results"
OUTCOMES = FrozenList((
    "passed",
    "failed",
    "skipped",
    "error",
    "xfailed",
    "xpassed",
))
FAILURES = FrozenList(("failed", "error", "xpassed"))


def pytest_addoption(parser: pytest.Parser) -> None:
    parser.addoption(OPTION, help="the file the module's counts are written to")


def reported(session: pytest.Session, outcome: str) -> FrozenOrderedSet[str]:
    reporter: object = session.config.pluginmanager.get_plugin(
        "terminalreporter"
    )
    if not isinstance(reporter, pytest.TerminalReporter):
        return FrozenOrderedSet()
    reports: list[object] = reporter.stats.get(outcome, [])
    return FrozenOrderedSet(
        report.nodeid
        for report in reports
        if isinstance(report, pytest.TestReport | pytest.CollectReport)
    )


def pytest_sessionfinish(session: pytest.Session) -> None:
    option: object = session.config.getoption(OPTION)
    if not isinstance(option, str):
        return
    ran = FrozenOrderedSet(
        node for outcome in OUTCOMES for node in reported(session, outcome)
    )
    failed = FrozenList(
        node for outcome in FAILURES for node in reported(session, outcome)
    )
    skipped = reported(session, "skipped") - FrozenOrderedSet(failed)
    text = written(len(ran), len(skipped), failed)
    _ = Path(option).write_text(text, encoding="utf-8")
