/* This file is generated from Rust schemas by narrativeengine-codegen. */

export interface NarrativeBlock {
  id: string;
  content: string;
}

export interface NarrativeLore {
  id: string;
  title: string;
  blocks: NarrativeBlock[];
}

export interface LabConfig {
  temperature: number;
  max_candidates: number;
  seed: number;
}

export interface HybridCandidate {
  id: string;
  block: NarrativeBlock;
  score: number;
  rationale: string;
}

