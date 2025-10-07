import { defineConfig } from 'vite';
import path from 'node:path';


const wasmContentTypePlugin = {
  name: "wasm-content-type-plugin",
  configureServer(server: any) {
    server.middlewares.use((req: any, res: any, next: any) => {
      if (req.url.endsWith(".wasm")) {
        res.setHeader("Content-Type", "application/wasm");
      }
      next();
    });
  },
};

const coopHeaders = {
  'Cross-Origin-Opener-Policy': 'same-origin',
  'Cross-Origin-Embedder-Policy': 'require-corp',
};

export default defineConfig({
  server: {
    open: true,
    headers: coopHeaders,
    fs: {
      allow: [path.resolve(__dirname, '..', '..', '..')],
    },
  },
  preview: {
    headers: coopHeaders,
  },
  build: {
    outDir: 'dist',
    emptyOutDir: true,
    target: 'esnext',
    rollupOptions: {
      output: {
        format: 'es',
      },
    },
  },
  assetsInclude: ['**/*.wasm'],
  plugins: [wasmContentTypePlugin],
});
