"""Per-commit progress feed (requested by Jim, 2026-10-25).

For each commit in BEFORE..AFTER, measure the review scoreboard straight from
Git blobs (no checkout, no build) and print one Markdown block per commit with
the change against its parent. Checks cannot be run here (they need the ROM),
so the count comes from the commit's own `Score:` line and is marked reported.

Usage: python .github/scripts/progress_feed.py BEFORE AFTER > comment.md
"""
import re
import subprocess
import sys

SRC = "sm/src/"
REPO = "https://github.com/JimLundin/sm-decomp"
SOURCE_PREFIXES = ("sm/", "tools/", "movies/", ".github/")
B_QUEUES = {"B1": ["sm_90", "sm_81", "sm_80", "sm_82"], "B2": ["sm_88", "sm_8b"],
            "B3": ["sm_89", "sm_a0", "sm_8f", "sm_b2"]}


def git(*args):
    return subprocess.run(["git", *args], check=True, capture_output=True).stdout


def measure(sha):
    names = git("ls-tree", "-r", "--name-only", sha, "--", SRC).decode().split("\n")
    names = [n for n in names if n.endswith(".rs")]
    stdin = "".join(f"{sha}:{n}\n" for n in names).encode()
    out = subprocess.run(["git", "cat-file", "--batch"], input=stdin, check=True,
                         capture_output=True).stdout
    m = {"production": 0, "verification": 0, "legacy": 0, "unsafe": 0, "banks": 0}
    pos = 0
    for name in names:
        header_end = out.index(b"\n", pos)
        size = int(out[pos:header_end].split()[2])
        body = out[header_end + 1:header_end + 1 + size]
        pos = header_end + 1 + size + 1
        lines = body.count(b"\n") + (1 if body and not body.endswith(b"\n") else 0)
        base = name.rsplit("/", 1)[-1]
        if re.search(r"/(verification|verify)/", name) or base.startswith("verify"):
            m["verification"] += lines
        else:
            m["production"] += lines
        if re.fullmatch(r"sm/src/game/sm_[0-9a-f]+\.rs", name):
            m["legacy"] += lines
            m["banks"] += 1
        m["unsafe"] += sum(1 for line in body.split(b"\n") if b"unsafe" in line)
    return m


def segment_rows(sha):
    try:
        text = git("show", f"{sha}:movies/segments.toml").decode()
    except subprocess.CalledProcessError:
        return 0
    return len(re.findall(r"^\[\[", text, re.M))


def lane(subject):
    if subject.startswith("Review check-in:"):
        return "review"
    if subject.startswith("A:"):
        return "A"
    bank = re.match(r"B:(sm_[0-9a-f]+)", subject)
    if bank:
        for queue, banks in B_QUEUES.items():
            if bank.group(1) in banks:
                return f"{queue} {bank.group(1)}"
        return f"B {bank.group(1)}"
    return None


def reported_checks(sha):
    """The checks count from the newest `Score:` line at or before `sha`."""
    log = git("log", "-50", "--format=%B%x00", sha).decode()
    found = re.search(r"^Score:.*?checks (\d+)/(\d+)", log, re.M)
    return (int(found.group(1)), int(found.group(2))) if found else None


def block(sha):
    subject, body = git("show", "-s", "--format=%s%x00%b", sha).decode().split("\x00", 1)
    parent = git("rev-parse", f"{sha}^").decode().strip()
    files = git("diff", "--name-only", parent, sha).decode().split()
    new, old = measure(sha), measure(parent)
    tag = lane(subject)
    score = re.search(r"^Score:.*$", body, re.M)
    head = f"**[`{sha[:7]}`]({REPO}/commit/{sha})** {subject}"
    parts = [f"{k} {new[k] - old[k]:+,}" for k in ("legacy", "production", "unsafe", "verification")
             if new[k] != old[k]]
    rust = sum(1 for f in files if f.startswith(SRC) and f.endswith(".rs"))
    if rust and not parts:
        parts.append(f"{rust} Rust file(s) edited, net 0 lines")
    elif not rust:
        parts.append("no Rust change")
    rows = segment_rows(sha) - segment_rows(parent)
    if rows:
        parts.append(f"segment rows {rows:+}")
    checks, before_checks = reported_checks(sha), reported_checks(parent)
    if checks and score and checks != before_checks:
        gained = checks[0] - before_checks[0] if before_checks else 0
        parts.insert(0, f"**checks {checks[0]}/{checks[1]} ({gained:+})**")
    notes = []
    removed = old["legacy"] - new["legacy"]
    owned_added = (new["production"] - old["production"]) + removed
    if removed > 0:
        notes.append(f"replacement ratio {max(owned_added, 0) / removed:.0%}")
    if new["banks"] < old["banks"]:
        notes.append(f"**{old['banks'] - new['banks']} bank file(s) deleted**")
    if tag != "review":
        if tag is None:
            notes.append("⚠ unlabelled")
        if not score:
            notes.append("⚠ no Score line")
        if not any(f.startswith(SOURCE_PREFIXES) for f in files):
            notes.append("docs only")
        if new["unsafe"] > old["unsafe"]:
            notes.append("⚠ unsafe rose")
    label = f"lane {tag}" if tag and tag != "review" else (tag or "unlabelled")
    lines = [head, f"{label} · " + " · ".join(parts)]
    if notes:
        lines.append(" · ".join(notes))
    return "\n".join(lines)


def main(before, after):
    try:
        shas = git("rev-list", "--reverse", "--no-merges", f"{before}..{after}").decode().split()
    except subprocess.CalledProcessError:
        shas = [after]
    shas = shas or [after]
    blocks = [block(s) for s in shas[-20:]]
    totals = measure(after)
    blocks.append(f"<sub>Now: {totals['banks']} bank files · legacy {totals['legacy']:,} · "
                  f"production {totals['production']:,} · unsafe {totals['unsafe']:,} · "
                  f"verification {totals['verification']:,}</sub>")
    print("\n\n".join(blocks))


if __name__ == "__main__":
    main(*sys.argv[1:3])
