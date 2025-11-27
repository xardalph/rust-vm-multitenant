import { create } from 'zustand';
import api, { loginApi } from '../api/client';

export const useAuthStore = create((set) => ({
  isAuthenticated: false,
  user: null,
  loading: true,
  
  login: async (username, password) => {
    try {
      const formData = new URLSearchParams();
      formData.append('username', username);
      formData.append('password', password);
      
      const response = await loginApi.post('/login', formData);
      
      // Vérifier que le login a réussi en testant /agent
      try {
        await api.get('/agent');
        set({ isAuthenticated: true, user: { username } });
        localStorage.setItem('username', username);
        return response;
      } catch (error) {
        // Si /agent échoue, c'est que le login a échoué
        localStorage.removeItem('username');
        throw new Error('Login failed');
      }
    } catch (error) {
      throw error;
    }
  },
  
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
  
  checkAuth: async () => {
    try {
      // Essayer d'accéder à /agent pour vérifier qu'on est connecté
      const response = await api.get('/agent');
      
      // Si on reçoit une redirection (307), on n'est pas authentifié
      if (response.status === 307 || response.status === 302 || response.status === 301) {
        ('checkAuth - received redirect, not authenticated');
        set({ isAuthenticated: false, user: null, loading: false });
        localStorage.removeItem('username');
        return;
      }
      
      // Si on arrive ici avec 200, on est authentifié
      const username = localStorage.getItem('username') || 'User';
      set({ isAuthenticated: true, user: { username }, loading: false });
    } catch (error) {
      // Si ça fail (erreur réseau, 401, etc), on n'est pas connecté
      set({ isAuthenticated: false, user: null, loading: false });
      localStorage.removeItem('username');
    }
  },
}));

