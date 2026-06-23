/**
 * Management tools — universe init, entity CRUD, VCS operations, remotes, etc.
 *
 * These tools MODIFY state. All have destructiveHint: false because they
 * are versioned (every change is a Git commit) and can be rolled back.
 */
import { z } from "zod";
import { initUniverse, createEntity, deleteEntity, listBranches, createBranch, switchBranch, listTags, createTag, listRemotes, addRemote, removeRemote, pushUniverse, pullUniverse, computeContentHash, validateManifest, NapApiError, } from "../services/api.js";
import { CONFIG, ENTITY_TYPES } from "../constants.js";
export function registerManagementTools(server) {
    // ── nap_init_universe ──────────────────────────────────────────────────
    server.registerTool("nap_init_universe", {
        title: "Initialize Universe",
        description: `Initialize a new NAP universe repository.

Creates a new Git-backed universe with the standard directory structure
(characters/, locations/, scenes/, props/) and an initial world manifest.

Args:
  universe (string): Universe name (e.g., 'starwars', 'toystory').

Returns: { success: boolean, universe: string, path: string }

Examples:
  - "Create a Star Wars universe" → universe="starwars"
  - "Start a new Toy Story universe" → universe="toystory"`,
        inputSchema: z
            .object({
            universe: z
                .string()
                .min(1)
                .describe("Universe name (e.g., 'starwars', 'toystory', 'middleearth'). Use lowercase and dashes/underscores."),
        })
            .strict(),
        annotations: {
            readOnlyHint: false,
            destructiveHint: false,
            idempotentHint: true,
            openWorldHint: true,
        },
    }, async (params) => {
        try {
            const result = await initUniverse(params.universe);
            return {
                content: [{ type: "text", text: JSON.stringify(result, null, 2) }],
            };
        }
        catch (err) {
            return handleManagementError(err);
        }
    });
    // ── nap_create_entity ──────────────────────────────────────────────────
    const CreateEntityInputSchema = z
        .object({
        universe: z
            .string()
            .min(1)
            .describe("Universe name (e.g., 'starwars')."),
        entity_type: z
            .enum(ENTITY_TYPES)
            .describe("Entity type: 'character', 'location', 'scene', 'prop', 'world'."),
        entity_id: z
            .string()
            .min(1)
            .describe("Entity ID (slug). Lowercase, no spaces. e.g., 'lukeskywalker', 'tatooine'."),
        name: z
            .string()
            .min(1)
            .describe("Human-readable display name. e.g., 'Luke Skywalker'."),
        author: z
            .string()
            .default("nap-agent")
            .describe("Author identifier (e.g., 'agent@studio.com')."),
    })
        .strict();
    server.registerTool("nap_create_entity", {
        title: "Create Entity",
        description: `Create a new narrative entity in a universe.

This creates a new manifest file, commits it to the universe's Git history,
and returns the NAP URI and commit ID. The entity starts with no properties
— use nap_set_property to add data.

Args:
  universe (string): Universe name (e.g., 'starwars')
  entity_type (string): Entity type ('character', 'location', 'scene', 'prop', 'world')
  entity_id (string): Slug identifier (e.g., 'lukeskywalker')
  name (string): Display name (e.g., 'Luke Skywalker')
  author (string, optional): Author identifier (default: 'nap-agent')

Returns: { uri: string, commit_id: string, version: number }

Examples:
  - "Create Luke Skywalker" → entity_type="character", entity_id="lukeskywalker", name="Luke Skywalker"
  - "Create Tatooine" → entity_type="location", entity_id="tatooine", name="Tatooine"
  - "Create the Cantina scene" → entity_type="scene", entity_id="cantina", name="Mos Eisley Cantina"`,
        inputSchema: CreateEntityInputSchema,
        annotations: {
            readOnlyHint: false,
            destructiveHint: false,
            idempotentHint: false,
            openWorldHint: true,
        },
    }, async (params) => {
        try {
            const result = await createEntity(params.universe, params.entity_type, params.entity_id, { name: params.name, author: params.author });
            return {
                content: [{ type: "text", text: JSON.stringify(result, null, 2) }],
            };
        }
        catch (err) {
            return handleManagementError(err);
        }
    });
    // ── nap_delete_entity ──────────────────────────────────────────────────
    const DeleteEntityInputSchema = z
        .object({
        universe: z
            .string()
            .min(1)
            .describe("Universe name (e.g., 'starwars')."),
        entity_type: z
            .enum(ENTITY_TYPES)
            .describe("Entity type to delete."),
        entity_id: z
            .string()
            .min(1)
            .describe("Entity ID (slug) to delete."),
        author: z
            .string()
            .default("nap-agent")
            .describe("Author identifier for the deletion commit."),
    })
        .strict();
    server.registerTool("nap_delete_entity", {
        title: "Delete Entity",
        description: `Delete a narrative entity from a universe.

Permanently removes the entity manifest and commits the deletion to history.
The deletion can be reverted using nap_revert_commit with the resulting commit hash.

Args:
  universe (string): Universe name
  entity_type (string): Entity type
  entity_id (string): Entity ID to delete
  author (string, optional): Author identifier (default: 'nap-agent')

Returns: { commit_id: string }

Examples:
  - "Delete Luke Skywalker" → entity_type="character", entity_id="lukeskywalker"
  - "Remove Tatooine" → entity_type="location", entity_id="tatooine"`,
        inputSchema: DeleteEntityInputSchema,
        annotations: {
            readOnlyHint: false,
            destructiveHint: true,
            idempotentHint: false,
            openWorldHint: true,
        },
    }, async (params) => {
        try {
            const result = await deleteEntity(params.universe, params.entity_type, params.entity_id, params.author);
            return {
                content: [{ type: "text", text: JSON.stringify(result, null, 2) }],
            };
        }
        catch (err) {
            return handleManagementError(err);
        }
    });
    // ── nap_list_branches ──────────────────────────────────────────────────
    const ListBranchesInputSchema = z
        .object({
        universe: z
            .string()
            .min(1)
            .describe("Universe name (e.g., 'starwars')."),
    })
        .strict();
    server.registerTool("nap_list_branches", {
        title: "List Branches",
        description: `List all Git branches in a universe repository.

Returns the list of branch names including the current active branch.

Args:
  universe (string): Universe name

Returns: { branches: string[] }

Examples:
  - "What branches exist in Star Wars?" → universe="starwars"
  - "Show me all branches in Toy Story" → universe="toystory"`,
        inputSchema: ListBranchesInputSchema,
        annotations: {
            readOnlyHint: true,
            destructiveHint: false,
            idempotentHint: true,
            openWorldHint: false,
        },
    }, async (params) => {
        try {
            const result = await listBranches(params.universe);
            return {
                content: [{ type: "text", text: JSON.stringify(result, null, 2) }],
            };
        }
        catch (err) {
            return handleManagementError(err);
        }
    });
    // ── nap_create_branch ──────────────────────────────────────────────────
    const CreateBranchInputSchema = z
        .object({
        universe: z
            .string()
            .min(1)
            .describe("Universe name (e.g., 'starwars')."),
        name: z
            .string()
            .min(1)
            .describe("Branch name (e.g., 'canon', 'legends', 'what-if', 'my-experimental')."),
    })
        .strict();
    server.registerTool("nap_create_branch", {
        title: "Create Branch",
        description: `Create a new Git branch in a universe repository.

Branches let you maintain parallel versions of a universe. Each branch has
its own independent commit history. Switch between them with nap_switch_branch.

Args:
  universe (string): Universe name
  name (string): Branch name (e.g., 'canon', 'legends', 'what-if')

Returns: { branch: string }

Examples:
  - "Create a canon branch" → universe="starwars", name="canon"
  - "Create a legends/what-if branch" → universe="starwars", name="legends"
  - "Create an experimental Toy Story branch" → universe="toystory", name="experimental"`,
        inputSchema: CreateBranchInputSchema,
        annotations: {
            readOnlyHint: false,
            destructiveHint: false,
            idempotentHint: false,
            openWorldHint: true,
        },
    }, async (params) => {
        try {
            const result = await createBranch(params.universe, params.name);
            return {
                content: [{ type: "text", text: JSON.stringify(result, null, 2) }],
            };
        }
        catch (err) {
            return handleManagementError(err);
        }
    });
    // ── nap_switch_branch ──────────────────────────────────────────────────
    const SwitchBranchInputSchema = z
        .object({
        universe: z
            .string()
            .min(1)
            .describe("Universe name (e.g., 'starwars')."),
        name: z
            .string()
            .min(1)
            .describe("Branch name to switch to (e.g., 'canon', 'legends'). Use nap_list_branches to see available branches."),
    })
        .strict();
    server.registerTool("nap_switch_branch", {
        title: "Switch Branch",
        description: `Switch the active Git branch in a universe repository.

After switching, all subsequent read and write operations use the new branch.
Use nap_list_branches to discover available branches.

Args:
  universe (string): Universe name
  name (string): Branch name to switch to

Returns: { branch: string }

Examples:
  - "Switch to the canon branch" → universe="starwars", name="canon"
  - "Switch to the legends branch" → universe="starwars", name="legends"
  - "Go back to main" → universe="toystory", name="main"`,
        inputSchema: SwitchBranchInputSchema,
        annotations: {
            readOnlyHint: false,
            destructiveHint: false,
            idempotentHint: false,
            openWorldHint: true,
        },
    }, async (params) => {
        try {
            const result = await switchBranch(params.universe, params.name);
            return {
                content: [{ type: "text", text: JSON.stringify(result, null, 2) }],
            };
        }
        catch (err) {
            return handleManagementError(err);
        }
    });
    // ── nap_list_tags ──────────────────────────────────────────────────────
    const ListTagsInputSchema = z
        .object({
        universe: z
            .string()
            .min(1)
            .describe("Universe name (e.g., 'starwars')."),
    })
        .strict();
    server.registerTool("nap_list_tags", {
        title: "List Tags",
        description: `List all Git tags in a universe repository.

Tags are read-only snapshots of a branch at a specific point in time.
They are useful for marking releases or important milestones.

Args:
  universe (string): Universe name

Returns: { tags: string[] }

Examples:
  - "What tags exist in Star Wars?" → universe="starwars"
  - "List all tags in Toy Story" → universe="toystory"`,
        inputSchema: ListTagsInputSchema,
        annotations: {
            readOnlyHint: true,
            destructiveHint: false,
            idempotentHint: true,
            openWorldHint: false,
        },
    }, async (params) => {
        try {
            const result = await listTags(params.universe);
            return {
                content: [{ type: "text", text: JSON.stringify(result, null, 2) }],
            };
        }
        catch (err) {
            return handleManagementError(err);
        }
    });
    // ── nap_create_tag ─────────────────────────────────────────────────────
    const CreateTagInputSchema = z
        .object({
        universe: z
            .string()
            .min(1)
            .describe("Universe name (e.g., 'starwars')."),
        name: z
            .string()
            .min(1)
            .describe("Tag name (e.g., 'v1.0', 'episode-4-release', 'pilot-episode')."),
    })
        .strict();
    server.registerTool("nap_create_tag", {
        title: "Create Tag",
        description: `Create a Git tag at the current HEAD in a universe repository.

Tags are immutable references to a specific commit. Use them to mark releases,
important milestones, or any version you want to reference later.

Args:
  universe (string): Universe name
  name (string): Tag name (e.g., 'v1.0', 'episode-4')

Returns: { tag: string }

Examples:
  - "Tag the current state as v1.0" → universe="starwars", name="v1.0"
  - "Mark this as the pilot episode" → universe="toystory", name="pilot"`,
        inputSchema: CreateTagInputSchema,
        annotations: {
            readOnlyHint: false,
            destructiveHint: false,
            idempotentHint: false,
            openWorldHint: true,
        },
    }, async (params) => {
        try {
            const result = await createTag(params.universe, params.name);
            return {
                content: [{ type: "text", text: JSON.stringify(result, null, 2) }],
            };
        }
        catch (err) {
            return handleManagementError(err);
        }
    });
    // ── nap_remote_list ────────────────────────────────────────────────────
    const RemoteListInputSchema = z
        .object({
        universe: z
            .string()
            .min(1)
            .describe("Universe name (e.g., 'starwars')."),
    })
        .strict();
    server.registerTool("nap_remote_list", {
        title: "List Remotes",
        description: `List all Git remotes configured for a universe repository.

Returns remote names and their URLs. Use this to discover where data is synced.

Args:
  universe (string): Universe name

Returns: { remotes: Array<{ name: string, url: string }> }

Examples:
  - "What remotes does Star Wars have?" → universe="starwars"
  - "Show remote configuration for Toy Story" → universe="toystory"`,
        inputSchema: RemoteListInputSchema,
        annotations: {
            readOnlyHint: true,
            destructiveHint: false,
            idempotentHint: true,
            openWorldHint: false,
        },
    }, async (params) => {
        try {
            const result = await listRemotes(params.universe);
            return {
                content: [{ type: "text", text: JSON.stringify(result, null, 2) }],
            };
        }
        catch (err) {
            return handleManagementError(err);
        }
    });
    // ── nap_remote_add ─────────────────────────────────────────────────────
    const RemoteAddInputSchema = z
        .object({
        universe: z
            .string()
            .min(1)
            .describe("Universe name (e.g., 'starwars')."),
        name: z
            .string()
            .min(1)
            .describe("Remote name (e.g., 'origin', 'backup', 'upstream')."),
        url: z
            .string()
            .min(1)
            .describe("Remote URL (e.g., 'git@github.com:user/starwars.git', 'https://github.com/user/repo.git')."),
    })
        .strict();
    server.registerTool("nap_remote_add", {
        title: "Add Remote",
        description: `Add a Git remote to a universe repository.

Remotes enable pushing and pulling universe data between repositories.
Use nap_push and nap_pull to synchronize after configuring a remote.

Args:
  universe (string): Universe name
  name (string): Remote name (e.g., 'origin')
  url (string): Remote URL (e.g., 'git@github.com:user/repo.git')

Returns: { remote: string, url: string }

Examples:
  - "Add origin remote for Star Wars" → name="origin", url="git@github.com:studio/starwars.git"
  - "Add a backup remote" → name="backup", url="https://github.com/backup/starwars.git"`,
        inputSchema: RemoteAddInputSchema,
        annotations: {
            readOnlyHint: false,
            destructiveHint: false,
            idempotentHint: false,
            openWorldHint: true,
        },
    }, async (params) => {
        try {
            const result = await addRemote(params.universe, params.name, params.url);
            return {
                content: [{ type: "text", text: JSON.stringify(result, null, 2) }],
            };
        }
        catch (err) {
            return handleManagementError(err);
        }
    });
    // ── nap_remote_remove ──────────────────────────────────────────────────
    const RemoteRemoveInputSchema = z
        .object({
        universe: z
            .string()
            .min(1)
            .describe("Universe name (e.g., 'starwars')."),
        name: z
            .string()
            .min(1)
            .describe("Remote name to remove (e.g., 'origin', 'backup')."),
    })
        .strict();
    server.registerTool("nap_remote_remove", {
        title: "Remove Remote",
        description: `Remove a Git remote from a universe repository.

Args:
  universe (string): Universe name
  name (string): Remote name to remove

Returns: { removed: string }

Examples:
  - "Remove the backup remote" → universe="starwars", name="backup"`,
        inputSchema: RemoteRemoveInputSchema,
        annotations: {
            readOnlyHint: false,
            destructiveHint: false,
            idempotentHint: true,
            openWorldHint: true,
        },
    }, async (params) => {
        try {
            const result = await removeRemote(params.universe, params.name);
            return {
                content: [{ type: "text", text: JSON.stringify(result, null, 2) }],
            };
        }
        catch (err) {
            return handleManagementError(err);
        }
    });
    // ── nap_push ───────────────────────────────────────────────────────────
    const PushInputSchema = z
        .object({
        universe: z
            .string()
            .min(1)
            .describe("Universe name (e.g., 'starwars')."),
        remote: z
            .string()
            .optional()
            .describe("Remote name (default: tracking branch's remote, or 'origin')."),
        branch: z
            .string()
            .optional()
            .describe("Branch to push (default: current branch)."),
    })
        .strict();
    server.registerTool("nap_push", {
        title: "Push Universe",
        description: `Push the current branch to a remote repository.

Synchronizes local commits to the remote. Use nap_remote_add first to
configure a remote, or nap_remote_list to see existing remotes.

Args:
  universe (string): Universe name
  remote (string, optional): Remote name (default: 'origin')
  branch (string, optional): Branch to push (default: current branch)

Returns: { universe: string }

Examples:
  - "Push Star Wars to origin" → universe="starwars"
  - "Push the canon branch to origin" → universe="starwars", branch="canon"
  - "Push to a specific remote" → universe="starwars", remote="backup"`,
        inputSchema: PushInputSchema,
        annotations: {
            readOnlyHint: false,
            destructiveHint: false,
            idempotentHint: false,
            openWorldHint: true,
        },
    }, async (params) => {
        try {
            const result = await pushUniverse(params.universe, params.remote, params.branch);
            return {
                content: [{ type: "text", text: JSON.stringify(result, null, 2) }],
            };
        }
        catch (err) {
            return handleManagementError(err);
        }
    });
    // ── nap_pull ───────────────────────────────────────────────────────────
    const PullInputSchema = z
        .object({
        universe: z
            .string()
            .min(1)
            .describe("Universe name (e.g., 'starwars')."),
        remote: z
            .string()
            .optional()
            .describe("Remote name (default: tracking branch's remote, or 'origin')."),
        branch: z
            .string()
            .optional()
            .describe("Branch to pull (default: current branch)."),
    })
        .strict();
    server.registerTool("nap_pull", {
        title: "Pull Universe",
        description: `Pull the latest changes from a remote repository.

Fetches and merges remote changes into the current branch. Use this to
synchronize with other agents or team members.

Args:
  universe (string): Universe name
  remote (string, optional): Remote name (default: 'origin')
  branch (string, optional): Branch to pull (default: current branch)

Returns: { universe: string }

Examples:
  - "Pull latest Star Wars from origin" → universe="starwars"
  - "Pull canon branch" → universe="starwars", branch="canon"
  - "Pull from specific remote" → universe="starwars", remote="upstream"`,
        inputSchema: PullInputSchema,
        annotations: {
            readOnlyHint: false,
            destructiveHint: false,
            idempotentHint: false,
            openWorldHint: true,
        },
    }, async (params) => {
        try {
            const result = await pullUniverse(params.universe, params.remote, params.branch);
            return {
                content: [{ type: "text", text: JSON.stringify(result, null, 2) }],
            };
        }
        catch (err) {
            return handleManagementError(err);
        }
    });
    // ── nap_content_hash ───────────────────────────────────────────────────
    const ContentHashInputSchema = z
        .object({
        data: z
            .string()
            .min(1)
            .describe("Base64-encoded data to compute the SHA-256 hash of. " +
            "Encode file contents as base64 before calling."),
    })
        .strict();
    server.registerTool("nap_content_hash", {
        title: "Compute Content Hash",
        description: `Compute the SHA-256 content hash of base64-encoded data.

Returns the hash in 'sha256:<hex>' format. Use this to verify content integrity
or to generate hashes for representation references.

Args:
  data (string): Base64-encoded data to hash

Returns: { hash: string, algorithm: string }

Examples:
  - "Hash some content" → data="SGVsbG8gV29ybGQ="
  - "Compute hash of encoded image data" → data="<base64-encoded-data>"`,
        inputSchema: ContentHashInputSchema,
        annotations: {
            readOnlyHint: true,
            destructiveHint: false,
            idempotentHint: true,
            openWorldHint: false,
        },
    }, async (params) => {
        try {
            const result = await computeContentHash(params.data);
            return {
                content: [{ type: "text", text: JSON.stringify(result, null, 2) }],
            };
        }
        catch (err) {
            return handleManagementError(err);
        }
    });
    // ── nap_validate ───────────────────────────────────────────────────────
    const ValidateInputSchema = z
        .object({
        universe: z
            .string()
            .min(1)
            .describe("Universe name (e.g., 'starwars')."),
        entity_type: z
            .enum(ENTITY_TYPES)
            .describe("Entity type to validate."),
        entity_id: z
            .string()
            .min(1)
            .describe("Entity ID to validate."),
    })
        .strict();
    server.registerTool("nap_validate", {
        title: "Validate Manifest",
        description: `Validate an entity manifest against the NAP JSON Schema.

Checks that the manifest has all required fields and that values match
their expected types. Returns a list of validation errors, if any.

Args:
  universe (string): Universe name
  entity_type (string): Entity type
  entity_id (string): Entity ID to validate

Returns: { valid: boolean, errors: string[] }

Examples:
  - "Validate Luke Skywalker's manifest" → universe="starwars", entity_type="character", entity_id="lukeskywalker"
  - "Check Tatooine for schema violations" → universe="starwars", entity_type="location", entity_id="tatooine"`,
        inputSchema: ValidateInputSchema,
        annotations: {
            readOnlyHint: true,
            destructiveHint: false,
            idempotentHint: true,
            openWorldHint: false,
        },
    }, async (params) => {
        try {
            const result = await validateManifest(params.universe, params.entity_type, params.entity_id);
            return {
                content: [{ type: "text", text: JSON.stringify(result, null, 2) }],
            };
        }
        catch (err) {
            return handleManagementError(err);
        }
    });
}
// ── Shared error formatting ──────────────────────────────────────────────
function handleManagementError(err) {
    if (err instanceof NapApiError) {
        if (err.status === 404) {
            return {
                isError: true,
                content: [
                    {
                        type: "text",
                        text: `Error: Universe not found. ` +
                            `💡 Use nap_list_universes to see available universes.`,
                    },
                ],
            };
        }
        if (err.status === 409) {
            return {
                isError: true,
                content: [
                    {
                        type: "text",
                        text: `Error: Already exists. A universe with this name may already exist.` +
                            `\n  💡 Use a different universe name.`,
                    },
                ],
            };
        }
        if (err.status === 502) {
            return {
                isError: true,
                content: [
                    {
                        type: "text",
                        text: `Error: Cannot connect to NAP server at ${CONFIG.napServerUrl}. ` +
                            `💡 Make sure nap-server is running: \`cargo run -p nap-server\``,
                    },
                ],
            };
        }
        return {
            isError: true,
            content: [{ type: "text", text: `Error: ${err.message}` }],
        };
    }
    return {
        isError: true,
        content: [
            {
                type: "text",
                text: `Error: Unexpected error — ${err instanceof Error ? err.message : String(err)}. Please try again.`,
            },
        ],
    };
}
//# sourceMappingURL=management.js.map