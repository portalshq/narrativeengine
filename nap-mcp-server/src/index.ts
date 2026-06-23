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

import { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import { StdioServerTransport } from "@modelcontextprotocol/sdk/server/stdio.js";
import express from "express";
import { StreamableHTTPServerTransport } from "@modelcontextprotocol/sdk/server/streamableHttp.js";

import { registerResolveTools } from "./tools/resolve.js";
import { registerDiscoveryTools } from "./tools/discovery.js";
import { registerMutationTools } from "./tools/mutation.js";
import { registerSchemaTools } from "./tools/schema.js";
import { registerManagementTools } from "./tools/management.js";
import { CONFIG } from "./constants.js";

// ── Server Initialization ─────────────────────────────────────────────────

const server = new McpServer({
  name: "nap-mcp-server",
  version: "1.0.0",
});

// ── Tool Registration ─────────────────────────────────────────────────────

// Read-only: resolution
registerResolveTools(server);
// Read-only: discovery
registerDiscoveryTools(server);
// Read-write: mutations
registerMutationTools(server);
// Read-only: schema introspection
registerSchemaTools(server);
// Read-write: init, CRUD, VCS, remotes, etc.
registerManagementTools(server);

// ── Resource Registration ─────────────────────────────────────────────────

import type { Manifest } from "./types.js";
import { parseNapUri, resolveManifest } from "./services/api.js";

// nap:// URI resources — agents can read any NAP resource by URI
server.registerResource(
  {
    uri: "nap://{universe}/{entity_type}/{entity_id}",
    name: "NAP Resource",
    description:
      "Access any NAP narrative resource by its nap:// URI. " +
      "Returns the full manifest as JSON. " +
      "Supports fragment syntax for subtree queries (e.g., nap://starwars/character/luke#properties.species).",
    mimeType: "application/json",
  },
  async (uri: string) => {
    try {
      const parts = parseNapUri(uri);
      const manifest = await resolveManifest(parts);
      return {
        contents: [
          {
            uri,
            mimeType: "application/json",
            text: JSON.stringify(manifest, null, 2),
          },
        ],
      };
    } catch (err) {
      const errorMessage =
        err instanceof Error ? err.message : String(err);
      return {
        contents: [
          {
            uri,
            mimeType: "text/plain",
            text: `Error resolving ${uri}: ${errorMessage}`,
          },
        ],
      };
    }
  },
);

// List all NAP resources (universes)
server.registerResourceList(async () => {
  const { listUniverses } = await import("./services/api.js");
  const universes = await listUniverses();
  return {
    resources: [
      {
        uri: "nap://universes",
        name: "All Universes",
        description: "List of all NAP universes",
        mimeType: "application/json",
      },
      ...universes.map((u: string) => ({
        uri: `nap://${u}/` as const,
        name: `${u} Universe`,
        description: `Entities in the ${u} universe`,
        mimeType: "application/json",
      })),
    ],
  };
});

// ── Transport & Startup ───────────────────────────────────────────────────

async function runStdio(): Promise<void> {
  const transport = new StdioServerTransport();
  await server.connect(transport);
  console.error(
    `[nap-mcp-server] Running via stdio → ${CONFIG.napServerUrl}`,
  );
}

async function runHTTP(): Promise<void> {
  const app = express();
  app.use(express.json());

  app.post("/mcp", async (req, res) => {
    const transport = new StreamableHTTPServerTransport({
      sessionIdGenerator: undefined,
      enableJsonResponse: true,
    });
    res.on("close", () => {
      transport.close();
    });
    await server.connect(transport);
    await transport.handleRequest(req, res, req.body);
  });

  const port = parseInt(process.env.PORT || "3000", 10);
  app.listen(port, () => {
    console.error(
      `[nap-mcp-server] Running via HTTP on :${port}/mcp → ${CONFIG.napServerUrl}`,
    );
  });
}

const transport = process.env.TRANSPORT || "stdio";
if (transport === "http") {
  runHTTP().catch((err) => {
    console.error("Fatal server error:", err);
    process.exit(1);
  });
} else {
  runStdio().catch((err) => {
    console.error("Fatal server error:", err);
    process.exit(1);
  });
}
