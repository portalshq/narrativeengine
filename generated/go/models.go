// This file is generated from Rust schemas by narrativeengine-codegen.
package narrativeengine

type NarrativeBlock struct {
	Id string `json:"id"`
	Content string `json:"content"`
}

type NarrativeLore struct {
	Id string `json:"id"`
	Title string `json:"title"`
	Blocks []NarrativeBlock `json:"blocks"`
}

type LabConfig struct {
	Temperature float64 `json:"temperature"`
	MaxCandidates uint32 `json:"max_candidates"`
	Seed uint64 `json:"seed"`
}

type HybridCandidate struct {
	Id string `json:"id"`
	Block NarrativeBlock `json:"block"`
	Score float64 `json:"score"`
	Rationale string `json:"rationale"`
}

