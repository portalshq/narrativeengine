# PX standalone site

This is the zero-build static export of the current PX landing page in the
portals marketing app. It includes the image-rich Eye Candy sections, the
Portals footer, and the Saga WebGL canvas. Preview it with `npx serve site`.

The Pages origin is deployment plumbing only. The public canonical URL is
always `https://portals.works/px`.

## Source mapping

| Target in `site/` | Source in `cloud/frontend/` |
|---|---|
| `index.html` | `app/(marketing)/px/page.tsx` metadata/JSON-LD + the current `src/components/px/PxLandingPage.tsx`, flattened to HTML |
| `styles.css` | Rendered `app/globals.css`/`src/saga.css` rules plus `src/components/px/PxLandingPage.module.css` and the current utility classes |
| `script.js` | `src/components/px/PxCodeBlock.tsx` clipboard behavior, including the `execCommand` fallback |
| `saga-webgl.js`, `models/scene.glb` | `public/saga-webgl.js` and `public/models/scene.glb`, with asset paths made relative for Pages |
| `eye-candy` image URLs | Optimized WebP assets in `cloud/frontend/public/eye-candy/optimized/`, served from `https://portals.works` so the marketing site remains the image origin |
| `fonts/*.woff2` | `public/fonts/` — six PX font files, renamed for stable relative URLs |
| `favicon.svg` | Minimal PX mark; the portals product favicon is not reused |
| OG image (`og:image`, `twitter:image`) | Absolute FQDN asset in `cloud/frontend/public/og-image-px.png`, served from `https://portals.works` |
| `.nojekyll` | Empty marker required for verbatim Pages serving |

The workflow refreshes the technical code blocks at build time from
`docs/authored/` on every commit to `main`. `README.md` is checked as the
public documentation mirror for the installation, Codex MCP, representation,
and MCP-summary examples; a drift between the authored docs and README fails
the Pages build. When the authored docs or their README mirror change, the
generated install, MCP, initialize, representation, TypeScript, and Python
examples update in the same Pages deployment. The published HTML is static
after deployment; it does not fetch snippets at runtime.

Two dead references were intentionally dropped: `styles.card` and
`styles.sagaBannerFrame` have no CSS-module rules, so omitting them is
pixel-identical.

## Deploy

`.github/workflows/pages.yml` refreshes authored content, validates the static
site, and uploads `site/` to GitHub Pages on every push to `main`. Run
`node scripts/update-site-content.mjs` and `node scripts/validate-site.mjs`
locally before pushing.
