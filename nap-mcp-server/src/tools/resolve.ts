/**
 * Resolution tools — resolve manifests, query subtrees, and inspect entities.
 *
 * These are READ-ONLY tools. They never mutate state.
 */

import { z } from "zod";
import type { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import {
  parseNapUri,
  resolveManifest,
  getHistory,
  NapApiError,
  NapNotFoundError,
} from "../services/api.js";
import { CONFIG, ENTITY_TYPES } from "../constants.js";

// ── Zod Schemas ───────────────────────────────────────────────────────────

const NapUrnField = z
  .string()
  .describe(
    "NAP URI, e.g. 'nap://starwars/character/lukeskywalker'. " +
      "Include a fragment for subtree queries: 'nap://starwars/character/lukeskywalker#properties.species'.",
  );

const ResolveSelectorSchema = z
  .object({
    branch: z
      .string()
      .optional()
      .describe("Branch name (e.g., 'canon', 'legends', 'what-if'). Omit for default branch."),
    commit: z
      .string()
      .optional()
      .describe("Specific commit hash to resolve at. Omit for HEAD."),
    tag: z
      .string()
      .optional()
      .describe("Tag name (e.g., 'episode-4'). Omit for HEAD."),
  })
  .strict();

// ── Tool: nap_resolve_manifest ────────────────────────────────────────────

const ResolveManifestInputSchema = z
  .object({
    uri: NapUrnField,
    branch: z.string().optional().describe("Branch name to resolve at (e.g., 'canon')."),
    commit: z.string().optional().describe("Specific commit hash."),
    tag: z.string().optional().describe("Tag name (e.g., 'episode-4')."),
    path: z
      .string()
      .optional()
      .describe(
        "Subtree query path. Overrides URI fragment. " +
          "Examples: 'properties.species', 'representations.reference_image.hash', " +
          "'references.appears_in', 'provenance.model'.",
      ),
  })
  .strict();

type ResolveManifestInput = z.infer<typeof ResolveManifestInputSchema>;

/** Maximum items in arrays before we summarize. */
const MAX_ARRAY_ITEMS = 20;

/**
 * Recursively truncate large arrays in a resolved value to avoid
 * blowing past the character limit.
 */
function truncateLargeArrays(value: unknown, depth = 0): unknown {
  if (depth > 10) return value;
  if (Array.isArray(value)) {
    if (value.length > MAX_ARRAY_ITEMS) {
      return [
        ...value.slice(0, MAX_ARRAY_ITEMS),
        `… (${value.length - MAX_ARRAY_ITEMS} more items)`,
      ];
    }
    return value.map((v) => truncateLargeArrays(v, depth + 1));
  }
  if (value && typeof value === "object" && !(value instanceof Date)) {
    const obj = value as Record<string, unknown>;
    const result: Record<string, unknown> = {};
    for (const [k, v] of Object.entries(obj)) {
      result[k] = truncateLargeArrays(v, depth + 1);
    }
    return result;
  }
  return value;
}

export function registerResolveTools(server: McpServer): void {
  // ── resolve_manifest ───────────────────────────────────────────────
  server.registerTool(
    "nap_resolve_manifest",
    {
      title: "Resolve NAP Manifest",
      description: `Resolve a NAP URI to its full manifest or a specific subtree.

Returns the complete manifest as JSON, including all properties, representations,
references, provenance, and version metadata.  Use the 'path' parameter or a URI
fragment (e.g. nap://starwars/character/luke#properties.species) to fetch only
the data you need — this is MUCH more token-efficient.

Args:
  uri (string): NAP URI (e.g., 'nap://starwars/character/lukeskywalker')
  branch (string, optional): Branch name (e.g., 'canon', 'legends')
  commit (string, optional): Specific commit hash
  tag (string, optional): Tag name (e.g., 'episode-4')
  path (string, optional): Subtree path override (e.g., 'properties.species')

Returns: JSON object with the manifest or the queried subtree value.

Examples:
  - "Get Luke Skywalker's full manifest" → uri="nap://starwars/character/lukeskywalker"
  - "What species is Luke?" → uri="nap://starwars/character/lukeskywalker", path="properties.species"
  - "What scenes does Luke appear in?" → uri="nap://starwars/character/lukeskywalker", path="references.appears_in"
  - "Get the canon version of Tatooine" → uri="nap://starwars/location/tatooine", branch="canon"`,
      inputSchema: ResolveManifestInputSchema,
      annotations: {
        readOnlyHint: true,
        destructiveHint: false,
        idempotentHint: true,
        openWorldHint: true,
      },
    },
    async (params: ResolveManifestInput) => {
      try {
        const decoded = parseNapUri(params.uri);
        const manifest = await resolveManifest(decoded, {
          branch: params.branch,
          commit: params.commit,
          tag: params.tag,
          path: params.path,
        });

        const text = JSON.stringify(truncateLargeArrays(manifest), null, 2);
        const truncated =
          text.length > CONFIG.characterLimit
            ? text.slice(0, CONFIG.characterLimit) +
              `\n… (truncated, ${text.length - CONFIG.characterLimit} more chars)`
            : text;

        return {
          content: [{ type: "text", text: truncated }],
        };
      } catch (err) {
        return formatError(err);
      }
    },
  );

  // ── query_property ────────────────────────────────────────────────
  const QueryPropertyInputSchema = z
    .object({
      uri: NapUrnField,
      path: z
        .string()
        .describe(
          "Dot-notation path to the property. " +
            "Examples: 'properties.species', 'representations.reference_image.hash', " +
            "'references.appears_in.0', 'provenance.model'.",
        ),
      branch: z.string().optional().describe("Branch name."),
      commit: z.string().optional().describe("Specific commit hash."),
      tag: z.string().optional().describe("Tag name."),
    })
    .strict();

  type QueryPropertyInput = z.infer<typeof QueryPropertyInputSchema>;

  server.registerTool(
    "nap_query_property",
    {
      title: "Query Property",
      description: `Extract a specific property or subtree from a manifest.

This is the most token-efficient way to read NAP data. Instead of fetching the
entire manifest (potentially thousands of tokens), you fetch only the field you
need (typically <100 tokens).

Args:
  uri (string): NAP URI (e.g., 'nap://starwars/character/lukeskywalker')
  path (string): Dot-notation path (e.g., 'properties.species', 'references.appears_in')
  branch (string, optional): Branch name
  commit (string, optional): Specific commit hash
  tag (string, optional): Tag name

Returns: The value at the given path (string, number, array, or object).

Examples:
  - "What species is Luke?" → path="properties.species"
  - "Where is Luke from?" → path="properties.homeworld"
  - "What scenes does he appear in?" → path="references.appears_in"
  - "What AI model generated this?" → path="provenance.model"
  - "What's the hash of the reference image?" → path="representations.reference_image.hash"`,
      inputSchema: QueryPropertyInputSchema,
      annotations: {
        readOnlyHint: true,
        destructiveHint: false,
        idempotentHint: true,
        openWorldHint: true,
      },
    },
    async (params: QueryPropertyInput) => {
      try {
        const decoded = parseNapUri(params.uri);
        const value = await resolveManifest(decoded, {
          branch: params.branch,
          commit: params.commit,
          tag: params.tag,
          path: params.path,
        });

        const text = JSON.stringify(value, null, 2);
        return {
          content: [{ type: "text", text }],
        };
      } catch (err) {
        return formatError(err);
      }
    },
  );

  // ── get_history ───────────────────────────────────────────────────
  const GetHistoryInputSchema = z
    .object({
      uri: NapUrnField,
      limit: z
        .number()
        .int()
        .min(1)
        .max(100)
        .default(20)
        .describe("Maximum number of commits to return (1-100, default 20)."),
    })
    .strict();

  type GetHistoryInput = z.infer<typeof GetHistoryInputSchema>;

  server.registerTool(
    "nap_get_history",
    {
      title: "Get Entity History",
      description: `View the commit history for a narrative entity.

Returns a chronological list of commits (most recent first) with commit hash,
author, message, and timestamp.  Useful for auditing changes, understanding
how a character evolved, and finding specific versions to resolve.

Args:
  uri (string): NAP URI (e.g., 'nap://starwars/character/lukeskywalker')
  limit (number, optional): Maximum commits to return (default 20)

Returns: Array of commit entries with id, author, message, timestamp.

Examples:
  - "Show me the last 10 changes to Luke Skywalker" → uri="nap://starwars/character/lukeskywalker", limit=10
  - "Who changed Tatooine's climate?" → uri="nap://starwars/location/tatooine"`,
      inputSchema: GetHistoryInputSchema,
      annotations: {
        readOnlyHint: true,
        destructiveHint: false,
        idempotentHint: true,
        openWorldHint: true,
      },
    },
    async (params: GetHistoryInput) => {
      try {
        const decoded = parseNapUri(params.uri);
        const history = await getHistory(decoded, params.limit);

        if (history.length === 0) {
          return {
            content: [
              {
                type: "text",
                text: `No commit history found for '${params.uri}'.`,
              },
            ],
          };
        }

        const output = {
          uri: params.uri,
          count: history.length,
          commits: history,
        };

        return {
          content: [{ type: "text", text: JSON.stringify(output, null, 2) }],
        };
      } catch (err) {
        return formatError(err);
      }
    },
  );
}

// ── Shared error formatting ──────────────────────────────────────────────

function formatError(err: unknown): {
  isError: boolean;
  content: Array<{ type: "text"; text: string }>;
} {
  if (err instanceof NapNotFoundError) {
    return {
      isError: true,
      content: [
        {
          type: "text",
          text: `Error: Resource not found. The URI may be incorrect, or the entity doesn't exist yet.` +
            `\n  Suggestion: Use nap_list_entities to see available entities.` +
            `\n  Suggestion: Use nap_list_repositories to see available repositories.`,
        },
      ],
    };
  }
  if (err instanceof NapApiError) {
    const suggestions: string[] = [];
    if (err.status === 400) {
      suggestions.push(
        "NAP URIs look like: nap://starwars/character/lukeskywalker",
      );
      suggestions.push("Valid entity types: character, location, scene, prop, world");
    }
    if (err.status === 502 || err.status === 408) {
      suggestions.push(
        "Make sure nap-server is running: `cargo run -p nap-server`",
      );
    }

    const suggestionText =
      suggestions.length > 0
        ? `\n${suggestions.map((s) => `  💡 ${s}`).join("\n")}`
        : "";

    return {
      isError: true,
      content: [
        {
          type: "text",
          text: `Error: ${err.message}${suggestionText}`,
        },
      ],
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
