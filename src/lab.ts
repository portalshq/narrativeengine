import type { NarrativeEngine } from "./engine";
import { randomUUID } from "node:crypto";

export const GLOBAL_KEY = Symbol.for("narrative.engine.registry");
export const LAB_TOKEN = Symbol.for("narrative.lab.token");

if (!(global as any)[LAB_TOKEN]) {
  (global as any)[LAB_TOKEN] = process.env.LAB_SECRET || `lab_${randomUUID()}`;
}
export const SESSION_SECRET = (global as any)[LAB_TOKEN];

export function configureLabEngine(engine: NarrativeEngine): void {
  (global as any)[GLOBAL_KEY] = engine;
}

export function getActiveEngine(): NarrativeEngine | undefined {
  return (global as any)[GLOBAL_KEY];
}
