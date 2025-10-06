# Raug WASM Demo

This example bundles the `raug` audio worklet implementation with TypeScript and Vite.

## Prerequisites

```bash
npm install
```

## Scripts

- `npm run dev` – Start the Vite dev server with COOP/COEP headers.
- `npm run build` – Generate a production build in `dist/`.
- `npm run preview` – Preview the production build locally.
- `npm run typecheck` – Run TypeScript without emitting files.

All source files live under `src/` as TypeScript modules. The wasm artifacts are imported from `pkg/` directly.
