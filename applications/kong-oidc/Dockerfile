FROM kong:3.8

USER root

# Install dependencies
RUN apt-get update && \
    apt-get install -y git wget && \
    rm -rf /var/lib/apt/lists/*

# Install lua-resty-openidc and its dependencies manually
# This avoids luarocks dependency resolution issues

# Install lua-resty-http
RUN wget -qO- https://github.com/pintsized/lua-resty-http/archive/refs/tags/v0.17.1.tar.gz | tar -xz -C /tmp && \
    cp -r /tmp/lua-resty-http-0.17.1/lib/resty/* /usr/local/share/lua/5.1/resty/ && \
    rm -rf /tmp/lua-resty-http-0.17.1

# Install lua-resty-session
RUN wget -qO- https://github.com/bungle/lua-resty-session/archive/refs/tags/v4.0.5.tar.gz | tar -xz -C /tmp && \
    cp -r /tmp/lua-resty-session-4.0.5/lib/resty/* /usr/local/share/lua/5.1/resty/ && \
    rm -rf /tmp/lua-resty-session-4.0.5

# Install lua-resty-jwt
RUN wget -qO- https://github.com/SkyLothar/lua-resty-jwt/archive/refs/tags/v0.2.3.tar.gz | tar -xz -C /tmp && \
    cp -r /tmp/lua-resty-jwt-0.2.3/lib/resty/* /usr/local/share/lua/5.1/resty/ && \
    rm -rf /tmp/lua-resty-jwt-0.2.3

# Install lua-resty-openidc
RUN wget -qO- https://github.com/zmartzone/lua-resty-openidc/archive/refs/tags/v1.7.6.tar.gz | tar -xz -C /tmp && \
    cp -r /tmp/lua-resty-openidc-1.7.6/lib/resty/* /usr/local/share/lua/5.1/resty/ && \
    rm -rf /tmp/lua-resty-openidc-1.7.6

# Install kong-oidc plugin
RUN git clone --branch v1.3.0 --depth 1 https://github.com/revomatico/kong-oidc.git /tmp/kong-oidc && \
    mkdir -p /usr/local/share/lua/5.1/kong/plugins/oidc && \
    cp -r /tmp/kong-oidc/kong/plugins/oidc/* /usr/local/share/lua/5.1/kong/plugins/oidc/ && \
    rm -rf /tmp/kong-oidc

# Switch back to kong user
USER kong

# The plugin will be available as 'oidc' (not 'openid-connect')
