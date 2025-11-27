import { create } from 'zustand';
import api, { loginApi } from '../api/client';

/**
 * Authentication Store (Zustand)
 * Manages user authentication state, login, logout, and session checking.
 * Uses localStorage to persist username across page reloads.
 */
export const useAuthStore = create((set) => ({
  isAuthenticated: false,
  user: null,
  loading: true,
  
  /**
   * Authenticates user with username and password.
   * Sends credentials as form data to /login endpoint.
   * Verifies success by attempting to access /agent endpoint.
   * @param {string} username - User's username
   * @param {string} password - User's password
   * @returns {Promise} - Resolves on successful login
   * @throws {Error} - Throws on authentication failure
   */
  login: async (username, password) => {
    try {
      const formData = new URLSearchParams();
      formData.append('username', username);
      formData.append('password', password);
      
      const response = await loginApi.post('/login', formData);
      
      // Verify login success by testing /agent endpoint
      try {
        await api.get('/agent');
        set({ isAuthenticated: true, user: { username } });
        localStorage.setItem('username', username);
        return response;
      } catch (error) {
        // If /agent fails, login actually failed
        localStorage.removeItem('username');
        throw new Error('Login failed');
      }
    } catch (error) {
      throw error;
    }
  },
  
  /**
   * Logs out the current user.
   * Clears authentication state and removes username from localStorage.
   */
  logout: async () => {
    try {
      await api.get('/logout');
      set({ isAuthenticated: false, user: null });
      localStorage.removeItem('username');
    } catch (error) {
      console.error('Logout error:', error);
      set({ isAuthenticated: false, user: null });
      localStorage.removeItem('username');
    }
  },
  
  /**
   * Checks if the user is currently authenticated.
   * Attempts to access /agent endpoint to verify session validity.
   * Updates authentication state based on response.
   */
  checkAuth: async () => {
    try {
      // Try to access /agent to verify authentication
      const response = await api.get('/agent');
      
      // If redirect received (307), user is not authenticated
      if (response.status === 307 || response.status === 302 || response.status === 301) {
        console.log('checkAuth - received redirect, not authenticated');
        set({ isAuthenticated: false, user: null, loading: false });
        localStorage.removeItem('username');
        return;
      }
      
      // If we get here with 200, user is authenticated
      const username = localStorage.getItem('username') || 'User';
      set({ isAuthenticated: true, user: { username }, loading: false });
    } catch (error) {
      // If request fails (network error, 401, etc), user is not logged in
      set({ isAuthenticated: false, user: null, loading: false });
      localStorage.removeItem('username');
    }
  },
}));

