import type { Plugin } from "vite";

import { colors } from "../src/theme/tokens";

const MANIFEST_FILE = "manifest.webmanifest";

/** Emits the web manifest and theme-color tags so colors come only from the theme tokens. */
export function manifestPlugin(): Plugin {
  let base = "/";
  return {
    name: "muniko-manifest",
    configResolved(config) {
      base = config.base;
    },
    generateBundle() {
      const manifest = {
        name: "Muniko Share",
        short_name: "Muniko",
        start_url: base,
        scope: base,
        display: "standalone",
        background_color: colors.light.bg,
        theme_color: colors.light.primary,
        icons: [{ src: `${base}icon.svg`, sizes: "any", type: "image/svg+xml", purpose: "any" }],
      };
      this.emitFile({ type: "asset", fileName: MANIFEST_FILE, source: JSON.stringify(manifest, null, 2) });
    },
    transformIndexHtml() {
      return [
        { tag: "link", attrs: { rel: "manifest", href: `${base}${MANIFEST_FILE}` }, injectTo: "head" },
        { tag: "link", attrs: { rel: "icon", href: `${base}icon.svg` }, injectTo: "head" },
        { tag: "meta", attrs: { name: "theme-color", content: colors.light.bg, media: "(prefers-color-scheme: light)" }, injectTo: "head" },
        { tag: "meta", attrs: { name: "theme-color", content: colors.dark.bg, media: "(prefers-color-scheme: dark)" }, injectTo: "head" },
      ];
    },
  };
}
