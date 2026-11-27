"""Check a clean member's declared targets and effective dependencies without running Cargo."""

import sys
import tomllib
from pathlib import Path, PureWindowsPath


Table = dict[str, object]
Approvals = dict[str, tuple[str, list[str], set[str]]]


def finding(tag: str, detail: str) -> None:
    print(f"{tag}\t{detail}")


def table(value: object) -> Table:
    if not isinstance(value, dict):
        raise ValueError("expected a TOML table")
    return value


def string(value: object) -> str:
    if not isinstance(value, str):
        raise ValueError("expected a TOML string")
    return value


def features_of(dep: Table) -> list[str]:
    features = dep.get("features", [])
    if not isinstance(features, list):
        raise ValueError("expected a list of dependency features")
    return [string(feature) for feature in features]


def dependency(value: object) -> Table:
    return {"version": value} if isinstance(value, str) else table(value)


def check_dependency(
    name: str,
    declaration: object,
    kind: str,
    crate: Path,
    root: Path,
    inherited_deps: Table,
    approved: Approvals,
    allowed_paths: set[Path],
) -> None:
    dep = dependency(declaration)
    base = crate
    if "workspace" in dep:
        inherited = inherited_deps.get(name)
        if dep["workspace"] is not True or inherited is None:
            finding("dep-workspace", f"{name}: missing workspace dependency")
            return
        if set(dep) - {"workspace", "features", "default-features"}:
            finding(
                "dep-registry",
                f"{name}: invalid override on an inherited dependency",
            )
            return
        definition = dependency(inherited)
        dep = {
            **definition,
            "features": features_of(definition) + features_of(dep),
        }
        base = root
    if {"git", "registry", "registry-index"} & dep.keys():
        finding("dep-git", f"{name}: git or alternate-registry dependency")
        return
    if "path" in dep:
        path = (base / string(dep["path"])).resolve()
        if path not in allowed_paths:
            finding(
                "dep-path",
                f"{name}: path {dep['path']} resolves outside the clean crates ({path})",
            )
        return
    package = string(dep.get("package", name))
    if package not in approved:
        finding("dep-registry", f"{name}: {package} is not in allowed_deps.txt")
        return
    version, kinds, features = approved[package]
    if dep.get("version") != version or kind not in kinds:
        finding(
            "dep-registry",
            f"{name}: {kind} dependency requires approval at its exact version",
        )
    for key in dep.keys() - {
        "version",
        "features",
        "default-features",
        "package",
    }:
        finding(
            "dep-registry",
            f"{name}: the key `{key}` is not allowed on a registry dependency",
        )
    for feature in set(features_of(dep)) - features:
        finding(
            "dep-registry",
            f"{name}: feature `{feature}` is not in allowed_deps.txt",
        )


def check(
    crate: Path, root: Path, allowfile: Path, allowed_paths: set[Path]
) -> None:
    manifest: Table = tomllib.loads(
        (crate / "Cargo.toml").read_text(encoding="utf-8")
    )
    workspace_file = root / "Cargo.toml"
    workspace: Table = {}
    if workspace_file.is_file():
        workspace = table(
            tomllib.loads(workspace_file.read_text(encoding="utf-8")).get(
                "workspace", {}
            )
        )
    trusted = tomllib.loads(
        (allowfile.parents[2] / "Cargo.toml").read_text(encoding="utf-8")
    )
    pinned = table(table(trusted["workspace"])["dependencies"])
    inherited_deps = table(workspace.get("dependencies", {}))
    approved: Approvals = {}
    for line in allowfile.read_text(encoding="utf-8").splitlines():
        fields = line.partition("#")[0].split()
        if fields:
            name, kinds, features = fields
            version = string(dependency(pinned[name])["version"])
            if not version.startswith("="):
                raise ValueError(f"{name} needs an exact workspace version")
            approved[name] = (
                version,
                kinds.split(","),
                set() if features == "-" else set(features.split(",")),
            )
    for section in manifest.keys() & {"workspace", "patch", "replace"}:
        finding("manifest-section", f"[{section}]")
    package = table(manifest.get("package", {}))
    if (
        "workspace" in package
        and (crate / string(package["workspace"])).resolve() != root
    ):
        finding("dep-workspace", "package selects another workspace")
    for key in ("build", "links"):
        if key in package:
            finding(f"{key}-key", f"package.{key}")
    edition = package.get("edition")
    if edition == {"workspace": True}:
        edition = table(workspace.get("package", {})).get("edition")
    if edition != "2024":
        finding("edition", f"expected edition 2024, found {edition!r}")
    if table(manifest.get("lints", {})).get("workspace") is not True:
        finding("lints-key", "no `[lints] workspace = true`")
    for key in ("lib", "bin", "test", "example", "bench"):
        targets = manifest.get(key, [])
        for item in targets if isinstance(targets, list) else [targets]:
            target = table(item)
            if "proc-macro" in target or "proc_macro" in target:
                finding("proc-macro", key)
            path = string(target.get("path", ""))
            if path and (
                Path(path).is_absolute()
                or ".." in Path(path).parts
                or PureWindowsPath(path).drive
            ):
                finding("target-path", path)
    for item in (manifest, *table(manifest.get("target", {})).values()):
        scope = table(item)
        if "build-dependencies" in scope or "build_dependencies" in scope:
            finding("build-deps", "build dependencies")
        for section, kind in (
            ("dependencies", "normal"),
            ("dev-dependencies", "dev"),
            ("dev_dependencies", "dev"),
        ):
            for name, dep in table(scope.get(section, {})).items():
                check_dependency(
                    name,
                    dep,
                    kind,
                    crate,
                    root,
                    inherited_deps,
                    approved,
                    allowed_paths,
                )


if __name__ == "__main__":
    crate_dir, workspace_root, allow_file, *allowed = map(Path, sys.argv[1:])
    try:
        check(crate_dir, workspace_root, allow_file, set(allowed))
    except (AttributeError, KeyError, OSError, TypeError, ValueError) as error:
        finding("manifest", f"invalid manifest or dependency policy: {error}")
