# NarrativeEngine TypeScript SDK

The TypeScript SDK is a thin NAPI-RS wrapper over the canonical Rust implementation. All TypeScript interfaces are generated from Rust schemas.

## Installation

```sh
npm install narrativeengine
```

## Usage

```ts
import { createBlock, generateCandidate } from "narrativeengine";

const block = createBlock("intro", "A signal appears in the archive.");
const candidate = generateCandidate(
  { id: "lore-1", title: "Archive Signal", blocks: [block] },
  { temperature: 0.7, max_candidates: 4, seed: 7 },
);
```

## Development

```sh
npm install
npm run build
npm test
npm run lint
```

## Build

Native bindings are compiled by `@napi-rs/cli` from `crates/typescript-bindings`.

## Release

Packages are built through GitHub Actions and published by `scripts/publish-all.sh`.

