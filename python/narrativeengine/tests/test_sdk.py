from __future__ import annotations

import pytest

from narrativeengine import (
    LabConfig,
    NarrativeLore,
    create_block,
    generate_candidate,
    render_lore_summary,
)


@pytest.mark.skip(reason="create_block_json not yet implemented in new architecture")
def test_create_block_and_candidate() -> None:
    block = create_block("intro", "A signal appears in the archive.")
    lore = NarrativeLore(id="lore-1", title="Archive Signal", blocks=[block])
    candidate = generate_candidate(lore, LabConfig(temperature=0.7, max_candidates=4, seed=7))

    assert candidate.id == "candidate-lore-1-7"
    assert candidate.block.id == "lore-1:hybrid"
    assert render_lore_summary(lore) == "Archive Signal contains 1 block(s) and 6 word(s)."

