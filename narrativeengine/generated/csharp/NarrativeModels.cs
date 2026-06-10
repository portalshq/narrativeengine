// This file is generated from Rust schemas by narrativeengine-codegen.
using System.Collections.Generic;

namespace NarrativeEngine;

public sealed record NarrativeBlock(
    string Id,
    string Content
);

public sealed record NarrativeLore(
    string Id,
    string Title,
    IReadOnlyList<NarrativeBlock> Blocks
);

public sealed record LabConfig(
    double Temperature,
    uint MaxCandidates,
    ulong Seed
);

public sealed record HybridCandidate(
    string Id,
    NarrativeBlock Block,
    double Score,
    string Rationale
);

