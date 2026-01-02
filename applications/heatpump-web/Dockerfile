# Build stage
FROM node:20-alpine AS builder
WORKDIR /app

# Disable V8 optimizations that cause issues in QEMU ARM64 emulation
ENV NODE_OPTIONS="--max-old-space-size=4096"

# No build-time environment variables needed - all config is runtime
COPY package*.json ./
RUN npm ci
COPY . .
RUN npm run build

# Production stage
FROM nginx:alpine

# Copy built application
COPY --from=builder /app/dist /usr/share/nginx/html

# Copy nginx configuration
COPY nginx.conf /etc/nginx/conf.d/default.conf

# Create env-config template
RUN echo 'window.ENV = {' > /usr/share/nginx/html/env-config.js.template && \
    echo '  API_URL: "${VITE_API_URL}",' >> /usr/share/nginx/html/env-config.js.template && \
    echo '  AUTHENTIK_URL: "${VITE_AUTHENTIK_URL}",' >> /usr/share/nginx/html/env-config.js.template && \
    echo '  OAUTH_CLIENT_ID: "${VITE_OAUTH_CLIENT_ID}",' >> /usr/share/nginx/html/env-config.js.template && \
    echo '  OAUTH_REDIRECT_URI: "${VITE_OAUTH_REDIRECT_URI}"' >> /usr/share/nginx/html/env-config.js.template && \
    echo '};' >> /usr/share/nginx/html/env-config.js.template

# Create entrypoint script
RUN echo '#!/bin/sh' > /docker-entrypoint.sh && \
    echo 'set -e' >> /docker-entrypoint.sh && \
    echo '' >> /docker-entrypoint.sh && \
    echo '# Substitute environment variables in env-config.js' >> /docker-entrypoint.sh && \
    echo 'envsubst < /usr/share/nginx/html/env-config.js.template > /usr/share/nginx/html/env-config.js' >> /docker-entrypoint.sh && \
    echo '' >> /docker-entrypoint.sh && \
    echo 'echo "Environment configuration generated:"' >> /docker-entrypoint.sh && \
    echo 'cat /usr/share/nginx/html/env-config.js' >> /docker-entrypoint.sh && \
    echo '' >> /docker-entrypoint.sh && \
    echo '# Start nginx' >> /docker-entrypoint.sh && \
    echo 'exec nginx -g "daemon off;"' >> /docker-entrypoint.sh && \
    chmod +x /docker-entrypoint.sh

EXPOSE 80

ENTRYPOINT ["/docker-entrypoint.sh"]
