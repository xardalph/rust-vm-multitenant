import axios from 'axios';

/**
 * API Client Configuration
 * Creates axios instances for API communication with the backend.
 * Uses relative URLs since Nginx proxies requests to the backend.
 */

// Backend URL - empty string means same origin (proxied via Nginx)
const API_BASE_URL = '';

/**
 * Main API client for authenticated requests.
 * Sends JSON content and includes credentials (cookies) for session management.
 * Disables automatic redirects to handle auth errors manually.
 */
const api = axios.create({
  baseURL: API_BASE_URL,
  withCredentials: true,
  maxRedirects: 0,
  headers: {
    'Content-Type': 'application/json',
  },
});

/**
 * Separate API client for login requests.
 * Used for form-urlencoded data (username/password).
 * Includes credentials for cookie-based session.
 */
export const loginApi = axios.create({
  baseURL: API_BASE_URL,
  withCredentials: true,
});

/**
 * Response interceptor for main API client.
 * Passes errors to components for handling (no automatic redirects).
 */
api.interceptors.response.use(
  (response) => response,
  (error) => {
    // Don't redirect here, let each component handle errors
    return Promise.reject(error);
  }
);

/**
 * Response interceptor for login API client.
 * Allows login component to display error messages without redirecting.
 */
loginApi.interceptors.response.use(
  (response) => response,
  (error) => {
    // Don't redirect on login - let the component handle the error
    return Promise.reject(error);
  }
);

export default api;
