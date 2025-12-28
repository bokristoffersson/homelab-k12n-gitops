import axios from 'axios';
import { oauthService } from './oauth';

const API_BASE_URL = import.meta.env.VITE_API_URL || 'https://api.k12n.com';

export const api = axios.create({
  baseURL: API_BASE_URL,
  timeout: 10000,
});

// Add request interceptor to add OAuth access token from localStorage
api.interceptors.request.use(
  async (config) => {
    // Check if token needs refresh before making request
    if (oauthService.needsRefresh()) {
      await oauthService.refreshToken();
    }

    const token = oauthService.getToken();
    if (token) {
      config.headers.Authorization = `Bearer ${token}`;
    }
    return config;
  },
  (error) => {
    return Promise.reject(error);
  }
);

// Add response interceptor to handle 401
api.interceptors.response.use(
  (response) => response,
  async (error) => {
    if (error.response?.status === 401) {
      // Try to refresh token once
      const refreshed = await oauthService.refreshToken();

      if (refreshed && error.config) {
        // Retry the original request with new token
        const token = oauthService.getToken();
        if (token) {
          error.config.headers.Authorization = `Bearer ${token}`;
        }
        return axios.request(error.config);
      }

      // Refresh failed or no config, logout
      oauthService.logout();
    }
    return Promise.reject(error);
  }
);

// OAuth initialization is handled in App.tsx



