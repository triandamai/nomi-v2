# syntax=docker/dockerfile:1

FROM node:22-bookworm-slim AS builder
WORKDIR /app
COPY package.json package-lock.json ./
RUN npm ci
COPY . .
# The release tag (e.g. v1.2.3). It's built into the app and sent to the backend as
# X-Client-Version. Without it, the build is labelled with package.json's version plus "-dev"
# (the image has no git history).
ARG APP_VERSION=""
ENV APP_VERSION=${APP_VERSION}
# The release notes of this build's environment (staging or production), as base64 JSON, read
# from GitHub by CI (release-notes.js). Without them the app has no notes to show.
ARG RELEASE_NOTES=""
ENV RELEASE_NOTES=${RELEASE_NOTES}
RUN npm run build

FROM node:22-bookworm-slim AS runtime
WORKDIR /app
ENV NODE_ENV=production
ENV BODY_SIZE_LIMIT=Infinity
# The live-update relay (ws-proxy) runs outside the build and reads it here.
ARG APP_VERSION=""
ENV APP_VERSION=${APP_VERSION}
COPY package.json package-lock.json ./
# Only the relay's own dependency (ws): the app itself is bundled into build/. --ignore-scripts:
# `prepare` needs the dev tools, which the runtime neither has nor needs.
RUN npm ci --omit=dev --ignore-scripts
COPY --from=builder /app/build ./build
COPY server.js ./server.js
COPY ws-proxy ./ws-proxy

# API_URL (where the backend is reachable) comes from the deployment environment, like ORIGIN;
# see .env.example. PORT/HOST default to 3000/0.0.0.0 (server.js).
EXPOSE 3000
CMD ["node", "server.js"]
