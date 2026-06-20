# NAP Structured Merge Engine v2

## Canonical Merge Semantics

This document is the **source of truth** for NAP merge behavior.

All implementations must implement these rules exactly. Protocol invariants
are hardcoded — they are never configurable, never expressed in SDL, and
must never vary across implementations.

---

## Table of Contents

1. [Architecture](#1-architecture)
2. [Protocol Invariants](#2-protocol-invariants)
3. [Pipeline](#3-pipeline)
4. [Rule 1 — JSON AST Conversion](#4-rule-1--json-ast-conversion)
5. [Rule 2 — Missing ≠ Null](#5-rule-2--missing--null)
6. [Rule 3 — Normalize Before Diff](#6-rule-3--normalize-before-diff)
7. [Rule 4 — Diffs After Normalization](#7-rule-4--diffs-after-normalization)
8. [Rule 5 — Path-Based Three-Way Merge](#8-rule-5--path-based-three-way-merge)
9. [Rule 6 — Conflict Matrix](#9-rule-6--conflict-matrix)
10. [Rule 7 — Conflicts Never Modify State](#10-rule-7--conflicts-never-modify-state)
11. [Rule 8 — Conflict Representation](#11-rule-8--conflict-representation)
12. [Rule 9 — Schema Controls Merge Strategy](#12-rule-9--schema-controls-merge-strategy)
13. [SDL Specification](#13-sdl-specification)
14. [Merge Strategies](#14-merge-strategies)
15. [Identity Rules](#15-identity-rules)
16. [Path Resolution](#16-path-resolution)
17. [Validation Stages](#17-validation-stages)
18. [Persistence](#18-persistence)
19. [Performance](#19-performance)
20. [Common Failure Modes](#20-common-failure-modes)

---

## 1. Architecture

```
SDL (YAML)
  ↓
Schema Loader
  ↓
Normalization Engine
  ↓
JSON AST Conversion
  ↓
Path Union Discovery
  ↓
Three-Way Merge Engine
  ↓
Conflict Generator
  ↓
Validation Engine
  ↓
Atomic Persistence
  ↓
Git Commit (storage only)
```

**Key separation:**

| Layer | Responsibility |
|---|---|
| **SDL** | Schema + merge strategy metadata (varies by schema) |
| **Merge Semantics v2** | Protocol invariants (never vary) |
| **nap-core** | Implementation |

---

## 2. Protocol Invariants

These are **non-negotiable** — they apply to every merge, every schema,
every implementation.

| # | Invariant | Description |
|---|---|---|
| 1 | `missing ≠ null` | A missing field means "no opinion". Explicit `null` means "delete". |
| 2 | Normalize before merge | Candidates must be normalized against base before any comparison. |
| 3 | Merge over path union | The merge must evaluate the union of all paths in base, current, and proposed. |
| 4 | Identity immutable | Identity fields cannot change. Identity mutation is always a conflict. |
| 5 | Validate before persist | Every merge result must pass SDL validation before writing. |
| 6 | Validate after merge | The merged document must be re-validated against SDL types. |
| 7 | Deterministic execution | Same inputs must always produce same outputs. |
| 8 | Atomic persistence | Writes must use temp-file + flush + fsync + rename. |
| 9 | No Git conflict markers | Conflicts are structured objects, never text markers. |

---

## 3. Pipeline

```
1. Load + validate SDL (Stage 1)
2. Convert base/current/proposed to JSON AST (serde_json::Value)
3. Validate inputs against SDL types (Stage 2)
4. Normalize both candidates against base
5. Build path maps for all three documents
6. Compute path union
7. For each path in union:
   a. Resolve values by path
   b. Look up merge strategy from SDL
   c. Apply conflict matrix (Rule 6)
   d. For array strategies: identity-aware merge
   e. Check identity immutability
8. If conflicts → return conflicts (no write)
9. Validate merged result (Stage 3)
10. Return merged document
```

---

## 4. Rule 1 — JSON AST Conversion

Documents are converted to JSON AST before merge.

```
YAML → serde_yaml → serde_json::Value → merge engine
```

The merge engine **never** operates directly on YAML text.

---

## 5. Rule 2 — Missing ≠ Null

This is the most important protocol invariant.

| State | Meaning | Behavior |
|---|---|---|
| Missing | No opinion | Copy from base during normalization / treat as "unchanged" |
| `null` | Explicit deletion | Preserve — do NOT copy from base |
| Value | Modified | Preserve as-is |

**Example:**

```json
// Base
{ "name": "Obi Wan", "homeworld": "Stewjon" }

// Proposed
{ "name": "Obi Wan Kenobi" }

// Result (missing = no change)
{ "name": "Obi Wan Kenobi", "homeworld": "Stewjon" }
```

**Deletion example:**

```json
// Base
{ "homeworld": "Stewjon" }

// Proposed
{ "homeworld": null }

// Result (null = explicit deletion)
{ "homeworld": null }
```

---

## 6. Rule 3 — Normalize Before Diff

```python
normalize(base, candidate)
```

For every leaf path in `base`:

- If path **missing** in `candidate`: copy from `base`
- If path **exists** in `candidate`: leave unchanged
- If path is **explicitly null**: leave null

Normalization ensures that omission is never treated as deletion.

---

## 7. Rule 4 — Diffs After Normalization

```
base, current, proposed
  ↓
normalized_current = normalize(base, current)
normalized_proposed = normalize(base, proposed)
  ↓
diff(normalized_current, base) → patchA
diff(normalized_proposed, base) → patchB
```

Diff is a **presentation layer** — the merge engine does NOT depend on it.
Diff is exposed as a public API for review workflows, agent conflict
resolution, and UI rendering.

---

## 8. Rule 5 — Path-Based Three-Way Merge

Every path in the **path union** (all paths across base, current, proposed)
is evaluated independently. The merge is per-path, not per-document.

---

## 9. Rule 6 — Conflict Matrix

For each path, evaluate three values: `base`, `current`, `proposed`.

| Case | Condition | Result |
|---|---|---|
| Accept proposed | `current == base && proposed != base` | Accept `proposed` |
| Accept current | `proposed == base && current != base` | Accept `current` |
| Accept either | `current == proposed` | Accept either (both same) |
| Accept base | All equal | Accept `base` |
| **Conflict** | `current != base && proposed != base && current != proposed` | Conflict |

---

## 10. Rule 7 — Conflicts Never Modify State

When a merge produces conflicts, **no write occurs**.

The caller receives `MergeResult::Conflicts(Vec<Conflict>)` and must
resolve conflicts before re-attempting the merge.

---

## 11. Rule 8 — Conflict Representation

Conflicts are **structured** — no textual markers, no Git syntax,
no YAML corruption.

```rust
pub struct Conflict {
    pub path: String,           // e.g. "root.properties.homeworld"
    pub conflict_type: ConflictType,
    pub base: Value,
    pub current: Value,
    pub proposed: Value,
}

pub enum ConflictType {
    ValueMismatch,       // values differ
    TypeMismatch,        // types differ (string vs number)
    StructuralConflict,  // structure differs (object vs array)
    IdentityMutation,    // identity field changed
}
```

---

## 12. Rule 9 — Schema Controls Merge Strategy

Every property MUST define `type` and `merge` in SDL.

No inferred merge behavior.
No fallback heuristics.
Schema omission is a validation error.

---

## 13. SDL Specification

### Format

```yaml
schema:
  version: "1.0"
  required:
    - id
  properties:
    <property_path>:
      type: <type>
      merge:
        type: <strategy>
        identity:          # required for ordered_unique, set_union, edge_list
          mode: key        # key | primitive_value
          key: id          # required when mode = key
        source_key: ...    # required for edge_list
        target_key: ...    # required for edge_list
```

### Supported Types

| Type | Description |
|---|---|
| `string` | UTF-8 string value |
| `number` | Integer or float |
| `boolean` | true/false |
| `object` | Nested JSON object (for deep_merge) |
| `array` | Ordered list (for ordered_unique, set_union, edge_list) |

### Supported Merge Strategies

| Strategy | Purpose |
|---|---|
| `replace` | Simple value overwrite per conflict matrix |
| `deep_merge` | Recursive object merge (object type only) |
| `atomic` | Whole-value replacement — any divergence = conflict |
| `ordered_unique` | Ordered list with identity deduplication |
| `set_union` | Unordered set with identity deduplication |
| `edge_list` | Graph edge list with source/target/identity |

---

## 14. Merge Strategies

### replace

Simple value overwrite using the conflict matrix. No recursion, no
identity handling. Suitable for strings, numbers, booleans, and
simple scalar fields.

### deep_merge

Only valid for `object`-type properties. Recursively applies the
conflict matrix to each sub-key. Additions from both sides are
preserved. Conflicts when both sides modify the same sub-key differently.

### atomic

The entire value is treated as a single unit. Any divergent change
between current and proposed creates a conflict, even if the changes
would be semantically mergeable (e.g., adding non-overlapping keys
to an object).

### ordered_unique

For ordered lists where element identity matters:

- Preserves base order
- Appends new items from current and proposed
- Deduplicates by identity
- Identity can be:
  - `primitive_value` — for arrays of scalars (strings, numbers)
  - `key` — for arrays of objects with an identity field
- Conflict when both branches modify the same identity differently

**Example:**

```yaml
# Base
characters:
  - { id: A }

# Current
characters:
  - { id: A }
  - { id: B }

# Proposed
characters:
  - { id: A }
  - { id: C }

# Result
characters:
  - { id: A }
  - { id: B }
  - { id: C }
```

### set_union

For unordered sets:

- Union of all unique items from base, current, proposed
- Deduplicates by identity
- Order is NOT preserved
- Conflict when both branches modify the same identity differently

### edge_list

For graph relationship lists:

- Each edge has a unique identity, source, and target
- Independent edge additions merge automatically
- Conflict when the same edge identity is modified differently
- Never use array index — always use identity

---

## 15. Identity Rules

### Primitive Value Mode

```yaml
identity:
  mode: primitive_value
```

The value itself IS the identity. Used for arrays of strings or numbers.

```yaml
tags: ["fantasy", "sci-fi"]
```

Identity of `"fantasy"` is the string `"fantasy"` itself.

### Key Mode

```yaml
identity:
  mode: key
  key: id
```

The identity is the value of a named field within each object element.

```yaml
characters:
  - id: obiwan
    name: Obi-Wan Kenobi
```

Identity of `{ id: obiwan, ... }` is the string `"obiwan"`.

### Identity Immutability

Identity fields **cannot change**. An attempt to mutate an identity
field produces `Conflict::IdentityMutation`.

```json
// Base:      { id: "obiwan", name: "Obi-Wan" }
// Current:   { id: "ben_kenobi", name: "Obi-Wan" }
// Result:    CONFLICT — identity mutation
```

Detection is **position-based**: if the item at position 0 in the base
array has identity "obiwan" and the item at position 0 in the current
array has identity "ben_kenobi", that is a mutation even if the
identity-based lookup maps show two unrelated items.

---

## 16. Path Resolution

### Canonical Path Format

```
property.sub_property
array_identity[identity_value].sub_property
```

**Examples:**

| Path | Description |
|---|---|
| `name` | Top-level string |
| `properties.homeworld` | Nested property |
| `characters[obiwan]` | Array element by identity |
| `characters[obiwan].name` | Sub-field of array element |
| `tags[0]` | Array element by index (fallback, no identity) |

### Rules

- **Never use array index** for identity-keyed arrays
- Always use identity value: `characters[obiwan]` not `characters[0]`
- Index-based paths are only produced for arrays without identity rules
- The `root.` prefix is presentation-only — never stored internally

### Path Union

The merge engine evaluates the **union** of all paths across base,
current, and proposed:

```
path_union = paths(base) ∪ paths(current) ∪ paths(proposed)
```

This ensures that paths added by either branch are evaluated, even
if they don't exist in base.

### Sub-path Filtering

For identity-keyed arrays, sub-paths of array items (like
`characters[obiwan].name` or `tags[fantasy]`) are **skipped** during
merge — the array-level strategy handles the entire array. Sub-paths
are still available in the diff API for detailed change inspection.

---

## 17. Validation Stages

### Stage 1 — SDL Validation

Before any merge, validate the SDL document:

- All properties have `type` and `merge` fields
- `ordered_unique` and `set_union` have identity rules
- `edge_list` has `source_key`, `target_key`, and identity
- `deep_merge` is only used with `object` type
- Identity keys are non-empty

### Stage 2 — Manifest Validation

Before merge, validate base/current/proposed against SDL:

- Required fields exist
- Field types match the schema
- Array items with key-mode identity have the identity field

### Stage 3 — Post-Merge Validation

After merge, validate the result:

- Same checks as Stage 2
- Catches engine bugs that could produce invalid state

### Ordering

```
Validate SDL (Stage 1)
  ↓
Validate inputs (Stage 2)
  ↓
Merge
  ↓
Validate result (Stage 3)
  ↓
Atomic persist
```

Validation must complete before persistence.

---

## 18. Persistence

### Atomic Write Protocol

```python
write_temp_file(path, content)
flush()
fsync()
rename(temp_path, path)
```

### Implementation

```rust
pub fn atomic_write(path: &Path, content: &[u8]) -> Result<(), AtomicWriteError>
```

1. Create a temporary file in the same directory (same filesystem)
2. Write all content
3. Flush the stream
4. `fsync` the file (ensure data hits disk)
5. `rename` to the target path (atomic on Unix)
6. `fsync` the parent directory (durable rename)

On error, the temp file is cleaned up. The target file is never corrupted.

---

## 19. Performance

### Requirements

Support 5,000–10,000 fields without quadratic scans.

### Design

- **Precomputed path maps** — flat `BTreeMap<String, Value>` built in
  a single traversal. No repeated tree walks.
- **Identity lookup maps** — `BTreeMap<String, Value>` for O(log n)
  identity-based lookups.
- **Hash-based membership checks** — `BTreeSet` for deduplication and
  membership testing.
- **Path union** — deduplicated, sorted for deterministic iteration.

### What NOT to do

- Do not repeatedly traverse the entire JSON tree
- Do not use linear scans for identity lookups
- Do not compute diffs during merge (diff is a separate concern)

---

## 20. Common Failure Modes

### Failure 1: Treating missing field as delete

**Forbidden.** Missing = no change. Only explicit `null` is deletion.

### Failure 2: Diff before normalization

**Forbidden.** Always normalize candidates against base before
generating diffs.

### Failure 3: Using array indices as graph identity

**Forbidden.** Identity-keyed arrays must use the identity value,
never the array index.

### Failure 4: Schema-less properties

**Forbidden.** Every property must have a type and merge strategy
in SDL. Schema omission is a validation error.

### Failure 5: Git merge markers entering repository

**Forbidden.** Conflicts are structured objects. Never write textual
conflict markers to any file.

### Failure 6: Validation after persistence

**Forbidden.** Validation must complete before the write occurs.
A merged document that fails validation must never be persisted.

### Failure 7: Identity mutation

**Forbidden.** Identity fields are immutable. Any change to an
identity value (detected positionally) produces a conflict.

---

## Appendix A: SDL Example

```yaml
schema:
  version: "1.0"
  required:
    - id
  properties:
    id:
      type: string
      merge:
        type: atomic
    name:
      type: string
      merge:
        type: replace
    version:
      type: number
      merge:
        type: atomic
    properties.homeworld:
      type: string
      merge:
        type: replace
    tags:
      type: array
      merge:
        type: ordered_unique
        identity:
          mode: primitive_value
    characters:
      type: array
      merge:
        type: ordered_unique
        identity:
          mode: key
          key: id
    edges:
      type: array
      merge:
        type: edge_list
        source_key: source_id
        target_key: target_id
        identity:
          mode: key
          key: edge_id
    metadata:
      type: object
      merge:
        type: deep_merge
```

## Appendix B: Conflict Example

```json
{
  "conflicts": [
    {
      "path": "root.properties.homeworld",
      "conflict_type": "value_mismatch",
      "base": "Stewjon",
      "current": "Tatooine",
      "proposed": "Coruscant"
    },
    {
      "path": "root.characters[obiwan].affiliation",
      "conflict_type": "value_mismatch",
      "base": "Jedi Order",
      "current": "Jedi Order",
      "proposed": "Galactic Republic"
    }
  ]
}
```

---

## Appendix C: Determinism Guarantee

The same three inputs (`base`, `current`, `proposed`) with the same
SDL document must ALWAYS produce the same output.

This is guaranteed by:

- Sorted path iteration (BTreeMap + sorted Vec)
- Deterministic merge strategies
- No randomness in any algorithm
- Well-defined conflict matrix

---

*This document is the canonical specification for NAP Structured
Merge Engine v2. All implementations must conform to these semantics.*
