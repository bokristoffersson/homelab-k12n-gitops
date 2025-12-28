FROM kong:3.8

USER root

# Install git and dependencies for building Lua rocks
RUN apt-get update && \
    apt-get install -y git unzip build-essential && \
    rm -rf /var/lib/apt/lists/*

# Install lua-resty dependencies in correct order
# These are needed for lua-resty-openidc
RUN luarocks install lua-resty-http && \
    luarocks install lua-resty-session && \
    luarocks install lua-resty-jwt

# Install lua-resty-openidc (dependency for kong-oidc)
RUN luarocks install lua-resty-openidc

# Install kong-oidc plugin
# Using a stable version from the original repo
RUN git clone --branch v1.3.0 https://github.com/revomatico/kong-oidc.git /tmp/kong-oidc && \
    cd /tmp/kong-oidc && \
    luarocks make && \
    rm -rf /tmp/kong-oidc

# Switch back to kong user
USER kong

# The plugin will be available as 'oidc' (not 'openid-connect')
