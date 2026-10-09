# Vue 3 + Vite (single-page apps)

Use Vue when the user asks for it, for a browser-only app. If the app needs a server or a
database, prefer SvelteKit unless the user insists on Vue.

`package.json`:
```json
{
  "name": "app",
  "private": true,
  "version": "0.0.0",
  "type": "module",
  "scripts": { "dev": "vite", "build": "vue-tsc -b && vite build", "check": "vue-tsc -b", "preview": "vite preview" },
  "dependencies": { "vue": "^3.5.42" },
  "devDependencies": {
    "@tailwindcss/vite": "^4.3.0", "@types/node": "^24.13.3", "@vitejs/plugin-vue": "^6.0.8",
    "@vue/tsconfig": "^0.9.1", "tailwindcss": "^4.3.0", "typescript": "~6.0.2", "vite": "^8.3.0",
    "vue-tsc": "^3.3.11"
  }
}
```
Add `vue-router@^4` for pages and `pinia` for shared state when needed.

`vite.config.ts`:
```ts
import tailwindcss from '@tailwindcss/vite'
import vue from '@vitejs/plugin-vue'
import { defineConfig } from 'vite'

export default defineConfig({ plugins: [vue(), tailwindcss()] })
```

`tsconfig.json` = `{ "files": [], "references": [{ "path": "./tsconfig.app.json" }, { "path": "./tsconfig.node.json" }] }`

`tsconfig.app.json`:
```json
{
  "extends": "@vue/tsconfig/tsconfig.dom.json",
  "compilerOptions": {
    "tsBuildInfoFile": "./node_modules/.tmp/tsconfig.app.tsbuildinfo",
    "types": ["vite/client"], "noUnusedLocals": true, "noUnusedParameters": true, "noFallthroughCasesInSwitch": true
  },
  "include": ["src/**/*.ts", "src/**/*.tsx", "src/**/*.vue"]
}
```
`tsconfig.node.json` is the same as in the React guide.

`index.html` has `<div id="app"></div>` and `<script type="module" src="/src/main.ts"></script>`.
`src/main.ts`:
```ts
import { createApp } from 'vue'
import './style.css'
import App from './App.vue'

createApp(App).mount('#app')
```
`src/style.css`: `@import 'tailwindcss';`

Conventions: `<script setup lang="ts">` single-file components with the Composition API (`ref`,
`computed`, `watch`, `defineProps<{...}>()`, `defineEmits<{...}>()`); no Options API.
