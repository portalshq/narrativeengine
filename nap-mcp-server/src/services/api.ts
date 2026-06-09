/**
 * Shared NAP HTTP API client.
 *
 * All tools delegate to the nap-server REST API.  This module provides
 * a single, typed `napRequest` helper that all tools use, avoiding
 * duplication of URL-building, error handling, and response parsing.
 */

import type { ApiError, Manifest, CommitEntry } from "../types.js";
import { CONFIG, ENTITY_TYPES } from "../constants.js";

// ── Error Types ───────────────────────────────────────────────────────────

export class NapApiError extends Error {
  constructor(
    message: string,
    public readonly status: number,
    public readonly code?: string,
  ) {
    super(message);
    this.name = "NapApiError";
  }
}

export class NapNotFoundError extends NapApiError {
  constructor(message: string) {
    super(message, 404, "NOT_FOUND");
    this.name = "NapNotFoundError";
  }
}

// ── URI Parsing ───────────────────────────────────────────────────────────

export interface NapUriParts {
  universe: string;
  entity_type: string;
  entity_id: string;
}

/**
 * Parse a `nap://` URI into its components.
 * Throws if the URI is malformed.
 */
export function parseNapUri(uri: string): NapUriParts & { fragment?: string } {
  if (!uri.startsWith("nap://")) {
    throw new NapApiError(
      `Invalid NAP URI: must start with 'nap://', got '${uri.slice(0, 20)}…'`,
      400,
      "INVALID_URI",
    );
  }

  const withoutScheme = uri.slice("nap://".length);
  const [pathPart, fragment] = withoutScheme.split("#");
  const segments = pathPart.split("/").filter(Boolean);

  if (segments.length < 3) {
    throw new NapApiError(
      `Invalid NAP URI '${uri}': expected at least 3 segments (universe/type/id), got ${segments.length}`,
      400,
      "INVALID_URI",
    );
  }

  return {
    universe: segments[0],
    entity_type: segments[1],
    entity_id: segments.slice(2).join("/"),
    fragment,
  };
}

// ── HTTP Client ───────────────────────────────────────────────────────────

/** Raw fetch wrapper with JSON parsing and unified error handling. */
async function napFetch<T>(
  path: string,
  options: RequestInit = {},
): Promise<T> {
  const url = `${CONFIG.napServerUrl}${path}`;
  const controller = new AbortController();
  const timeout = setTimeout(() => controller.abort(), 30_000);

  try {
    const response = await fetch(url, {
      ...options,
      signal: controller.signal,
      headers: {
        "Content-Type": "application/json",
        Accept: "application/json",
        ...options.headers,
      },
    });

    if (!response.ok) {
      let apiError: ApiError;
      try {
        apiError = (await response.json()) as ApiError;
      } catch {
        apiError = { error: response.statusText, code: "HTTP_ERROR" };
      }
      throw new NapApiError(apiError.error, response.status, apiError.code);
    }

    return (await response.json()) as T;
  } catch (err) {
    if (err instanceof NapApiError) throw err;
    if (err instanceof Error && err.name === "AbortError") {
      throw new NapApiError(
        "Request timed out after 30s. Check that nap-server is running.",
        408,
        "TIMEOUT",
      );
    }
    throw new NapApiError(
      `Could not connect to NAP server at ${CONFIG.napServerUrl}. ` +
        `Make sure nap-server is running. Details: ${err instanceof Error ? err.message : String(err)}`,
      502,
      "CONNECTION_REFUSED",
    );
  } finally {
    clearTimeout(timeout);
  }
}

// ── High-Level API Operations ─────────────────────────────────────────────

/** Resolve a manifest (with optional selectors). */
export async function resolveManifest(
  parts: NapUriParts,
  params?: { branch?: string; commit?: string; tag?: string; path?: string },
): Promise<Manifest | unknown> {
  const query = new URLSearchParams();
  if (params?.branch) query.set("branch", params.branch);
  if (params?.commit) query.set("commit", params.commit);
  if (params?.tag) query.set("tag", params.tag);
  if (params?.path) query.set("path", params.path);

  const qs = query.toString();
  const path = `/resolve/${parts.universe}/${parts.entity_type}/${parts.entity_id}${qs ? `?${qs}` : ""}`;
  return napFetch<Manifest>(path);
}

/** Get entity commit history. */
export async function getHistory(
  parts: NapUriParts,
  limit = 20,
): Promise<CommitEntry[]> {
  return napFetch<CommitEntry[]>(
    `/history/${parts.universe}/${parts.entity_type}/${parts.entity_id}?limit=${limit}`,
  );
}

/** List all universes. */
export async function listUniverses(): Promise<string[]> {
  const result = await napFetch<{ universes: string[] }>("/universes");
  return result.universes;
}

/** List entities in a universe, optionally filtered by type. */
export async function listEntities(
  universe: string,
  entityType?: string,
): Promise<Record<string, string[]>> {
  const params = entityType ? `?type=${entityType}` : "";
  return napFetch<Record<string, string[]>>(
    `/universes/${universe}/entities${params}`,
  );
}

/** Commit a property change to an entity. */
export async function commitChanges(
  parts: NapUriParts,
  body: {
    message: string;
    author: string;
    properties: Record<string, unknown>;
  },
): Promise<{ commit_id: string; version: number }> {
  return napFetch<{ commit_id: string; version: number }>(
    `/commit/${parts.universe}/${parts.entity_type}/${parts.entity_id}`,
    {
      method: "POST",
      body: JSON.stringify(body),
    },
  );
}

/** Revert a commit by hash across an entire universe. */
export async function revertCommit(
  universe: string,
  body: { commit: string; author: string },
): Promise<{ reverted_commit: string; new_commit: string; author: string }> {
  return napFetch<{ reverted_commit: string; new_commit: string; author: string }>(
    `/revert/${universe}`,
    { method: "POST", body: JSON.stringify(body) },
  );
}

/** Check server health. */
export async function healthCheck(): Promise<{
  status: string;
  protocol: string;
  version: string;
}> {
  return napFetch("/health");
}

/** Fetch the JSON Schema for a given schema name. */
export async function getSchema(
  schemaName: string,
): Promise<Record<string, unknown>> {
  return napFetch<Record<string, unknown>>(`/schema/${schemaName}`);
}

/** Search entities in a universe by substring match against name/id. */
export async function searchEntities(
  universe: string,
  query: string,
  entityType?: string,
): Promise<Array<{ uri: string; name: string; entity_type: string }>> {
  const types = entityType
    ? [entityType]
    : ENTITY_TYPES.filter((t) => t !== "world");

  const results: Array<{ uri: string; name: string; entity_type: string }> =
    [];

  for (const et of types) {
    const entities = await listEntities(universe, et);
    const uris = entities[et] ?? [];
    for (const uri of uris) {
      try {
        const parts = parseNapUri(uri);
        const manifest = (await resolveManifest(parts)) as Manifest;
        if (
          manifest.name?.toLowerCase().includes(query.toLowerCase()) ||
          parts.entity_id.toLowerCase().includes(query.toLowerCase())
        ) {
          results.push({
            uri,
            name: manifest.name,
            entity_type: et,
          });
        }
      } catch {
        // Silently skip entities we can't resolve
      }
    }
  }

  return results;
}
