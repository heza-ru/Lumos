import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import { resolve } from "path";

export default defineConfig({
  plugins: [react()],
  server: { port: 1420, strictPort: true },
  build: {
    outDir: "dist",
    emptyOutDir: true,
    rollupOptions: {
      input: {
        toolbar: resolve(__dirname, "index.html"),
        canvas:  resolve(__dirname, "canvas.html"),
      },
    },
  },
});
