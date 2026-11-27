"""Cache retention must preserve publication, ref and compatibility boundaries."""

import unittest
import os
from datetime import datetime, timedelta, timezone
from unittest.mock import patch

from retain_cargo_caches import Api, Cache, candidates, main


PREFIXES = [f"cargo-{job}-Linux-toolchain-lock-" for job in ("quality", "replay")]
CURRENT = "a" * 40


def cache(identifier, prefix=PREFIXES[0], revision=CURRENT, ref="refs/heads/main"):
    return Cache(identifier, prefix + revision, ref,
                 datetime(2026, 1, 1, tzinfo=timezone.utc) + timedelta(seconds=identifier))


class RetentionTests(unittest.TestCase):
    def test_paginated_duplicate_newest_preserves_second_newest(self):
        api = Api("owner/repo", "test-token")
        def entry(identifier, revision):
            return {"id": identifier, "key": PREFIXES[0] + revision,
                    "ref": "refs/heads/main", "created_at": f"2026-01-01T00:00:0{identifier}Z"}
        oldest = entry(1, "b" * 40)
        second = entry(2, "c" * 40)
        newest = entry(3, CURRENT)
        pages = [{"actions_caches": [oldest, second, newest] + [newest] * 97},
                 {"actions_caches": [oldest, newest]}]
        def response(suffix, method="GET"):
            return pages.pop(0) if method == "GET" else None
        with patch.object(api, "request", side_effect=response) as request, patch("builtins.print"):
            api.prune(PREFIXES[:1], CURRENT)
        deletions = [call.args for call in request.call_args_list if len(call.args) == 2]
        self.assertEqual(deletions, [("/1", "DELETE")])

    def test_conflicting_duplicate_id_prevents_all_deletion(self):
        api = Api("owner/repo", "test-token")
        entries = [cache(1, revision="b" * 40), cache(2, revision="c" * 40), cache(3),
                   cache(3, ref="refs/pull/1/merge")]
        with patch.object(api, "inventory", return_value=entries), patch.object(api, "request") as request:
            with self.assertRaises(ValueError):
                api.prune(PREFIXES[:1], CURRENT)
            request.assert_not_called()

    def test_keep_newest_two_and_overlapping_current(self):
        entries = [cache(1), cache(2, revision="b" * 40), cache(3, revision="c" * 40),
                   cache(4, revision="d" * 40)]
        self.assertEqual([entry.id for entry in candidates(entries, PREFIXES[:1], CURRENT)], [2])

    def test_only_exact_main_family_is_owned(self):
        entries = [cache(1, revision="b" * 40), cache(2, revision="c" * 40), cache(3),
                   cache(4, ref="refs/pull/1/merge"), cache(5, prefix="cargo-other-"),
                   cache(6, prefix=PREFIXES[0] + "extra-"), cache(7, revision="invalid")]
        self.assertEqual([entry.id for entry in candidates(entries, PREFIXES[:1], CURRENT)], [1])

    def test_a_job_that_did_not_run_keeps_its_family(self):
        entries = [cache(1, revision="b" * 40), cache(2, revision="c" * 40), cache(3),
                   cache(4, PREFIXES[1], "b" * 40), cache(5, PREFIXES[1], "c" * 40),
                   cache(6, PREFIXES[1], "d" * 40)]
        self.assertEqual([entry.id for entry in candidates(entries, PREFIXES, CURRENT)], [1])

    def test_creation_time_orders_not_access_time_and_ties_are_deterministic(self):
        entries = [cache(1, revision="b" * 40), cache(2, revision="c" * 40), cache(3)]
        entries = [Cache(entry.id, entry.key, entry.ref, entries[0].created) for entry in entries]
        self.assertEqual([entry.id for entry in candidates(entries, PREFIXES[:1], CURRENT)], [1])

    def test_inventory_reads_every_page(self):
        api = Api("owner/repo", "test-token")
        entries = [{"id": identifier, "key": PREFIXES[0] + (CURRENT if identifier == 101 else f"{identifier:040x}"),
                    "ref": "refs/heads/main", "created_at": "2026-01-01T00:00:00Z"}
                   for identifier in range(1, 102)]
        def request(suffix, method="GET"):
            if method == "DELETE":
                self.assertEqual(calls[:2], ["?per_page=100&page=1", "?per_page=100&page=2"])
                self.assertNotIn(suffix, ("/100", "/101"))
                calls.append(suffix)
                return None
            calls.append(suffix)
            return {"actions_caches": entries[:100] if len(calls) == 1 else entries[100:]}
        calls = []
        with patch.object(api, "request", side_effect=request), patch("builtins.print"):
            api.prune(PREFIXES[:1], CURRENT)
        self.assertEqual(len(calls), 101)

    def test_current_cache_versions_are_all_protected(self):
        entries = [cache(1), cache(2), cache(3, revision="b" * 40), cache(4, revision="c" * 40)]
        self.assertEqual(candidates(entries, PREFIXES[:1], CURRENT), [])

    def test_hashfiles_matches_github_single_file_digest(self):
        # Known SHA256-of-empty-file digest, hashed as binary by GitHub's hashFiles.
        expected = "5df6e0e2761359d30a8275058e299fcc0381534545f55cf43e41983f5d4c9456"
        with patch.dict(os.environ, {"GITHUB_REF": "refs/heads/main", "GITHUB_EVENT_NAME": "push",
                                     "GITHUB_SHA": CURRENT, "GITHUB_REPOSITORY": "owner/repo", "GH_TOKEN": "test"}), \
                patch("retain_cargo_caches.Path.read_bytes", return_value=b""), \
                patch.object(Api, "prune") as prune:
            main()
        self.assertEqual(prune.call_args.args,
                         ([f"cargo-{job}-Linux-{expected}-{expected}-" for job in ("quality", "replay")], CURRENT))

    def test_consumer_cannot_prune(self):
        with patch.dict(os.environ, {"GITHUB_REF": "refs/pull/1/merge", "GITHUB_EVENT_NAME": "pull_request"}), \
                patch.object(Api, "prune") as prune:
            with self.assertRaises(ValueError):
                main()
            prune.assert_not_called()

    def test_malformed_inventory_prevents_deletion(self):
        api = Api("owner/repo", "test-token")
        with patch.object(api, "request", return_value={"actions_caches": [{"id": 1}]}) as request:
            with self.assertRaises(KeyError):
                api.prune(PREFIXES, CURRENT)
            self.assertEqual(request.call_count, 1)

    def test_delete_failure_stops_without_touching_retained_entries(self):
        api = Api("owner/repo", "test-token")
        entries = [cache(1, revision="b" * 40), cache(2, revision="c" * 40), cache(3)]
        with patch.object(api, "inventory", return_value=entries), patch.object(api, "request", side_effect=OSError) as request:
            with self.assertRaises(OSError):
                api.prune(PREFIXES[:1], CURRENT)
            self.assertEqual(request.call_args.args, ("/1", "DELETE"))


if __name__ == "__main__":
    unittest.main()
