# PX standalone site

Static export of the `px` landing page for GitHub Pages. No build step, no
runtime dependencies — any static file server (or `npx serve site`) renders it.

## Source mapping

Ported from `portals-cloud/frontend` (no framework, no Tailwind toolchain):

| This file | Portals source |
|---|---|
| `index.html` | `app/(marketing)/px/page.tsx` (metadata, JSON-LD) + `src/components/px/PxLandingPage.tsx` with `chrome="standalone"`, copy flattened into HTML |
| `styles.css` | design tokens from `app/globals.css` `@theme` + `src/saga-repro.css` fonts; `t-*` type scale copied verbatim from the Tailwind v4 build (`src/saga.css`); sections from `src/components/px/PxLandingPage.module.css` |
| `script.js` | `src/components/px/PxCodeBlock.tsx` copy button |
| `fonts/` | `public/fonts/*.woff2` (renamed for clean URLs) |
| `favicon.svg` | new minimal PX mark (the portals favicon is product chrome, not reused) |
| `og-image.svg` | static port of `app/(marketing)/px/opengraph-image.tsx` |

The code samples in `#install` mirror `src/lib/px-content.ts`
(`getPxTechnicalContent()`), which reads them from `docs/authored/` in this
repo. If those docs change, update the matching `<code>` blocks here.

Two dead references were intentionally dropped: `styles.card` and
`styles.sagaBannerFrame` are read in the component but have no rule in the CSS
module, so they contribute no styling.

## Deploy

Pushing to `main` runs `.github/workflows/pages.yml`, which uploads `site/`
to GitHub Pages. First-time setup needs one click: repo
**Settings → Pages → Build and deployment → Source: GitHub Actions**.
