# Development Guide

## Prerequisites

- Node.js 20+ and npm
- `kubectl` access to the cluster, to reach the API through a port-forward

## Setup

1. Clone the repository:
```bash
git clone https://github.com/bokristoffersson/homelab-k12n-gitops.git
cd applications/heatpump-web
```

2. Install dependencies:
```bash
npm install
```

3. Forward the API from the cluster in a second terminal:
```bash
kubectl port-forward -n homelab-api svc/homelab-api 8080:8080
```

4. Create a `.env.local` file:
```env
VITE_API_URL=http://localhost:8080
VITE_AUTHENTIK_URL=https://auth.k12n.com
VITE_OAUTH_CLIENT_ID=heatpump-web
VITE_OAUTH_REDIRECT_URI=http://localhost:5173/auth/callback
```

`VITE_AUTHENTIK_URL` is the OIDC provider URL. The name predates the move to Authelia.
The redirect URI above is registered for the `heatpump-web` client, and
`http://localhost:5173` is in the API's CORS allow-list.

5. Start the development server:
```bash
npm run dev
```

The application will be available at `http://localhost:5173`.

## Project Structure

```
heatpump-web/
├── src/
│   ├── components/
│   │   └── Dashboard/
│   │       ├── Dashboard.tsx
│   │       ├── PowerGauge.tsx
│   │       ├── HourlyChart.tsx
│   │       └── TemperatureChart.tsx
│   ├── services/
│   │   └── api.ts
│   ├── types/
│   │   ├── energy.ts
│   │   └── heatpump.ts
│   └── App.tsx
├── public/
│   └── env-config.js (generated at runtime)
├── Dockerfile
├── nginx.conf
└── package.json
```

## Building

Build the production bundle:
```bash
npm run build
```

The output will be in the `dist/` directory.

## Linting

Run ESLint:
```bash
npm run lint
```

## Docker Build

Build the multi-arch Docker image:
```bash
docker buildx build --platform linux/amd64,linux/arm64 -t heatpump-web:dev .
```

## CI/CD

`.github/workflows/heatpump-web.yml` runs on every PR and on `main`:
- `npm audit`, lint and a type-checked build
- on `main`, a multi-arch image pushed to GHCR as `:main`
