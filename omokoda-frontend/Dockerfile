# Ọmọ Kọ́dà — legacy Next.js frontend (the human interface layer).
# NOTE: Axiom (:8876) is the canonical dashboard; this is the TypeScript PWA surface.

FROM node:20-slim AS deps
WORKDIR /app
COPY package.json package-lock.json* ./
RUN npm ci

FROM node:20-slim AS builder
WORKDIR /app
COPY --from=deps /app/node_modules ./node_modules
COPY . .
RUN npm run build

FROM node:20-slim AS runtime
WORKDIR /app
ENV NODE_ENV=production
# Talk to the kernel service by its compose hostname.
ENV KERNEL=http://kernel:7777
COPY --from=builder /app/.next ./.next
COPY --from=builder /app/node_modules ./node_modules
COPY --from=builder /app/package.json ./package.json
EXPOSE 3000
CMD ["npm", "run", "start"]
