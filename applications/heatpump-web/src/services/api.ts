import axios from 'axios';

// Use runtime configuration from window.ENV (loaded from env-config.js)
const API_BASE_URL = window.ENV?.API_URL || 'https://heatpump.k12n.com';

export const api = axios.create({
  baseURL: API_BASE_URL,
  timeout: 10000,
  withCredentials: true,  // Send cookies with requests
});

// Handle 401 responses by redirecting to OIDC login
api.interceptors.response.use(
  (response) => response,
  (error) => {
    if (error.response?.status === 401) {
      // Store current path for redirect after auth
      sessionStorage.setItem('auth_redirect', window.location.pathname);
      // Redirect to auth login to trigger OIDC flow
      window.location.href = '/auth/login';
      return new Promise(() => {}); // Prevent error propagation during redirect
    }
    return Promise.reject(error);
  }
);



