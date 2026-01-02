/// <reference types="vite/client" />

// Runtime environment configuration loaded from env-config.js
interface WindowEnv {
  API_URL: string;
  AUTHENTIK_URL: string;
  OAUTH_CLIENT_ID: string;
  OAUTH_REDIRECT_URI: string;
}

declare interface Window {
  ENV: WindowEnv;
}
