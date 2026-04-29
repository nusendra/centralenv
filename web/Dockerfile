# syntax=docker/dockerfile:1.7

FROM node:22-alpine AS build

WORKDIR /app
COPY package.json ./
RUN npm install --no-audit --no-fund

COPY svelte.config.js tsconfig.json vite.config.ts ./
COPY src ./src

RUN npm run build

# ---

FROM node:22-alpine

RUN apk add --no-cache tini && \
    addgroup -S app && adduser -S -G app app

WORKDIR /app
COPY --from=build /app/build ./build

# PUBLIC_API_URL is read at RUNTIME via SvelteKit's $env/dynamic/public.
# Override in docker-compose / Coolify env to point at a separate API host.
ENV PORT=3000 \
    HOST=0.0.0.0 \
    NODE_ENV=production \
    PUBLIC_API_URL=

USER app
EXPOSE 3000

ENTRYPOINT ["/sbin/tini", "--"]
CMD ["node", "build"]
