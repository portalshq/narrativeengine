from __future__ import annotations

import json
from typing import Any, cast

from . import _native
from .models import HybridCandidate, LabConfig, NarrativeBlock, NarrativeLore


def create_block(id: str, content: str) -> NarrativeBlock:
    return _coerce(NarrativeBlock, json.loads(_native.create_block_json(id, content)))


def generate_candidate(lore: NarrativeLore, config: LabConfig) -> HybridCandidate:
    return _coerce(
        HybridCandidate,
        json.loads(_native.generate_candidate_json(_to_json(lore), _to_json(config))),
    )


def render_lore_summary(lore: NarrativeLore) -> str:
    return _native.render_lore_summary_json(_to_json(lore))


def schema_bundle() -> dict[str, Any]:
    return cast(dict[str, Any], json.loads(_native.schema_bundle_json()))


def version() -> str:
    return _native.version()


def _to_json(value: object) -> str:
    if hasattr(value, "model_dump"):
        return json.dumps(value.model_dump(), separators=(",", ":"))
    return json.dumps(value, separators=(",", ":"))


def _coerce[T](model: type[T], data: dict[str, Any]) -> T:
    if hasattr(model, "model_validate"):
        return cast(T, model.model_validate(data))  # type: ignore[attr-defined]
    return cast(T, data)


# ─────────────────────────────────────────────────────────────────────────────
# NarrativeEngine class
# ─────────────────────────────────────────────────────────────────────────────

class NarrativeEngine:
    def __init__(self) -> None:
        self._engine: Any = _native.PyNarrativeEngine()  # type: ignore[attr-defined]

    def generate_context(self, channel_id: str, query: str) -> str:
        return cast(str, self._engine.generate_context(channel_id, query))  # type: ignore[no-any-return]

    def generate_block(
        self, channel_id: str, input_query: str, parameters: dict[str, Any]
    ) -> dict[str, Any]:
        return json.loads(
            self._engine.generate_block(channel_id, input_query, _to_json(parameters))
        )

    def generate_blocks_sequential(
        self, channel_id: str, previous_context: str, options: dict[str, Any]
    ) -> dict[str, Any]:
        return json.loads(
            self._engine.generate_blocks_sequential(
                channel_id, previous_context, _to_json(options)
            )
        )

    def generate_blocks_parallel(
        self, channel_id: str, branch_contexts: list[str], options: dict[str, Any]
    ) -> dict[str, Any]:
        return json.loads(
            self._engine.generate_blocks_parallel(
                channel_id, branch_contexts, _to_json(options)
            )
        )

    def set_lab_config(self, config: dict[str, Any]) -> None:
        self._engine.set_lab_config(_to_json(config))

    def get_lab_config(self) -> dict[str, Any]:
        return json.loads(self._engine.get_lab_config())


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
    "NarrativeEngine",
]

