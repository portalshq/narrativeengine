from __future__ import annotations

import json
from typing import Any, TypeVar, cast

from . import _native
from .types import HybridCandidate, LabConfig, NarrativeBlock, NarrativeLore

T = TypeVar("T")


def create_block(id: str, content: str) -> NarrativeBlock:
    return _coerce(NarrativeBlock, json.loads(_native.create_block_json(id, content)))


def generate_candidate(lore: NarrativeLore, config: LabConfig) -> HybridCandidate:
    return _coerce(
        HybridCandidate,
        json.loads(_native.generate_candidate_json(_to_json(lore), _to_json(config))),
    )


def render_lore_summary(lore: NarrativeLore) -> str:
    return cast(str, _native.render_lore_summary_json(_to_json(lore)))


def schema_bundle() -> dict[str, Any]:
    return cast(dict[str, Any], json.loads(_native.schema_bundle_json()))


def version() -> str:
    return cast(str, _native.version())


def _to_json(value: object) -> str:
    if hasattr(value, "model_dump"):
        return json.dumps(value.model_dump(), separators=(",", ":"))
    return json.dumps(value, separators=(",", ":"))


def _coerce(model: type[T], data: dict[str, Any]) -> T:
    if hasattr(model, "model_validate"):
        return cast(T, model.model_validate(data))
    return cast(T, data)


__all__ = [
    "HybridCandidate",
    "LabConfig",
    "NarrativeBlock",
    "NarrativeLore",
    "create_block",
    "generate_candidate",
    "render_lore_summary",
    "schema_bundle",
    "version",
]

