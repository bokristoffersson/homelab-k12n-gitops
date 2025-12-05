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
# .yarnrc.yml specifies yarnPath pointing to .yarn/releases/yarn-4.4.1.cjs
COPY .yarnrc.yml ./
COPY .yarn ./.yarn

# Ensure Yarn binary is executable
RUN chmod +x .yarn/releases/yarn-4.4.1.cjs

# Copy all package.json files and lockfile together
# This ensures the lockfile matches the package.json files
# Copy root package.json and lockfile first
COPY package.json yarn.lock* package-lock.json* ./

# Copy package.json files from all workspaces to preserve structure
COPY packages/app/package.json ./packages/app/
COPY packages/backend/package.json ./packages/backend/
# Copy plugins directory (may be empty, but structure is needed)
COPY plugins/ ./plugins/

# Verify Yarn is available and can find the binary
RUN yarn --version

# Install dependencies
RUN yarn install --immutable

# Copy source code
COPY . .

# Ensure Yarn is still available after copying source
RUN chmod +x .yarn/releases/yarn-4.4.1.cjs && yarn --version

# Build the application
RUN yarn tsc
RUN yarn build:backend
RUN yarn build:all

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

