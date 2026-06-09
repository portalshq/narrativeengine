/**
 * Discovery tools — list universes, entities, and search across them.
 *
 * These are READ-ONLY tools. They never mutate state.
 */

import { z } from "zod";
import type { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import {
  listUniverses,
  listEntities,
  searchEntities,
  healthCheck,
  NapApiError,
} from "../services/api.js";
import { CONFIG, ENTITY_TYPES } from "../constants.js";

export function registerDiscoveryTools(server: McpServer): void {
  // ── nap_list_universes ──────────────────────────────────────────────
  server.registerTool(
    "nap_list_universes",
    {
      title: "List Universes",
      description: `List all fictional universes available in the NAP resolver.

Returns an array of universe names (e.g., ['starwars', 'toystory', 'middleearth']).
Each universe is an independent Git repository containing characters, locations,
scenes, and props.

Args: None

Returns: { universes: string[] }

Examples:
  - "What universes do we have?" → call with no args
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
        const universes = await listUniverses();
        if (universes.length === 0) {
          return {
            content: [
              {
                type: "text",
                text:
                  "No universes found. Make sure nap-server is pointing at a directory " +
                  "that contains universe repositories (directories with .nap/ subdirectories).",
              },
            ],
          };
        }
        const output = { universes };
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
      universe: z
        .string()
        .min(1)
        .describe("Universe name (e.g., 'starwars', 'toystory')."),
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
      description: `List entities within a universe, optionally filtered by type.

Returns a map of entity type → array of NAP URIs.  Use this to discover what
exists in a universe before resolving specific manifests.

Args:
  universe (string): Universe name (e.g., 'starwars')
  entity_type (string, optional): Filter by type ('character', 'location', 'scene', 'prop', 'world')

Returns: { character: string[], location: string[], scene: string[], prop: string[] }

Examples:
  - "List everything in Star Wars" → universe="starwars"
  - "List all characters in Star Wars" → universe="starwars", entity_type="character"
  - "What locations exist in Toy Story?" → universe="toystory", entity_type="location"`,
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
        const result = await listEntities(params.universe, params.entity_type);

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
                  `No entities found in universe '${params.universe}'. ` +
                  (params.entity_type
                    ? `No ${params.entity_type} entities exist yet.`
                    : "The universe exists but has no entities yet."),
              },
            ],
          };
        }

        // Add summary counts
        const output: Record<string, unknown> = { universe: params.universe };
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
      universe: z
        .string()
        .min(1)
        .describe("Universe name (e.g., 'starwars')."),
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
      description: `Search for entities in a universe by name or ID.

Performs a case-insensitive substring match against entity names and IDs.
Useful when you don't know the exact entity ID but know part of the name.

Args:
  universe (string): Universe name (e.g., 'starwars')
  query (string): Search term (e.g., 'luke', 'tatooine', 'vader')
  entity_type (string, optional): Filter by type

Returns: Array of matching entities with uri, name, and entity_type.

Examples:
  - "Find characters named Luke" → universe="starwars", query="luke", entity_type="character"
  - "Find anything related to Tatooine" → universe="starwars", query="tatooine"
  - "Find Vader across all types" → universe="starwars", query="vader"`,
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
          params.universe,
          params.query,
          params.entity_type,
        );

        if (results.length === 0) {
          return {
            content: [
              {
                type: "text",
                text:
                  `No entities matching '${params.query}' found in '${params.universe}'. ` +
                  `Try a different search term, or use nap_list_entities to see all available entities.`,
              },
            ],
          };
        }

        const output = {
          universe: params.universe,
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
