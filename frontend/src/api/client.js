import axios from 'axios';

// URL du backend - via le proxy Nginx sur le même port
const API_BASE_URL = '';

const api = axios.create({
  baseURL: API_BASE_URL,
  withCredentials: true,
  maxRedirects: 0,
  headers: {
    'Content-Type': 'application/json',
  },
});

// Client séparé pour le login (form data)
export const loginApi = axios.create({
  baseURL: API_BASE_URL,
  withCredentials: true,
});

// Interceptor response pour gérer les erreurs
api.interceptors.response.use(
  (response) => response,
  (error) => {
    // Ne pas rediriger ici, laisser chaque composant gérer
    return Promise.reject(error);
  }
);

loginApi.interceptors.response.use(
  (response) => response,
  (error) => {
    if (error.response?.status === 401) {
      window.location.href = '/login';
    }
    return Promise.reject(error);
  }
);

export default api;
