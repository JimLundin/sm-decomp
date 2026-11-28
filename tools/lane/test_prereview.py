import os
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import prereview

TOP = {"docs", "tools", "AGENTS.md"}
LONG = "word " * 25


def nothing_missing(paths: object) -> set[str]:
    return set()


class Lines(unittest.TestCase):
    def test_a_dash_is_refused(self) -> None:
        self.assertEqual(prereview.line_findings("a.md:1", "one — two"), ["a.md:1: a dash"])
        self.assertEqual(prereview.line_findings("a.md:1", "1–2"), ["a.md:1: a dash"])
        self.assertEqual(prereview.line_findings("a.md:1", "a plain - hyphen"), [])

    def test_a_line_over_100_columns_is_refused(self) -> None:
        self.assertEqual(prereview.line_findings("a.md:3", "x" * 100), [])
        self.assertEqual(prereview.line_findings("a.md:3", "x" * 101), ["a.md:3: 101 columns, over 100"])

    def test_table_rows_and_link_only_lines_may_be_long(self) -> None:
        url = "https://example.invalid/" + "x" * 100
        for line in (f"| {LONG} | {LONG} |", f"[{LONG}]({url})", f"- [{LONG}](../{'x' * 100}.md).", url,
                     f"<{url}>", f"[ref]: {url}"):
            with self.subTest(line=line):
                self.assertEqual(prereview.line_findings("a.md:1", line), [])
        self.assertEqual(len(prereview.line_findings("a.md:1", f"See [{LONG}]({url})")), 1)

    def test_a_dash_in_a_commit_message_is_refused(self) -> None:
        self.assertEqual(prereview.message_findings("abc", "area: title\n\nBody — aside.\n"),
                         ["commit abc: a dash in its message"])
        self.assertEqual(prereview.message_findings("abc", "area: title [exact]\n\nBody.\n"), [])

    def test_only_added_lines_are_read_with_their_new_numbers(self) -> None:
        diff = ("diff --git a/d.md b/d.md\n--- a/d.md\n+++ b/d.md\n@@ -3,0 +4,2 @@\n+four\n+five\n"
                "@@ -9 +11 @@\n-old\n+eleven\n")
        self.assertEqual(list(prereview.added_lines(diff)), [("d.md", 4, "four"), ("d.md", 5, "five"),
                                                             ("d.md", 11, "eleven")])


class Paths(unittest.TestCase):
    def paths(self, text: str) -> list[str]:
        return [path for _, path in prereview.backticked_paths(text, TOP)]

    def test_repository_paths_are_read_from_spans_and_commands(self) -> None:
        text = "Run `bash tools/a.sh --x` and read `docs/b.md#part`, `AGENTS.md`, `tools/c.rs:12`, `docs/d/`."
        self.assertEqual(self.paths(text), ["tools/a.sh", "docs/b.md", "AGENTS.md", "tools/c.rs", "docs/d"])

    def test_placeholders_globs_temp_and_urls_are_not_paths(self) -> None:
        self.assertEqual(self.paths("`$TEMP/docs/x` `docs/*.md` `docs/<name>.md` `docs/{a,b}` `https://docs/x` "
                                    "`~/docs/x`"), [])

    def test_words_outside_the_repository_top_are_not_paths(self) -> None:
        self.assertEqual(self.paths("`origin/main` `gate.sh` `crates/x.rs` `docs` `ridley/flyby`"), [])

    def test_fenced_blocks_are_skipped(self) -> None:
        self.assertEqual(self.paths("```sh\necho `tools/x.sh`\n```\n`tools/y.sh`\n"), ["tools/y.sh"])

    def test_front_matter_and_tilde_fences_are_skipped(self) -> None:
        text = "---\nname: `tools/x.sh`\n---\n~~~\n`tools/y.sh`\n```\n`tools/z.sh`\n~~~\n`tools/w.sh`\n"
        self.assertEqual(list(prereview.backticked_paths(text, TOP)), [(9, "tools/w.sh")])

    def test_quotes_and_possessives_are_stripped(self) -> None:
        self.assertEqual(self.paths("`tools/a.sh's` `'tools/b.sh'` `\"docs/c.md\".`"),
                         ["tools/a.sh", "tools/b.sh", "docs/c.md"])

    def test_a_path_marked_planned_is_not_read(self) -> None:
        self.assertEqual(self.paths("`tools/later.py` (planned) and `tools/now.py`(planned), `tools/here.py`"),
                         ["tools/here.py"])

    def test_a_missing_path_is_refused_with_its_line(self) -> None:
        def missing(paths: object) -> set[str]:
            return {"tools/gone.sh"}
        self.assertEqual(prereview.path_findings("docs/a.md", "`tools/here.sh`\nsee `tools/gone.sh`\n", TOP, missing),
                         ["docs/a.md:2: `tools/gone.sh` does not exist" + prereview.HINT])
        self.assertEqual(prereview.path_findings("docs/a.md", "`tools/gone.sh`\n", TOP, nothing_missing), [])


class Branch(unittest.TestCase):
    def setUp(self) -> None:
        self.repo = Path(self.enterContext(tempfile.TemporaryDirectory(
            prefix="prereview-test-", dir=os.environ.get("TEMP") or os.environ.get("TMPDIR"))))
        (self.repo / "docs").mkdir()
        (self.repo / "tools").mkdir()
        (self.repo / ".gitignore").write_text("/tools/*.inputs\n")
        (self.repo / "tools/run.sh").write_text("")
        (self.repo / "docs/a.md").write_text(f"Old {LONG}{LONG}line — kept.\nRun `tools/run.sh`.\n")
        self.git("init", "-q", "-b", "main")
        self.commit("base: an invented tree — with a dash")
        self.git("checkout", "-q", "-b", "lane")

    def git(self, *args: str) -> str:
        return subprocess.run(["git", *args], cwd=self.repo, check=True, capture_output=True, text=True).stdout

    def commit(self, message: str) -> None:
        self.git("add", "-A")
        self.git("-c", "user.name=fixture", "-c", "user.email=fixture@invalid", "commit", "-q", "--allow-empty",
                 "-m", message)

    def findings(self) -> list[str]:
        return prereview.findings(self.repo, "main")

    def test_what_main_holds_is_not_the_branch_s_finding(self) -> None:
        self.commit("lane: nothing changes [exact]")
        self.assertEqual(self.findings(), [])

    def test_the_branch_s_messages_and_changed_lines_are_refused(self) -> None:
        with (self.repo / "docs/a.md").open("a") as doc:
            doc.write(f"New {LONG}{LONG}\n")
        self.commit("lane: a title [exact]\n\nA body — with an aside.")
        (self.repo / "docs/new.md").write_text("Untracked – range.\n")
        self.assertEqual(sorted(self.findings()), sorted([
            "commit " + self.git("rev-parse", "--short", "HEAD").strip() + ": a dash in its message",
            "docs/a.md:3: 254 columns, over 100", "docs/new.md:1: a dash"]))

    def test_a_renamed_file_counts_only_its_edited_lines(self) -> None:
        (self.repo / "docs/a.md").rename(self.repo / "docs/moved.md")
        with (self.repo / "docs/moved.md").open("a") as doc:
            doc.write(f"New {LONG}{LONG}\n")
        self.commit("lane: a.md moves [exact]")
        self.assertEqual(self.findings(), ["docs/moved.md:3: 254 columns, over 100"])

    def test_long_lines_in_code_blocks_and_front_matter_may_stay(self) -> None:
        (self.repo / "docs/c.md").write_text(f"---\ndescription: {LONG}{LONG}\n---\n```\n{LONG}{LONG}\n```\n"
                                             f"{LONG}{LONG}\n")
        self.commit("lane: c.md [exact]")
        self.assertEqual(self.findings(), ["docs/c.md:7: 250 columns, over 100"])

    def test_a_git_failure_is_a_clear_error(self) -> None:
        result = subprocess.run([sys.executable, "-I", str(Path(prereview.__file__)), "no-such-base"], cwd=self.repo,
                                capture_output=True, text=True, check=False)
        self.assertEqual(result.returncode, 2)
        self.assertRegex(result.stderr, r"^prereview: git merge-base no-such-base HEAD failed: ")
        self.assertNotIn("Traceback", result.stderr)

    def test_a_removed_file_a_doc_names_is_refused_unless_ignored(self) -> None:
        (self.repo / "tools/run.sh").unlink()
        (self.repo / "docs/b.md").write_text("Read `tools/any.inputs`.\n")
        self.commit("lane: run.sh is gone [exact]")
        self.assertEqual(self.findings(), ["docs/a.md:2: `tools/run.sh` does not exist" + prereview.HINT])


if __name__ == "__main__":
    unittest.main()
