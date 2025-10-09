# Raug WASM Demo

This example shows an editor interface on the web for Raug.

## Prerequisites

```bash
npm install
```

## Scripts

- `npm run dev` – Start the Vite dev server.
- `npm run build` – Generate a production build in `dist/`.
- `npm run preview` – Preview the production build locally.
- `npm run typecheck` – Run TypeScript without emitting files.

All source files live under `src/` as TypeScript modules. The wasm artifacts are imported from `pkg/` directly.
