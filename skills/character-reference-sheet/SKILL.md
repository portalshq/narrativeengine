---
name: character-reference-sheet
description: Create a high-fidelity three-view character reference sheet from character samples. Use whenever a user asks to create a character, character sheet, or reference sheet.
---

# Character Reference Sheet

## When to Apply

Reference these guidelines when the user asks to create a character, character sheet, or reference sheet. Resolve project and entity context per `px-resolve`, generate the sheet per the spec below, then persist it per `px-update` (all PX operations via `px-mcp-server`; the `px` CLI is not available for agentic use).

## Character Reference Sheet Spec

You are an expert character designer specializing in creating high-fidelity character reference sheets. Analyze the provided character samples and generate a single cohesive 3:2 image that serves as a professional reference.

### Project Context

Project-owned style references are the baseline for rendering, materials, lighting, palette, and asset conventions. The character's properties and representations apply as its identity-specific refinements. Repository context is resolved per `repository-stewardship.md` and entity context per `resolve-workflow.md` before generation. If project and character instructions truly conflict, pause and ask the user for direction. If the repository manifest cannot be read, warn the user and ask how to proceed; never silently generate without project context.

### Layout Requirements

Divide the image into three distinct columns:

- **Detailed Portrait (left):** Close-up focus on the character's face. Capture the exact eye color, facial features, makeup, and head-worn accessories in high detail.
- **Full-Body Front View (center):** Head-to-toe view from the front. Clearly show the whole outfit, proportions, and frontal details.
- **Full-Body Back View (right):** Head-to-toe view from the back. Show hair styling, rear outfit details, and accessories not visible from the front.
- **Character Name (bottom right):** The character's correctly written name in large, plain text.

### Style and Fidelity

- **Exact style match:** This is not a hand-drawn or rough sketch sheet. Match the samples' material style, lighting, and rendering quality exactly, whether photorealistic, 3D-rendered, or a specific digital-art style.
- **Character consistency:** Preserve eye color, clothing textures, specific accessories, and color palette from the supplied materials.
- **Background:** Use a clean, flat, simple background that complements the design and keeps the character as the sole focus.

### Goal and Action

Produce an official-quality reference asset with complete stylistic and design continuity from the resolved project context and supplied character samples. Generate the image, then persist it per `update-workflow.md` as the entity's `character_sheet` representation and commit the updated manifest.


## Repository Context (`repository.yaml`)

`repository.yaml` is the repository's manifest and the sole source of truth for project-wide context. It owns durable global visual or narrative style, reusable asset conventions, global exclusions, and canonical references through its `properties`, `representations`, and `references`.

Before creating an entity or generating content, resolve the repository.yaml from the same target branch as the entity work (via `px_resolve` on the repository URI). Read its `properties`, `representations`, and `references`, and resolve only the referenced resources relevant to the requested work. Treat repository context as the project-wide baseline; entity-specific identity, behavior, and representation data apply as refinements. Keep entity manifests focused on identity and entity-specific facts; do not add a repository-level summary or reference for every entity.

If project and entity instructions truly conflict, pause and ask the user for direction rather than choosing one. If the repository.yaml cannot be read, warn the user and ask how to proceed; never silently generate without project context.

Update `repository.yaml` only when the user explicitly defines or approves a project-wide property or reference. If entity work reveals a potentially reusable global fact, present it as a proposed repository update and wait for user approval before writing it. Preserve existing global context when adding an approved change. Never promote inferred project-wide facts into `repository.yaml` as part of an entity update.

Unless the user explicitly requests a different provider or storage location, call `px_init` with no provider argument and preserve the configured provider and default PX directory. Never infer a local provider from an example. Never choose an isolated storage location merely to isolate a repository. Pass a provider only when the user explicitly requests a provider change, and a storage location only when the user explicitly names one.

**No tagging.** Lore VCS has no native tag support — branches are the only mechanism for human-readable names on a revision point. Do not append tags to URIs.

Checking the current workspace is not required: PX usually stores all repositories in a centralized directory unless configured otherwise.


## Active Entity Continuity

Once a PX entity URI is established in a task, carry forward:

- the active URI, repository, entity type, and entity ID
- the active revision branch and the **target branch** (see `target-branch.md`)
- stable representation keys such as `character_sheet`, `face_sheet`, `portrait`, `model_sheet`, or `reference_image`
- user-approved identity constraints and negative constraints

Later turns that keep refining the same entity are continuity work. They must trigger `px-update` even when the user does not mention PX again and does not say "save" or "commit" or repeat the URI. Carry forward stable representation keys and identity constraints so attributes do not get dropped between turns.

