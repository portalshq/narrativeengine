import fs from 'node:fs'
import path from 'node:path'
import {fileURLToPath} from 'node:url'

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..')
const siteRoot = path.join(repoRoot, 'site')
const html = fs.readFileSync(path.join(siteRoot, 'index.html'), 'utf8')
const css = fs.readFileSync(path.join(siteRoot, 'styles.css'), 'utf8')

const requiredFiles = [
  'index.html',
  'styles.css',
  'script.js',
  'favicon.svg',
  'og-image.svg',
  '.nojekyll',
  'saga-webgl.js',
  'models/scene.glb',
  'fonts/die-grotesk-b-regular.woff2',
  'fonts/die-grotesk-b-medium.woff2',
  'fonts/die-grotesk-c-regular.woff2',
  'fonts/aeonik-fono-regular.woff2',
  'fonts/stk-bureau-book.woff2',
  'fonts/stk-bureau-light.woff2',
]
for (const relativePath of requiredFiles) {
  if (!fs.existsSync(path.join(siteRoot, relativePath))) throw new Error(`Missing site asset: ${relativePath}`)
}

const classNames = [...html.matchAll(/class="([^"]*)"/g)]
  .flatMap((match) => match[1].split(/\s+/))
  .filter(Boolean)
const uniqueClassNames = [...new Set(classNames)]
const manuallyResolved = new Set(['2xl:grid-cols-[1.5fr_1fr]', '2xl:block', 'row-start-last'])
const missing = uniqueClassNames.filter((className) => {
  if (className === 'group' || className.startsWith('lucide') || manuallyResolved.has(className)) return false
  const escaped = className.replace(/[^a-zA-Z0-9_-]/g, (character) => `\\${character}`)
  return !css.includes(`.${escaped}`)
})
if (missing.length) throw new Error(`Classes without CSS rules: ${missing.join(', ')}`)

if ((html.match(/<h1\b/g) ?? []).length !== 1) throw new Error('PX site must contain exactly one h1')
if (html.includes('class="undefined') || html.includes('portalshq.github.io')) throw new Error('Static HTML contains an unresolved class or public Pages origin')
if (!html.includes('data-webgl-theme="px"') || !html.includes('src="./saga-webgl.js"')) throw new Error('WebGL runtime is not wired into the static page')

if (!html.includes('<link rel="canonical" href="https://portals.works/px"')) throw new Error('Canonical URL is not portals.works/px')
if (!html.includes('content="https://portals.works/px/og-image.svg"')) throw new Error('OG/Twitter image is not canonical')
const publicMarkup = html.replace(/<script[^>]*>[\s\S]*?base\.href=\"\/px\/\"[\s\S]*?<\/script>/g, '')
if (publicMarkup.includes('href="/')) throw new Error('Static HTML contains an origin-relative link')

console.log(`Validated ${requiredFiles.length} site assets and ${uniqueClassNames.length - 1} CSS classes.`)
