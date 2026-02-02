import axios from 'axios';

// Use runtime configuration from window.ENV (loaded from env-config.js)
const API_BASE_URL = window.ENV?.API_URL || 'https://heatpump.k12n.com';

export const api = axios.create({
  baseURL: API_BASE_URL,
  timeout: 10000,
  withCredentials: true,  // Send cookies with requests
});

// Simple response interceptor to handle 401
// oauth2-proxy will redirect to Authentik login page automatically
api.interceptors.response.use(
  (response) => response,
  (error) => {
    if (error.response?.status === 401) {
      // Redirect to login (oauth2-proxy will handle this)
      window.location.href = '/oauth2/sign_in';
    }
    return Promise.reject(error);
  }
);

// Authentication is handled by oauth2-proxy via HTTP-only cookies
// No client-side OAuth2 code needed
