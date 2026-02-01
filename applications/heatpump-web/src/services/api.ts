import axios from 'axios';

// Use runtime configuration from window.ENV (loaded from env-config.js)
const API_BASE_URL = window.ENV?.API_URL || 'https://heatpump.k12n.com';

export const api = axios.create({
  baseURL: API_BASE_URL,
  timeout: 10000,
  withCredentials: true,  // Send cookies with requests
});

// TODO: Re-enable 401 redirect when traefikoidc is working
// For now, just let errors propagate to the UI
api.interceptors.response.use(
  (response) => response,
  (error) => {
    // Log auth errors for debugging
    if (error.response?.status === 401) {
      console.warn('API returned 401 - authentication required');
    }
    return Promise.reject(error);
  }
);



