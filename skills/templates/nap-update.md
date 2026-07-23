---
name: nap-update
description: Update entity properties in a workflow and properly store properties and assets back into the entity manifest, ensuring proper provenance.
metadata:
  author: portals
  version: "{{version}}"
---

# NAP Skill: Entity Update and Creative Workflow Integration

Update entity properties in a workflow and properly store properties and assets back into the entity manifest, ensuring proper provenance.

## When to Apply

Reference these guidelines when:
- Making changes to entity properties in a workflow
- Generating new assets as part of a workflow
- Storing assets back into the entity manifest

## Core Commands

* **Set a Property:** To modify or add a property to an entity's manifest, use `nap set <URI> <property_key> <value>`.
  * *Example:* `nap set nap://toystory/character/woody toy_type pullstring_cowboy`.
  * *Example:* `nap set nap://toystory/character/woody location "nap://toystory/location/andysroom"`.

## Workflow Pipeline
When using a tool (like a text model, Midjourney, or video generation platform) to generate assets for a NAP entity, you must follow this exact sequence:

1. **Resolve/Query:** Fetch necessary context using `nap resolve` or `nap query` (see Entity Access Skill).
2. **Generate:** Use your available creative tools to generate the text, image, or video based on the context.
3. **Store Representation:** When saving the generated asset back to the entity's manifest, you must track its provenance. Ensure the manifest is updated with the following structured YAML/JSON:
   * **Hash:** Every piece of content must be strictly indexed by its BLAKE3 hash. Do not use SHA-256.
   * **Provenance Tracking:** Record the AI generation metadata, including the `model` used (e.g., "midjourney-v6" or the specific LLM), the `prompt_hash`, and any `derived_from` URIs.

*Example of updating a manifest's representations block via CLI/script editing:*
```yaml
representations:
  ai_description:
    hash: "blake3:abc123def..." 
    format: text
    provenance:
      model: "claude-3-opus"
      prompt_hash: "blake3:def456..."
      derived_from: "nap://toystory/character/woody"
```

## CLI Reference

{{include docs/generated/cli.md}}

## Global Options

{{include docs/generated/options.md}}

## Environment Variables

{{include docs/generated/environment.md}}
