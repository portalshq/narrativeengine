/**
 * Management tools — repository init, entity CRUD, VCS operations, remotes, etc.
 *
 * These tools MODIFY state. All have destructiveHint: false because they
 * are versioned (every change is a Git commit) and can be rolled back.
 */

import { z } from "zod";
import type { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import {
  initUniverse,
  createEntity,
  deleteEntity,
  listBranches,
  createBranch,
  switchBranch,
  listTags,
  createTag,
  listRemotes,
  addRemote,
  removeRemote,
  pushUniverse,
  pullUniverse,
  computeContentHash,
  validateManifest,
  NapApiError,
} from "../services/api.js";
import { CONFIG, ENTITY_TYPES } from "../constants.js";

export function registerManagementTools(server: McpServer): void {
  // ── nap_init ────────────────────────────────────────────────────────
  server.registerTool(
    "nap_init",
    {
      title: "Initialize Repository",
      description: `Initialize a new NAP repository repository.

Creates a new Git-backed repository with the standard directory structure
(characters/, locations/, scenes/, props/) and an initial world manifest.

Args:
  repository (string): Repository name (e.g., 'starwars', 'toystory').

Returns: { success: boolean, repository: string, path: string }

Examples:
  - "Create a Star Wars repository" → repository="starwars"
  - "Start a new Toy Story repository" → repository="toystory"`,
      inputSchema: z
        .object({
          repository: z
            .string()
            .min(1)
            .describe(
              "Repository name (e.g., 'starwars', 'toystory', 'middleearth'). Use lowercase and dashes/underscores.",
            ),
        })
        .strict(),
      annotations: {
        readOnlyHint: false,
        destructiveHint: false,
        idempotentHint: true,
        openWorldHint: true,
      },
    },
    async (params: { repository: string }) => {
      try {
        const result = await initUniverse(params.repository);
        return {
          content: [{ type: "text", text: JSON.stringify(result, null, 2) }],
        };
      } catch (err) {
        return handleManagementError(err);
      }
    },
  );

  // ── nap_create_entity ──────────────────────────────────────────────────
  const CreateEntityInputSchema = z
    .object({
      repository: z
        .string()
        .min(1)
        .describe("Repository name (e.g., 'starwars')."),
      entity_type: z
        .enum(ENTITY_TYPES as unknown as [string, ...string[]])
        .describe("Entity type: 'character', 'location', 'scene', 'prop', 'world'."),
      entity_id: z
        .string()
        .min(1)
        .describe(
          "Entity ID (slug). Lowercase, no spaces. e.g., 'lukeskywalker', 'tatooine'.",
        ),
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

  type CreateEntityInput = z.infer<typeof CreateEntityInputSchema>;

  server.registerTool(
    "nap_create_entity",
    {
      title: "Create Entity",
      description: `Create a new narrative entity in a repository.

This creates a new manifest file, commits it to the repository's Git history,
and returns the NAP URI and commit ID. The entity starts with no properties
— use nap_set_property to add data.

Args:
  repository (string): Repository name (e.g., 'starwars')
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
    },
    async (params: CreateEntityInput) => {
      try {
        const result = await createEntity(
          params.repository,
          params.entity_type,
          params.entity_id,
          { name: params.name, author: params.author },
        );
        return {
          content: [{ type: "text", text: JSON.stringify(result, null, 2) }],
        };
      } catch (err) {
        return handleManagementError(err);
      }
    },
  );

  // ── nap_delete_entity ──────────────────────────────────────────────────
  const DeleteEntityInputSchema = z
    .object({
      repository: z
        .string()
        .min(1)
        .describe("Repository name (e.g., 'starwars')."),
      entity_type: z
        .enum(ENTITY_TYPES as unknown as [string, ...string[]])
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

  type DeleteEntityInput = z.infer<typeof DeleteEntityInputSchema>;

  server.registerTool(
    "nap_delete_entity",
    {
      title: "Delete Entity",
      description: `Delete a narrative entity from a repository.

Permanently removes the entity manifest and commits the deletion to history.
The deletion can be reverted using nap_revert_commit with the resulting commit hash.

Args:
  repository (string): Repository name
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
    },
    async (params: DeleteEntityInput) => {
      try {
        const result = await deleteEntity(
          params.repository,
          params.entity_type,
          params.entity_id,
          params.author,
        );
        return {
          content: [{ type: "text", text: JSON.stringify(result, null, 2) }],
        };
      } catch (err) {
        return handleManagementError(err);
      }
    },
  );

  // ── nap_list_branches ──────────────────────────────────────────────────
  const ListBranchesInputSchema = z
    .object({
      repository: z
        .string()
        .min(1)
        .describe("Repository name (e.g., 'starwars')."),
    })
    .strict();

  server.registerTool(
    "nap_list_branches",
    {
      title: "List Branches",
      description: `List all Git branches in a repository repository.

Returns the list of branch names including the current active branch.

Args:
  repository (string): Repository name

Returns: { branches: string[] }

Examples:
  - "What branches exist in Star Wars?" → repository="starwars"
  - "Show me all branches in Toy Story" → repository="toystory"`,
      inputSchema: ListBranchesInputSchema,
      annotations: {
        readOnlyHint: true,
        destructiveHint: false,
        idempotentHint: true,
        openWorldHint: false,
      },
    },
    async (params: { repository: string }) => {
      try {
        const result = await listBranches(params.repository);
        return {
          content: [{ type: "text", text: JSON.stringify(result, null, 2) }],
        };
      } catch (err) {
        return handleManagementError(err);
      }
    },
  );

  // ── nap_create_branch ──────────────────────────────────────────────────
  const CreateBranchInputSchema = z
    .object({
      repository: z
        .string()
        .min(1)
        .describe("Repository name (e.g., 'starwars')."),
      name: z
        .string()
        .min(1)
        .describe(
          "Branch name (e.g., 'canon', 'legends', 'what-if', 'my-experimental').",
        ),
    })
    .strict();

  server.registerTool(
    "nap_create_branch",
    {
      title: "Create Branch",
      description: `Create a new Git branch in a repository repository.

Branches let you maintain parallel versions of a repository. Each branch has
its own independent commit history. Switch between them with nap_switch_branch.

Args:
  repository (string): Repository name
  name (string): Branch name (e.g., 'canon', 'legends', 'what-if')

Returns: { branch: string }

Examples:
  - "Create a canon branch" → repository="starwars", name="canon"
  - "Create a legends/what-if branch" → repository="starwars", name="legends"
  - "Create an experimental Toy Story branch" → repository="toystory", name="experimental"`,
      inputSchema: CreateBranchInputSchema,
      annotations: {
        readOnlyHint: false,
        destructiveHint: false,
        idempotentHint: false,
        openWorldHint: true,
      },
    },
    async (params: { repository: string; name: string }) => {
      try {
        const result = await createBranch(params.repository, params.name);
        return {
          content: [{ type: "text", text: JSON.stringify(result, null, 2) }],
        };
      } catch (err) {
        return handleManagementError(err);
      }
    },
  );

  // ── nap_switch_branch ──────────────────────────────────────────────────
  const SwitchBranchInputSchema = z
    .object({
      repository: z
        .string()
        .min(1)
        .describe("Repository name (e.g., 'starwars')."),
      name: z
        .string()
        .min(1)
        .describe(
          "Branch name to switch to (e.g., 'canon', 'legends'). Use nap_list_branches to see available branches.",
        ),
    })
    .strict();

  server.registerTool(
    "nap_switch_branch",
    {
      title: "Switch Branch",
      description: `Switch the active Git branch in a repository repository.

After switching, all subsequent read and write operations use the new branch.
Use nap_list_branches to discover available branches.

Args:
  repository (string): Repository name
  name (string): Branch name to switch to

Returns: { branch: string }

Examples:
  - "Switch to the canon branch" → repository="starwars", name="canon"
  - "Switch to the legends branch" → repository="starwars", name="legends"
  - "Go back to main" → repository="toystory", name="main"`,
      inputSchema: SwitchBranchInputSchema,
      annotations: {
        readOnlyHint: false,
        destructiveHint: false,
        idempotentHint: false,
        openWorldHint: true,
      },
    },
    async (params: { repository: string; name: string }) => {
      try {
        const result = await switchBranch(params.repository, params.name);
        return {
          content: [{ type: "text", text: JSON.stringify(result, null, 2) }],
        };
      } catch (err) {
        return handleManagementError(err);
      }
    },
  );

  // ── nap_list_tags ──────────────────────────────────────────────────────
  const ListTagsInputSchema = z
    .object({
      repository: z
        .string()
        .min(1)
        .describe("Repository name (e.g., 'starwars')."),
    })
    .strict();

  server.registerTool(
    "nap_list_tags",
    {
      title: "List Tags",
      description: `List all Git tags in a repository repository.

Tags are read-only snapshots of a branch at a specific point in time.
They are useful for marking releases or important milestones.

Args:
  repository (string): Repository name

Returns: { tags: string[] }

Examples:
  - "What tags exist in Star Wars?" → repository="starwars"
  - "List all tags in Toy Story" → repository="toystory"`,
      inputSchema: ListTagsInputSchema,
      annotations: {
        readOnlyHint: true,
        destructiveHint: false,
        idempotentHint: true,
        openWorldHint: false,
      },
    },
    async (params: { repository: string }) => {
      try {
        const result = await listTags(params.repository);
        return {
          content: [{ type: "text", text: JSON.stringify(result, null, 2) }],
        };
      } catch (err) {
        return handleManagementError(err);
      }
    },
  );

  // ── nap_create_tag ─────────────────────────────────────────────────────
  const CreateTagInputSchema = z
    .object({
      repository: z
        .string()
        .min(1)
        .describe("Repository name (e.g., 'starwars')."),
      name: z
        .string()
        .min(1)
        .describe(
          "Tag name (e.g., 'v1.0', 'episode-4-release', 'pilot-episode').",
        ),
    })
    .strict();

  server.registerTool(
    "nap_create_tag",
    {
      title: "Create Tag",
      description: `Create a Git tag at the current HEAD in a repository repository.

Tags are immutable references to a specific commit. Use them to mark releases,
important milestones, or any version you want to reference later.

Args:
  repository (string): Repository name
  name (string): Tag name (e.g., 'v1.0', 'episode-4')

Returns: { tag: string }

Examples:
  - "Tag the current state as v1.0" → repository="starwars", name="v1.0"
  - "Mark this as the pilot episode" → repository="toystory", name="pilot"`,
      inputSchema: CreateTagInputSchema,
      annotations: {
        readOnlyHint: false,
        destructiveHint: false,
        idempotentHint: false,
        openWorldHint: true,
      },
    },
    async (params: { repository: string; name: string }) => {
      try {
        const result = await createTag(params.repository, params.name);
        return {
          content: [{ type: "text", text: JSON.stringify(result, null, 2) }],
        };
      } catch (err) {
        return handleManagementError(err);
      }
    },
  );

  // ── nap_remote_list ────────────────────────────────────────────────────
  const RemoteListInputSchema = z
    .object({
      repository: z
        .string()
        .min(1)
        .describe("Repository name (e.g., 'starwars')."),
    })
    .strict();

  server.registerTool(
    "nap_remote_list",
    {
      title: "List Remotes",
      description: `List all Git remotes configured for a repository repository.

Returns remote names and their URLs. Use this to discover where data is synced.

Args:
  repository (string): Repository name

Returns: { remotes: Array<{ name: string, url: string }> }

Examples:
  - "What remotes does Star Wars have?" → repository="starwars"
  - "Show remote configuration for Toy Story" → repository="toystory"`,
      inputSchema: RemoteListInputSchema,
      annotations: {
        readOnlyHint: true,
        destructiveHint: false,
        idempotentHint: true,
        openWorldHint: false,
      },
    },
    async (params: { repository: string }) => {
      try {
        const result = await listRemotes(params.repository);
        return {
          content: [{ type: "text", text: JSON.stringify(result, null, 2) }],
        };
      } catch (err) {
        return handleManagementError(err);
      }
    },
  );

  // ── nap_remote_add ─────────────────────────────────────────────────────
  const RemoteAddInputSchema = z
    .object({
      repository: z
        .string()
        .min(1)
        .describe("Repository name (e.g., 'starwars')."),
      name: z
        .string()
        .min(1)
        .describe(
          "Remote name (e.g., 'origin', 'backup', 'upstream').",
        ),
      url: z
        .string()
        .min(1)
        .describe(
          "Remote URL (e.g., 'git@github.com:user/starwars.git', 'https://github.com/user/repo.git').",
        ),
    })
    .strict();

  server.registerTool(
    "nap_remote_add",
    {
      title: "Add Remote",
      description: `Add a Git remote to a repository repository.

Remotes enable pushing and pulling repository data between repositories.
Use nap_push and nap_pull to synchronize after configuring a remote.

Args:
  repository (string): Repository name
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
    },
    async (params: { repository: string; name: string; url: string }) => {
      try {
        const result = await addRemote(
          params.repository,
          params.name,
          params.url,
        );
        return {
          content: [{ type: "text", text: JSON.stringify(result, null, 2) }],
        };
      } catch (err) {
        return handleManagementError(err);
      }
    },
  );

  // ── nap_remote_remove ──────────────────────────────────────────────────
  const RemoteRemoveInputSchema = z
    .object({
      repository: z
        .string()
        .min(1)
        .describe("Repository name (e.g., 'starwars')."),
      name: z
        .string()
        .min(1)
        .describe("Remote name to remove (e.g., 'origin', 'backup')."),
    })
    .strict();

  server.registerTool(
    "nap_remote_remove",
    {
      title: "Remove Remote",
      description: `Remove a Git remote from a repository repository.

Args:
  repository (string): Repository name
  name (string): Remote name to remove

Returns: { removed: string }

Examples:
  - "Remove the backup remote" → repository="starwars", name="backup"`,
      inputSchema: RemoteRemoveInputSchema,
      annotations: {
        readOnlyHint: false,
        destructiveHint: false,
        idempotentHint: true,
        openWorldHint: true,
      },
    },
    async (params: { repository: string; name: string }) => {
      try {
        const result = await removeRemote(params.repository, params.name);
        return {
          content: [{ type: "text", text: JSON.stringify(result, null, 2) }],
        };
      } catch (err) {
        return handleManagementError(err);
      }
    },
  );

  // ── nap_push ───────────────────────────────────────────────────────────
  const PushInputSchema = z
    .object({
      repository: z
        .string()
        .min(1)
        .describe("Repository name (e.g., 'starwars')."),
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

  server.registerTool(
    "nap_push",
    {
      title: "Push Repository",
      description: `Push the current branch to a remote repository.

Synchronizes local commits to the remote. Use nap_remote_add first to
configure a remote, or nap_remote_list to see existing remotes.

Args:
  repository (string): Repository name
  remote (string, optional): Remote name (default: 'origin')
  branch (string, optional): Branch to push (default: current branch)

Returns: { repository: string }

Examples:
  - "Push Star Wars to origin" → repository="starwars"
  - "Push the canon branch to origin" → repository="starwars", branch="canon"
  - "Push to a specific remote" → repository="starwars", remote="backup"`,
      inputSchema: PushInputSchema,
      annotations: {
        readOnlyHint: false,
        destructiveHint: false,
        idempotentHint: false,
        openWorldHint: true,
      },
    },
    async (params: { repository: string; remote?: string; branch?: string }) => {
      try {
        const result = await pushUniverse(
          params.repository,
          params.remote,
          params.branch,
        );
        return {
          content: [{ type: "text", text: JSON.stringify(result, null, 2) }],
        };
      } catch (err) {
        return handleManagementError(err);
      }
    },
  );

  // ── nap_pull ───────────────────────────────────────────────────────────
  const PullInputSchema = z
    .object({
      repository: z
        .string()
        .min(1)
        .describe("Repository name (e.g., 'starwars')."),
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

  server.registerTool(
    "nap_pull",
    {
      title: "Pull Repository",
      description: `Pull the latest changes from a remote repository.

Fetches and merges remote changes into the current branch. Use this to
synchronize with other agents or team members.

Args:
  repository (string): Repository name
  remote (string, optional): Remote name (default: 'origin')
  branch (string, optional): Branch to pull (default: current branch)

Returns: { repository: string }

Examples:
  - "Pull latest Star Wars from origin" → repository="starwars"
  - "Pull canon branch" → repository="starwars", branch="canon"
  - "Pull from specific remote" → repository="starwars", remote="upstream"`,
      inputSchema: PullInputSchema,
      annotations: {
        readOnlyHint: false,
        destructiveHint: false,
        idempotentHint: false,
        openWorldHint: true,
      },
    },
    async (params: { repository: string; remote?: string; branch?: string }) => {
      try {
        const result = await pullUniverse(
          params.repository,
          params.remote,
          params.branch,
        );
        return {
          content: [{ type: "text", text: JSON.stringify(result, null, 2) }],
        };
      } catch (err) {
        return handleManagementError(err);
      }
    },
  );

  // ── nap_content_hash ───────────────────────────────────────────────────
  const ContentHashInputSchema = z
    .object({
      data: z
        .string()
        .min(1)
        .describe(
          "Base64-encoded data to compute the SHA-256 hash of. " +
            "Encode file contents as base64 before calling.",
        ),
    })
    .strict();

  server.registerTool(
    "nap_content_hash",
    {
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
    },
    async (params: { data: string }) => {
      try {
        const result = await computeContentHash(params.data);
        return {
          content: [{ type: "text", text: JSON.stringify(result, null, 2) }],
        };
      } catch (err) {
        return handleManagementError(err);
      }
    },
  );

  // ── nap_validate ───────────────────────────────────────────────────────
  const ValidateInputSchema = z
    .object({
      repository: z
        .string()
        .min(1)
        .describe("Repository name (e.g., 'starwars')."),
      entity_type: z
        .enum(ENTITY_TYPES as unknown as [string, ...string[]])
        .describe("Entity type to validate."),
      entity_id: z
        .string()
        .min(1)
        .describe("Entity ID to validate."),
    })
    .strict();

  server.registerTool(
    "nap_validate",
    {
      title: "Validate Manifest",
      description: `Validate an entity manifest against the NAP JSON Schema.

Checks that the manifest has all required fields and that values match
their expected types. Returns a list of validation errors, if any.

Args:
  repository (string): Repository name
  entity_type (string): Entity type
  entity_id (string): Entity ID to validate

Returns: { valid: boolean, errors: string[] }

Examples:
  - "Validate Luke Skywalker's manifest" → repository="starwars", entity_type="character", entity_id="lukeskywalker"
  - "Check Tatooine for schema violations" → repository="starwars", entity_type="location", entity_id="tatooine"`,
      inputSchema: ValidateInputSchema,
      annotations: {
        readOnlyHint: true,
        destructiveHint: false,
        idempotentHint: true,
        openWorldHint: false,
      },
    },
    async (params: { repository: string; entity_type: string; entity_id: string }) => {
      try {
        const result = await validateManifest(
          params.repository,
          params.entity_type,
          params.entity_id,
        );
        return {
          content: [{ type: "text", text: JSON.stringify(result, null, 2) }],
        };
      } catch (err) {
        return handleManagementError(err);
      }
    },
  );
}

// ── Shared error formatting ──────────────────────────────────────────────

function handleManagementError(err: unknown): {
  isError: boolean;
  content: Array<{ type: "text"; text: string }>;
} {
  if (err instanceof NapApiError) {
    if (err.status === 404) {
      return {
        isError: true,
        content: [
          {
            type: "text",
            text:
              `Error: Repository not found. ` +
              `💡 Use nap_list_repositories to see available repositories.`,
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
            text:
              `Error: Already exists. A repository with this name may already exist.` +
              `\n  💡 Use a different repository name.`,
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
            text:
              `Error: Cannot connect to NAP server at ${CONFIG.napServerUrl}. ` +
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
