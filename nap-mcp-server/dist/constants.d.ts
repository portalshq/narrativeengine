/**
 * Shared constants for the NAP MCP Server.
 */
import type { ServerConfig } from "./types.js";
/** Default NAP HTTP server URL. */
export declare const DEFAULT_NAP_SERVER_URL = "http://localhost:3100";
/** Maximum characters in a tool response before truncation. */
export declare const CHARACTER_LIMIT = 25000;
/** Resolved server configuration (from env vars with defaults). */
export declare const CONFIG: ServerConfig;
/** Valid entity types in the NAP protocol. */
export declare const ENTITY_TYPES: readonly ["character", "location", "scene", "prop", "world"];
export type EntityType = (typeof ENTITY_TYPES)[number];
//# sourceMappingURL=constants.d.ts.map