"""
TrajectoryCollector — consumes completed trajectories from Rust and stores them for analysis.
"""

import uuid
from dataclasses import dataclass, field
from datetime import datetime, timezone
from typing import Optional
import json


@dataclass
class TrajectoryTurn:
    """A single turn in a trajectory."""
    turn_index: int
    role: str  # system | user | assistant | tool
    content: str
    tool_calls: list[dict] = field(default_factory=list)


@dataclass
class Trajectory:
    """A complete multi-tool execution sequence."""
    id: uuid.UUID
    workspace_id: uuid.UUID
    session_id: uuid.UUID
    user_message: str
    turns: list[TrajectoryTurn]
    outcome: str  # completed_chat | completed_tools | partial | failed | blocked_policy | blocked_safety
    duration_ms: int
    model: Optional[str] = None
    created_at: datetime = field(default_factory=lambda: datetime.now(timezone.utc))

    def to_dict(self) -> dict:
        return {
            "id": str(self.id),
            "workspace_id": str(self.workspace_id),
            "session_id": str(self.session_id),
            "user_message": self.user_message,
            "turns": [
                {
                    "turn_index": t.turn_index,
                    "role": t.role,
                    "content": t.content,
                    "tool_calls": t.tool_calls,
                }
                for t in self.turns
            ],
            "outcome": self.outcome,
            "duration_ms": self.duration_ms,
            "model": self.model,
            "created_at": self.created_at.isoformat(),
        }


class TrajectoryCollector:
    """
    Consumes trajectories from Rust via gRPC or direct database insert.
    Stores raw trajectories for later analysis by the skill synthesizer.
    """

    def __init__(self, db_url: str):
        self.db_url = db_url

    async def store(self, trajectory: Trajectory) -> None:
        """
        Store a trajectory into the analysis database.
        Called by Rust after request completion.
        """
        # TODO: Insert into analysis database
        print(f"[TrajectoryCollector] Stored trajectory {trajectory.id}")

    async def fetch_recent(
        self,
        workspace_id: uuid.UUID,
        limit: int = 100,
    ) -> list[Trajectory]:
        """
        Fetch recent trajectories for a workspace.
        Used by the skill synthesizer.
        """
        # TODO: Query from analysis database
        return []

    def extract_patterns(self, trajectories: list[Trajectory]) -> list[dict]:
        """
        Extract common tool-call sequences as patterns.
        Returns list of {pattern: [tool_ids], frequency: int, examples: [trajectory_id]}.
        """
        from collections import Counter

        patterns: list[tuple[tuple[str, ...], list[uuid.UUID]]] = []

        for traj in trajectories:
            tool_seq = tuple(
                tc["tool_id"]
                for turn in traj.turns
                for tc in turn.tool_calls
            )
            if tool_seq:
                patterns.append((tool_seq, [traj.id]))

        # Cluster by sequence
        counter: Counter[tuple[str, ...]] = Counter(k for k, _ in patterns)
        results = []
        for seq, count in counter.most_common(50):
            results.append({
                "pattern": list(seq),
                "frequency": count,
            })

        return results
