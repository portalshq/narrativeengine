from __future__ import annotations

import json

from narrativeengine import LabConfig, NarrativeLore, create_block, generate_candidate, render_lore_summary


def main() -> None:
    block = create_block("intro", "A signal appears in the archive.")
    lore = NarrativeLore(id="lore-1", title="Archive Signal", blocks=[block])
    config = LabConfig(temperature=0.7, max_candidates=4, seed=7)
    candidate = generate_candidate(lore, config)
    value = {
        "block": _dump(block),
        "candidate": _dump(candidate),
        "summary": render_lore_summary(lore),
    }
    print(json.dumps(value, separators=(",", ":")))


def _dump(value: object) -> object:
    if hasattr(value, "model_dump"):
        return value.model_dump()
    return value


if __name__ == "__main__":
    main()

