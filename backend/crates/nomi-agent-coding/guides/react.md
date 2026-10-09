# React 19 + Vite (single-page apps)

Use React when the user asks for it, or for a browser-only app (no server, no shared database).
Data can live in `localStorage` or come from a public API. If the app needs a server or a
database, say so and prefer SvelteKit (Nomi's full-stack default) unless the user insists.

`package.json`:
```json
{
  "name": "app",
  "private": true,
  "version": "0.0.0",
  "type": "module",
  "scripts": { "dev": "vite", "build": "tsc -b && vite build", "check": "tsc -b", "preview": "vite preview" },
  "dependencies": { "react": "^19.2.8", "react-dom": "^19.2.8" },
  "devDependencies": {
    "@tailwindcss/vite": "^4.3.0", "@types/node": "^24.13.3", "@types/react": "^19.2.18",
    "@types/react-dom": "^19.2.7", "@vitejs/plugin-react": "^6.1.1", "tailwindcss": "^4.3.0",
    "typescript": "~6.0.2", "vite": "^8.3.0"
  }
}
```
For routing add `react-router@^8.4.0` (`createBrowserRouter`, `RouterProvider` from `react-router`).

`vite.config.ts`:
```ts
import tailwindcss from '@tailwindcss/vite'
import react from '@vitejs/plugin-react'
import { defineConfig } from 'vite'

export default defineConfig({ plugins: [react(), tailwindcss()] })
```

`tsconfig.json` = `{ "files": [], "references": [{ "path": "./tsconfig.app.json" }, { "path": "./tsconfig.node.json" }] }`

`tsconfig.app.json`:
```json
{
  "compilerOptions": {
    "tsBuildInfoFile": "./node_modules/.tmp/tsconfig.app.tsbuildinfo",
    "target": "es2023", "lib": ["ES2023", "DOM"], "module": "esnext", "types": ["vite/client"],
    "skipLibCheck": true, "moduleResolution": "bundler", "allowImportingTsExtensions": true,
    "verbatimModuleSyntax": true, "moduleDetection": "force", "noEmit": true, "jsx": "react-jsx",
    "strict": true, "noUnusedLocals": true, "noUnusedParameters": true, "noFallthroughCasesInSwitch": true
  },
  "include": ["src"]
}
```

`tsconfig.node.json`:
```json
{
  "compilerOptions": {
    "tsBuildInfoFile": "./node_modules/.tmp/tsconfig.node.tsbuildinfo",
    "target": "es2023", "lib": ["ES2023"], "types": ["node"], "skipLibCheck": true,
    "module": "nodenext", "allowImportingTsExtensions": true, "verbatimModuleSyntax": true,
    "moduleDetection": "force", "noEmit": true, "strict": true
  },
  "include": ["vite.config.ts"]
}
```

`index.html` has `<div id="root"></div>` and `<script type="module" src="/src/main.tsx"></script>`.
`src/main.tsx`:
```tsx
import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
import './index.css'
import App from './App.tsx'

createRoot(document.getElementById('root')!).render(<StrictMode><App /></StrictMode>)
```
`src/index.css`: `@import 'tailwindcss';`

Conventions: function components and hooks only; import types with `import type`; import local
files with their extension (`./App.tsx`); keep components in `src/components/`.
