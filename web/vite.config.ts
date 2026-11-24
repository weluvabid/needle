import { defineConfig, type Plugin } from "vite";
import tailwindcss from "@tailwindcss/vite";

function inlineCssPlugin(): Plugin {
  return {
    name: "inline-build-css",
    apply: "build",
    generateBundle(_options, bundle) {
      const html = bundle["index.html"];
      if (!html || html.type !== "asset" || typeof html.source !== "string") {
        this.error("Expected Vite to emit index.html as a text asset.");
      }

      let document = html.source;
      const styles: string[] = [];

      for (const [fileName, asset] of Object.entries(bundle)) {
        if (asset.type !== "asset" || !fileName.endsWith(".css")) continue;

        const css = typeof asset.source === "string"
          ? asset.source
          : new TextDecoder().decode(asset.source);
        styles.push(css);

        document = document.replace(/<link\b[^>]*>/g, (tag) => {
          const href = tag.match(/\bhref=["']([^"']+)["']/)?.[1];
          return href?.endsWith(fileName) ? "" : tag;
        });
        delete bundle[fileName];
      }

      if (styles.length > 0) {
        document = document.replace(
          "</head>",
          `<style>${styles.join("\n")}</style>\n</head>`,
        );
      }

      html.source = document;
    },
  };
}

export default defineConfig(({ mode }) => ({
  base: mode === "production" ? "/needle/" : "/",
  plugins: [tailwindcss(), inlineCssPlugin()],
  server: {
    port: 4173,
    strictPort: true,
  },
  build: {
    target: "es2022",
    outDir: "dist",
    emptyOutDir: true,
    cssCodeSplit: false,
  },
}));
