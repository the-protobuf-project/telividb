import { defineConfig } from "vite";
import { sveltekit } from "@sveltejs/kit/vite";
import tailwindcss from "@tailwindcss/vite";
import process from "node:process";

// Indexed rather than dotted: `env` is an index signature, so `strict` rejects
// the property form. Absent unless `tauri dev` set it, which is the normal case.
const host = process.env["TAURI_DEV_HOST"];

// https://vite.dev/config/
export default defineConfig(() => ({
  // Tailwind v4 is a Vite plugin; there is no PostCSS config and no
  // tailwind.config.js. It must precede sveltekit().
  plugins: [tailwindcss(), sveltekit()],

  // Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
  //
  // 1. prevent Vite from obscuring rust errors
  clearScreen: false,
  // 2. tauri expects a fixed port, fail if that port is not available
  server: {
    port: 1420,
    strictPort: true,
    host: host || "127.0.0.1",
    hmr: host
      ? {
          protocol: "ws",
          host,
          port: 1421,
        }
      : undefined,
    watch: {
      // 3. tell Vite to ignore watching `src-tauri`
      ignored: ["**/src-tauri/**"],
    },
  },
}));
