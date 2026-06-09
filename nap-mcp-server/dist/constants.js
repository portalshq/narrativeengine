/**
 * Shared constants for the NAP MCP Server.
 */
/** Default NAP HTTP server URL. */
export const DEFAULT_NAP_SERVER_URL = "http://localhost:3100";
/** Maximum characters in a tool response before truncation. */
export const CHARACTER_LIMIT = 25_000;
/** Resolved server configuration (from env vars with defaults). */
export const CONFIG = {
    napServerUrl: process.env.NAP_SERVER_URL ?? DEFAULT_NAP_SERVER_URL,
    characterLimit: CHARACTER_LIMIT,
};
/** Valid entity types in the NAP protocol. */
export const ENTITY_TYPES = [
    "character",
    "location",
    "scene",
    "prop",
    "world",
];
//# sourceMappingURL=constants.js.map