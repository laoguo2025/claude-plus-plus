import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import packageJson from "./package.json";
import { readFileSync } from "node:fs";

function readDefaultProxyPort(): number {
  const constants = readFileSync("src-tauri/src/constants.rs", "utf8");
  const match = constants.match(/pub const DEFAULT_PROXY_PORT:\s*u16\s*=\s*(\d+);/);
  if (!match) {
    throw new Error("Unable to read DEFAULT_PROXY_PORT from Rust constants");
  }
  return Number(match[1]);
}

const defaultProxyPort = readDefaultProxyPort();

// @ts-expect-error process is a nodejs global
const host = process.env.TAURI_DEV_HOST;

// https://vite.dev/config/
export default defineConfig(async () => ({
  plugins: [react()],
  define: {
    __APP_VERSION__: JSON.stringify(packageJson.version),
    __DEFAULT_PROXY_PORT__: JSON.stringify(defaultProxyPort),
  },

  // Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
  //
  // 1. prevent Vite from obscuring rust errors
  clearScreen: false,
  // 2. tauri expects a fixed port, fail if that port is not available
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
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
