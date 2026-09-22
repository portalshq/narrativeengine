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

function replaceText(html, id, value) {
  const pattern = new RegExp(`(<p id="${id}"[^>]*>)[\\s\\S]*?(</p>)`)
  if (!pattern.test(html)) throw new Error(`Missing site text block: ${id}`)
  const formatted = escapeHtml(value).replaceAll(/`([^`]+)`/g, '<code>$1</code>')
  return html.replace(pattern, `$1${formatted}$2`)
}

const installation = read('docs/authored/installation.md')
const primitives = read('docs/authored/primitives.md')
const mcpOverview = read('docs/authored/mcp/overview.md')
const packageJson = JSON.parse(read('typescript/px-sdk/package.json'))

const install = firstCodeBlockAfter(installation, '### Installation Script')
const skillsHeading = installation.indexOf('### Skills Install')
const skills = skillsHeading === -1
  ? install.split('&&').at(-1).trim()
  : firstCodeBlockAfter(installation, '### Skills Install')
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
const mcpSummary = firstParagraphAfter(mcpOverview, '## MCP Server')
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
html = replaceCode(html, 'code-install', install)
html = replaceCode(html, 'code-skills', skills)
html = replaceCode(html, 'code-init', initialize)
html = replaceCode(html, 'code-repr', representations)
html = replaceCode(html, 'code-ts', typescriptSdk)
html = replaceCode(html, 'code-py', pythonSdk)
html = replaceText(html, 'mcp-summary', mcpSummary)
fs.writeFileSync(sitePath, html)

console.log('Updated PX site technical content from docs/authored/.')
