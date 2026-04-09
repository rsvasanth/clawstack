"""
SkillSynthesizer — promotes successful trajectories into reusable skills.
"""

import uuid
import os
import httpx
from dataclasses import dataclass
from datetime import datetime, timezone
from typing import Optional


@dataclass
class SkillInput:
    name: str
    description: str
    required: bool
    schema: dict


@dataclass
class Skill:
    """A synthesized skill."""
    id: str
    name: str
    description: str
    version: int = 1
    scope: str = "general"
    inputs: list[SkillInput] = None
    steps: list[str] = None
    success_signals: list[str] = None
    risk_level: str = "medium"
    required_capabilities: list[str] = None
    source: str = "learned"
    enabled: bool = False
    created_at: datetime = None

    def __post_init__(self):
        if self.inputs is None:
            self.inputs = []
        if self.steps is None:
            self.steps = []
        if self.success_signals is None:
            self.success_signals = []
        if self.required_capabilities is None:
            self.required_capabilities = []
        if self.created_at is None:
            self.created_at = datetime.now(timezone.utc)

    def to_dict(self) -> dict:
        return {
            "id": self.id,
            "name": self.name,
            "description": self.description,
            "version": self.version,
            "scope": self.scope,
            "inputs": [
                {
                    "name": i.name,
                    "description": i.description,
                    "required": i.required,
                    "schema": i.schema,
                }
                for i in self.inputs
            ],
            "steps": self.steps,
            "success_signals": self.success_signals,
            "risk_level": self.risk_level,
            "required_capabilities": self.required_capabilities,
            "source": self.source,
            "enabled": self.enabled,
            "created_at": self.created_at.isoformat(),
        }


@dataclass
class SkillDraft:
    """A proposed skill waiting for Rust approval."""
    id: uuid.UUID
    proposed_by: str = "skill_synthesizer"
    skill: Skill = None
    evidence_trajectory_ids: list[uuid.UUID] = None
    confidence_score: float = 0.0
    status: str = "pending"
    created_at: datetime = None

    def __post_init__(self):
        if self.skill is None:
            self.skill = Skill(id=str(self.id), name="")
        if self.evidence_trajectory_ids is None:
            self.evidence_trajectory_ids = []
        if self.created_at is None:
            self.created_at = datetime.now(timezone.utc)

    def to_dict(self) -> dict:
        return {
            "id": str(self.id),
            "proposed_by": self.proposed_by,
            "skill": self.skill.to_dict(),
            "evidence_trajectory_ids": [str(t) for t in self.evidence_trajectory_ids],
            "confidence_score": self.confidence_score,
            "status": self.status,
            "created_at": self.created_at.isoformat(),
        }


class SkillSynthesizer:
    """
    Analyzes trajectories and synthesizes reusable skills.
    Proposes SkillDrafts back to Rust for approval.
    """

    def __init__(self, collector, rust_grpc_addr: str):
        self.collector = collector
        self.rust_grpc_addr = rust_grpc_addr
        self.http_client = httpx.AsyncClient(timeout=30.0)

    async def synthesize_from_pattern(
        self,
        pattern: dict,
        trajectories: list,
    ) -> Optional[SkillDraft]:
        """
        Convert a frequent tool-call pattern into a SkillDraft.
        Called periodically by the learning loop.
        """
        tool_ids = pattern["pattern"]
        frequency = pattern["frequency"]

        # Minimum frequency threshold to synthesize
        if frequency < 5:
            return None

        # Heuristic: derive skill name from tool sequence
        skill_name = "_".join(tool_ids[:3])[:64]
        skill_id = f"learned.{skill_name}.v1"

        skill = Skill(
            id=skill_id,
            name=f"Auto-learned: {' + '.join(tool_ids)}",
            description=f"Automatically synthesized from {frequency} successful executions",
            version=1,
            scope="general",
            steps=[f"Use tool: {tid}" for tid in tool_ids],
            success_signals=["All tools returned success"],
            risk_level=self._assess_risk(tool_ids),
            required_capabilities=self._infer_capabilities(tool_ids),
            source="learned",
            enabled=False,  # Must be approved by Rust
        )

        draft = SkillDraft(
            id=uuid.uuid4(),
            proposed_by="skill_synthesizer",
            skill=skill,
            evidence_trajectory_ids=[t.id for t in trajectories[:10]],
            confidence_score=min(frequency / 20.0, 1.0),
            status="pending",
        )

        return draft

    def _assess_risk(self, tool_ids: list[str]) -> str:
        """Heuristic risk assessment based on tool types."""
        high_risk = {"repo.write", "shell.exec", "secrets.use", "browser.auth"}
        medium_risk = {"repo.read", "shell.test", "browser.use"}
        low_risk = {"memory.read", "web.fetch", "channel.send"}

        if any(t in high_risk for t in tool_ids):
            return "high"
        elif any(t in medium_risk for t in tool_ids):
            return "medium"
        return "low"

    def _infer_capabilities(self, tool_ids: list[str]) -> list[str]:
        """Infer required capabilities from tool IDs."""
        caps: set[str] = set()
        for tid in tool_ids:
            if tid.startswith("memory"):
                caps.add("memory.read")
            elif tid.startswith("repo"):
                caps.add(tid.replace(".", "_").replace("_", ".", 1) if "." in tid else tid)
            elif tid.startswith("shell"):
                caps.add("shell.exec")
        return list(caps)

    async def run_cycle(self, workspace_id: uuid.UUID) -> list[SkillDraft]:
        """
        Full synthesis cycle: fetch patterns, generate drafts, send to Rust.
        Called periodically by the scheduler.
        
        Args:
            workspace_id: The workspace to analyze trajectories for.
        """
        # Step 1: Fetch recent trajectories from collector for this workspace
        trajectories = await self.collector.fetch_recent(workspace_id, limit=500)
        if not trajectories:
            return []

        # Step 2: Extract frequent patterns
        patterns = self.collector.extract_patterns(trajectories)

        # Step 3: Synthesize drafts for each pattern
        drafts = []
        for pattern in patterns:
            draft = await self.synthesize_from_pattern(pattern, trajectories)
            if draft:
                drafts.append(draft)

        # Step 4: Send drafts to Rust for approval via REST API
        await self._submit_drafts_to_rust(drafts)

        return drafts

    async def _submit_drafts_to_rust(self, drafts: list[SkillDraft]) -> None:
        """
        Submit skill drafts to Rust gateway for approval.
        Uses REST API as fallback when gRPC is not available.
        """
        rust_url = os.environ.get("RUST_GATEWAY_URL", "http://localhost:8080")
        
        for draft in drafts:
            try:
                # Try REST API first
                response = await self.http_client.post(
                    f"{rust_url}/api/v1/skills/drafts",
                    json=draft.to_dict(),
                )
                if response.status_code == 201:
                    print(f"[SkillSynthesizer] Submitted draft: {draft.skill.name}")
                else:
                    print(f"[SkillSynthesizer] Failed to submit draft: {response.status_code}")
            except Exception as e:
                # gRPC would be used here in production
                print(f"[SkillSynthesizer] Could not reach Rust gateway: {e}")
                print(f"[SkillSynthesizer] Draft pending: {draft.skill.name}")
