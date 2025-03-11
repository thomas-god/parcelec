/// <reference types="vitest/config" />
import { defineConfig } from "vite";
import { sveltekit } from "@sveltejs/kit/vite";
import tailwindcss from "@tailwindcss/vite";
import { svelteTesting } from "@testing-library/svelte/vite";

// https://vite.dev/config/
export default defineConfig({
  plugins: [sveltekit(), tailwindcss(), svelteTesting()],
  server: {
    host: "127.0.0.1",
    port: 5173,
  },
  test: {
    include: ["**/*.test.ts"],
    globals: true,
    environment: "jsdom",
    setupFiles: ["./vitest-setup.js"],
  },
});
