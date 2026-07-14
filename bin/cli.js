#!/usr/bin/env node
import { resolve } from "path";
async function boot() {
    const args = process.argv.slice(2).filter(arg => arg !== "--");
    const [command, entryPath] = args;
    if (command !== "lab") {
        console.error("Usage: npx narrativeengine lab");
        console.error("\nStart the NarrativeEngine Lab:");
        console.error("  npx narrativeengine lab");
        process.exit(1);
    }
    try {
        if (entryPath) {
            const absolutePath = resolve(process.cwd(), entryPath);
            console.log(`[Lab] Loading consumer configuration: ${entryPath}`);
            try {
                await import(absolutePath);
            }
            catch (err) {
                console.error(`[Lab] Failed to load ${entryPath}:`, err);
                process.exit(1);
            }
        }
        const { startLabServer } = await import("../dist/lab/server.js");
        await startLabServer();
    }
    catch (err) {
        console.error("[Lab] Failed to start lab server:", err);
        process.exit(1);
    }
}
boot();
