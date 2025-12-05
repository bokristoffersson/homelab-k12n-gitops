# Build stage
FROM --platform=$BUILDPLATFORM node:20-slim AS builder

ARG TARGETPLATFORM
ARG BUILDPLATFORM
WORKDIR /app

# Install dependencies for building native modules
RUN apt-get update && apt-get install -y \
    python3 \
    make \
    g++ \
    git \
    && rm -rf /var/lib/apt/lists/*

# Enable Corepack for Yarn v4 (Berry)
RUN corepack enable

# Copy Yarn configuration files (required for Yarn v4)
COPY .yarnrc.yml ./
COPY .yarn/releases/ ./.yarn/releases/

# Copy package files
COPY package.json yarn.lock* package-lock.json* ./
COPY packages/*/package.json ./packages/
COPY plugins/*/package.json ./plugins/

# Install dependencies
RUN yarn install --frozen-lockfile

# Copy source code
COPY . .

# Build the application
RUN yarn tsc
RUN yarn build:backend --config app-config.yaml
RUN yarn build

# Runtime stage
FROM node:20-slim

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    python3 \
    git \
    && rm -rf /var/lib/apt/lists/*

# Copy built artifacts from builder
COPY --from=builder /app/packages/backend/dist ./packages/backend/dist
COPY --from=builder /app/packages/backend/package.json ./packages/backend/
COPY --from=builder /app/packages/app/dist ./packages/app/dist
COPY --from=builder /app/packages/app/package.json ./packages/app/
COPY --from=builder /app/package.json ./app-config.yaml ./
COPY --from=builder /app/node_modules ./node_modules
COPY --from=builder /app/packages/*/node_modules ./packages/*/node_modules
COPY --from=builder /app/plugins/*/node_modules ./plugins/*/node_modules

# Set environment variables
ENV NODE_ENV=production
ENV APP_CONFIG=/app-config/app-config.yaml

# Expose port
EXPOSE 7007

# Start the backend
WORKDIR /app/packages/backend
CMD ["node", "dist/index.js"]

