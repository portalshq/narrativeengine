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
/** Search entities in a universe by substring match against name/id. */
export declare function searchEntities(universe: string, query: string, entityType?: string): Promise<Array<{
    uri: string;
    name: string;
    entity_type: string;
}>>;
//# sourceMappingURL=api.d.ts.map