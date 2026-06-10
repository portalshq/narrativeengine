# This file is generated from Rust schemas by narrativeengine-codegen.
from __future__ import annotations

from typing import Any

try:
    from pydantic import BaseModel, ConfigDict
    _HAS_PYDANTIC = True
except ImportError:
    BaseModel = object  # type: ignore[assignment]
    ConfigDict = dict  # type: ignore[assignment]
    _HAS_PYDANTIC = False


if _HAS_PYDANTIC:
    class NarrativeBlock(BaseModel):
        model_config = ConfigDict(extra="forbid")
        id: str
        content: str

    class NarrativeLore(BaseModel):
        model_config = ConfigDict(extra="forbid")
        id: str
        title: str
        blocks: list[NarrativeBlock]

    class LabConfig(BaseModel):
        model_config = ConfigDict(extra="forbid")
        temperature: float
        max_candidates: int
        seed: int

    class HybridCandidate(BaseModel):
        model_config = ConfigDict(extra="forbid")
        id: str
        block: NarrativeBlock
        score: float
        rationale: str

else:
    NarrativeBlock = dict[str, Any]
    NarrativeLore = dict[str, Any]
    LabConfig = dict[str, Any]
    HybridCandidate = dict[str, Any]


__all__ = [
    "NarrativeBlock",
    "NarrativeLore",
    "LabConfig",
    "HybridCandidate",
]
