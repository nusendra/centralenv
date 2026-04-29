# syntax=docker/dockerfile:1.7

FROM node:22-alpine AS build

WORKDIR /app
COPY package.json ./
RUN npm install --no-audit --no-fund

COPY svelte.config.js tsconfig.json vite.config.ts ./
COPY src ./src

# Empty default = same-origin (single-domain deploy with reverse proxy).
# Override at build time:  --build-arg VITE_API_URL=https://api.example.com
ARG VITE_API_URL=
ENV VITE_API_URL=$VITE_API_URL

RUN npm run build

# ---

FROM node:22-alpine

RUN apk add --no-cache tini && \
    addgroup -S app && adduser -S -G app app

WORKDIR /app
COPY --from=build /app/build ./build

ENV PORT=3000 \
    HOST=0.0.0.0 \
    NODE_ENV=production

USER app
EXPOSE 3000

ENTRYPOINT ["/sbin/tini", "--"]
CMD ["node", "build"]
