/**
 * Type definitions for the NAP MCP Server.
 *
 * Mirrors the core NAP data model (nap-core/src/) so the MCP server
 * can produce typed, structured output without needing the Rust crate.
 */
/** Parsed components of a `nap://` URI. */
export interface NapUriComponents {
    universe: string;
    entity_type: string;
    entity_id: string;
    fragment?: string;
}
/** A NAP manifest — the canonical representation of a narrative resource. */
export interface Manifest {
    id: string;
    name: string;
    entity_type: string;
    version: number;
    properties: Record<string, unknown>;
    representations: Record<string, Representation>;
    references: Record<string, unknown>;
    provenance?: Provenance;
    head?: string;
}
/** A content-addressed representation (image, voice model, mesh, etc.). */
export interface Representation {
    hash: string;
    format: string;
    uri?: string;
    tier?: string;
}
/** AI generation provenance metadata. */
export interface Provenance {
    model?: string;
    prompt_hash?: string;
    seed?: string;
    parameters?: Record<string, string>;
    derived_from?: string;
    created_at?: string;
}
/** Standard NAP API error response. */
export interface ApiError {
    error: string;
    code: string;
}
/** An individual commit entry in entity history. */
export interface CommitEntry {
    id: string;
    parent?: string;
    author: string;
    message: string;
    timestamp: string;
}
export interface ServerConfig {
    /** Base URL of the NAP HTTP resolver server. */
    napServerUrl: string;
    /** Maximum characters in a tool response before truncation. */
    characterLimit: number;
}
//# sourceMappingURL=types.d.ts.map