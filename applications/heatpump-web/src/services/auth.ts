import { api } from './api';

export interface LoginCredentials {
  username: string;
  password: string;
}

export interface LoginResponse {
  token: string;
  username: string;
  expires_in: number;
}

const TOKEN_KEY = 'heatpump_auth_token';
const USERNAME_KEY = 'heatpump_username';

export const authService = {
  login: async (credentials: LoginCredentials): Promise<LoginResponse> => {
    const response = await api.post<LoginResponse>('/api/v1/auth/login', credentials);
    const { token, username } = response.data;
    
    localStorage.setItem(TOKEN_KEY, token);
    localStorage.setItem(USERNAME_KEY, username);
    // Token will be automatically added by request interceptor in api.ts
    
    return response.data;
  },
  
  logout: () => {
    localStorage.removeItem(TOKEN_KEY);
    localStorage.removeItem(USERNAME_KEY);
    // Token will be automatically removed from requests by interceptor
  },
  
  getToken: (): string | null => localStorage.getItem(TOKEN_KEY),
  getUsername: (): string | null => localStorage.getItem(USERNAME_KEY),
  
  isAuthenticated: (): boolean => {
    const token = localStorage.getItem(TOKEN_KEY);
    if (!token) return false;
    
    try {
      const payload = JSON.parse(atob(token.split('.')[1]));
      return Date.now() < payload.exp * 1000;
    } catch {
      return false;
    }
  },
  
  init: () => {
    // Token will be automatically added by request interceptor in api.ts
    // This method is kept for potential future initialization logic
  },
};



