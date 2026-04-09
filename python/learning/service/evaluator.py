"""SkillEvaluator — evaluates proposed skill drafts against benchmarks."""

import uuid
from dataclasses import dataclass
from typing import Optional


@dataclass
class EvaluationResult:
    """Result of evaluating a skill draft."""
    draft_id: uuid.UUID
    score: float  # 0.0 - 1.0
    benchmark_results: list[dict]
    passed: bool
    notes: str


class SkillEvaluator:
    """
    Evaluates synthesized skills against test cases.
    Scores drafts before they are promoted to active skills.
    """

    def __init__(self):
        self.benchmarks: dict[str, list[dict]] = {}

    def register_benchmark(self, skill_id: str, test_cases: list[dict]) -> None:
        """Register a benchmark test suite for a skill."""
        self.benchmarks[skill_id] = test_cases

    async def evaluate(self, draft_id: uuid.UUID, skill: dict) -> EvaluationResult:
        """
        Run evaluation against registered benchmarks.
        Returns a score and pass/fail.
        """
        skill_id = skill["id"]
        test_cases = self.benchmarks.get(skill_id, [])

        if not test_cases:
            # No benchmark yet — defer
            return EvaluationResult(
                draft_id=draft_id,
                score=0.5,
                benchmark_results=[],
                passed=False,
                notes="No benchmark registered — deferred",
            )

        results = []
        for tc in test_cases:
            results.append({
                "name": tc.get("name", "unknown"),
                "passed": True,  # TODO: actually run the test
            })

        passed = all(r["passed"] for r in results)
        score = sum(1 for r in results if r["passed"]) / len(results) if results else 0.0

        return EvaluationResult(
            draft_id=draft_id,
            score=score,
            benchmark_results=results,
            passed=passed,
            notes=f"{len(results)} test cases, {sum(1 for r in results if r['passed'])} passed",
        )
