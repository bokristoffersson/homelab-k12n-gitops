/**
 * OAuth2 Authorization Code Flow with PKCE
 * Integrates with Authentik for authentication
 * Uses runtime configuration from window.ENV (loaded from env-config.js)
 */

const AUTHENTIK_URL = window.ENV?.AUTHENTIK_URL || 'https://authentik.k12n.com';
const CLIENT_ID = window.ENV?.OAUTH_CLIENT_ID || 'heatpump-web';
const REDIRECT_URI = window.ENV?.OAUTH_REDIRECT_URI || `${window.location.origin}/auth/callback`;
const SCOPES = 'openid profile email read:energy read:heatpump read:settings write:settings';

const TOKEN_KEY = 'oauth_access_token';
const REFRESH_TOKEN_KEY = 'oauth_refresh_token';
const TOKEN_EXPIRY_KEY = 'oauth_token_expiry';
const USER_INFO_KEY = 'oauth_user_info';

interface TokenResponse {
  access_token: string;
  token_type: string;
  expires_in: number;
  refresh_token?: string;
  scope: string;
}

interface UserInfo {
  sub: string;
  email: string;
  name?: string;
  preferred_username?: string;
}

/**
 * Generate cryptographically random string for PKCE
 */
function generateRandomString(length: number): string {
  const array = new Uint8Array(length);
  crypto.getRandomValues(array);
  return Array.from(array, byte => byte.toString(16).padStart(2, '0')).join('');
}

/**
 * Generate PKCE code verifier and challenge
 */
async function generatePKCE() {
  const codeVerifier = generateRandomString(32);
  const encoder = new TextEncoder();
  const data = encoder.encode(codeVerifier);
  const hash = await crypto.subtle.digest('SHA-256', data);
  const codeChallenge = btoa(String.fromCharCode(...new Uint8Array(hash)))
    .replace(/\+/g, '-')
    .replace(/\//g, '_')
    .replace(/=/g, '');

  return { codeVerifier, codeChallenge };
}

export const oauthService = {
  /**
   * Start OAuth2 authorization flow
   * Redirects user to Authentik login page
   */
  async login() {
    const { codeVerifier, codeChallenge } = await generatePKCE();
    const state = generateRandomString(16);

    // Store PKCE verifier and state for later validation
    sessionStorage.setItem('oauth_code_verifier', codeVerifier);
    sessionStorage.setItem('oauth_state', state);

    const authUrl = new URL(`${AUTHENTIK_URL}/application/o/authorize/`);
    authUrl.searchParams.set('response_type', 'code');
    authUrl.searchParams.set('client_id', CLIENT_ID);
    authUrl.searchParams.set('redirect_uri', REDIRECT_URI);
    authUrl.searchParams.set('scope', SCOPES);
    authUrl.searchParams.set('state', state);
    authUrl.searchParams.set('code_challenge', codeChallenge);
    authUrl.searchParams.set('code_challenge_method', 'S256');

    // Redirect to Authentik
    window.location.href = authUrl.toString();
  },

  /**
   * Handle OAuth2 callback
   * Exchange authorization code for access token
   */
  async handleCallback(code: string, state: string): Promise<UserInfo> {
    // Validate state to prevent CSRF
    const storedState = sessionStorage.getItem('oauth_state');
    if (!storedState || state !== storedState) {
      throw new Error('Invalid state parameter - possible CSRF attack');
    }

    const codeVerifier = sessionStorage.getItem('oauth_code_verifier');
    if (!codeVerifier) {
      throw new Error('Missing code verifier');
    }

    // Exchange code for token
    const tokenUrl = `${AUTHENTIK_URL}/application/o/token/`;
    const body = new URLSearchParams({
      grant_type: 'authorization_code',
      code,
      redirect_uri: REDIRECT_URI,
      client_id: CLIENT_ID,
      code_verifier: codeVerifier,
    });

    const response = await fetch(tokenUrl, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/x-www-form-urlencoded',
      },
      body: body.toString(),
    });

    if (!response.ok) {
      const error = await response.text();
      throw new Error(`Token exchange failed: ${error}`);
    }

    const tokenData: TokenResponse = await response.json();

    // Store tokens
    localStorage.setItem(TOKEN_KEY, tokenData.access_token);
    if (tokenData.refresh_token) {
      localStorage.setItem(REFRESH_TOKEN_KEY, tokenData.refresh_token);
    }

    // Calculate and store expiry time
    const expiryTime = Date.now() + (tokenData.expires_in * 1000);
    localStorage.setItem(TOKEN_EXPIRY_KEY, expiryTime.toString());

    // Fetch user info
    const userInfo = await this.fetchUserInfo(tokenData.access_token);
    localStorage.setItem(USER_INFO_KEY, JSON.stringify(userInfo));

    // Clean up session storage
    sessionStorage.removeItem('oauth_code_verifier');
    sessionStorage.removeItem('oauth_state');

    return userInfo;
  },

  /**
   * Fetch user information from Authentik
   */
  async fetchUserInfo(accessToken: string): Promise<UserInfo> {
    const userInfoUrl = `${AUTHENTIK_URL}/application/o/userinfo/`;
    const response = await fetch(userInfoUrl, {
      headers: {
        'Authorization': `Bearer ${accessToken}`,
      },
    });

    if (!response.ok) {
      throw new Error('Failed to fetch user info');
    }

    return response.json();
  },

  /**
   * Refresh access token using refresh token
   */
  async refreshToken(): Promise<boolean> {
    const refreshToken = localStorage.getItem(REFRESH_TOKEN_KEY);
    if (!refreshToken) {
      return false;
    }

    try {
      const tokenUrl = `${AUTHENTIK_URL}/application/o/token/`;
      const body = new URLSearchParams({
        grant_type: 'refresh_token',
        refresh_token: refreshToken,
        client_id: CLIENT_ID,
      });

      const response = await fetch(tokenUrl, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/x-www-form-urlencoded',
        },
        body: body.toString(),
      });

      if (!response.ok) {
        return false;
      }

      const tokenData: TokenResponse = await response.json();

      // Update stored tokens
      localStorage.setItem(TOKEN_KEY, tokenData.access_token);
      if (tokenData.refresh_token) {
        localStorage.setItem(REFRESH_TOKEN_KEY, tokenData.refresh_token);
      }

      const expiryTime = Date.now() + (tokenData.expires_in * 1000);
      localStorage.setItem(TOKEN_EXPIRY_KEY, expiryTime.toString());

      return true;
    } catch (error) {
      console.error('Token refresh failed:', error);
      return false;
    }
  },

  /**
   * Logout user
   * Clears local storage and redirects to Authentik logout
   */
  async logout() {
    const token = localStorage.getItem(TOKEN_KEY);

    // Clear local storage
    localStorage.removeItem(TOKEN_KEY);
    localStorage.removeItem(REFRESH_TOKEN_KEY);
    localStorage.removeItem(TOKEN_EXPIRY_KEY);
    localStorage.removeItem(USER_INFO_KEY);

    // Redirect to Authentik logout
    if (token) {
      const logoutUrl = new URL(`${AUTHENTIK_URL}/application/o/heatpump-web/end-session/`);
      logoutUrl.searchParams.set('post_logout_redirect_uri', window.location.origin);
      window.location.href = logoutUrl.toString();
    } else {
      window.location.href = '/login';
    }
  },

  /**
   * Get current access token
   */
  getToken(): string | null {
    return localStorage.getItem(TOKEN_KEY);
  },

  /**
   * Get stored user info
   */
  getUserInfo(): UserInfo | null {
    const userInfoStr = localStorage.getItem(USER_INFO_KEY);
    if (!userInfoStr) return null;

    try {
      return JSON.parse(userInfoStr);
    } catch {
      return null;
    }
  },

  /**
   * Check if user is authenticated
   */
  isAuthenticated(): boolean {
    const token = localStorage.getItem(TOKEN_KEY);
    const expiryStr = localStorage.getItem(TOKEN_EXPIRY_KEY);

    if (!token || !expiryStr) {
      return false;
    }

    const expiry = parseInt(expiryStr, 10);
    const now = Date.now();

    // Consider token expired if less than 5 minutes remaining
    const bufferTime = 5 * 60 * 1000;
    return now < (expiry - bufferTime);
  },

  /**
   * Check if token needs refresh
   */
  needsRefresh(): boolean {
    const expiryStr = localStorage.getItem(TOKEN_EXPIRY_KEY);
    if (!expiryStr) return false;

    const expiry = parseInt(expiryStr, 10);
    const now = Date.now();

    // Refresh if less than 10 minutes remaining
    const refreshThreshold = 10 * 60 * 1000;
    return now > (expiry - refreshThreshold);
  },

  /**
   * Initialize OAuth service
   * Check token validity and refresh if needed
   */
  async init() {
    if (this.isAuthenticated()) {
      if (this.needsRefresh()) {
        const refreshed = await this.refreshToken();
        if (!refreshed) {
          // Refresh failed, clear tokens
          this.logout();
        }
      }
    }
  },
};
