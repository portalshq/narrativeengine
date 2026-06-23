/**
 * Shared NAP HTTP API client.
 *
 * All tools delegate to the nap-server REST API.  This module provides
 * a single, typed `napRequest` helper that all tools use, avoiding
 * duplication of URL-building, error handling, and response parsing.
 */
import { CONFIG, ENTITY_TYPES } from "../constants.js";
// ── Error Types ───────────────────────────────────────────────────────────
export class NapApiError extends Error {
    status;
    code;
    constructor(message, status, code) {
        super(message);
        this.status = status;
        this.code = code;
        this.name = "NapApiError";
    }
}
export class NapNotFoundError extends NapApiError {
    constructor(message) {
        super(message, 404, "NOT_FOUND");
        this.name = "NapNotFoundError";
    }
}
/**
 * Parse a `nap://` URI into its components.
 * Throws if the URI is malformed.
 */
export function parseNapUri(uri) {
    if (!uri.startsWith("nap://")) {
        throw new NapApiError(`Invalid NAP URI: must start with 'nap://', got '${uri.slice(0, 20)}…'`, 400, "INVALID_URI");
    }
    const withoutScheme = uri.slice("nap://".length);
    const [pathPart, fragment] = withoutScheme.split("#");
    const segments = pathPart.split("/").filter(Boolean);
    if (segments.length < 3) {
        throw new NapApiError(`Invalid NAP URI '${uri}': expected at least 3 segments (universe/type/id), got ${segments.length}`, 400, "INVALID_URI");
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
async function napFetch(path, options = {}) {
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
            let apiError;
            try {
                apiError = (await response.json());
            }
            catch {
                apiError = { error: response.statusText, code: "HTTP_ERROR" };
            }
            throw new NapApiError(apiError.error, response.status, apiError.code);
        }
        return (await response.json());
    }
    catch (err) {
        if (err instanceof NapApiError)
            throw err;
        if (err instanceof Error && err.name === "AbortError") {
            throw new NapApiError("Request timed out after 30s. Check that nap-server is running.", 408, "TIMEOUT");
        }
        throw new NapApiError(`Could not connect to NAP server at ${CONFIG.napServerUrl}. ` +
            `Make sure nap-server is running. Details: ${err instanceof Error ? err.message : String(err)}`, 502, "CONNECTION_REFUSED");
    }
    finally {
        clearTimeout(timeout);
    }
}
// ── High-Level API Operations ─────────────────────────────────────────────
/** Resolve a manifest (with optional selectors). */
export async function resolveManifest(parts, params) {
    const query = new URLSearchParams();
    if (params?.branch)
        query.set("branch", params.branch);
    if (params?.commit)
        query.set("commit", params.commit);
    if (params?.tag)
        query.set("tag", params.tag);
    if (params?.path)
        query.set("path", params.path);
    const qs = query.toString();
    const path = `/resolve/${parts.universe}/${parts.entity_type}/${parts.entity_id}${qs ? `?${qs}` : ""}`;
    return napFetch(path);
}
/** Get entity commit history. */
export async function getHistory(parts, limit = 20) {
    return napFetch(`/history/${parts.universe}/${parts.entity_type}/${parts.entity_id}?limit=${limit}`);
}
/** List all universes. */
export async function listUniverses() {
    const result = await napFetch("/universes");
    return result.universes;
}
/** List entities in a universe, optionally filtered by type. */
export async function listEntities(universe, entityType) {
    const params = entityType ? `?type=${entityType}` : "";
    return napFetch(`/universes/${universe}/entities${params}`);
}
/** Commit a property change to an entity. */
export async function commitChanges(parts, body) {
    return napFetch(`/commit/${parts.universe}/${parts.entity_type}/${parts.entity_id}`, {
        method: "POST",
        body: JSON.stringify(body),
    });
}
/** Revert a commit by hash across an entire universe. */
export async function revertCommit(universe, body) {
    return napFetch(`/revert/${universe}`, { method: "POST", body: JSON.stringify(body) });
}
/** Check server health. */
export async function healthCheck() {
    return napFetch("/health");
}
/** Fetch the JSON Schema for a given schema name. */
export async function getSchema(schemaName) {
    return napFetch(`/schema/${schemaName}`);
}
// ── Init Universe ─────────────────────────────────────────────
/** Initialize a new universe repository. */
export async function initUniverse(universe) {
    return napFetch(`/init/${universe}`, { method: "POST" });
}
// ── Create Entity ─────────────────────────────────────────────
/** Create a new entity in a universe. */
export async function createEntity(universe, entityType, entityId, body) {
    return napFetch(`/create/${universe}/${entityType}/${entityId}`, { method: "POST", body: JSON.stringify(body) });
}
// ── Delete Entity ─────────────────────────────────────────────
/** Delete an entity from a universe. */
export async function deleteEntity(universe, entityType, entityId, author) {
    return napFetch(`/${universe}/${entityType}/${entityId}`, { method: "DELETE", body: JSON.stringify({ author }) });
}
// ── Branch Operations ─────────────────────────────────────────
/** List branches in a universe. */
export async function listBranches(universe) {
    return napFetch(`/branches/${universe}`);
}
/** Create a branch in a universe. */
export async function createBranch(universe, name) {
    return napFetch(`/branches/${universe}`, { method: "POST", body: JSON.stringify({ name }) });
}
/** Switch to a branch in a universe. */
export async function switchBranch(universe, name) {
    return napFetch(`/switch/${universe}`, { method: "POST", body: JSON.stringify({ name }) });
}
// ── Tag Operations ────────────────────────────────────────────
/** List tags in a universe. */
export async function listTags(universe) {
    return napFetch(`/tags/${universe}`);
}
/** Create a tag in a universe. */
export async function createTag(universe, name) {
    return napFetch(`/tags/${universe}`, { method: "POST", body: JSON.stringify({ name }) });
}
// ── Remote Operations ─────────────────────────────────────────
/** List remotes in a universe. */
export async function listRemotes(universe) {
    return napFetch(`/remotes/${universe}`);
}
/** Add a remote to a universe. */
export async function addRemote(universe, name, url) {
    return napFetch(`/remotes/${universe}`, { method: "POST", body: JSON.stringify({ name, url }) });
}
/** Remove a remote from a universe. */
export async function removeRemote(universe, name) {
    return napFetch(`/remotes/${universe}/${name}`, { method: "DELETE" });
}
// ── Push / Pull ───────────────────────────────────────────────
/** Push a universe to its remote. */
export async function pushUniverse(universe, remote, branch) {
    return napFetch(`/push/${universe}`, { method: "POST", body: JSON.stringify({ remote, branch }) });
}
/** Pull a universe from its remote. */
export async function pullUniverse(universe, remote, branch) {
    return napFetch(`/pull/${universe}`, { method: "POST", body: JSON.stringify({ remote, branch }) });
}
// ── Content Hash ──────────────────────────────────────────────
/** Compute the SHA-256 content hash of data (base64-encoded). */
export async function computeContentHash(data) {
    return napFetch(`/content-hash`, { method: "POST", body: JSON.stringify({ data }) });
}
// ── Validate Manifest ─────────────────────────────────────────
/** Validate a manifest against the NAP schema. */
export async function validateManifest(universe, entityType, entityId) {
    return napFetch(`/validate/${universe}/${entityType}/${entityId}`);
}
// ── Search Entities ───────────────────────────────────────────
export async function searchEntities(universe, query, entityType) {
    const types = entityType
        ? [entityType]
        : ENTITY_TYPES.filter((t) => t !== "world");
    const results = [];
    for (const et of types) {
        const entities = await listEntities(universe, et);
        const uris = entities[et] ?? [];
        for (const uri of uris) {
            try {
                const parts = parseNapUri(uri);
                const manifest = (await resolveManifest(parts));
                if (manifest.name?.toLowerCase().includes(query.toLowerCase()) ||
                    parts.entity_id.toLowerCase().includes(query.toLowerCase())) {
                    results.push({
                        uri,
                        name: manifest.name,
                        entity_type: et,
                    });
                }
            }
            catch {
                // Silently skip entities we can't resolve
            }
        }
    }
    return results;
}
//# sourceMappingURL=api.js.map