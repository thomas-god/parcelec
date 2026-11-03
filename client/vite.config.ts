import adapter from "@sveltejs/adapter-static";
import { vitePreprocess } from "@sveltejs/vite-plugin-svelte";

import { defineConfig } from "vitest/config";
import { sveltekit } from "@sveltejs/kit/vite";
import tailwindcss from "@tailwindcss/vite";
import { svelteTesting } from "@testing-library/svelte/vite";

// https://vite.dev/config/
export default defineConfig({
  plugins: [
    sveltekit({
      // Consult https://svelte.dev/docs/kit/integrations
      // for more information about preprocessors
      preprocess: vitePreprocess(),

      adapter: adapter({
        pages: "build",
        assets: "build",
        fallback: "index.html",
        precompress: false,
        strict: true,
      }),
    }),
    tailwindcss(),
    svelteTesting(),
  ],
  server: { host: true, port: 5173 },
  test: {
    include: ["**/*.test.ts"],
    globals: true,
    environment: "jsdom",
    setupFiles: ["./vitest-setup.ts"],
  },
});
