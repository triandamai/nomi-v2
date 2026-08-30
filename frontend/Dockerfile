# syntax=docker/dockerfile:1

FROM node:22-bookworm-slim AS builder
WORKDIR /app
COPY package.json package-lock.json ./
RUN npm ci
COPY . .
RUN npm run build

FROM node:22-bookworm-slim AS runtime
WORKDIR /app
ENV NODE_ENV=production
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
