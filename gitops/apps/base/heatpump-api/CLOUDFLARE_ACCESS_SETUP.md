# Cloudflare Access Setup for heatpump-api

The heatpump-api service is exposed via Cloudflare Tunnel at `api.heatpump.k12n.com`. To protect it with authentication, you need to configure Cloudflare Access in the Cloudflare Zero Trust dashboard.

## Quick Setup (5 minutes)

### 1. Create an Access Application

1. Go to [Cloudflare Zero Trust Dashboard](https://one.dash.cloudflare.com/)
2. Navigate to **Access** → **Applications**
3. Click **Add an application** → **Self-hosted**
4. Configure:
   - **Application name**: `heatpump-api`
   - **Application domain**: `api.heatpump.k12n.com`
   - **Session duration**: Choose based on your needs (e.g., 24 hours)

### 2. Set Up Authentication Policy

For a mobile app, you have two options:

#### Option A: Service Token (Recommended for Mobile Apps)

1. In the Access application settings, go to **Policies**
2. Click **Add a policy**
3. Configure:
   - **Policy name**: `heatpump-api-service-token`
   - **Action**: Allow
   - **Include**: Select **Service Token**
4. Click **Create Service Token**
5. Save the token securely - you'll use this in your mobile app
6. The token will be sent as a header: `CF-Access-Token: <your-token>`

#### Option B: Email-based Authentication

1. In the Access application settings, go to **Policies**
2. Click **Add a policy**
3. Configure:
   - **Policy name**: `heatpump-api-users`
   - **Action**: Allow
   - **Include**: Select **Emails** and add your email addresses
   - Or use **Email domain** if you have a domain

### 3. Configure Your Mobile App

#### Using Service Token (Option A)

Add the service token to your mobile app's API requests:

```http
GET https://api.heatpump.k12n.com/health
CF-Access-Token: <your-service-token>
```

#### Using Email Auth (Option B)

Users will be redirected to Cloudflare's login page on first access. After authentication, they'll receive a session cookie that your app can use.

## Testing

1. Try accessing `https://api.heatpump.k12n.com/health` without authentication - you should be blocked
2. With proper authentication (service token or logged-in session), you should get a successful response

## Security Notes

- Service tokens are long-lived and should be stored securely in your mobile app
- Consider rotating service tokens periodically
- For production, use environment-specific tokens
- Service tokens cannot be revoked individually - you must regenerate them

## Troubleshooting

- If you can't access the API, check that the tunnel is running: `kubectl get pods -n cloudflare-tunnel`
- Verify the route is configured: `kubectl get configmap cloudflared-config -n cloudflare-tunnel -o yaml`
- Check Cloudflare Access logs in the Zero Trust dashboard

