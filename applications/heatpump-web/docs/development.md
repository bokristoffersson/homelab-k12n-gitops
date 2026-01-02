# Development Guide

## Prerequisites

- Node.js 20+
- npm or yarn
- Access to homelab-api backend (for development)

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

3. Create a `.env.local` file for development:
```env
VITE_API_URL=http://localhost:8000
VITE_AUTHENTIK_URL=https://authentik.k12n.com
VITE_OAUTH_CLIENT_ID=heatpump-web
VITE_OAUTH_REDIRECT_URI=http://localhost:3000/auth/callback
```

4. Start the development server:
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
docker buildx build --platform linux/amd64,linux/arm64 -t heatpump-web:latest .
```

## CI/CD

GitHub Actions automatically builds and publishes Docker images to GHCR on push to main:
- Runs lint checks
- Builds TypeScript
- Builds and pushes multi-arch Docker image

See `.github/workflows/heatpump-web.yml` for details.
