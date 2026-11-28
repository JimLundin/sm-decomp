import shlex
lazy from collections.abc import Mapping, Sequence
lazy from pathlib import PurePath

from ordered import FrozenList, FrozenOrderedDict

USAGE = """usage: replay baseline RECEIPT | replay case ID [ARG...]

baseline: checks the required witnesses and the whole reviewed baseline at once, the witnesses
beside three indexed shards, and merges the shard receipts into RECEIPT, byte for byte the receipt
one unsharded check writes. Every part runs to its end and prints its log under its own label;
any failing part fails the check. Needs SM_CACHE, SM_ROM and SM_AUDIO_NATIVE.

case: checks one indexed diagnostic case, building the verifier from the checkout unless SM_TOOLS
names one; ARGs go to `tools replay check`."""
INDEX = "scenarios/replay/index.toml"
BASELINE = "scenarios/replay/baseline.toml"
SHARDS = 3
BUILT_TOOLS = "target/verify/tools"
DEFAULT_ROM = "rom/sm.smc"
REQUIRED = FrozenList(("SM_CACHE", "SM_ROM", "SM_AUDIO_NATIVE"))
WITNESSES = FrozenList(("required-witnesses",))
HEAD = FrozenList(("git", "rev-parse", "HEAD"))


def tools(env: Mapping[str, str]) -> str:
    return env.get("SM_TOOLS") or BUILT_TOOLS


def check(
    env: Mapping[str, str],
    checker: Sequence[str],
    selection: Sequence[str],
    extra: Sequence[str],
) -> FrozenList[str]:
    return FrozenList((
        *checker,
        "replay",
        "check",
        INDEX,
        *selection,
        "--baseline",
        BASELINE,
        "--cache",
        env.get("SM_CACHE", ""),
        "--rom",
        env.get("SM_ROM") or DEFAULT_ROM,
        "--native",
        env.get("SM_AUDIO_NATIVE", ""),
        "--bundle",
        env.get("SM_BUNDLE", ""),
        *extra,
    ))


def verifier(env: Mapping[str, str]) -> FrozenList[str]:
    if env.get("SM_TOOLS"):
        return FrozenList((tools(env),))
    return FrozenList((
        env.get("CARGO") or "cargo",
        "run",
        "--quiet",
        "--locked",
        "--profile",
        "verify",
        "-p",
        "tools",
        "--bin",
        "tools",
        "--",
    ))


def case(
    env: Mapping[str, str], name: str, extra: Sequence[str]
) -> FrozenList[str]:
    return check(env, verifier(env), ("--case", name), extra)


def reproduce(head: str, name: str) -> str:
    return (
        f"Reproduce on {head} with uv run --locked --project tools replay case"
        f" {shlex.quote(name)}"
    )


def missing_input(env: Mapping[str, str]) -> str | None:
    for name in REQUIRED:
        if not env.get(name):
            return f"replay: {name} is required"
    return None


def shard_receipt(logs: PurePath, shard: int) -> str:
    return str(logs / f"shard-{shard}.toml")


def parts(
    env: Mapping[str, str], logs: PurePath
) -> FrozenOrderedDict[str, FrozenList[str]]:
    shards = FrozenOrderedDict(
        (
            f"shard-{shard}",
            check(
                env,
                (tools(env),),
                ("--all",),
                (
                    "--shard",
                    f"{shard}/{SHARDS}",
                    "--write",
                    shard_receipt(logs, shard),
                ),
            ),
        )
        for shard in range(SHARDS)
    )
    return FrozenOrderedDict({"witnesses": WITNESSES}) | shards


def part_report(label: str, status: int, log: str) -> str:
    lines = log.splitlines()
    if status == 0:
        return "\n".join((f"::group::{label} passed", *lines, "::endgroup::"))
    return "\n".join((f"::error::{label} failed with exit {status}", *lines))


def merge(
    env: Mapping[str, str], logs: PurePath, receipt: str
) -> FrozenList[str]:
    receipts = FrozenList(
        argument
        for shard in range(SHARDS)
        for argument in ("--receipt", shard_receipt(logs, shard))
    )
    return FrozenList((
        tools(env),
        "replay",
        "merge",
        INDEX,
        "--all",
        "--baseline",
        BASELINE,
        *receipts,
        "--write",
        receipt,
    ))
