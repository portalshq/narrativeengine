import fs from 'node:fs'
import path from 'node:path'
import {fileURLToPath} from 'node:url'

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..')
const sitePath = path.join(repoRoot, 'site', 'index.html')

function read(relativePath) {
  return fs.readFileSync(path.join(repoRoot, relativePath), 'utf8')
}

function firstCodeBlockAfter(markdown, heading) {
  const start = markdown.indexOf(heading)
  if (start === -1) throw new Error(`Missing authored heading: ${heading}`)
  const match = markdown.slice(start + heading.length).match(/```[^\n]*\n([\s\S]*?)```/)
  if (!match) throw new Error(`Missing authored code block after: ${heading}`)
  return match[1].trim()
}

function firstParagraphAfter(markdown, heading) {
  const start = markdown.indexOf(heading)
  if (start === -1) throw new Error(`Missing authored heading: ${heading}`)
  const afterHeading = markdown.slice(start + heading.length).replace(/^[^\n]*\n/, '')
  const paragraph = afterHeading
    .split(/\n\s*\n/)
    .map((part) => part.trim())
    .find((part) => part && !part.startsWith('#') && !part.startsWith('```'))
  if (!paragraph) throw new Error(`Missing authored paragraph after: ${heading}`)
  return paragraph.replaceAll('\n', ' ')
}

function escapeHtml(value) {
  return value
    .replaceAll('&', '&amp;')
    .replaceAll('<', '&lt;')
    .replaceAll('>', '&gt;')
    .replaceAll('"', '&quot;')
    .replaceAll("'", '&#39;')
}

function replaceCode(html, id, value) {
  const pattern = new RegExp(`(<code id="${id}">)[\\s\\S]*?(</code>)`)
  if (!pattern.test(html)) throw new Error(`Missing site code block: ${id}`)
  return html.replace(pattern, `$1${escapeHtml(value)}$2`)
}

const installation = read('docs/authored/installation.md')
const mcpInstall = read('docs/authored/mcp/install.md')
const primitives = read('docs/authored/primitives.md')
const mcpOverview = read('docs/authored/mcp/overview.md')
const packageJson = JSON.parse(read('typescript/px-sdk/package.json'))

const install = firstCodeBlockAfter(installation, '### Installation Script')
const codexMcp = firstCodeBlockAfter(mcpInstall, '## Connect with Codex')
const initialize = [
  'px init bears --provider local',
  '',
  '# Create a character that the world can resolve',
  'px create character lonnie -u bears -n "Lonnie"',
].join('\n')
const representations = firstCodeBlockAfter(primitives, '### Scene Clips as Representations')
  .split('\n')
  .slice(0, 2)
  .join('\n')
const typescriptSdk = [
  `import {repoCreateEntity} from '${packageJson.name}'`,
  '',
  "const lonnie = repoCreateEntity('bears', 'character', 'lonnie', 'Lonnie')",
].join('\n')
const pythonSdk = [
  'from px_sdk import repo_create_entity',
  '',
  "lonnie = repo_create_entity('bears', 'character', 'lonnie', 'Lonnie')",
].join('\n')

let html = read(path.relative(repoRoot, sitePath))
html = html.replace(/\s*<meta name="px-(?:mcp-summary|skills-install)"[^>]*\/>/g, '')
html = html.replace(/\s*<script[^>]*>[\s\S]*?base\.href=\"\/px\/\"[\s\S]*?<\/script>/g, '')
html = html.replace(
  '<span class="underline decoration-2 underline-offset-4">Explore Portals</span>',
  '<a href="https://portals.works" class="underline decoration-2 underline-offset-4">Explore Portals</a>',
)
html = replaceCode(html, 'px-code-1', install)
html = replaceCode(html, 'px-code-2', codexMcp)
html = replaceCode(html, 'px-code-3', initialize)
html = replaceCode(html, 'px-code-4', representations)
html = replaceCode(html, 'px-code-5', typescriptSdk)
html = replaceCode(html, 'px-code-6', pythonSdk)
const mcpSummary = escapeHtml(firstParagraphAfter(mcpOverview, '## MCP Server'))
const skillsSource = installation.includes('### Skills Install')
  ? firstCodeBlockAfter(installation, '### Skills Install')
  : install.split('&&').at(-1).trim()
const skills = escapeHtml(skillsSource)
const runtimeBase = `<script data-px-base>(function(){if(location.hostname==="portals.works"||location.hostname==="www.portals.works"){var base=document.createElement("base");base.href="/px/";document.head.insertBefore(base,document.head.firstChild);}})();</script>`
html = html.replace('<link rel="icon"', `${runtimeBase}\n  <link rel="icon"`)
html = html.replace('</head>', `  <meta name="px-mcp-summary" content="${mcpSummary.replaceAll('"', '&quot;')}" />\n  <meta name="px-skills-install" content="${skills.replaceAll('"', '&quot;')}" />\n</head>`)
fs.writeFileSync(sitePath, html)

console.log('Updated PX technical examples from docs/authored/.')
