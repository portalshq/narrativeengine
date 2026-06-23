/**
 * Management tools — universe init, entity CRUD, VCS operations, remotes, etc.
 *
 * These tools MODIFY state. All have destructiveHint: false because they
 * are versioned (every change is a Git commit) and can be rolled back.
 */
import type { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
export declare function registerManagementTools(server: McpServer): void;
//# sourceMappingURL=management.d.ts.map