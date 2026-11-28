"""The cutover cannot discard a claim or relabel a missing owner as passing proof."""

import copy
import hashlib
from typing import Any
import unittest

from check_replay_transition import InvalidTransition, validate


class TransitionTests(unittest.TestCase):
    def setUp(self) -> None:
        text = "movement first 4\nmovement second 0\n"
        digest = hashlib.sha256(text.encode()).hexdigest()
        self.prior: dict[str, Any] = {"source": "fixed", "snapshot": [{
            "path": "floors.txt", "text": text, "sha256": digest, "noncomment_lines": 2,
        }]}
        self.transition: dict[str, Any] = {
            "format": 1, "source": "fixed", "status": "reviewed", "review_sha256": "a" * 64,
            "claim": [{"source_path": "floors.txt", "source_sha256": digest,
                       "identity": f"movement/{name}", "before_values": [count],
                       "disposition": "replaced", "reason": "Independent scoped witness.",
                       "witness_case_ids": ["movement/case"], "lost_observations": []}
                      for name, count in [("first", 4), ("second", 0)]],
        }
        self.baseline: dict[str, Any] = {"case": [{"id": "movement/case", "claim": "behaviour",
                                   "expected": {"status": "pass"}}]}

    def test_zero_floor_still_requires_a_disposition(self) -> None:
        self.assertEqual(validate(self.prior, self.transition, self.baseline), 2)
        altered = copy.deepcopy(self.transition)
        altered["claim"].pop()
        with self.assertRaises(InvalidTransition):
            validate(self.prior, altered, self.baseline)

    def test_unsupported_is_not_replacement_behaviour_proof(self) -> None:
        self.baseline["case"][0].update(claim="boundary", expected={"status": "unsupported"})
        with self.assertRaises(InvalidTransition):
            validate(self.prior, self.transition, self.baseline)

    def test_retirement_requires_an_explicit_loss(self) -> None:
        self.transition["claim"][0]["disposition"] = "retired"
        with self.assertRaises(InvalidTransition):
            validate(self.prior, self.transition, self.baseline)
        self.transition["claim"][0]["lost_observations"] = ["former complete-row coverage"]
        self.assertEqual(validate(self.prior, self.transition, self.baseline), 2)

    def test_duplicate_and_modified_history_are_rejected(self) -> None:
        altered = copy.deepcopy(self.transition)
        altered["claim"].append(copy.deepcopy(altered["claim"][0]))
        with self.assertRaises(InvalidTransition):
            validate(self.prior, altered, self.baseline)
        self.transition["claim"][0]["before_values"] = [3]
        with self.assertRaises(InvalidTransition):
            validate(self.prior, self.transition, self.baseline)

    def test_internal_content_receipt_requires_a_sha256_pin(self) -> None:
        requirement = {
            "source_path": "content-tests.rs", "source_sha256": "b" * 64,
            "identity": "content/pose", "before_values": [1], "meaning": "pose content integrity",
        }
        self.prior["requirement"] = [requirement]
        self.transition["claim"].append({
            **requirement, "disposition": "internal", "reason": "content test retained",
            "witness_case_ids": [], "test_ids": ["pose"], "lost_observations": [],
        })
        self.transition["development_check"] = [{
            "id": "pose", "status": "pass", "receipt_sha256": "x" * 64,
        }]
        with self.assertRaises(InvalidTransition):
            validate(self.prior, self.transition, self.baseline)
        self.transition["development_check"][0]["receipt_sha256"] = "c" * 64
        self.assertEqual(validate(self.prior, self.transition, self.baseline), 3)


if __name__ == "__main__":
    unittest.main()
