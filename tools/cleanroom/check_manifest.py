"""Check a clean member's declared targets and effective dependencies without running Cargo."""

import sys
from pathlib import Path, PureWindowsPath
import tomllib


def finding(tag, detail):
    print(f"{tag}\t{detail}")


def table(value):
    return {"version": value} if isinstance(value, str) else value


def check_dependency(name, declaration, kind, crate, root, workspace, approved, allowed_paths):
    dep = table(declaration)
    base = crate
    if "workspace" in dep:
        inherited = workspace.get("dependencies", {}).get(name)
        if dep["workspace"] is not True or inherited is None:
            finding("dep-workspace", f"{name}: missing workspace dependency")
            return
        if set(dep) - {"workspace", "features", "default-features"}:
            finding("dep-registry", f"{name}: invalid override on an inherited dependency")
            return
        definition = table(inherited)
        dep = {**definition, "features": definition.get("features", []) + dep.get("features", [])}
        base = root
    if {"git", "registry", "registry-index"} & dep.keys():
        finding("dep-git", f"{name}: git or alternate-registry dependency")
        return
    if "path" in dep:
        path = (base / dep["path"]).resolve()
        if path not in allowed_paths:
            finding("dep-path", f"{name}: path {dep['path']} resolves outside the clean crates ({path})")
        return
    package = dep.get("package", name)
    if package not in approved:
        finding("dep-registry", f"{name}: {package} is not in allowed_deps.txt")
        return
    version, kinds, features = approved[package]
    if dep.get("version") != version or kind not in kinds:
        finding("dep-registry", f"{name}: {kind} dependency requires approval at its exact version")
    for key in dep.keys() - {"version", "features", "default-features", "package"}:
        finding("dep-registry", f"{name}: the key `{key}` is not allowed on a registry dependency")
    for feature in set(dep.get("features", [])) - features:
        finding("dep-registry", f"{name}: feature `{feature}` is not in allowed_deps.txt")


def check(crate, root, allowfile, allowed_paths):
    manifest = tomllib.loads((crate / "Cargo.toml").read_text(encoding="utf-8"))
    workspace_file = root / "Cargo.toml"
    workspace = {}
    if workspace_file.is_file():
        workspace = tomllib.loads(workspace_file.read_text(encoding="utf-8")).get("workspace", {})
    trusted = tomllib.loads((allowfile.parents[2] / "Cargo.toml").read_text(encoding="utf-8"))
    pinned = trusted["workspace"]["dependencies"]
    approved = {}
    for line in allowfile.read_text(encoding="utf-8").splitlines():
        fields = line.partition("#")[0].split()
        if fields:
            name, kinds, features = fields
            version = table(pinned[name])["version"]
            if not version.startswith("="):
                raise ValueError(f"{name} needs an exact workspace version")
            approved[name] = version, kinds.split(","), set() if features == "-" else set(features.split(","))
    for section in manifest.keys() & {"workspace", "patch", "replace"}:
        finding("manifest-section", f"[{section}]")
    package = manifest.get("package", {})
    if "workspace" in package and (crate / package["workspace"]).resolve() != root:
        finding("dep-workspace", "package selects another workspace")
    for key in ("build", "links"):
        if key in package:
            finding(f"{key}-key", f"package.{key}")
    edition = package.get("edition")
    if edition == {"workspace": True}:
        edition = workspace.get("package", {}).get("edition")
    if edition != "2024":
        finding("edition", f"expected edition 2024, found {edition!r}")
    if manifest.get("lints", {}).get("workspace") is not True:
        finding("lints-key", "no `[lints] workspace = true`")
    for key in ("lib", "bin", "test", "example", "bench"):
        targets = manifest.get(key, [])
        for target in [targets] if isinstance(targets, dict) else targets:
            if "proc-macro" in target or "proc_macro" in target:
                finding("proc-macro", key)
            path = target.get("path")
            if path and (Path(path).is_absolute() or ".." in Path(path).parts or PureWindowsPath(path).drive):
                finding("target-path", path)
    for scope in (manifest, *manifest.get("target", {}).values()):
        if "build-dependencies" in scope or "build_dependencies" in scope:
            finding("build-deps", "build dependencies")
        for section, kind in (("dependencies", "normal"), ("dev-dependencies", "dev"), ("dev_dependencies", "dev")):
            for name, dep in scope.get(section, {}).items():
                check_dependency(name, dep, kind, crate, root, workspace, approved, allowed_paths)


if __name__ == "__main__":
    crate_dir, workspace_root, allow_file, *allowed = map(Path, sys.argv[1:])
    try:
        check(crate_dir, workspace_root, allow_file, set(allowed))
    except (AttributeError, KeyError, OSError, TypeError, ValueError) as error:
        finding("manifest", f"invalid manifest or dependency policy: {error}")
