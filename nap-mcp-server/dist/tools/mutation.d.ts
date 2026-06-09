/**
 * Mutation tools — create entities, set properties, and commit changes.
 *
 * These tools MODIFY state.  All have destructiveHint: false because they
 * are versioned (every change is a Git commit) and can be rolled back,
 * but they DO change the universe state.
 */
import type { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
export declare function registerMutationTools(server: McpServer): void;
//# sourceMappingURL=mutation.d.ts.map