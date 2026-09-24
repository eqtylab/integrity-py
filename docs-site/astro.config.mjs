// @ts-check
import docs, { resolveDocsEnv } from "@eqtylab/docs";
import { defineConfig } from "astro/config";

// `base` has to be literal in Astro's config at build time, so the GitHub Pages
// sub-path deploy threads it through the environment as DOCS_BASE. Unset locally
// and on the custom domain, where the site serves from `/`.
const env = resolveDocsEnv({ site: process.env.DOCS_SITE ?? "https://eqtylab.github.io" });

export default defineConfig({
  site: env.site,
  base: env.base,
  trailingSlash: "ignore",
  build: { format: "directory" },
  devToolbar: { enabled: false },

  vite: {
    // Equality ships CSS modules in its dist, which Node cannot load once Vite
    // externalises the package for SSR. Every npm consumer of @eqtylab/docs needs this.
    resolve: { noExternal: ["@eqtylab/equality"] },
    // Pages import ../examples/*.py and ../docs/generated/*.txt with `?raw`, which sit
    // above this project's root. Build allows it; dev needs to be told.
    server: { fs: { allow: [".."] } },
  },

  integrations: [
    docs({
      env,
      title: "Integrity Python SDK",
      description:
        "Documentation for eqty_sdk, the EQTY Integrity Python SDK for tracking data provenance, asset lineage and computation integrity.",
      favicon: "/favicon.ico",
      // The site documents the current release only. Older releases are served frozen from the
      // old site's folders, overlaid by .github/workflows/docs.yml.
      versions: false,
      header: {
        links: [
          { label: "GitHub", href: "https://github.com/eqtylab/integrity-py", external: true },
          {
            label: "Changelog",
            href: "https://github.com/eqtylab/integrity-py/blob/main/CHANGELOG.md",
            external: true,
          },
          // Absolute: @eqtylab/docs does not prefix header links with `base`, so a
          // root-relative `/latest/` would leave the /integrity-py/ sub-path.
          {
            label: "Older versions",
            href: "https://eqtylab.github.io/integrity-py/latest/",
            external: true,
          },
        ],
      },
      footer: {
        // Ends at the content directory: the route appends the entry's path relative to it.
        editUrl: "https://github.com/eqtylab/integrity-py/edit/main/docs-site/src/content/docs/",
      },
    }),
  ],
});
