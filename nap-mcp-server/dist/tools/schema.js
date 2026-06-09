/**
 * Schema tools — access NAP JSON Schemas and data model documentation.
 *
 * These are READ-ONLY tools that help agents understand the NAP data model.
 */
import { z } from "zod";
import { getSchema, NapApiError } from "../services/api.js";
export function registerSchemaTools(server) {
    // ── nap_get_schema ──────────────────────────────────────────────────
    const GetSchemaInputSchema = z
        .object({
        name: z
            .enum(["manifest", "commit"])
            .describe("Schema name: 'manifest' (resource structure) or 'commit' (change history structure)."),
    })
        .strict();
    server.registerTool("nap_get_schema", {
        title: "Get NAP Schema",
        description: `Get the JSON Schema for a NAP data type.

Returns the complete JSON Schema document describing the structure of manifests
or commits.  Use this to understand the exact field names, types, and constraints
of the NAP data model.

Args:
  name (string): Schema name — 'manifest' or 'commit'

Returns: JSON Schema document (draft-07) describing the type.

Examples:
  - "Show me the manifest schema" → name="manifest"
  - "What fields does a commit have?" → name="commit"`,
        inputSchema: GetSchemaInputSchema,
        annotations: {
            readOnlyHint: true,
            destructiveHint: false,
            idempotentHint: true,
            openWorldHint: false,
        },
    }, async (params) => {
        try {
            const schema = await getSchema(params.name);
            return {
                content: [
                    { type: "text", text: JSON.stringify(schema, null, 2) },
                ],
            };
        }
        catch (err) {
            if (err instanceof NapApiError) {
                return {
                    isError: true,
                    content: [
                        {
                            type: "text",
                            text: err.status === 404
                                ? `Error: Schema '${params.name}' not found. Available schemas: 'manifest', 'commit'.`
                                : `Error: ${err.message}`,
                        },
                    ],
                };
            }
            return {
                isError: true,
                content: [
                    {
                        type: "text",
                        text: `Error: ${err instanceof Error ? err.message : String(err)}`,
                    },
                ],
            };
        }
    });
}
//# sourceMappingURL=schema.js.map