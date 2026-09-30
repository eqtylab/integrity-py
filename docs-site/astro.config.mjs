// @ts-check
import docs, { resolveDocsEnv } from "@eqtylab/docs";
import { defineConfig } from "astro/config";
import folders from "./archive/folders.json" with { type: "json" };

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
    // The Min Version page imports ../docs/generated/*.txt with `?raw`, which sits above
    // this project's root. Build allows it; dev needs to be told.
    server: { fs: { allow: [".."] } },
  },

  integrations: [
    docs({
      env,
      title: "Integrity Python SDK",
      description:
        "Documentation for eqty_sdk, the EQTY Integrity Python SDK for tracking data provenance, asset lineage and computation integrity.",
      favicon: "/favicon.ico",
      // Old versions come from the saved folders below, not from release tags, so Vercel's
      // clone, which has no tags, builds the same site as CI. An archived page keeps only what
      // is written in it: scripts/render_api_docs.py puts the reference and examples there.
      versions: {
        tags: "v*",
        granularity: "minor",
        source: "folders",
        // Archived versions: 2.0 to 2.4 converted from MkDocs by scripts/archive_version.py,
        // later ones saved by release.yml with their wheel reports. The folders are frozen; the
        // highest is the current release, and waits until a newer release makes it old.
        folders,
      },
      header: {
        links: [
          { label: "GitHub", href: "https://github.com/eqtylab/integrity-py", external: true },
          {
            label: "Changelog",
            href: "https://github.com/eqtylab/integrity-py/blob/main/CHANGELOG.md",
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
