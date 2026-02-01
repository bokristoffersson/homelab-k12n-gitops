import axios from 'axios';

// Use runtime configuration from window.ENV (loaded from env-config.js)
const API_BASE_URL = window.ENV?.API_URL || 'https://heatpump.k12n.com';

export const api = axios.create({
  baseURL: API_BASE_URL,
  timeout: 10000,
  withCredentials: true,  // Send cookies with requests
});

// Response interceptor to handle 401 - redirect to trigger OIDC flow
// traefikoidc middleware handles authentication via session cookies
api.interceptors.response.use(
  (response) => response,
  (error) => {
    if (error.response?.status === 401) {
      // Store current location for post-login redirect
      sessionStorage.setItem('auth_redirect', window.location.pathname);
      // Redirect to /auth/login which is protected by traefikoidc
      // This triggers the OIDC flow with Authentik
      window.location.href = '/auth/login';
    }
    return Promise.reject(error);
  }
);

// Authentication is handled by traefikoidc middleware via session cookies
// No client-side OAuth2 code needed



