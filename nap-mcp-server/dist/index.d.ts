#!/usr/bin/env node
/**
 * NAP MCP Server — Model Context Protocol server for the Narrative Addressing Protocol.
 *
 * Exposes NAP operations as MCP tools so that any MCP-compatible agent (Claude,
 * Cursor, LangChain, etc.) can resolve, query, create, and manage narrative
 * universes.
 *
 * # Usage
 *
 * ## stdio (local development)
 *   npm start
 *
 * ## Streamable HTTP (remote / multi-client)
 *   TRANSPORT=http NAP_PORT=3000 npm start
 *
 * # Configuration
 *
 * | Env Variable       | Default                  | Description                     |
 * |--------------------|--------------------------|---------------------------------|
 * | NAP_SERVER_URL     | http://localhost:3100     | Base URL of NAP HTTP server     |
 * | TRANSPORT          | stdio                    | Transport: stdio or http        |
 * | PORT               | 3000                     | Port for HTTP transport         |
 */
export {};
//# sourceMappingURL=index.d.ts.map