// lab/server.ts
import express from "express";
import * as fs from "fs";
import * as path2 from "path";
import { fileURLToPath } from "url";
import { dirname, resolve } from "path";
import { randomUUID } from "crypto";
import cors from "cors";
import { createServer } from "http";
import { NarrativeEngine, InMemoryNarrativeProvider, GLOBAL_KEY, LAB_TOKEN, SESSION_SECRET } from "narrative-engine";

// lab/logger.ts
import path from "path";
var verboseLog = {
  lab: (...args) => {
    if (process.env.NARRATIVE_VERBOSE === "true" || process.env.NODE_ENV === "development") {
      console.log(`[NarrativeLab]`, ...args);
    }
  },
  request: (method, path3, details) => {
    if (process.env.NARRATIVE_VERBOSE === "true" || process.env.NODE_ENV === "development") {
      console.log(`[NarrativeLab] \u2192 ${method} ${path3}`, details ?? "");
    }
  },
  response: (method, path3, status, duration) => {
    if (process.env.NARRATIVE_VERBOSE === "true" || process.env.NODE_ENV === "development") {
      const durationStr = duration ? ` (${duration}ms)` : "";
      console.log(`[NarrativeLab] \u2190 ${status} ${method} ${path3}${durationStr}`);
    }
  },
  security: (event, details) => {
    if (process.env.NARRATIVE_VERBOSE === "true" || process.env.NODE_ENV === "development") {
      console.warn(`[NarrativeLab/Security] ${event}:`, details);
    }
  },
  config: (label, config) => {
    if (process.env.NARRATIVE_VERBOSE === "true" || process.env.NODE_ENV === "development") {
      console.log(`[NarrativeLab] Config [${label}]:`, JSON.stringify(config, null, 2));
    }
  },
  trace: (action, count) => {
    if (process.env.NARRATIVE_VERBOSE === "true" || process.env.NODE_ENV === "development") {
      console.log(`[NarrativeLab] Trace [${action}]:`, count !== void 0 ? `${count} entries` : "");
    }
  }
};
var ledgerPath = path.join(process.cwd(), ".traces", "narrative_ledger.jsonl");

// lab/server.ts
var __filename = fileURLToPath(import.meta.url);
var __dirname = dirname(__filename);
function securityGate(req, res, next) {
  const remoteAddress = req.socket.remoteAddress;
  const isLocal = remoteAddress === "127.0.0.1" || remoteAddress === "::1" || remoteAddress === "::ffff:127.0.0.1";
  if (!isLocal) {
    verboseLog.security("BLOCKED_EXTERNAL", { address: remoteAddress, path: req.path });
    return res.status(403).json({ error: "Access restricted to local loopback." });
  }
  const authHeader = req.headers["authorization"];
  if (authHeader !== `Bearer ${SESSION_SECRET}`) {
    verboseLog.security("BLOCKED_INVALID_TOKEN", {
      hasHeader: !!authHeader,
      path: req.path,
      remoteAddress
    });
    return res.status(401).json({ error: "Invalid or missing Narrative-Lab-Token." });
  }
  verboseLog.security("ALLOWED", { path: req.path, remoteAddress });
  next();
}
function getActiveEngine() {
  const existing = global[GLOBAL_KEY];
  if (existing) {
    const providerType = existing["provider"]?.getProviderType?.() ?? "unknown";
    console.log("[getActiveEngine] Found existing engine in registry, provider type:", providerType);
    return existing;
  }
  console.log("[getActiveEngine] No engine in registry - GLOBAL_KEY exists:", GLOBAL_KEY.description);
  console.log("[getActiveEngine] global[GLOBAL_KEY] is:", global[GLOBAL_KEY]);
  verboseLog.lab("No engine in registry, creating new InMemoryNarrativeProvider with browser storage");
  const channelId = process.env.LAB_CHANNEL_ID || "lab-default";
  const provider = new InMemoryNarrativeProvider(void 0, void 0, {
    useBrowserStorage: true,
    channelId
  });
  return new NarrativeEngine(provider);
}
async function startLabServer(port = 5002) {
  const app = express();
  const server = createServer(app);
  if (!global[LAB_TOKEN]) {
    global[LAB_TOKEN] = `lab_${randomUUID().slice(0, 8)}`;
  }
  app.use(cors({
    origin: [
      "http://127.0.0.1:5173",
      "http://127.0.0.1:5002"
    ],
    methods: ["GET", "POST", "OPTIONS"],
    allowedHeaders: ["Content-Type", "Authorization"]
  }));
  app.use(express.json());
  app.use("/__narrative_lab", securityGate);
  const engine = getActiveEngine();
  verboseLog.lab("Lab server initialized with engine");
  app.get("/__narrative_lab/config", (req, res) => {
    verboseLog.request("GET", "/config");
    const startTime = Date.now();
    const config = engine.getLabConfig();
    verboseLog.config("Current", config);
    verboseLog.response("GET", "/config", 200, Date.now() - startTime);
    res.json({ config });
  });
  app.post("/__narrative_lab/generate", async (req, res) => {
    const startTime = Date.now();
    verboseLog.request("POST", "/generate", req.body);
    try {
      const { channelId, query, config, newBlock } = req.body;
      if (config) {
        verboseLog.lab("Updating engine config:", config);
        engine.setLabConfig(config);
      }
      let blockSaved = false;
      if (newBlock && newBlock.content) {
        const provider2 = engine["provider"];
        if (provider2 && typeof provider2.addBlock === "function") {
          const currentBlockCount = await provider2.getBlockCount(channelId || "lab-default");
          const block = {
            id: currentBlockCount + 1,
            index: currentBlockCount + 1,
            content: newBlock.content,
            happenedAt: Date.now(),
            isNotable: newBlock.isNotable ?? false
          };
          await provider2.addBlock(channelId || "lab-default", block);
          verboseLog.lab("Block added to storage", { blockId: block.id, content: block.content.substring(0, 30) });
          blockSaved = true;
        } else {
          verboseLog.lab("Provider does not support addBlock - simulation block not persisted", {
            providerType: provider2?.getProviderType?.() ?? "unknown"
          });
        }
      }
      const result = await engine.generateContext(channelId || "lab-default", query || "");
      const provider = engine.provider;
      verboseLog.lab("Context generated", {
        channelId: channelId || "lab-default",
        queryLength: (query || "").length,
        contextLength: result.length,
        providerType: provider?.getProviderType?.() ?? "custom"
      });
      const isTracingEnabled = process.env.NODE_ENV === "development" || process.env.NARRATIVE_VERBOSE === "true";
      verboseLog.response("POST", "/generate", 200, Date.now() - startTime);
      res.json({
        channelId,
        context: result,
        config: engine.getLabConfig(),
        providerType: provider?.getProviderType?.() ?? "custom",
        traceStored: isTracingEnabled,
        blockSaved
      });
    } catch (err) {
      verboseLog.lab("Generation failed:", err instanceof Error ? err.message : String(err));
      verboseLog.response("POST", "/generate", 500, Date.now() - startTime);
      res.status(500).json({ error: err instanceof Error ? err.message : "Generation failed" });
    }
  });
  app.get("/__narrative_lab/traces", (req, res) => {
    const startTime = Date.now();
    verboseLog.request("GET", "/traces");
    try {
      let fileContentRaw = "";
      try {
        fileContentRaw = fs.readFileSync(ledgerPath, "utf-8");
      } catch (readError) {
        if (readError.code === "ENOENT") {
          verboseLog.trace("Read", 0);
          verboseLog.response("GET", "/traces", 200, Date.now() - startTime);
          return res.json({ traces: [] });
        }
        console.error("[NarrativeLab] Trace read contention:", readError);
        throw readError;
      }
      const ledgerLines = fileContentRaw.split("\n").filter((line) => line.trim() !== "");
      const parsedTraces = ledgerLines.map((line) => JSON.parse(line));
      console.log("[TracesEndpoint] Returning", parsedTraces.length, "traces");
      if (parsedTraces.length > 0) {
        const latest = parsedTraces[parsedTraces.length - 1];
        console.log("[TracesEndpoint] Latest trace providerType:", latest.providerType, "timestamp:", latest.timestamp);
      }
      verboseLog.trace("Read", parsedTraces.length);
      verboseLog.response("GET", "/traces", 200, Date.now() - startTime);
      res.json({ traces: parsedTraces });
    } catch (err) {
      console.error("[NarrativeLab] Failed to parse narrative ledger:", err);
      verboseLog.response("GET", "/traces", 500, Date.now() - startTime);
      res.status(500).json({ error: "Failed to read traces due to I/O lock or corruption" });
    }
  });
  app.delete("/__narrative_lab/traces", (req, res) => {
    const startTime = Date.now();
    verboseLog.request("DELETE", "/traces");
    try {
      if (fs.existsSync(ledgerPath)) {
        fs.unlinkSync(ledgerPath);
        verboseLog.trace("Cleared");
      } else {
        verboseLog.trace("Cleared (no file existed)");
      }
      verboseLog.response("DELETE", "/traces", 200, Date.now() - startTime);
      res.json({ status: "ok", message: "Ledger cleared" });
    } catch (err) {
      verboseLog.lab("Failed to clear traces:", err);
      verboseLog.response("DELETE", "/traces", 500, Date.now() - startTime);
      res.status(500).json({ error: "Failed to clear traces" });
    }
  });
  const distPath = resolve(__dirname, "..", "dist");
  if (process.env.NODE_ENV === "development") {
    verboseLog.lab("Development mode - serve static files from dist/lab or use vite dev server");
  }
  verboseLog.lab("Production mode: serving static files");
  app.use(express.static(distPath));
  app.get("/{*splat}", (req, res) => res.sendFile(path2.join(distPath, "index.html")));
  server.listen(port, "127.0.0.1", () => {
    console.log(`
\u2554\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2557`);
    console.log(`\u2551          NarrativeEngine Lab Started                   \u2551`);
    console.log(`\u2560\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2563`);
    console.log(`\u2551  URL:      http://127.0.0.1:${port}                       `);
    console.log(`\u2551  Token:    ${SESSION_SECRET}`);
    console.log(`\u2551  Auth:     Authorization: Bearer ${SESSION_SECRET}`);
    console.log(`\u2551  Verbose:  NARRATIVE_VERBOSE=true`);
    console.log(`\u255A\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u2550\u255D
`);
    verboseLog.lab("Server listening on port", port);
  });
}
export {
  getActiveEngine,
  securityGate,
  startLabServer
};
