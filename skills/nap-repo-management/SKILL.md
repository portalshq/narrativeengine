# NAP Skill: Repository Management

## Description
This skill enables the agent to initialize and manage Narrative Addressing Protocol (NAP) universes using the CLI. A universe is the top-level repository that contains entities like characters, locations, and assets.

## Core Commands

* **Initialize a Universe:** To create a new universe repository in the current directory, use `nap init <universe_name>`.
  * *Example:* `nap init toystory`.
  * *Note:* This creates a directory containing a `.nap/` configuration folder, a `universe.yaml` manifest, and subdirectories for entity types (characters, locations, etc.).

* **Branching:** To create a new timeline or snapshot, use `nap branch <universe_name> <branch_name>`.
  * *Example:* `nap branch toystory classic`.

## Critical Guardrails & Context
* **No Tagging:** Do not attempt to use `nap tag` or append tags to URIs. The underlying Lore VCS does not natively support tags. Branches are the primary and only way to apply a human-readable name to a specific point in the revision history.