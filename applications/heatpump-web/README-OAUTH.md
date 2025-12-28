# OAuth2 Authentication Setup

This application uses OAuth2 Authorization Code Flow with PKCE to authenticate with Authentik.

## Architecture

```
User Browser (heatpump-web)
      ↓ 1. Click "Sign in"
      ↓
Authentik (authentik.k12n.com)
      ↓ 2. Login page
      ↓ 3. User enters credentials
      ↓ 4. Redirect to callback with code
      ↓
heatpump-web (/auth/callback)
      ↓ 5. Exchange code for token
      ↓ 6. Store access token
      ↓
Kong API Gateway (api.k12n.com)
      ↓ 7. Validate token with Authentik
      ↓ 8. Forward request to backend
      ↓
Backend Service (redpanda-sink, etc.)
```

## Configuration

### 1. Authentik Application Setup

Create an OAuth2 provider in Authentik:

1. Navigate to **Applications** → **Providers** → **Create**
2. Select **OAuth2/OpenID Provider**

**Provider Settings:**
- **Name**: `Heatpump Web`
- **Authorization flow**: `default-provider-authorization-implicit-consent`
- **Client type**: `Public` (SPA, no client secret)
- **Client ID**: `heatpump-web`
- **Redirect URIs**:
  - Production: `https://heatpump.k12n.com/auth/callback`
  - Local dev: `http://localhost:5173/auth/callback`
- **PKCE**: Required
- **Scopes**: `openid profile email read:energy read:heatpump read:settings write:settings`

3. Create **Application**:
   - **Name**: `Heatpump Dashboard`
   - **Slug**: `heatpump-web`
   - **Provider**: Select the provider created above
   - **Launch URL**: `https://heatpump.k12n.com`

### 2. Environment Variables

Create `.env` file (for local development):

```bash
VITE_API_URL=http://localhost:8000
VITE_AUTHENTIK_URL=http://localhost:9000
VITE_OAUTH_CLIENT_ID=heatpump-web
VITE_OAUTH_REDIRECT_URI=http://localhost:5173/auth/callback
```

For production, these are set in `.env.production`.

### 3. CORS Configuration

Authentik must allow the frontend origin:

- Navigate to **System** → **Settings** → **CORS**
- Add origin: `https://heatpump.k12n.com` (production)
- Add origin: `http://localhost:5173` (development)

## OAuth2 Flow

### Authorization Request

When user clicks "Sign in", the app redirects to:

```
https://authentik.k12n.com/application/o/authorize/?
  response_type=code&
  client_id=heatpump-web&
  redirect_uri=https://heatpump.k12n.com/auth/callback&
  scope=openid profile email read:energy&
  state=<random-state>&
  code_challenge=<pkce-challenge>&
  code_challenge_method=S256
```

### Token Exchange

After authentication, Authentik redirects to callback with `code` parameter.
The app exchanges the code for an access token:

```
POST https://authentik.k12n.com/application/o/token/
Content-Type: application/x-www-form-urlencoded

grant_type=authorization_code&
code=<authorization-code>&
redirect_uri=https://heatpump.k12n.com/auth/callback&
client_id=heatpump-web&
code_verifier=<pkce-verifier>
```

Response:
```json
{
  "access_token": "...",
  "token_type": "Bearer",
  "expires_in": 86400,
  "refresh_token": "...",
  "scope": "openid profile email read:energy"
}
```

### Token Storage

Tokens are stored in `localStorage`:
- `oauth_access_token`: Access token
- `oauth_refresh_token`: Refresh token (if provided)
- `oauth_token_expiry`: Expiry timestamp
- `oauth_user_info`: User information from UserInfo endpoint

### API Requests

All API requests include the access token:

```
GET https://api.k12n.com/api/v1/energy/latest
Authorization: Bearer <access-token>
```

Kong validates the token with Authentik's introspection endpoint and forwards
the request to the backend with authenticated user headers.

## Token Refresh

The app automatically refreshes tokens when:
- Token expires in less than 10 minutes
- API returns 401 Unauthorized

Refresh request:
```
POST https://authentik.k12n.com/application/o/token/
Content-Type: application/x-www-form-urlencoded

grant_type=refresh_token&
refresh_token=<refresh-token>&
client_id=heatpump-web
```

## Logout

When user logs out:
1. Clear `localStorage` tokens
2. Redirect to Authentik logout endpoint:

```
https://authentik.k12n.com/application/o/heatpump-web/end-session/?
  post_logout_redirect_uri=https://heatpump.k12n.com
```

## Security Features

- **PKCE (Proof Key for Code Exchange)**: Prevents authorization code interception
- **State parameter**: Prevents CSRF attacks
- **Secure token storage**: Tokens in localStorage (consider httpOnly cookies for production)
- **Automatic token refresh**: Reduces re-authentication
- **Token expiry buffer**: Refreshes before expiration

## Development

```bash
# Install dependencies
npm install

# Run development server
npm run dev

# Build for production
npm run build
```

## Testing Locally

1. Port-forward Authentik:
   ```bash
   kubectl port-forward -n authentik svc/authentik-server 9000:9000
   ```

2. Port-forward Kong:
   ```bash
   kubectl port-forward -n kong svc/kong-proxy 8000:8000
   ```

3. Start frontend:
   ```bash
   npm run dev
   ```

4. Visit http://localhost:5173 and click "Sign in"

## Troubleshooting

### "Invalid redirect_uri"
- Check that redirect URI in Authentik matches exactly (including trailing slash)
- Verify VITE_OAUTH_REDIRECT_URI environment variable

### "Invalid state parameter"
- Clear browser sessionStorage
- Check for CSRF protection issues

### "Failed to fetch user info"
- Verify token is valid
- Check Authentik UserInfo endpoint: `/application/o/userinfo/`

### API returns 401
- Check token expiration
- Verify Kong OIDC plugin configuration
- Check Authentik introspection endpoint
