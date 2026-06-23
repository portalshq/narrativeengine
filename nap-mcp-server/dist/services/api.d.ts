/**
 * Shared NAP HTTP API client.
 *
 * All tools delegate to the nap-server REST API.  This module provides
 * a single, typed `napRequest` helper that all tools use, avoiding
 * duplication of URL-building, error handling, and response parsing.
 */
import type { Manifest, CommitEntry } from "../types.js";
export declare class NapApiError extends Error {
    readonly status: number;
    readonly code?: string | undefined;
    constructor(message: string, status: number, code?: string | undefined);
}
export declare class NapNotFoundError extends NapApiError {
    constructor(message: string);
}
export interface NapUriParts {
    universe: string;
    entity_type: string;
    entity_id: string;
}
/**
 * Parse a `nap://` URI into its components.
 * Throws if the URI is malformed.
 */
export declare function parseNapUri(uri: string): NapUriParts & {
    fragment?: string;
};
/** Resolve a manifest (with optional selectors). */
export declare function resolveManifest(parts: NapUriParts, params?: {
    branch?: string;
    commit?: string;
    tag?: string;
    path?: string;
}): Promise<Manifest | unknown>;
/** Get entity commit history. */
export declare function getHistory(parts: NapUriParts, limit?: number): Promise<CommitEntry[]>;
/** List all universes. */
export declare function listUniverses(): Promise<string[]>;
/** List entities in a universe, optionally filtered by type. */
export declare function listEntities(universe: string, entityType?: string): Promise<Record<string, string[]>>;
/** Commit a property change to an entity. */
export declare function commitChanges(parts: NapUriParts, body: {
    message: string;
    author: string;
    properties: Record<string, unknown>;
}): Promise<{
    commit_id: string;
    version: number;
}>;
/** Revert a commit by hash across an entire universe. */
export declare function revertCommit(universe: string, body: {
    commit: string;
    author: string;
}): Promise<{
    reverted_commit: string;
    new_commit: string;
    author: string;
}>;
/** Check server health. */
export declare function healthCheck(): Promise<{
    status: string;
    protocol: string;
    version: string;
}>;
/** Fetch the JSON Schema for a given schema name. */
export declare function getSchema(schemaName: string): Promise<Record<string, unknown>>;
/** Initialize a new universe repository. */
export declare function initUniverse(universe: string): Promise<{
    success: boolean;
    universe: string;
    path: string;
}>;
/** Create a new entity in a universe. */
export declare function createEntity(universe: string, entityType: string, entityId: string, body: {
    name: string;
    author: string;
}): Promise<{
    success: boolean;
    uri: string;
    commit_id: string;
    version: number;
}>;
/** Delete an entity from a universe. */
export declare function deleteEntity(universe: string, entityType: string, entityId: string, author: string): Promise<{
    success: boolean;
    commit_id: string;
}>;
/** List branches in a universe. */
export declare function listBranches(universe: string): Promise<{
    universe: string;
    branches: string[];
}>;
/** Create a branch in a universe. */
export declare function createBranch(universe: string, name: string): Promise<{
    success: boolean;
    branch: string;
}>;
/** Switch to a branch in a universe. */
export declare function switchBranch(universe: string, name: string): Promise<{
    success: boolean;
    branch: string;
}>;
/** List tags in a universe. */
export declare function listTags(universe: string): Promise<{
    universe: string;
    tags: string[];
}>;
/** Create a tag in a universe. */
export declare function createTag(universe: string, name: string): Promise<{
    success: boolean;
    tag: string;
}>;
/** List remotes in a universe. */
export declare function listRemotes(universe: string): Promise<{
    universe: string;
    remotes: Array<{
        name: string;
        url: string;
    }>;
}>;
/** Add a remote to a universe. */
export declare function addRemote(universe: string, name: string, url: string): Promise<{
    success: boolean;
    remote: string;
    url: string;
}>;
/** Remove a remote from a universe. */
export declare function removeRemote(universe: string, name: string): Promise<{
    success: boolean;
    removed: string;
}>;
/** Push a universe to its remote. */
export declare function pushUniverse(universe: string, remote?: string, branch?: string): Promise<{
    success: boolean;
    universe: string;
}>;
/** Pull a universe from its remote. */
export declare function pullUniverse(universe: string, remote?: string, branch?: string): Promise<{
    success: boolean;
    universe: string;
}>;
/** Compute the SHA-256 content hash of data (base64-encoded). */
export declare function computeContentHash(data: string): Promise<{
    hash: string;
    algorithm: string;
}>;
/** Validate a manifest against the NAP schema. */
export declare function validateManifest(universe: string, entityType: string, entityId: string): Promise<{
    valid: boolean;
    uri: string;
    errors: string[];
}>;
export declare function searchEntities(universe: string, query: string, entityType?: string): Promise<Array<{
    uri: string;
    name: string;
    entity_type: string;
}>>;
//# sourceMappingURL=api.d.ts.map