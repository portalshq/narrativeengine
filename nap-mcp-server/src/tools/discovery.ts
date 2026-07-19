/**
 * Discovery tools — list repositories, entities, and search across them.
 *
 * These are READ-ONLY tools. They never mutate state.
 */

import { z } from "zod";
import type { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import {
  listRepositories,
  listEntities,
  searchEntities,
  healthCheck,
  NapApiError,
} from "../services/api.js";
import { CONFIG, ENTITY_TYPES } from "../constants.js";

export function registerDiscoveryTools(server: McpServer): void {
  // ── nap_list_repositories ──────────────────────────────────────────────
  server.registerTool(
    "nap_list_repositories",
    {
      title: "List Repositories",
      description: `List all fictional repositories available in the NAP resolver.

Returns an array of repository names (e.g., ['starwars', 'toystory', 'middleearth']).
Each repository is an independent Git repository containing characters, locations,
scenes, and props.

Args: None

Returns: { repositories: string[] }

Examples:
  - "What repositories do we have?" → call with no args
  - "Show me all available worlds" → call with no args`,
      inputSchema: z.object({}).strict(),
      annotations: {
        readOnlyHint: true,
        destructiveHint: false,
        idempotentHint: true,
        openWorldHint: true,
      },
    },
    async () => {
      try {
        const repositories = await listRepositories();
        if (repositories.length === 0) {
          return {
            content: [
              {
                type: "text",
                text:
                  "No repositories found. Make sure nap-server is pointing at a directory " +
                  "that contains repository repositories (directories with .nap/ subdirectories).",
              },
            ],
          };
        }
        const output = { repositories };
        return {
          content: [{ type: "text", text: JSON.stringify(output, null, 2) }],
        };
      } catch (err) {
        return handleDiscoveryError(err);
      }
    },
  );

  // ── nap_list_entities ───────────────────────────────────────────────
  const ListEntitiesInputSchema = z
    .object({
      repository: z
        .string()
        .min(1)
        .describe("Repository name (e.g., 'starwars', 'toystory')."),
      entity_type: z
        .enum(ENTITY_TYPES as unknown as [string, ...string[]])
        .optional()
        .describe(
          "Optional filter: 'character', 'location', 'scene', 'prop', 'world'. " +
            "If omitted, all entity types are returned.",
        ),
    })
    .strict();

  type ListEntitiesInput = z.infer<typeof ListEntitiesInputSchema>;

  server.registerTool(
    "nap_list_entities",
    {
      title: "List Entities",
      description: `List entities within a repository, optionally filtered by type.

Returns a map of entity type → array of NAP URIs.  Use this to discover what
exists in a repository before resolving specific manifests.

Args:
  repository (string): Repository name (e.g., 'starwars')
  entity_type (string, optional): Filter by type ('character', 'location', 'scene', 'prop', 'world')

Returns: { character: string[], location: string[], scene: string[], prop: string[] }

Examples:
  - "List everything in Star Wars" → repository="starwars"
  - "List all characters in Star Wars" → repository="starwars", entity_type="character"
  - "What locations exist in Toy Story?" → repository="toystory", entity_type="location"`,
      inputSchema: ListEntitiesInputSchema,
      annotations: {
        readOnlyHint: true,
        destructiveHint: false,
        idempotentHint: true,
        openWorldHint: true,
      },
    },
    async (params: ListEntitiesInput) => {
      try {
        const result = await listEntities(params.repository, params.entity_type);

        // Check if empty
        const allEmpty = Object.values(result).every(
          (arr) => arr.length === 0,
        );
        if (allEmpty) {
          return {
            content: [
              {
                type: "text",
                text:
                  `No entities found in repository '${params.repository}'. ` +
                  (params.entity_type
                    ? `No ${params.entity_type} entities exist yet.`
                    : "The repository exists but has no entities yet."),
              },
            ],
          };
        }

        // Add summary counts
        const output: Record<string, unknown> = { repository: params.repository };
        for (const [type, uris] of Object.entries(result)) {
          if (uris.length > 0) {
            output[type] = {
              count: uris.length,
              uris,
            };
          }
        }

        return {
          content: [{ type: "text", text: JSON.stringify(output, null, 2) }],
        };
      } catch (err) {
        return handleDiscoveryError(err);
      }
    },
  );

  // ── nap_search_entities ─────────────────────────────────────────────
  const SearchEntitiesInputSchema = z
    .object({
      repository: z
        .string()
        .min(1)
        .describe("Repository name (e.g., 'starwars')."),
      query: z
        .string()
        .min(1)
        .describe(
          "Search term. Matches against entity names and IDs (case-insensitive substring match). " +
            "Examples: 'luke', 'tatooine', 'vader'.",
        ),
      entity_type: z
        .enum(ENTITY_TYPES as unknown as [string, ...string[]])
        .optional()
        .describe("Optional filter: 'character', 'location', 'scene', 'prop'."),
    })
    .strict();

  type SearchEntitiesInput = z.infer<typeof SearchEntitiesInputSchema>;

  server.registerTool(
    "nap_search_entities",
    {
      title: "Search Entities",
      description: `Search for entities in a repository by name or ID.

Performs a case-insensitive substring match against entity names and IDs.
Useful when you don't know the exact entity ID but know part of the name.

Args:
  repository (string): Repository name (e.g., 'starwars')
  query (string): Search term (e.g., 'luke', 'tatooine', 'vader')
  entity_type (string, optional): Filter by type

Returns: Array of matching entities with uri, name, and entity_type.

Examples:
  - "Find characters named Luke" → repository="starwars", query="luke", entity_type="character"
  - "Find anything related to Tatooine" → repository="starwars", query="tatooine"
  - "Find Vader across all types" → repository="starwars", query="vader"`,
      inputSchema: SearchEntitiesInputSchema,
      annotations: {
        readOnlyHint: true,
        destructiveHint: false,
        idempotentHint: true,
        openWorldHint: true,
      },
    },
    async (params: SearchEntitiesInput) => {
      try {
        const results = await searchEntities(
          params.repository,
          params.query,
          params.entity_type,
        );

        if (results.length === 0) {
          return {
            content: [
              {
                type: "text",
                text:
                  `No entities matching '${params.query}' found in '${params.repository}'. ` +
                  `Try a different search term, or use nap_list_entities to see all available entities.`,
              },
            ],
          };
        }

        const output = {
          repository: params.repository,
          query: params.query,
          count: results.length,
          results,
        };

        return {
          content: [{ type: "text", text: JSON.stringify(output, null, 2) }],
        };
      } catch (err) {
        return handleDiscoveryError(err);
      }
    },
  );

  // ── nap_health_check ────────────────────────────────────────────────
  server.registerTool(
    "nap_health_check",
    {
      title: "Health Check",
      description: `Check if the NAP resolver server is running and healthy.

Returns the server status, protocol name, and version number.  Use this to
verify connectivity before attempting other operations.

Args: None

Returns: { status: string, protocol: string, version: string }`,
      inputSchema: z.object({}).strict(),
      annotations: {
        readOnlyHint: true,
        destructiveHint: false,
        idempotentHint: true,
        openWorldHint: true,
      },
    },
    async () => {
      try {
        const result = await healthCheck();
        return {
          content: [{ type: "text", text: JSON.stringify(result, null, 2) }],
        };
      } catch (err) {
        return handleDiscoveryError(err);
      }
    },
  );
}

// ── Shared error formatting ──────────────────────────────────────────────

function handleDiscoveryError(err: unknown): {
  isError: boolean;
  content: Array<{ type: "text"; text: string }>;
} {
  if (err instanceof NapApiError) {
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
