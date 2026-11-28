lazy from pathlib import Path

import pytest
from ordered import FrozenList

from sm_tools.lane.core.prereview import (
    HINT,
    Added,
    Mention,
    added_lines,
    backticked_paths,
    line_findings,
    message_findings,
    path_findings,
)
from sm_tools.lane.shell.prereview import findings, run
from sm_tools.shell.process import git_out

TOP = FrozenList(("docs", "tools", "AGENTS.md"))
LONG = "word " * 25
FIXTURE = FrozenList((
    "-c",
    "user.name=fixture",
    "-c",
    "user.email=fixture@invalid",
))


def paths(text: str) -> FrozenList[str]:
    return FrozenList(mention.path for mention in backticked_paths(text, TOP))


def test_a_dash_is_refused() -> None:
    assert line_findings("a.md:1", "one — two") == ("a.md:1: a dash",)
    assert line_findings("a.md:1", "1\u20132") == ("a.md:1: a dash",)
    assert line_findings("a.md:1", "a plain - hyphen") == ()


def test_a_line_over_100_columns_is_refused() -> None:
    assert line_findings("a.md:3", "x" * 100) == ()
    assert line_findings("a.md:3", "x" * 101) == (
        "a.md:3: 101 columns, over 100",
    )


def test_table_rows_and_link_only_lines_may_be_long() -> None:
    url = "https://example.invalid/" + "x" * 100
    lines = (
        f"| {LONG} | {LONG} |",
        f"[{LONG}]({url})",
        f"- [{LONG}](../{'x' * 100}.md).",
        url,
        f"<{url}>",
        f"[ref]: {url}",
    )
    assert [line for line in lines if line_findings("a.md:1", line)] == []
    assert len(line_findings("a.md:1", f"See [{LONG}]({url})")) == 1


def test_a_dash_in_a_commit_message_is_refused() -> None:
    assert message_findings("abc", "area: title\n\nBody — aside.\n") == (
        "commit abc: a dash in its message",
    )
    assert message_findings("abc", "area: title [exact]\n\nBody.\n") == ()


def test_only_added_lines_are_read_with_their_new_numbers() -> None:
    diff = (
        "diff --git a/d.md b/d.md\n--- a/d.md\n+++ b/d.md\n@@ -3,0 +4,2 @@\n"
        "+four\n+five\n@@ -9 +11 @@\n-old\n+eleven\n"
    )
    assert added_lines(diff) == (
        Added("d.md", 4, "four"),
        Added("d.md", 5, "five"),
        Added("d.md", 11, "eleven"),
    )


def test_repository_paths_are_read_from_spans_and_commands() -> None:
    text = (
        "Run `bash tools/a.sh --x` and read `docs/b.md#part`, `AGENTS.md`,"
        " `tools/c.rs:12`, `docs/d/`."
    )
    assert paths(text) == (
        "tools/a.sh",
        "docs/b.md",
        "AGENTS.md",
        "tools/c.rs",
        "docs/d",
    )


def test_placeholders_globs_temp_and_urls_are_not_paths() -> None:
    text = (
        "`$TEMP/docs/x` `docs/*.md` `docs/<name>.md` `docs/{a,b}`"
        " `https://docs/x` `~/docs/x`"
    )
    assert paths(text) == ()


def test_words_outside_the_repository_top_are_not_paths() -> None:
    text = "`origin/main` `gate.sh` `crates/x.rs` `docs` `ridley/flyby`"
    assert paths(text) == ()


def test_fenced_blocks_are_skipped() -> None:
    assert paths("```sh\necho `tools/x.sh`\n```\n`tools/y.sh`\n") == (
        "tools/y.sh",
    )


def test_front_matter_and_tilde_fences_are_skipped() -> None:
    text = (
        "---\nname: `tools/x.sh`\n---\n~~~\n`tools/y.sh`\n```\n`tools/z.sh`\n"
        "~~~\n`tools/w.sh`\n"
    )
    assert backticked_paths(text, TOP) == (Mention(9, "tools/w.sh"),)


def test_quotes_and_possessives_are_stripped() -> None:
    assert paths("`tools/a.sh's` `'tools/b.sh'` `\"docs/c.md\".`") == (
        "tools/a.sh",
        "tools/b.sh",
        "docs/c.md",
    )


def test_a_path_marked_planned_is_not_read() -> None:
    text = (
        "`tools/later.py` (planned) and `tools/now.py`(planned),"
        " `tools/here.py`"
    )
    assert paths(text) == ("tools/here.py",)


def test_a_missing_path_is_refused_with_its_line() -> None:
    found = backticked_paths("`tools/here.sh`\nsee `tools/gone.sh`\n", TOP)
    assert path_findings("docs/a.md", found, ("tools/gone.sh",)) == (
        "docs/a.md:2: `tools/gone.sh` does not exist" + HINT,
    )
    alone = backticked_paths("`tools/gone.sh`\n", TOP)
    assert path_findings("docs/a.md", alone, ()) == ()


@pytest.fixture
def repo(tmp_path: Path) -> Path:
    (tmp_path / "docs").mkdir()
    (tmp_path / "tools").mkdir()
    _ = (tmp_path / ".gitignore").write_text("/tools/*.inputs\n")
    _ = (tmp_path / "tools/run.sh").write_text("")
    _ = (tmp_path / "docs/a.md").write_text(
        f"Old {LONG}{LONG}line — kept.\nRun `tools/run.sh`.\n"
    )
    _ = git_out("init", "-q", "-b", "main", cwd=tmp_path)
    commit(tmp_path, "base: an invented tree — with a dash")
    _ = git_out("checkout", "-q", "-b", "lane", cwd=tmp_path)
    return tmp_path


def commit(repo: Path, message: str) -> None:
    _ = git_out("add", "-A", cwd=repo)
    _ = git_out(
        *FIXTURE, "commit", "-q", "--allow-empty", "-m", message, cwd=repo
    )


def append(path: Path, text: str) -> None:
    _ = path.write_text(
        path.read_text(encoding="utf-8") + text, encoding="utf-8"
    )


def test_what_main_holds_is_not_the_branch_s_finding(repo: Path) -> None:
    commit(repo, "lane: nothing changes [exact]")
    assert findings(repo, "main") == ()


def test_the_branch_s_messages_and_changed_lines_are_refused(
    repo: Path,
) -> None:
    append(repo / "docs/a.md", f"New {LONG}{LONG}\n")
    commit(repo, "lane: a title [exact]\n\nA body — with an aside.")
    _ = (repo / "docs/new.md").write_text("Untracked \u2013 range.\n")
    head = git_out("rev-parse", "--short", "HEAD", cwd=repo).strip()
    assert sorted(findings(repo, "main")) == sorted((
        f"commit {head}: a dash in its message",
        "docs/a.md:3: 254 columns, over 100",
        "docs/new.md:1: a dash",
    ))


def test_a_renamed_file_counts_only_its_edited_lines(repo: Path) -> None:
    _ = (repo / "docs/a.md").rename(repo / "docs/moved.md")
    append(repo / "docs/moved.md", f"New {LONG}{LONG}\n")
    commit(repo, "lane: a.md moves [exact]")
    assert findings(repo, "main") == ("docs/moved.md:3: 254 columns, over 100",)


def test_long_lines_in_code_blocks_and_front_matter_may_stay(
    repo: Path,
) -> None:
    _ = (repo / "docs/c.md").write_text(
        f"---\ndescription: {LONG}{LONG}\n---\n```\n{LONG}{LONG}\n```\n{LONG}{LONG}\n"
    )
    commit(repo, "lane: c.md [exact]")
    assert findings(repo, "main") == ("docs/c.md:7: 250 columns, over 100",)


def test_a_git_failure_is_a_clear_error(
    repo: Path,
    monkeypatch: pytest.MonkeyPatch,
    capsys: pytest.CaptureFixture[str],
) -> None:
    monkeypatch.chdir(repo)
    status = run(("no-such-base",))
    err = capsys.readouterr().err
    assert status == 2
    assert err.startswith(
        r"prereview: git merge-base no-such-base HEAD failed: "
    )
    assert "Traceback" not in err


def test_a_removed_file_a_doc_names_is_refused_unless_ignored(
    repo: Path,
) -> None:
    (repo / "tools/run.sh").unlink()
    _ = (repo / "docs/b.md").write_text("Read `tools/any.inputs`.\n")
    commit(repo, "lane: run.sh is gone [exact]")
    assert findings(repo, "main") == (
        "docs/a.md:2: `tools/run.sh` does not exist" + HINT,
    )
