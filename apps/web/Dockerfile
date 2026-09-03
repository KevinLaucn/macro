# syntax=docker/dockerfile:1
FROM oven/bun:1.3.5 AS builder
WORKDIR /app

# Copy root monorepo package manifests
COPY package.json bun.lock bunfig.toml ./
COPY packages ./packages
COPY apps/web ./apps/web
COPY services/ai-editing-worker ./services/ai-editing-worker
COPY services/bots ./services/bots
COPY services/lexical-service ./services/lexical-service

# Install dependencies and build web app
RUN bun install --frozen-lockfile
WORKDIR /app/apps/web
RUN bun run build

# Runner stage serving compiled static files with Nginx
FROM nginx:alpine AS runner
COPY apps/web/nginx.conf /etc/nginx/conf.d/default.conf
COPY --from=builder /app/apps/web/dist /usr/share/nginx/html
EXPOSE 80

CMD ["nginx", "-g", "daemon off;"]
