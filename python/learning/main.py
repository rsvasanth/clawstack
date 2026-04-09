"""
ClawStack Learning Service — FastAPI entry point.
"""

import uuid
import structlog
from contextlib import asynccontextmanager
from fastapi import FastAPI, HTTPException
from pydantic import BaseModel

from .service import TrajectoryCollector, SkillSynthesizer, SkillEvaluator

logger = structlog.get_logger()

collector: TrajectoryCollector | None = None
synthesizer: SkillSynthesizer | None = None
evaluator: SkillEvaluator | None = None


@asynccontextmanager
async def lifespan(app: FastAPI):
    global collector, synthesizer, evaluator
    import os
    db_url = os.environ.get("DATABASE_URL", "postgresql://localhost/clawstack")
    collector = TrajectoryCollector(db_url)
    synthesizer = SkillSynthesizer(collector, "http://localhost:50051")
    evaluator = SkillEvaluator()
    logger.info("learning_service.started")
    yield
    logger.info("learning_service.stopped")


app = FastAPI(
    title="ClawStack Learning Service",
    version="0.1.0",
    lifespan=lifespan,
)


class TrajectorySubmitRequest(BaseModel):
    id: str
    workspace_id: str
    session_id: str
    user_message: str
    turns: list[dict]
    outcome: str
    duration_ms: int
    model: str | None = None


class SkillDraftResponse(BaseModel):
    id: str
    proposed_by: str
    skill: dict
    evidence_trajectory_ids: list[str]
    confidence_score: float
    status: str


@app.post("/trajectories", status_code=201)
async def submit_trajectory(req: TrajectorySubmitRequest) -> dict:
    """Receive a completed trajectory from Rust and store it for analysis."""
    if not collector:
        raise HTTPException(503, "Service not ready")

    from .service.collector import Trajectory, TrajectoryTurn

    traj = Trajectory(
        id=uuid.UUID(req.id),
        workspace_id=uuid.UUID(req.workspace_id),
        session_id=uuid.UUID(req.session_id),
        user_message=req.user_message,
        turns=[TrajectoryTurn(**t) for t in req.turns],
        outcome=req.outcome,
        duration_ms=req.duration_ms,
        model=req.model,
    )

    await collector.store(traj)
    return {"status": "stored", "trajectory_id": req.id}


@app.post("/skills/synthesize", status_code=201)
async def run_synthesis_cycle() -> list[SkillDraftResponse]:
    """Trigger a full synthesis cycle — fetch trajectories, generate skill drafts."""
    if not synthesizer:
        raise HTTPException(503, "Service not ready")

    drafts = await synthesizer.run_cycle()
    return [SkillDraftResponse(**d.to_dict()) for d in drafts]


@app.get("/health")
async def health() -> dict:
    return {
        "status": "ok",
        "service": "clawstack-learning",
        "version": "0.1.0",
    }


@app.post("/skills/evaluate/{draft_id}")
async def evaluate_draft(draft_id: str, skill: dict) -> dict:
    """Evaluate a skill draft against benchmarks."""
    if not evaluator:
        raise HTTPException(503, "Service not ready")

    result = await evaluator.evaluate(uuid.UUID(draft_id), skill)
    return {
        "draft_id": str(result.draft_id),
        "score": result.score,
        "passed": result.passed,
        "notes": result.notes,
    }
