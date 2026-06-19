import { homedir } from "node:os";
import { join } from "node:path";
import native from "./native.js";

export function parseUri(uri: string): Record<string, unknown> {
  return JSON.parse(native.parseUri(uri)) as Record<string, unknown>;
}

export function parseManifest(yamlStr: string): Record<string, unknown> {
  return JSON.parse(native.parseManifest(yamlStr)) as Record<string, unknown>;
}

export function resolve(uri: string, repoPath?: string): Record<string, unknown> {
  if (!repoPath) {
    repoPath = process.env.NAP_DIR || join(homedir(), ".nap");
  }
  return JSON.parse(native.resolve(uri, repoPath)) as Record<string, unknown>;
}

export function version(): string {
  return native.version();
}

/**
 * Ingest raw media bytes into the content-addressed storage engine.
 *
 * The storage backend is determined by the ``NAP_STORAGE_BACKEND``
 * environment variable at the Rust layer (``local`` or ``s3``).
 *
 * @param data - Raw bytes of the media asset (image, audio, mesh, etc.).
 * @param format - File extension without a leading dot (e.g. ``"png"``,
 *   ``"jpg"``, ``"wav"``, ``"glb"``).
 * @returns A Promise resolving to the content-addressed hash ``sha256:<hex>``.
 */
export async function ingestMedia(data: Buffer, format: string): Promise<string> {
  return native.ingestMedia(data, format);
}
