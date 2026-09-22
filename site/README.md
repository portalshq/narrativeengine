# PX standalone site

This is the zero-build static export of the PX landing page. Preview it with
`npx serve site`; the Pages workflow deploys the same files on every commit to
`main`.

## Source mapping

Ported from the portals marketing app (`cloud/frontend`):

| Target in `site/` | Source in `cloud/frontend/` |
|---|---|
| `index.html` | `app/(marketing)/px/page.tsx` metadata/JSON-LD + `src/components/px/PxLandingPage.tsx` with `chrome="standalone"`, flattened to HTML |
| `styles.css` | `app/globals.css` `@theme`, `src/saga.css` type rules, `src/components/px/PxLandingPage.module.css`, and the component's hand-resolved Tailwind utility subset |
| `script.js` | `src/components/px/PxCodeBlock.tsx` clipboard behavior, including the `execCommand` fallback |
| `fonts/*.woff2` | `public/fonts/` — the six PX font files, renamed for stable relative URLs |
| `favicon.svg` | New minimal PX mark; the portals product favicon is not reused |
| `og-image.svg` | Static SVG port of `app/(marketing)/px/opengraph-image.tsx` |
| `.nojekyll` | Empty marker required for verbatim Pages serving |

The install, skills, initialize, representations, MCP, TypeScript SDK, and
Python SDK content is refreshed by `scripts/update-site-content.mjs`. Update the
matching `<code>` blocks whenever `../px/docs/authored/` changes.

`portalshq.github.io` is only the GitHub Pages deployment origin; the public
canonical URL is always `https://portals.works/px`.

Two dead references were intentionally dropped: `styles.card` and
`styles.sagaBannerFrame` are read in the former component but have no rule in
the CSS module, so they contribute no styling.

## Deploy

`.github/workflows/pages.yml` refreshes authored content, validates the static
site, and uploads `site/` to GitHub Pages on every push to `main`.
