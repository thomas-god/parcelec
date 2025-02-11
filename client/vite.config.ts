/// <reference types="vitest/config" />
import { defineConfig } from "vite";
import { sveltekit } from "@sveltejs/kit/vite";
import tailwindcss from "@tailwindcss/vite";

// https://vite.dev/config/
export default defineConfig({
  plugins: [sveltekit(), tailwindcss()],
  server: {
    host: "127.0.0.1",
    port: 5173
  },
  test: {
    include: ["**/*.test.ts"],
    globals: true,
  },
});
