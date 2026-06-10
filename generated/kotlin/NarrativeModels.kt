// This file is generated from Rust schemas by narrativeengine-codegen.
package com.narrativeengine

data class NarrativeBlock(
    val id: String,
    val content: String
)

data class NarrativeLore(
    val id: String,
    val title: String,
    val blocks: List<NarrativeBlock>
)

data class LabConfig(
    val temperature: Double,
    val max_candidates: Long,
    val seed: Long
)

data class HybridCandidate(
    val id: String,
    val block: NarrativeBlock,
    val score: Double,
    val rationale: String
)

