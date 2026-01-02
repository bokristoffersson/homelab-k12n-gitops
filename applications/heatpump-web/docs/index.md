# Heatpump Web

Heat pump dashboard web application for monitoring and visualizing real-time energy consumption and heatpump status.

## Overview

Heatpump Web is a React-based single-page application (SPA) that provides:

- **Live Power Monitoring**: Real-time energy consumption display via WebSocket
- **Heat Pump Status**: Current operating parameters and temperatures
- **Historical Data**: Energy consumption and temperature trends over 24 hours
- **OAuth2 Authentication**: Integrated with Authentik for secure access

## Features

- Real-time power gauge showing current energy consumption (max 17.5 kW)
- WebSocket connection for live data streaming
- Hourly energy consumption totals
- Heat pump operating temperatures
- Indoor temperature monitoring
- Dark/light theme toggle
- Responsive design for mobile and desktop

## Tech Stack

- **Frontend**: React 18, TypeScript, Vite
- **Charts**: Recharts
- **State Management**: React Query (TanStack Query)
- **Authentication**: OAuth2/OIDC (Authentik)
- **WebSocket**: Native WebSocket API with JWT authentication
- **Deployment**: Nginx, Docker, Kubernetes

## Quick Links

- [Architecture](architecture.md)
- [Development Guide](development.md)
- [Deployment Guide](deployment.md)
