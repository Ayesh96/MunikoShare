import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

import { manifestPlugin } from "./build/manifestPlugin";

export default defineConfig({
  base: "/m/",
  plugins: [react(), manifestPlugin()],
  build: { outDir: "dist", emptyOutDir: true },
  server: { port: 5174 },
});
