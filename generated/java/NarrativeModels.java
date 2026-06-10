// This file is generated from Rust schemas by narrativeengine-codegen.
package com.narrativeengine;

import java.util.List;

public final class NarrativeModels {
    private NarrativeModels() {}

    public record NarrativeBlock(
        String id,
        String content
    ) {}

    public record NarrativeLore(
        String id,
        String title,
        List<NarrativeBlock> blocks
    ) {}

    public record LabConfig(
        double temperature,
        int max_candidates,
        long seed
    ) {}

    public record HybridCandidate(
        String id,
        NarrativeBlock block,
        double score,
        String rationale
    ) {}

}
