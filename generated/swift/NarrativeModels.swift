// This file is generated from Rust schemas by narrativeengine-codegen.
import Foundation

public struct NarrativeBlock: Codable, Equatable {
    public let id: String
    public let content: String
}

public struct NarrativeLore: Codable, Equatable {
    public let id: String
    public let title: String
    public let blocks: [NarrativeBlock]
}

public struct LabConfig: Codable, Equatable {
    public let temperature: Double
    public let max_candidates: UInt64
    public let seed: UInt64
}

public struct HybridCandidate: Codable, Equatable {
    public let id: String
    public let block: NarrativeBlock
    public let score: Double
    public let rationale: String
}

