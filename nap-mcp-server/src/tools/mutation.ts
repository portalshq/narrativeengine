/**
 * Mutation tools — create entities, set properties, and commit changes.
 *
 * These tools MODIFY state.  All have destructiveHint: false because they
 * are versioned (every change is a Git commit) and can be rolled back,
 * but they DO change the universe state.
 */

import { z } from "zod";
import type { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import {
  parseNapUri,
  resolveManifest,
  commitChanges,
  revertCommit,
  NapApiError,
} from "../services/api.js";
import { ENTITY_TYPES } from "../constants.js";

// ── Tool: nap_set_property ────────────────────────────────────────────────

const SetPropertyInputSchema = z
  .object({
    uri: z
      .string()
      .describe(
        "NAP URI of the entity to modify. " +
          "e.g., 'nap://starwars/character/lukeskywalker'",
      ),
    key: z
      .string()
      .min(1)
      .describe(
        "Property key (dot-notation for nested paths). " +
          "Examples: 'species', 'homeworld', 'affiliation', " +
          "'provenance.model', 'references.appears_in'.",
      ),
    value: z
      .string()
      .min(1)
      .describe(
        "Property value as a string.  For structured data (arrays, objects), " +
          "use JSON syntax.  For cross-references, use a nap:// URI.\n" +
          "Examples:\n" +
          "  - Simple string: 'human'\n" +
          "  - Cross-reference: 'nap://starwars/location/tatooine'\n" +
          "  - Array: '[\"nap://starwars/scene/cantina\", \"nap://starwars/scene/trench-run\"]'\n" +
          "  - Object: '{\"target\": \"nap://starwars/character/darthvader\", \"type\": \"father\"}'",
      ),
    message: z
      .string()
      .default("set property")
      .describe("Commit message describing the change."),
    author: z
      .string()
      .default("nap-agent")
      .describe("Author identifier (e.g., 'agent@studio.com')."),
  })
  .strict();

type SetPropertyInput = z.infer<typeof SetPropertyInputSchema>;

export function registerMutationTools(server: McpServer): void {
  server.registerTool(
    "nap_set_property",
    {
      title: "Set Property",
      description: `Set a property on an entity manifest and commit the change.

Properties are the primary way to attach structured data to entities. Values
can be strings, numbers, booleans, arrays, or nested objects.  Use nap:// URIs
as values to create cross-references between entities.

Every change is versioned via Git — you can view history with nap_get_history
and resolve previous versions using the commit or branch selectors.

Args:
  uri (string): NAP URI of the entity (e.g., 'nap://starwars/character/lukeskywalker')
  key (string): Property key in dot-notation (e.g., 'species', 'homeworld', 'affiliation')
  value (string): Property value. Use JSON for arrays/objects, nap:// URIs for cross-references.
  message (string, optional): Commit message (default: 'set property')
  author (string, optional): Author identifier (default: 'nap-agent')

Returns: { commit_id: string, version: number }

Examples:
  - "Set Luke's species to human" → key="species", value="human"
  - "Set Luke's homeworld to Tatooine" → key="homeworld", value="nap://starwars/location/tatooine"
  - "Set what scenes Luke appears in" → key="references.appears_in", value='["nap://starwars/scene/cantina"]'
  - "Record the AI model used" → key="provenance.model", value="midjourney-v6"

Cross-reference examples:
  - Character → homeworld: use a nap://location URI
  - Scene → participants: use an array of nap://character URIs
  - Prop → owner: use a nap://character URI`,
      inputSchema: SetPropertyInputSchema,
      annotations: {
        readOnlyHint: false,
        destructiveHint: false, // Versioned — can be rolled back
        idempotentHint: false, // Each call creates a new commit
        openWorldHint: true,
      },
    },
    async (params: SetPropertyInput) => {
      try {
        const decoded = parseNapUri(params.uri);

        // Verify the entity exists before trying to modify it
        try {
          await resolveManifest(decoded);
        } catch {
          return {
            isError: true,
            content: [
              {
                type: "text",
                text:
                  `Error: Entity '${params.uri}' not found. ` +
                  `Use nap_list_entities to discover existing entities, ` +
                  `or check that the URI is correct.\n` +
                  `💡 NAP URIs look like: nap://starwars/character/lukeskywalker`,
              },
            ],
          };
        }

        // Parse value: try as JSON for structured values, fallback to string
        let parsedValue: unknown;
        try {
          parsedValue = JSON.parse(params.value);
        } catch {
          parsedValue = params.value;
        }

        const result = await commitChanges(decoded, {
          message: params.message,
          author: params.author,
          properties: { [params.key]: parsedValue },
        });

        const output = {
          uri: params.uri,
          property: params.key,
          value: parsedValue,
          commit_id: result.commit_id,
          version: result.version,
          message: params.message,
        };

        return {
          content: [{ type: "text", text: JSON.stringify(output, null, 2) }],
        };
      } catch (err) {
        return handleMutationError(err);
      }
    },
  );

  // ── nap_commit_manifest (multi-property update) ────────────────────
  const CommitManifestInputSchema = z
    .object({
      uri: z
        .string()
        .describe(
          "NAP URI of the entity to update. " +
            "e.g., 'nap://starwars/character/lukeskywalker'",
        ),
      properties: z
        .record(z.unknown())
        .describe(
          "Map of property keys to values. " +
            "Values can be strings, numbers, booleans, arrays, or objects. " +
            "Use nap:// URIs for cross-references.\n" +
            'Example: {"species": "human", "homeworld": "nap://starwars/location/tatooine"}',
        ),
      message: z.string().describe("Commit message describing the changes."),
      author: z
        .string()
        .default("nap-agent")
        .describe("Author identifier."),
    })
    .strict();

  type CommitManifestInput = z.infer<typeof CommitManifestInputSchema>;

  server.registerTool(
    "nap_commit_manifest",
    {
      title: "Commit Manifest Changes",
      description: `Update multiple properties on an entity in a single atomic commit.

Unlike nap_set_property which sets one field at a time, this tool lets you
update several properties at once — all recorded as a single commit with
one version bump.

Args:
  uri (string): NAP URI of the entity
  properties (object): Map of property keys to values
  message (string): Commit message describing the changes
  author (string, optional): Author identifier (default: 'nap-agent')

Returns: { commit_id: string, version: number, updated_fields: string[] }

Examples:
  - Create a character in one go:
    properties = {
      "species": "human",
      "homeworld": "nap://starwars/location/tatooine",
      "affiliation": "rebel_alliance"
    }
  - Update scene details:
    properties = {
      "mood": "tense",
      "time_of_day": "sunset"
    }
  - Record full provenance:
    properties = {
      "provenance.model": "midjourney-v6",
      "provenance.seed": "8675309"
    }`,
      inputSchema: CommitManifestInputSchema,
      annotations: {
        readOnlyHint: false,
        destructiveHint: false,
        idempotentHint: false,
        openWorldHint: true,
      },
    },
    async (params: CommitManifestInput) => {
      try {
        const decoded = parseNapUri(params.uri);

        // Verify entity exists
        try {
          await resolveManifest(decoded);
        } catch {
          return {
            isError: true,
            content: [
              {
                type: "text",
                text:
                  `Error: Entity '${params.uri}' not found. ` +
                  `Use nap_list_entities to discover existing entities.`,
              },
            ],
          };
        }

        const result = await commitChanges(decoded, {
          message: params.message,
          author: params.author,
          properties: params.properties as Record<string, unknown>,
        });

        const output = {
          uri: params.uri,
          updated_fields: Object.keys(params.properties),
          commit_id: result.commit_id,
          version: result.version,
          message: params.message,
        };

        return {
          content: [{ type: "text", text: JSON.stringify(output, null, 2) }],
        };
      } catch (err) {
        return handleMutationError(err);
      }
    },
  );

  // ── Tool: nap_revert_commit ──────────────────────────────────────────

  const RevertInputSchema = z
    .object({
      universe: z
        .string()
        .min(1)
        .describe(
          "Universe name (e.g., 'starwars', 'toystory'). " +
            "Use nap_list_universes to discover available universes.",
        ),
      commit: z
        .string()
        .min(1)
        .describe(
          "Commit hash to revert.  Must be a full SHA-1 hash (40 chars) " +
            "resolvable in the universe's Git history. " +
            "Use nap_get_history to find commit hashes.",
        ),
      author: z
        .string()
        .default("nap-agent")
        .describe(
          "Author identifier for the revert commit " +
            "(e.g., 'agent@studio.com').",
        ),
    })
    .strict();

  type RevertInput = z.infer<typeof RevertInputSchema>;

  server.registerTool(
    "nap_revert_commit",
    {
      title: "Revert Commit",
      description: `Revert a specific commit across an entire universe.

This is a universe-level operation — it undoes all changes made by the
specified commit, restoring every file to its previous state.  A new
revert commit is created in the Git history, so the revert itself is
also versioned and can be reverted if needed.

Use this to undo mistakes: if an agent wrote bad data to an entity,
find the commit hash with nap_get_history and revert it.

Args:
  universe (string): Universe name (e.g., 'starwars')
  commit (string): Full commit hash to revert
  author (string, optional): Author identifier (default: 'nap-agent')

Returns: { reverted_commit: string, new_commit: string }

Examples:
  - Undo a bad agent commit:
    nap_revert_commit(universe="starwars", commit="a58a3c73a332ca62e21da4543dba87621ef18ee0")
  - Revert with specific author tracking:
    nap_revert_commit(universe="toystory", commit="abc123...", author="admin@studio.com")`,
      inputSchema: RevertInputSchema,
      annotations: {
        readOnlyHint: false,
        destructiveHint: true, // Reverts state — use with care
        idempotentHint: false, // Each call creates a new commit
        openWorldHint: true,
      },
    },
    async (params: RevertInput) => {
      try {
        const result = await revertCommit(params.universe, {
          commit: params.commit,
          author: params.author,
        });

        const output = {
          universe: params.universe,
          reverted_commit: result.reverted_commit,
          new_commit: result.new_commit,
          author: params.author,
          note: "The revert is itself versioned — use nap_get_history to verify, or revert the revert to restore.",
        };

        return {
          content: [{ type: "text", text: JSON.stringify(output, null, 2) }],
        };
      } catch (err) {
        return handleMutationError(err);
      }
    },
  );
}

// ── Shared error formatting ──────────────────────────────────────────────

function handleMutationError(err: unknown): {
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
              `Error: Resource not found. The URI may be incorrect, or the universe/entity doesn't exist.\n` +
              `💡 Use nap_list_universes to see available universes.\n` +
              `💡 Use nap_list_entities to see entities in a universe.`,
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
