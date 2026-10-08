// @ts-check
import docs, { resolveDocsEnv } from "@eqtylab/docs";
import { defineConfig } from "astro/config";
import folders from "./archive/folders.json" with { type: "json" };

// `base` has to be literal in Astro's config at build time, so a sub-path deploy
// threads it through the environment as DOCS_BASE. None sets it now: the site
// serves from `/` locally and on integrity-py.docs.eqtylab.io. That domain is also the
// default site, so builds without DOCS_SITE (CI's artifact, local builds) name the real host.
const env = resolveDocsEnv({
  site: process.env.DOCS_SITE ?? "https://integrity-py.docs.eqtylab.io",
});

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
      repository: { url: "https://github.com/eqtylab/integrity-py" },
      license: "Apache 2.0",
      header: {
        links: [
          {
            label: "GitHub",
            href: "https://github.com/eqtylab/integrity-py",
            icon: "simple-icons:github",
            external: true,
          },
          {
            label: "Changelog",
            href: "https://github.com/eqtylab/integrity-py/blob/main/CHANGELOG.md",
            external: true,
          },
        ],
      },
      footer: {
        links: [
          {
            label: "GitHub",
            href: "https://github.com/eqtylab/integrity-py",
            icon: "simple-icons:github",
            external: true,
          },
          {
            label: "PyPI",
            href: "https://pypi.org/project/eqty-sdk/",
            icon: "simple-icons:pypi",
            external: true,
          },
        ],
      },
      // Old links into the API reference name headings by ids this site does not use. The
      // script finds the heading; it loads on every page, old versions included.
      clientScripts: ["./src/scripts/old-anchors.js"],
    }),
  ],
});
