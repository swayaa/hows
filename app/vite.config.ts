import { defineConfig, searchForWorkspaceRoot } from "vite";
import { svelte } from "@sveltejs/vite-plugin-svelte";

// Port 1420 ist der Tauri-Default (siehe src-tauri/tauri.conf.json, devUrl).
export default defineConfig({
  plugins: [svelte()],
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    // brands.json und marks.json liegen im Kern, damit Exporte dieselben Werte nutzen.
    fs: {
      allow: [
        searchForWorkspaceRoot(process.cwd()),
        "../core/export/brands.json",
        "../core/store/marks.json",
      ],
    },
  },
  build: {
    outDir: "dist",
    target: "es2022",
  },
});
