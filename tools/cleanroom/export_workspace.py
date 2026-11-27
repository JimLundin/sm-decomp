"""Generate a clean export's workspace and any missing crates from the requested revision."""

import json
import subprocess
import sys
import tomllib
from collections.abc import Mapping
from pathlib import Path
from typing import NotRequired, TypedDict


class Workspace(TypedDict):
    resolver: str
    package: NotRequired[dict[str, object]]
    dependencies: NotRequired[dict[str, str | dict[str, object]]]
    lints: dict[str, dict[str, object]]


def at(revision: str, path: str) -> str:
    return subprocess.check_output(
        ["git", "show", f"{revision}:{path}"], text=True
    )


def value(item: object) -> str:
    if isinstance(item, dict):
        return (
            "{ "
            + ", ".join(f"{key} = {value(val)}" for key, val in item.items())
            + " }"
        )
    return json.dumps(item, ensure_ascii=False)


def section(name: str, entries: Mapping[str, object]) -> str:
    return f"[{name}]\n" + "".join(
        f"{key} = {value(val)}\n" for key, val in entries.items()
    )


def scaffold(
    dest: Path,
    name: str,
    package: Mapping[str, object],
    description: str,
    doc: str,
    dependencies: Mapping[str, object],
    dev_dependencies: Mapping[str, object] | None = None,
) -> None:
    folder = dest / "crates/clean" / name
    if (folder / "Cargo.toml").exists():
        return
    (folder / "src").mkdir(parents=True, exist_ok=True)
    sections = [
        section(
            "package", {"name": name, **package, "description": description}
        ),
        section("dependencies", dependencies),
    ]
    if dev_dependencies:
        sections.append(section("dev-dependencies", dev_dependencies))
    sections.append(section("lints", {"workspace": True}))
    (folder / "Cargo.toml").write_text("\n".join(sections), encoding="utf-8")
    (folder / "src/lib.rs").write_text(f"//! {doc}\n", encoding="utf-8")


def write_workspace(revision: str, dest: Path, domain: str, model: str) -> None:
    workspace: Workspace = tomllib.loads(at(revision, "Cargo.toml"))[
        "workspace"
    ]
    clean = ["crates/clean/types", f"crates/clean/{model}"]
    sections = [
        section(
            "workspace", {"resolver": workspace["resolver"], "members": clean}
        )
    ]
    # A scaffold imported onto its source branch must use the package and dependency
    # declarations available there. Older branches declared these values in each member.
    shared = workspace.get("package")
    source: dict[str, object] = (
        shared
        or tomllib.loads(at(revision, "crates/tools/Cargo.toml"))["package"]
    )
    package = {
        key: source[key] for key in ("version", "edition", "license", "publish")
    }
    if shared:
        sections.append(section("workspace.package", package))
        package = {key: {"workspace": True} for key in package}

    approved: dict[str, str | dict[str, object]] = {}
    for line in at(revision, "tools/cleanroom/allowed_deps.txt").splitlines():
        fields = line.partition("#")[0].split()
        if fields:
            name = fields[0]
            approved[name] = (
                fields[1]
                if len(fields) == 4
                else workspace["dependencies"][name]
            )
    deps: dict[str, dict[str, object]]
    if "dependencies" in workspace:
        for name, declaration in approved.items():
            if isinstance(declaration, dict) and declaration.keys() - {
                "version",
                "features",
                "default-features",
            }:
                raise ValueError(f"{name} is not a crates.io dependency")
        sections.append(section("workspace.dependencies", approved))
        deps = {name: {"workspace": True} for name in approved}
    else:
        deps = {
            name: {"version": version} for name, version in approved.items()
        }

    for tool, lints in workspace["lints"].items():
        sections.append(section(f"workspace.lints.{tool}", lints))
    (dest / "Cargo.toml").write_text("\n".join(sections), encoding="utf-8")
    scaffold(
        dest,
        "types",
        package,
        "The domain types the clean models share",
        "The domain types the clean models share: quantities, poses and tables in player terms.",
        {},
    )
    scaffold(
        dest,
        model,
        package,
        f"The {domain} model, built from its behaviour spec",
        f"The {domain} model, built from its behaviour spec alone.",
        {"types": {"path": "../types"}, "enum-map": deps["enum-map"]},
        {
            "serde": {**deps["serde"], "features": ["derive"]},
            "toml": deps["toml"],
        },
    )


if __name__ == "__main__":
    revision, destination, domain, model = sys.argv[1:]
    write_workspace(revision, Path(destination), domain, model)
