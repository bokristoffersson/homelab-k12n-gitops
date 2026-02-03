// Export all type definitions
export * from './energy';
export * from './heatpump';
export * from './plugs';

// API response types
export interface ApiResponse<T> {
  data: T;
  message?: string;
}

export interface ApiError {
  message: string;
  error?: string;
  statusCode?: number;
}

// Auth types (re-exported from auth service for convenience)
export interface LoginCredentials {
  username: string;
  password: string;
}

export interface LoginResponse {
  token: string;
  username: string;
  expires_in: number;
}
