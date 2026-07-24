---
name: nap-resolve
description: Create new nap entities, resolve NAP URIs into manifests, and perform subtree queries to extract specific context for AI workflows.
metadata:
  author: portals
  version: "{{version}}"
---

# NAP Skill: Entity Creation and Resolution

Create new nap entities, resolve NAP URIs into manifests, and perform subtree queries to extract specific context for AI workflows. 

## When to Apply

Reference these guidelines when:
- Creating new entities (e.g., characters, locations, items, events)
- Resolving NAP URIs into manifests
- Querying subtree data for creative workflows

## Core Commands

* **Create an Entity:** Use `nap create <entity_type> <entity_id> -u <repository> -n "<Name>"`.
  * *Example:* `nap create character woody -u toystory -n "Sheriff Woody"`.

* **Resolve a Full Manifest:** To fetch the complete YAML manifest of a resource, use `nap resolve <URI>`.
  * *Example:* `nap resolve nap://toystory/character/woody`.
  * You can specify output formats using the `-f` flag, such as `-f json` or `-f yaml`.

* **Query a Subtree (Crucial for AI workflows):** To fetch only specific data and save token context, append a fragment to the URI or use the query command. Subtree queries let AI agents fetch exactly the data they need, preserving token budget.
  * *Example (Fragment):* `nap resolve nap://toystory/character/woody#properties.toy_type`.
  * *Example (Command):* `nap query nap://toystory/character/woody properties`.

## Generation Context Requirements

Before generating content from a NAP entity:

1. Resolve the entity and gather every property relevant to the requested content, medium, scene, identity, style, behavior, or continuity.
2. Gather applicable `representations` and `references`, including referenced entities that affect the requested result.
3. Inspect the entity's properties for negative or exclusion constraints. Property names are schema-flexible, so look for semantically relevant keys such as `negative_constraints`, `exclusions`, `avoid`, `forbidden`, or other domain-specific constraint names rather than assuming one fixed path.
4. Always provide discovered negative constraints to the content generator. Use a dedicated negative-prompt or exclusion field when the generator supports one; otherwise include them in the main prompt as explicit "do not" instructions. The absence of negative constraints does not block generation.
5. For visual appearance, treat image and video representations as the source of truth. Use textual properties to supply supporting detail and explicit constraints without replacing observable visual identity.
6. In multi-entity generations, keep each entity's properties, representations, references, and negative constraints associated with that entity so details do not bleed between entities.

## Creative Workflow Example (Context Gathering)
If tasked with generating an image of a location:
1. Resolve `nap://toystory/location/andysroom` to identify its available properties, representations, and references.
2. Query relevant properties for atmospheric details and any negative or exclusion constraints.
3. Resolve the applicable visual representations and referenced entities.
4. Construct the generation prompt from that context, providing all discovered negative constraints to the generator.

## CLI Reference

{{include docs/generated/cli.md}}

## Global Options

{{include docs/generated/options.md}}

## Environment Variables

{{include docs/generated/environment.md}}
