'use strict';

var fs = require('fs');
var path = require('path');
var crypto = require('crypto');

function _interopNamespace(e) {
  if (e && e.__esModule) return e;
  var n = Object.create(null);
  if (e) {
    Object.keys(e).forEach(function (k) {
      if (k !== 'default') {
        var d = Object.getOwnPropertyDescriptor(e, k);
        Object.defineProperty(n, k, d.get ? d : {
          enumerable: true,
          get: function () { return e[k]; }
        });
      }
    });
  }
  n.default = e;
  return Object.freeze(n);
}

var fs__namespace = /*#__PURE__*/_interopNamespace(fs);
var path__namespace = /*#__PURE__*/_interopNamespace(path);

// src/mocks.ts
var MOCK_BLOCKS = [
  { id: 1, index: 1, content: "Kael woke up in the cryo-pod with no memory.", happenedAt: 2e3, isNotable: true },
  { id: 2, index: 2, content: "The cryo-fluid drained slowly, hissing against the cold floor.", happenedAt: 2050, isNotable: false },
  { id: 3, index: 3, content: "He pushed the heavy glass lid open, shivering violently.", happenedAt: 2100, isNotable: false },
  { id: 4, index: 4, content: "Emergency lights painted the cryo-bay in a dull, pulsing red.", happenedAt: 2150, isNotable: false },
  { id: 5, index: 5, content: "He checked his cybernetic left arm; the servos whined but engaged.", happenedAt: 2200, isNotable: true },
  { id: 6, index: 6, content: "A sharp pain pulsed behind his eyes, a side effect of prolonged stasis.", happenedAt: 2300, isNotable: false },
  { id: 7, index: 7, content: "He stumbled to the nearest wall console to check the ship's status.", happenedAt: 2400, isNotable: false },
  { id: 8, index: 8, content: "The console screen was cracked, displaying a cascade of error logs.", happenedAt: 2450, isNotable: false },
  { id: 9, index: 9, content: "He bypassed the hardware fault using his arm's diagnostic port.", happenedAt: 2600, isNotable: true },
  { id: 10, index: 10, content: "Main power was offline. Auxiliary reserves were at 14%.", happenedAt: 2700, isNotable: false },
  { id: 11, index: 11, content: "Kael found his environmental suit in locker 4A and suited up.", happenedAt: 3e3, isNotable: false },
  { id: 12, index: 12, content: "The suit's HUD flickered to life, syncing with his neural link.", happenedAt: 3100, isNotable: false },
  { id: 13, index: 13, content: "He unsealed the cryo-bay doors manually.", happenedAt: 3300, isNotable: false },
  { id: 14, index: 14, content: "The main corridor was devoid of gravity; he pushed off the bulkhead.", happenedAt: 3400, isNotable: false },
  { id: 15, index: 15, content: "Debris floated aimlessly\u2014shattered data pads and frozen coolant droplets.", happenedAt: 3500, isNotable: false },
  { id: 16, index: 16, content: "He navigated through the engineering deck, looking for the AI core.", happenedAt: 3800, isNotable: false },
  { id: 17, index: 17, content: "A localized gravity field caught him off guard near the reactor.", happenedAt: 4e3, isNotable: false },
  { id: 18, index: 18, content: "He landed hard, testing the durability of his prosthetic shoulder.", happenedAt: 4050, isNotable: false },
  { id: 19, index: 19, content: "Booting the secondary AI node in engineering took three attempts.", happenedAt: 4300, isNotable: true },
  { id: 20, index: 20, content: "A synthetic voice finally chimed in his earpiece.", happenedAt: 4500, isNotable: false },
  { id: 21, index: 21, content: "'Captain Kael. Diagnostics restored,' ELARA stated flatly.", happenedAt: 4600, isNotable: true },
  { id: 22, index: 22, content: "Kael asked for a ship-wide damage assessment.", happenedAt: 4700, isNotable: false },
  { id: 23, index: 23, content: "ELARA processed the request, the HUD displaying structural schematics.", happenedAt: 4800, isNotable: false },
  { id: 24, index: 24, content: "Several red zones flashed across the mid-section.", happenedAt: 4900, isNotable: false },
  { id: 25, index: 25, content: "The ship's AI, ELARA, reported a hull breach in Sector 4.", happenedAt: 5e3, isNotable: false },
  { id: 26, index: 26, content: "Sector 4 housed the primary cargo bay and atmospheric processors.", happenedAt: 5100, isNotable: false },
  { id: 27, index: 27, content: "Kael sprinted down the transit tube toward the breach.", happenedAt: 5300, isNotable: false },
  { id: 28, index: 28, content: "The bulkheads had sealed, trapping the decompression.", happenedAt: 5500, isNotable: false },
  { id: 29, index: 29, content: "He grabbed a heavy plasma welder from a maintenance cache.", happenedAt: 5800, isNotable: true },
  { id: 30, index: 30, content: "Overriding the Sector 4 blast doors required his command codes.", happenedAt: 6e3, isNotable: false },
  { id: 31, index: 31, content: "The doors cracked open, and a violent rush of air pushed past him.", happenedAt: 6100, isNotable: false },
  { id: 32, index: 32, content: "He engaged his magnetic boots, locking himself to the deck.", happenedAt: 6200, isNotable: false },
  { id: 33, index: 33, content: "The breach was small, a micro-meteorite puncture, but widening.", happenedAt: 6500, isNotable: false },
  { id: 34, index: 34, content: "Kael applied a durasteel patch and welded the edges shut.", happenedAt: 6800, isNotable: true },
  { id: 35, index: 35, content: "The whistling sound of escaping oxygen finally ceased.", happenedAt: 7e3, isNotable: false },
  { id: 36, index: 36, content: "ELARA initiated repressurization of the cargo bay.", happenedAt: 7100, isNotable: false },
  { id: 37, index: 37, content: "With the immediate threat neutralized, Kael inspected the cargo.", happenedAt: 7500, isNotable: false },
  { id: 38, index: 38, content: "Several storage containers had come loose during the impact.", happenedAt: 7600, isNotable: false },
  { id: 39, index: 39, content: "He began securing the magnetic locks on the loose crates.", happenedAt: 7800, isNotable: false },
  { id: 40, index: 40, content: "A strange harmonic vibration caught his attention.", happenedAt: 8e3, isNotable: false },
  { id: 41, index: 41, content: "He tracked the hum to a damaged, unmarked stasis pod.", happenedAt: 8100, isNotable: false },
  { id: 42, index: 42, content: "The pod's biometric scanner was dead, the glass frosted over.", happenedAt: 8200, isNotable: false },
  { id: 43, index: 43, content: "Wiping the frost away, he expected to see a crew member.", happenedAt: 8300, isNotable: false },
  { id: 44, index: 44, content: "Instead, the pod was empty save for a single object resting on the floor.", happenedAt: 8400, isNotable: false },
  { id: 45, index: 45, content: "He pried the emergency release lever, popping the pod open.", happenedAt: 8450, isNotable: false },
  { id: 46, index: 46, content: "The ambient temperature dropped sharply as he reached inside.", happenedAt: 8480, isNotable: false },
  { id: 47, index: 47, content: "His cybernetic fingers brushed against cold, unnatural geometry.", happenedAt: 8490, isNotable: false },
  { id: 48, index: 48, content: "Kael found a strange obsidian cube in the cargo bay.", happenedAt: 8500, isNotable: true },
  { id: 49, index: 49, content: "ELARA warns that the cube is emitting a Void signature.", happenedAt: 8600, isNotable: false },
  { id: 50, index: 50, content: "Kael reaches for his rebreather as the alarms scream.", happenedAt: 9e3, isNotable: false },
  { id: 51, index: 51, content: "A black mist began venting from the cube's intricate etchings.", happenedAt: 9050, isNotable: true },
  { id: 52, index: 52, content: "The ship's gravity generators flickered and died completely.", happenedAt: 9100, isNotable: false },
  { id: 53, index: 53, content: "Kael dropped the cube, but it remained suspended in the zero-g environment.", happenedAt: 9200, isNotable: false },
  { id: 54, index: 54, content: "Shadows detached from the bulkheads, slithering toward the artifact.", happenedAt: 9300, isNotable: false },
  { id: 55, index: 55, content: "ELARA initiated a localized EMP pulse to disrupt the entity.", happenedAt: 9500, isNotable: true },
  { id: 56, index: 56, content: "The pulse shattered the shadows, sending them recoiling into the vents.", happenedAt: 9600, isNotable: false },
  { id: 57, index: 57, content: "Kael sealed the cube inside a hazardous materials lockbox.", happenedAt: 9800, isNotable: false },
  { id: 58, index: 58, content: "He needed to get the ship to the surface of Kepler-186f immediately.", happenedAt: 1e4, isNotable: true },
  { id: 59, index: 59, content: "Returning to the bridge, he strapped into the pilot's chair.", happenedAt: 10200, isNotable: false },
  { id: 60, index: 60, content: "ELARA calculated an aggressive atmospheric entry vector.", happenedAt: 10500, isNotable: false },
  { id: 61, index: 61, content: "The ship shuddered violently as it hit the exosphere.", happenedAt: 10800, isNotable: false },
  { id: 62, index: 62, content: "Plasma licked the viewport screens, glowing a terrifying orange.", happenedAt: 11e3, isNotable: false },
  { id: 63, index: 63, content: "Heat shields were holding at 85% capacity.", happenedAt: 11200, isNotable: false },
  { id: 64, index: 64, content: "Kael fought the manual controls to keep the nose aligned.", happenedAt: 11500, isNotable: false },
  { id: 65, index: 65, content: "The toxic green clouds of Kepler-186f swallowed the ship.", happenedAt: 11800, isNotable: true },
  { id: 66, index: 66, content: "Turbulence threw loose items across the bridge.", happenedAt: 12e3, isNotable: false },
  { id: 67, index: 67, content: "Altimeter readouts dropped rapidly: 10,000 meters, 8,000 meters.", happenedAt: 12200, isNotable: false },
  { id: 68, index: 68, content: "He fired the retro-thrusters, straining the ship's fatigued chassis.", happenedAt: 12500, isNotable: false },
  { id: 69, index: 69, content: "The ground approached rapidly\u2014a dense jungle of alien flora.", happenedAt: 12800, isNotable: false },
  { id: 70, index: 70, content: "The landing struts deployed with a heavy, mechanical thud.", happenedAt: 13e3, isNotable: true },
  { id: 71, index: 71, content: "Impact. The ship settled into the soft, acidic soil of the planet.", happenedAt: 13200, isNotable: true },
  { id: 72, index: 72, content: "Kael let out a breath he didn't realize he was holding.", happenedAt: 13400, isNotable: false },
  { id: 73, index: 73, content: "ELARA ran surface scans; the atmosphere was confirmed lethal.", happenedAt: 13600, isNotable: false },
  { id: 74, index: 74, content: "He checked his suit's oxygen reserves: 12 hours remaining.", happenedAt: 13800, isNotable: false },
  { id: 75, index: 75, content: "He packed a field kit, ensuring his plasma rifle was fully charged.", happenedAt: 14e3, isNotable: false },
  { id: 76, index: 76, content: "He secured the lockbox containing the cube to his tactical harness.", happenedAt: 14200, isNotable: false },
  { id: 77, index: 77, content: "The airlock cycled with a hiss, exposing him to the alien world.", happenedAt: 14500, isNotable: true },
  { id: 78, index: 78, content: "Visibility was low, choked by thick, luminescent spores.", happenedAt: 14700, isNotable: false },
  { id: 79, index: 79, content: "He stepped off the ramp, his boots sinking into the violet moss.", happenedAt: 15e3, isNotable: false },
  { id: 80, index: 80, content: "The lockbox hummed softly, vibrating against his back.", happenedAt: 15200, isNotable: false },
  { id: 81, index: 81, content: "He activated his wrist-mounted pathfinder module.", happenedAt: 15500, isNotable: false },
  { id: 82, index: 82, content: "A signal beacon was detected three kilometers to the north.", happenedAt: 15800, isNotable: true },
  { id: 83, index: 83, content: "He began the trek, cutting through dense, razor-sharp foliage.", happenedAt: 16e3, isNotable: false },
  { id: 84, index: 84, content: "Strange, multi-limbed creatures scurried away from his light.", happenedAt: 16200, isNotable: false },
  { id: 85, index: 85, content: "An acidic rain began to fall, pattering aggressively against his visor.", happenedAt: 16500, isNotable: false },
  { id: 86, index: 86, content: "His suit's corrosive resistant shielding held, but energy drained faster.", happenedAt: 16800, isNotable: false },
  { id: 87, index: 87, content: "The terrain shifted from jungle to a rocky, obsidian plateau.", happenedAt: 17e3, isNotable: false },
  { id: 88, index: 88, content: "He noticed the rock formations mirrored the etchings on the cube.", happenedAt: 17200, isNotable: true },
  { id: 89, index: 89, content: "A low rumble vibrated through the ground beneath him.", happenedAt: 17500, isNotable: false },
  { id: 90, index: 90, content: "The beacon signal grew stronger, leading him to a massive crater.", happenedAt: 17800, isNotable: false },
  { id: 91, index: 91, content: "At the center of the crater lay an ancient, derelict structure.", happenedAt: 18e3, isNotable: true },
  { id: 92, index: 92, content: "The structure was emitting a harmonic frequency matching the cube's.", happenedAt: 18200, isNotable: false },
  { id: 93, index: 93, content: "Shadows detached from the ruins, swirling in the green mist.", happenedAt: 18500, isNotable: false },
  { id: 94, index: 94, content: "Void-Eaters materialised, blocking his path to the entrance.", happenedAt: 18800, isNotable: true },
  { id: 95, index: 95, content: "Kael leveled his plasma rifle, disabling the safety.", happenedAt: 19e3, isNotable: false },
  { id: 96, index: 96, content: "He fired a concentrated blast, incinerating the closest entity.", happenedAt: 19200, isNotable: false },
  { id: 97, index: 97, content: "The lockbox unlatched itself, the cube hovering into the open air.", happenedAt: 19500, isNotable: true },
  { id: 98, index: 98, content: "The cube projected a beam of dark light towards the ruins.", happenedAt: 19700, isNotable: false },
  { id: 99, index: 99, content: "The doors of the ancient structure ground open, revealing an abyss.", happenedAt: 19900, isNotable: true },
  { id: 100, index: 100, content: "Kael took a deep breath of recycled air and stepped inside.", happenedAt: 2e4, isNotable: true }
];
var MOCK_LORE = [
  { id: "lore-1", content: "The atmosphere on Kepler-186f is toxic without a rebreather.", happenedAt: 1e3, isActive: true },
  { id: "lore-2", content: "Captain Kael has a cybernetic left arm from the Sol Wars.", happenedAt: 1005, isActive: true },
  { id: "lore-3", content: "The 'Void-Eaters' are attracted to high-energy signatures.", happenedAt: 1010, isActive: true },
  { id: "lore-4", content: "The USS Daedalus is a deep-space freighter retrofitted for reconnaissance.", happenedAt: 1015, isActive: true },
  { id: "lore-5", content: "ELARA stands for Electronic Logistics and Reconnaissance Automaton.", happenedAt: 1020, isActive: true },
  { id: "lore-6", content: "Obsidian Cubes are Class-Omega artifacts, prohibited by the Terran Coalition.", happenedAt: 1025, isActive: true },
  { id: "lore-7", content: "The Sol Wars ended in 2098 after the destruction of the Lunar colonies.", happenedAt: 1030, isActive: false },
  { id: "lore-8", content: "Kepler-186f's flora secretes a highly acidic neuro-sap.", happenedAt: 1035, isActive: true },
  { id: "lore-9", content: "Cryo-stasis memory fragmentation is a common but temporary condition.", happenedAt: 1040, isActive: true },
  { id: "lore-10", content: "Void signatures disrupt localized gravitational fields and electronics.", happenedAt: 1045, isActive: true },
  { id: "lore-11", content: "Kael's neural link allows him to interface directly with Terran tech.", happenedAt: 1050, isActive: true },
  { id: "lore-12", content: "Sector 4 of the Daedalus houses restricted atmospheric processors.", happenedAt: 1055, isActive: false },
  { id: "lore-13", content: "Standard plasma welders operate at 15,000 degrees Kelvin.", happenedAt: 1060, isActive: true },
  { id: "lore-14", content: "A localized EMP is the only known deterrent for low-tier Void entities.", happenedAt: 1065, isActive: true },
  { id: "lore-15", content: "Terran Hazard Lockboxes are lined with quantum-sealed lead.", happenedAt: 1070, isActive: true },
  { id: "lore-16", content: "The Ruins of Kepler were discovered by probes but never explored by humans.", happenedAt: 1075, isActive: true },
  { id: "lore-17", content: "Magnetic boots are standard issue for zero-gravity EVA and internal breaches.", happenedAt: 1080, isActive: true },
  { id: "lore-18", content: "Dark light projection is a theoretical physics phenomenon linked to the Void.", happenedAt: 1085, isActive: true },
  { id: "lore-19", content: "The Daedalus's heat shields were past their recommended maintenance cycle.", happenedAt: 1090, isActive: false },
  { id: "lore-20", content: "Kael lost his original arm during the Siege of Mare Crisium.", happenedAt: 1095, isActive: true }
];

// src/provider.ts
function getBrowserStorage(key, fallback) {
  if (typeof window === "undefined" || !window.localStorage) {
    return fallback;
  }
  try {
    const stored = localStorage.getItem(key);
    return stored ? JSON.parse(stored) : fallback;
  } catch {
    return fallback;
  }
}
function setBrowserStorage(key, data) {
  if (typeof window === "undefined" || !window.localStorage) {
    return;
  }
  try {
    localStorage.setItem(key, JSON.stringify(data));
  } catch {
  }
}
var InMemoryNarrativeProvider = class {
  blocks = [];
  lore = [];
  storageKeyBlocks;
  storageKeyLore;
  useBrowserStorage;
  constructor(initialBlocks = MOCK_BLOCKS, initialLore = MOCK_LORE, options) {
    this.useBrowserStorage = options?.useBrowserStorage ?? false;
    const channelId = options?.channelId ?? "default";
    this.storageKeyBlocks = `narrative_blocks_${channelId}`;
    this.storageKeyLore = `narrative_lore_${channelId}`;
    if (this.useBrowserStorage) {
      this.blocks = getBrowserStorage(this.storageKeyBlocks, initialBlocks);
      this.lore = getBrowserStorage(this.storageKeyLore, initialLore);
    } else {
      this.blocks = initialBlocks;
      this.lore = initialLore;
    }
  }
  getProviderType() {
    return this.useBrowserStorage ? "browser-storage" : "in-memory";
  }
  async getLoreAtoms(_channelId) {
    return this.lore.filter((l) => l.isActive !== false);
  }
  async getNotableEvents(_channelId) {
    return this.blocks.filter((b) => b.isNotable);
  }
  async getBlocksByIndices(_channelId, indices) {
    return this.blocks.filter((b) => indices.includes(Number(b.id)));
  }
  async getBlockCount(_channelId) {
    return this.blocks.length;
  }
  async getHybridSearchCandidates(_channelId, query, limit) {
    return this.blocks.filter((b) => b.content.toLowerCase().includes(query.toLowerCase())).slice(0, limit).map((b) => ({
      block: b,
      scoreVectorDense: 0.8,
      // Mock high-relevance
      scoreKeywordSparse: 0.8
    }));
  }
  async addBlock(_channelId, block) {
    this.blocks.push(block);
    if (this.useBrowserStorage) {
      setBrowserStorage(this.storageKeyBlocks, this.blocks);
    }
  }
};

// src/sequence.ts
var RAG_DIVISIONS = 5;
var RAG_MIN_BLOCKS = 3;
var calculateHarmonicConstant = (n) => {
  let sum = 0;
  for (let i = 1; i <= n; i++) {
    sum += 1 / i;
  }
  return sum;
};
var generateReciprocalSequence = (targetN, divisions) => {
  if (targetN <= 1 || divisions <= 0) return [1];
  const k = calculateHarmonicConstant(divisions);
  const scale = (targetN - 1) / k;
  const sequence = [1];
  for (let i = 1; i <= divisions; i++) {
    const lastValue = sequence[i - 1];
    const jump = scale / i;
    sequence.push(Number((lastValue + jump).toFixed(2)));
  }
  return sequence;
};
var sequenceToBlockIndices = (sequence) => {
  const rounded = sequence.map((v) => Math.max(1, Math.round(v)));
  const unique = Array.from(new Set(rounded));
  return unique.sort((a, b) => a - b);
};
function loggerNarrativeTrace(traceObject) {
  const isTracingEnabled = process.env.NODE_ENV === "development" || process.env.NARRATIVE_VERBOSE === "true";
  if (!isTracingEnabled) return;
  try {
    const traceDir = path__namespace.join(process.cwd(), ".traces");
    if (!fs__namespace.existsSync(traceDir)) {
      fs__namespace.mkdirSync(traceDir, { recursive: true });
    }
    const filepath = path__namespace.join(traceDir, "narrative_ledger.jsonl");
    const traceContent = JSON.stringify(traceObject) + "\n";
    fs__namespace.appendFileSync(filepath, traceContent, "utf-8");
  } catch (err) {
    console.warn("[Trace] Failed to write trace file:", err);
  }
}

// src/engine.ts
var LIMIT_HYBRID_TOP = 3;
var verboseLog = {
  group: (label, ...args) => {
    if (process.env.NARRATIVE_VERBOSE === "true" || process.env.NODE_ENV === "development") {
      console.group(`[NarrativeEngine] ${label}`);
      args.forEach((arg) => {
        if (typeof arg === "object") {
          console.log(JSON.stringify(arg, null, 2));
        } else {
          console.log(arg);
        }
      });
      console.groupEnd();
    }
  },
  info: (label, ...args) => {
    if (process.env.NARRATIVE_VERBOSE === "true" || process.env.NODE_ENV === "development") {
      console.info(`[NarrativeEngine] \u2139\uFE0F ${label}`, ...args);
    }
  },
  debug: (label, ...args) => {
    if (process.env.NARRATIVE_VERBOSE === "true" || process.env.NODE_ENV === "development") {
      console.debug(`[NarrativeEngine] \u{1F50D} ${label}`, ...args);
    }
  },
  warn: (label, ...args) => {
    if (process.env.NARRATIVE_VERBOSE === "true" || process.env.NODE_ENV === "development") {
      console.warn(`[NarrativeEngine] \u26A0\uFE0F ${label}`, ...args);
    }
  },
  success: (label, ...args) => {
    if (process.env.NARRATIVE_VERBOSE === "true" || process.env.NODE_ENV === "development") {
      console.log(`[NarrativeEngine] \u2705 ${label}`, ...args);
    }
  },
  phase: (phase, message, data) => {
    if (process.env.NARRATIVE_VERBOSE === "true" || process.env.NODE_ENV === "development") {
      const divider = "\u2500".repeat(50);
      console.log(`
${divider}`);
      console.log(`\u{1F680} PHASE: ${phase}`);
      console.log(`   ${message}`);
      if (data !== void 0) {
        if (typeof data === "object") {
          console.log(JSON.stringify(data, null, 4).split("\n").map((l) => `   ${l}`).join("\n"));
        } else {
          console.log(`   ${data}`);
        }
      }
      console.log(divider);
    }
  },
  table: (label, data) => {
    if (process.env.NARRATIVE_VERBOSE === "true" || process.env.NODE_ENV === "development") {
      console.log(`[NarrativeEngine] \u{1F4CA} ${label}`);
      console.table(data);
    }
  }
};
var DEFAULT_LAB_CONFIG = {
  saliencyThreshold: 0.65,
  weightDense: 0.7,
  significanceCoef: 1.5,
  temporalPhrasing: true,
  maxLoreAtoms: 20,
  timestamp: (/* @__PURE__ */ new Date()).toISOString()
};
var NarrativeEngine = class {
  constructor(provider = new InMemoryNarrativeProvider()) {
    this.provider = provider;
  }
  provider;
  labConfig = { ...DEFAULT_LAB_CONFIG };
  setLabConfig(config) {
    this.labConfig = {
      saliencyThreshold: config.saliencyThreshold ?? DEFAULT_LAB_CONFIG.saliencyThreshold,
      weightDense: config.weightDense ?? DEFAULT_LAB_CONFIG.weightDense,
      significanceCoef: config.significanceCoef ?? DEFAULT_LAB_CONFIG.significanceCoef,
      temporalPhrasing: config.temporalPhrasing ?? DEFAULT_LAB_CONFIG.temporalPhrasing,
      maxLoreAtoms: config.maxLoreAtoms ?? DEFAULT_LAB_CONFIG.maxLoreAtoms,
      timestamp: config.timestamp ?? DEFAULT_LAB_CONFIG.timestamp
    };
  }
  getLabConfig() {
    return { ...this.labConfig };
  }
  async generateContext(channelId, inputQuery) {
    verboseLog.phase("CONTEXT_GENERATION", `Starting for channel="${channelId}", query="${inputQuery.substring(0, 50)}${inputQuery.length > 50 ? "..." : ""}"`);
    const trace = {
      timestamp: (/* @__PURE__ */ new Date()).toISOString(),
      channelId,
      inputQuery,
      labConfig: { ...this.labConfig },
      providerType: this.provider.getProviderType?.() ?? "custom",
      phases: {}
    };
    try {
      verboseLog.phase("HARVEST", "Fetching blocks, lore atoms, and hybrid search candidates");
      verboseLog.debug("LabConfig", this.labConfig);
      const totalBlockCount = await this.provider.getBlockCount(channelId);
      verboseLog.info("BlockCount", `totalBlockCount=${totalBlockCount}`);
      const loreAtomsRaw = await this.provider.getLoreAtoms(channelId);
      verboseLog.info("LoreAtomsRaw", `found=${loreAtomsRaw.length} atoms`);
      const loreAtoms = loreAtomsRaw.sort((a, b) => b.happenedAt - a.happenedAt).slice(0, this.labConfig.maxLoreAtoms);
      verboseLog.info("LoreAtomsCapped", {
        raw: loreAtomsRaw.length,
        active: loreAtoms.length,
        maxAllowed: this.labConfig.maxLoreAtoms,
        oldestIncluded: loreAtoms.length > 0 ? loreAtoms[loreAtoms.length - 1].happenedAt : null,
        newestIncluded: loreAtoms.length > 0 ? loreAtoms[0].happenedAt : null
      });
      const candidatesHybrid = await this.provider.getHybridSearchCandidates(channelId, inputQuery, 20);
      verboseLog.info("HybridCandidates", `found=${candidatesHybrid.length} candidates`);
      if (candidatesHybrid.length > 0) {
        verboseLog.table("HybridCandidates.Sample", candidatesHybrid.slice(0, 5).map((c) => ({
          id: c.block.id,
          dense: c.scoreVectorDense.toFixed(3),
          sparse: c.scoreKeywordSparse.toFixed(3),
          notable: c.block.isNotable,
          snippet: c.block.content
        })));
      }
      let blocksHistorical = [];
      let blockSequenceIntervals = [];
      if (totalBlockCount >= RAG_MIN_BLOCKS) {
        const seq = generateReciprocalSequence(totalBlockCount, RAG_DIVISIONS);
        const indices = sequenceToBlockIndices(seq);
        blockSequenceIntervals = indices;
        verboseLog.debug("ReciprocalSkeleton", {
          totalBlocks: totalBlockCount,
          divisions: RAG_DIVISIONS,
          rawSequence: seq,
          blockIndices: indices
        });
        blocksHistorical = await this.provider.getBlocksByIndices(channelId, indices);
        verboseLog.info("HistoricalBlocks", `retrieved=${blocksHistorical.length} blocks via reciprocal skeleton`);
      } else {
        verboseLog.warn("ReciprocalSkeleton", `Skipped - blockCount(${totalBlockCount}) < RAG_MIN_BLOCKS(${RAG_MIN_BLOCKS})`);
      }
      verboseLog.phase("FUSION", "Applying weighted fusion and significance boost");
      const weightSparse = 1 - this.labConfig.weightDense;
      verboseLog.debug("FusionWeights", {
        weightDense: this.labConfig.weightDense,
        weightSparse: weightSparse.toFixed(3),
        formula: `scoreRaw = (dense * ${this.labConfig.weightDense}) + (sparse * ${weightSparse.toFixed(3)})`
      });
      const scoredCandidates = candidatesHybrid.map((candidate) => {
        const scoreRawFused = candidate.scoreVectorDense * this.labConfig.weightDense + candidate.scoreKeywordSparse * weightSparse;
        const scoreFinalFused = candidate.block.isNotable ? scoreRawFused * this.labConfig.significanceCoef : scoreRawFused;
        verboseLog.debug("ScoredCandidate", {
          id: candidate.block.id,
          scoreDense: candidate.scoreVectorDense.toFixed(3),
          scoreSparse: candidate.scoreKeywordSparse.toFixed(3),
          scoreRaw: scoreRawFused.toFixed(3),
          isNotable: candidate.block.isNotable,
          significanceCoef: candidate.block.isNotable ? this.labConfig.significanceCoef : 1,
          scoreFinal: scoreFinalFused.toFixed(3)
        });
        return {
          ...candidate,
          scoreRawFused,
          scoreFinalFused
        };
      });
      verboseLog.table("AllScoredCandidates", scoredCandidates.map((c) => ({
        id: c.block.id,
        raw: c.scoreRawFused.toFixed(3),
        final: c.scoreFinalFused.toFixed(3),
        notable: c.block.isNotable
      })));
      verboseLog.phase("SALIENCY_GATE", `Threshold=${this.labConfig.saliencyThreshold}, TopK=${LIMIT_HYBRID_TOP}`);
      const filteredBySaliency = scoredCandidates.filter((c) => c.scoreFinalFused >= this.labConfig.saliencyThreshold);
      verboseLog.info("SaliencyFilter", {
        total: scoredCandidates.length,
        passed: filteredBySaliency.length,
        threshold: this.labConfig.saliencyThreshold,
        evicted: scoredCandidates.length - filteredBySaliency.length
      });
      const evicted = scoredCandidates.filter((c) => c.scoreFinalFused < this.labConfig.saliencyThreshold);
      if (evicted.length > 0) {
        verboseLog.warn("EvictedCandidates", evicted.map((e) => ({
          id: e.block.id,
          score: e.scoreFinalFused.toFixed(3),
          belowBy: (this.labConfig.saliencyThreshold - e.scoreFinalFused).toFixed(3)
        })));
      }
      const survivors = scoredCandidates.filter((c) => c.scoreFinalFused >= this.labConfig.saliencyThreshold).sort(
        (a, b) => b.scoreFinalFused - a.scoreFinalFused || b.block.happenedAt - a.block.happenedAt
      ).slice(0, LIMIT_HYBRID_TOP);
      verboseLog.success("Survivors", {
        count: survivors.length,
        maxAllowed: LIMIT_HYBRID_TOP,
        ids: survivors.map((s) => s.block.id),
        topScore: survivors[0]?.scoreFinalFused.toFixed(3) ?? null
      });
      if (survivors.length > 0) {
        verboseLog.table("SurvivorRanking", survivors.map((s, i) => ({
          rank: i + 1,
          id: s.block.id,
          score: s.scoreFinalFused.toFixed(3),
          temporal: s.block.happenedAt,
          notable: s.block.isNotable
        })));
      }
      verboseLog.phase("TIMELINE", "Merging historical skeleton with relevance survivors");
      const blocksChrono = this.mergeAndSortChronologically(blocksHistorical, survivors);
      verboseLog.info("TimelineMerge", {
        historicalBlocks: blocksHistorical.length,
        survivorBlocks: survivors.length,
        totalUnique: blocksChrono.length,
        deduped: blocksHistorical.length + survivors.length - blocksChrono.length
      });
      verboseLog.table("FinalTimeline", blocksChrono.map((b, i) => ({
        position: i + 1,
        id: b.id,
        temporalOffset: `${totalBlockCount - (typeof b.index === "number" ? b.index : 0) + 1} blocks ago`,
        notable: b.isNotable,
        content: b.content
      })));
      verboseLog.phase("PROSE", "Composing final context prompt");
      const currentBlockCount = (blocksChrono.at(-1)?.index ?? 0) + 1;
      const finalizedPrompt = this.composeProse(
        blocksChrono,
        loreAtoms,
        inputQuery,
        currentBlockCount
      );
      verboseLog.debug("ProseStats", {
        loreSectionChars: loreAtoms.length > 0 ? loreAtoms.map((l) => l.content).join(" ").length : 0,
        blockSectionCount: blocksChrono.length,
        totalPromptChars: finalizedPrompt.length,
        temporalPhrasing: this.labConfig.temporalPhrasing
      });
      trace.phases = {
        harvest: {
          totalBlockCount,
          loreCount: loreAtoms.length,
          candidateCount: candidatesHybrid.length,
          loreAtoms: loreAtoms.map((l) => ({ id: l.id, content: l.content, happenedAt: l.happenedAt })),
          searchCandidates: candidatesHybrid.map((c) => ({
            id: c.block.id,
            content: c.block.content,
            scoreVectorDense: c.scoreVectorDense,
            scoreKeywordSparse: c.scoreKeywordSparse,
            isNotable: c.block.isNotable
          })),
          immediateContext: inputQuery
        },
        fusion: scoredCandidates.map((c) => ({
          id: c.block.id,
          scoreFinal: c.scoreFinalFused,
          scoreRaw: c.scoreRawFused,
          isNotable: c.block.isNotable
        })),
        saliency: {
          threshold: this.labConfig.saliencyThreshold,
          passed: survivors.map((s) => s.block.id),
          evicted: evicted.map((e) => e.block.id),
          filteredCount: filteredBySaliency.length,
          totalCandidates: scoredCandidates.length
        },
        timeline: {
          merged: blocksChrono.map((b) => ({ id: b.id, index: b.index, content: b.content })),
          fromHistorical: blocksHistorical.map((b) => b.id),
          fromSurvivors: survivors.map((s) => s.block.id),
          blockSequenceIntervals,
          currentBlockCount
        },
        prose: { promptLength: finalizedPrompt.length, loreAtoms: loreAtoms.length, blockCount: blocksChrono.length }
      };
      trace.finalizedPrompt = finalizedPrompt;
      loggerNarrativeTrace(trace);
      verboseLog.success("ContextGenerated", `Prompt length: ${finalizedPrompt.length} chars`);
      return finalizedPrompt;
    } catch (err) {
      verboseLog.warn("Error", err instanceof Error ? err.message : String(err));
      trace.error = err instanceof Error ? err.message : String(err);
      loggerNarrativeTrace(trace);
      throw err;
    }
  }
  mergeAndSortChronologically(blocksHistorical, candidatesSurvivor) {
    verboseLog.debug("MergeStart", {
      historical: blocksHistorical.length,
      survivors: candidatesSurvivor.length
    });
    const merged = /* @__PURE__ */ new Map();
    for (const block of blocksHistorical) {
      merged.set(block.id, block);
    }
    const dupsFromSurvivors = [];
    for (const candidate of candidatesSurvivor) {
      if (merged.has(candidate.block.id)) {
        dupsFromSurvivors.push(String(candidate.block.id));
      }
      merged.set(candidate.block.id, candidate.block);
    }
    if (dupsFromSurvivors.length > 0) {
      verboseLog.info("DuplicateBlocks", `Survivors overlapping with historical: ${dupsFromSurvivors.join(", ")}`);
    }
    const result = Array.from(merged.values()).sort((a, b) => a.happenedAt - b.happenedAt);
    verboseLog.debug("MergeComplete", {
      inputHistorical: blocksHistorical.length,
      inputSurvivors: candidatesSurvivor.length,
      duplicates: dupsFromSurvivors.length,
      output: result.length
    });
    return result;
  }
  composeProse(blocksChrono, loreAtoms, immediateContext, currentBlockCount) {
    const loreSection = loreAtoms.length > 0 ? loreAtoms.map((l) => l.content).join(" ") : "";
    verboseLog.debug("LoreSection", {
      hasLore: loreAtoms.length > 0,
      atomCount: loreAtoms.length,
      totalChars: loreSection.length,
      preview: loreSection.substring(0, 100) + (loreSection.length > 100 ? "..." : "")
    });
    const blockSections = blocksChrono.map((block) => {
      if (this.labConfig.temporalPhrasing && typeof block.index === "number") {
        const offsetHistorical = currentBlockCount - block.index + 1;
        const unit = offsetHistorical === 1 ? "storyblock" : "storyblocks";
        return `${offsetHistorical} ${unit} ago: ${block.content}`;
      }
      return `Entry ${block.id}: ${block.content}`;
    });
    verboseLog.debug("BlockSections", {
      blockCount: blocksChrono.length,
      temporalPhrasing: this.labConfig.temporalPhrasing,
      samples: blockSections.slice(0, 2).map((s) => s.substring(0, 60) + "...")
    });
    const parts = [];
    if (loreSection) {
      parts.push(`Essential facts of the story: ${loreSection}`);
    }
    if (blockSections.length > 0) {
      parts.push(blockSections.join("\n"));
    }
    parts.push(`${immediateContext}`);
    const result = parts.join("\n");
    verboseLog.debug("ProseComplete", {
      sections: parts.length,
      totalChars: result.length
    });
    return result;
  }
};

// src/utils.ts
function normalizeScore(value, min, max) {
  if (max === min) return 0;
  const normalized = (value - min) / (max - min);
  return Math.max(0, Math.min(1, normalized));
}
function validateProviderShape(provider) {
  const requiredMethods = [
    "getLoreAtoms",
    "getNotableEvents",
    "getBlocksByIndices",
    "getHybridSearchCandidates",
    "getBlockCount"
  ];
  const missing = requiredMethods.filter(
    (method) => typeof provider[method] !== "function"
  );
  if (missing.length > 0) {
    console.error(
      `[NarrativeEngine] Invalid Provider: Missing methods [${missing.join(", ")}]`
    );
    return false;
  }
  return true;
}
var GLOBAL_KEY = /* @__PURE__ */ Symbol.for("narrative.engine.registry");
var LAB_TOKEN = /* @__PURE__ */ Symbol.for("narrative.lab.token");
if (!global[LAB_TOKEN]) {
  global[LAB_TOKEN] = process.env.LAB_SECRET || `lab_${crypto.randomUUID()}`;
}
var SESSION_SECRET = global[LAB_TOKEN];
function configureLabEngine(engine) {
  global[GLOBAL_KEY] = engine;
}
function getActiveEngine() {
  return global[GLOBAL_KEY];
}

exports.GLOBAL_KEY = GLOBAL_KEY;
exports.InMemoryNarrativeProvider = InMemoryNarrativeProvider;
exports.LAB_TOKEN = LAB_TOKEN;
exports.NarrativeEngine = NarrativeEngine;
exports.SESSION_SECRET = SESSION_SECRET;
exports.configureLabEngine = configureLabEngine;
exports.getActiveEngine = getActiveEngine;
exports.normalizeScore = normalizeScore;
exports.validateProviderShape = validateProviderShape;
