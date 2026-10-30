# Nomi frontend (new)

The new Nomi web app, rebuilt from the pen.dev design system. It runs side by side with the
current app in `frontend/` and talks to the same backend API, unchanged.

## Run it

Node 22.12 or newer.

```sh
npm ci
npx playwright install chromium   # once, for the component tests
npm run dev                        # http://localhost:5175
```

## Check it

```sh
npm run check && npm run test:unit && npm run test:components && npm run build
```

- `test:unit`: pure TypeScript and server code, in Node.
- `test:components`: components in a real Chromium, with the app's CSS.

How to work in this folder (design rules, tests, i18n, boundaries): see [CLAUDE.md](CLAUDE.md).
What the backend would need to add for some designed flows: see
[BACKEND_TODO.md](BACKEND_TODO.md).
