# syntax=docker/dockerfile:1

FROM node:22-bookworm-slim AS builder
WORKDIR /app
COPY package.json package-lock.json ./
RUN npm ci
COPY . .
# The release tag (e.g. v1.2.3), passed by .github/workflows/release.yml. It's built into the
# app (shown in the UI, sent to the backend as X-Client-Version). Without it, the build is
# labelled with package.json's version plus "-dev" (the image has no git history).
ARG APP_VERSION=""
ENV APP_VERSION=${APP_VERSION}
RUN npm run build

FROM node:22-bookworm-slim AS runtime
WORKDIR /app
ENV NODE_ENV=production
# The live-chat WebSocket relay (ws-proxy) runs outside the build and reads it here.
ARG APP_VERSION=""
ENV APP_VERSION=${APP_VERSION}
COPY package.json package-lock.json ./
RUN npm ci --omit=dev
COPY --from=builder /app/build ./build
COPY server.js ./server.js
COPY ws-proxy ./ws-proxy

# API_URL (where the backend is reachable) is supplied by the deployment
# environment — see src/lib/server/api.ts. PORT/HOST default to 3000/0.0.0.0,
# overridable the same way — see server.js.
EXPOSE 3000
CMD ["node", "server.js"]
