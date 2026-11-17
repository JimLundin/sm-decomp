"""Retain recent clean main builds without touching other cache owners."""

import hashlib
import json
import os
import re
import urllib.request
from dataclasses import dataclass
from datetime import datetime
from pathlib import Path
from typing import Any


@dataclass(frozen=True)
class Cache:
    id: int
    key: str
    ref: str
    created: datetime


def candidates(caches: list[Cache], prefixes: list[str], revision: str) -> list[Cache]:
    """Plan all deletions before performing any; incomplete publication preserves all."""
    # Inventory pages can overlap as other runs access caches during pagination.
    unique: dict[int, Cache] = {}
    for cache in caches:
        previous = unique.get(cache.id)
        if previous is not None and previous != cache:
            raise ValueError("conflicting records for one cache id")
        unique[cache.id] = cache
    remove: list[Cache] = []
    for prefix in prefixes:
        family = [cache for cache in unique.values() if cache.ref == "refs/heads/main"
                  and re.fullmatch(re.escape(prefix) + r"[0-9a-f]{40}", cache.key)]
        current = [cache for cache in family if cache.key == prefix + revision]
        if not current:
            raise ValueError("current clean cache is not published for every job")
        ordered = sorted(family, key=lambda cache: (cache.created, cache.id), reverse=True)
        keep = {cache.id for cache in ordered[:2]} | {cache.id for cache in current}
        remove.extend(cache for cache in ordered if cache.id not in keep)
    return remove


class Api:
    def __init__(self, repository: str, token: str) -> None:
        if not re.fullmatch(r"[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+", repository):
            raise ValueError("invalid repository")
        self.url = f"https://api.github.com/repos/{repository}/actions/caches"
        self.token = token

    def request(self, suffix: str, method: str = "GET") -> Any:
        request = urllib.request.Request(self.url + suffix, method=method, headers={
            "Authorization": f"Bearer {self.token}",
            "Accept": "application/vnd.github+json",
            "X-GitHub-Api-Version": "2022-11-28",
        })
        with urllib.request.urlopen(request, timeout=60) as response:
            return json.load(response) if method == "GET" else None

    def inventory(self) -> list[Cache]:
        caches = []
        page = 1
        while True:
            entries = self.request(f"?per_page=100&page={page}")["actions_caches"]
            if not isinstance(entries, list):
                raise ValueError("invalid cache inventory page")
            for entry in entries:
                if (not isinstance(entry["id"], int) or isinstance(entry["id"], bool)
                        or not all(isinstance(entry[field], str)
                                   for field in ("key", "ref", "created_at"))):
                    raise ValueError("invalid cache inventory fields")
                cache = Cache(entry["id"], entry["key"], entry["ref"],
                              datetime.fromisoformat(entry["created_at"].replace("Z", "+00:00")))
                if cache.id <= 0 or cache.created.tzinfo is None:
                    raise ValueError("invalid cache inventory")
                caches.append(cache)
            if len(entries) < 100:
                return caches
            page += 1

    def prune(self, prefixes: list[str], revision: str) -> None:
        remove = candidates(self.inventory(), prefixes, revision)
        print(f"Clean main cache publication confirmed; retiring {len(remove)} old entries")
        for cache in remove:
            self.request(f"/{cache.id}", "DELETE")
            print(f"Deleted old clean cargo cache {cache.id}")


def main() -> None:
    if os.environ["GITHUB_REF"] != "refs/heads/main" or os.environ["GITHUB_EVENT_NAME"] not in {
        "push", "schedule", "workflow_dispatch"
    }:
        raise ValueError("cache retention requires a trusted main producer")
    revision = os.environ["GITHUB_SHA"]
    if not re.fullmatch(r"[0-9a-f]{40}", revision):
        raise ValueError("invalid revision")
    # hashFiles hashes the concatenated binary file digests; each pattern matches one file.
    toolchain, lock = (hashlib.sha256(hashlib.sha256(Path(name).read_bytes()).digest()).hexdigest()
                       for name in ("rust-toolchain.toml", "Cargo.lock"))
    prefixes = [f"cargo-{job}-Linux-{toolchain}-{lock}-"
                for job in ("quality", "content", "replay")]
    Api(os.environ["GITHUB_REPOSITORY"], os.environ["GH_TOKEN"]).prune(prefixes, revision)


if __name__ == "__main__":
    main()
