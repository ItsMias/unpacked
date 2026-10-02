import { rmSync } from "node:fs";
import { defineConfig } from "vite";
import { svelte } from "@sveltejs/vite-plugin-svelte";

// Real package data for development lives in public/dev/ (git-ignored); Vite would copy it into
// the build like the rest of public/, so it's removed from every build.
const keepDevDataOut = {
  name: "keep-dev-data-out",
  closeBundle() {
    rmSync(new URL("./dist/dev", import.meta.url), { recursive: true, force: true });
  },
};

export default defineConfig({
  plugins: [svelte(), keepDevDataOut],
  worker: { format: "es" },
  build: { target: "es2022" },
});
