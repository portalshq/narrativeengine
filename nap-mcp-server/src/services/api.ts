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
  repository: string;
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
      `Invalid NAP URI '${uri}': expected at least 3 segments (repository/type/id), got ${segments.length}`,
      400,
      "INVALID_URI",
    );
  }

  return {
    repository: segments[0],
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
  const path = `/resolve/${parts.repository}/${parts.entity_type}/${parts.entity_id}${qs ? `?${qs}` : ""}`;
  return napFetch<Manifest>(path);
}

/** Get entity commit history. */
export async function getHistory(
  parts: NapUriParts,
  limit = 20,
): Promise<CommitEntry[]> {
  return napFetch<CommitEntry[]>(
    `/history/${parts.repository}/${parts.entity_type}/${parts.entity_id}?limit=${limit}`,
  );
}

/** List all repositories. */
export async function listRepositories(): Promise<string[]> {
  const result = await napFetch<{ repositories: string[] }>("/repositories");
  return result.repositories;
}

/** List entities in a repository, optionally filtered by type. */
export async function listEntities(
  repository: string,
  entityType?: string,
): Promise<Record<string, string[]>> {
  const params = entityType ? `?type=${entityType}` : "";
  return napFetch<Record<string, string[]>>(
    `/repositories/${repository}/entities${params}`,
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
    `/commit/${parts.repository}/${parts.entity_type}/${parts.entity_id}`,
    {
      method: "POST",
      body: JSON.stringify(body),
    },
  );
}

/** Revert a commit by hash across an entire repository. */
export async function revertCommit(
  repository: string,
  body: { commit: string; author: string },
): Promise<{ reverted_commit: string; new_commit: string; author: string }> {
  return napFetch<{ reverted_commit: string; new_commit: string; author: string }>(
    `/revert/${repository}`,
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

// ── Init Repository ─────────────────────────────────────────────

/** Initialize a new repository repository. */
export async function initUniverse(
  repository: string,
): Promise<{ success: boolean; repository: string; path: string }> {
  return napFetch<{ success: boolean; repository: string; path: string }>(
    `/init/${repository}`,
    { method: "POST" },
  );
}

// ── Create Entity ─────────────────────────────────────────────

/** Create a new entity in a repository. */
export async function createEntity(
  repository: string,
  entityType: string,
  entityId: string,
  body: { name: string; author: string },
): Promise<{ success: boolean; uri: string; commit_id: string; version: number }> {
  return napFetch<{ success: boolean; uri: string; commit_id: string; version: number }>(
    `/create/${repository}/${entityType}/${entityId}`,
    { method: "POST", body: JSON.stringify(body) },
  );
}

// ── Delete Entity ─────────────────────────────────────────────

/** Delete an entity from a repository. */
export async function deleteEntity(
  repository: string,
  entityType: string,
  entityId: string,
  author: string,
): Promise<{ success: boolean; commit_id: string }> {
  return napFetch<{ success: boolean; commit_id: string }>(
    `/${repository}/${entityType}/${entityId}`,
    { method: "DELETE", body: JSON.stringify({ author }) },
  );
}

// ── Branch Operations ─────────────────────────────────────────

/** List branches in a repository. */
export async function listBranches(
  repository: string,
): Promise<{ repository: string; branches: string[] }> {
  return napFetch<{ repository: string; branches: string[] }>(
    `/branches/${repository}`,
  );
}

/** Create a branch in a repository. */
export async function createBranch(
  repository: string,
  name: string,
): Promise<{ success: boolean; branch: string }> {
  return napFetch<{ success: boolean; branch: string }>(
    `/branches/${repository}`,
    { method: "POST", body: JSON.stringify({ name }) },
  );
}

/** Switch to a branch in a repository. */
export async function switchBranch(
  repository: string,
  name: string,
): Promise<{ success: boolean; branch: string }> {
  return napFetch<{ success: boolean; branch: string }>(
    `/switch/${repository}`,
    { method: "POST", body: JSON.stringify({ name }) },
  );
}

// ── Tag Operations ────────────────────────────────────────────

/** List tags in a repository. */
export async function listTags(
  repository: string,
): Promise<{ repository: string; tags: string[] }> {
  return napFetch<{ repository: string; tags: string[] }>(`/tags/${repository}`);
}

/** Create a tag in a repository. */
export async function createTag(
  repository: string,
  name: string,
): Promise<{ success: boolean; tag: string }> {
  return napFetch<{ success: boolean; tag: string }>(
    `/tags/${repository}`,
    { method: "POST", body: JSON.stringify({ name }) },
  );
}

// ── Remote Operations ─────────────────────────────────────────

/** List remotes in a repository. */
export async function listRemotes(
  repository: string,
): Promise<{ repository: string; remotes: Array<{ name: string; url: string }> }> {
  return napFetch<{ repository: string; remotes: Array<{ name: string; url: string }> }>(
    `/remotes/${repository}`,
  );
}

/** Add a remote to a repository. */
export async function addRemote(
  repository: string,
  name: string,
  url: string,
): Promise<{ success: boolean; remote: string; url: string }> {
  return napFetch<{ success: boolean; remote: string; url: string }>(
    `/remotes/${repository}`,
    { method: "POST", body: JSON.stringify({ name, url }) },
  );
}

/** Remove a remote from a repository. */
export async function removeRemote(
  repository: string,
  name: string,
): Promise<{ success: boolean; removed: string }> {
  return napFetch<{ success: boolean; removed: string }>(
    `/remotes/${repository}/${name}`,
    { method: "DELETE" },
  );
}

// ── Push / Pull ───────────────────────────────────────────────

/** Push a repository to its remote. */
export async function pushUniverse(
  repository: string,
  remote?: string,
  branch?: string,
): Promise<{ success: boolean; repository: string }> {
  return napFetch<{ success: boolean; repository: string }>(
    `/push/${repository}`,
    { method: "POST", body: JSON.stringify({ remote, branch }) },
  );
}

/** Pull a repository from its remote. */
export async function pullUniverse(
  repository: string,
  remote?: string,
  branch?: string,
): Promise<{ success: boolean; repository: string }> {
  return napFetch<{ success: boolean; repository: string }>(
    `/pull/${repository}`,
    { method: "POST", body: JSON.stringify({ remote, branch }) },
  );
}

// ── Content Hash ──────────────────────────────────────────────

/** Compute the SHA-256 content hash of data (base64-encoded). */
export async function computeContentHash(
  data: string,
): Promise<{ hash: string; algorithm: string }> {
  return napFetch<{ hash: string; algorithm: string }>(
    `/content-hash`,
    { method: "POST", body: JSON.stringify({ data }) },
  );
}

// ── Validate Manifest ─────────────────────────────────────────

/** Validate a manifest against the NAP schema. */
export async function validateManifest(
  repository: string,
  entityType: string,
  entityId: string,
): Promise<{ valid: boolean; uri: string; errors: string[] }> {
  return napFetch<{ valid: boolean; uri: string; errors: string[] }>(
    `/validate/${repository}/${entityType}/${entityId}`,
  );
}

// ── Search Entities ───────────────────────────────────────────
export async function searchEntities(
  repository: string,
  query: string,
  entityType?: string,
): Promise<Array<{ uri: string; name: string; entity_type: string }>> {
  const types = entityType
    ? [entityType]
    : ENTITY_TYPES.filter((t) => t !== "world");

  const results: Array<{ uri: string; name: string; entity_type: string }> =
    [];

  for (const et of types) {
    const entities = await listEntities(repository, et);
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
