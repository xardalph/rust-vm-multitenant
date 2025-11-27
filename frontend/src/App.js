import React, { useEffect } from 'react';
import { BrowserRouter as Router, Routes, Route, Navigate } from 'react-router-dom';
import { useAuthStore } from './store/authStore';
import Login from './pages/Login';
import Dashboard from './pages/Dashboard';
import Metrics from './pages/Metrics';
import ProtectedRoute from './components/ProtectedRoute';
import './App.css';

/**
 * App Component - Main Application Entry Point
 * Handles routing and authentication state management.
 * Routes are protected using ProtectedRoute wrapper for authenticated-only access.
 * 
 * Routes:
 * - /login: Public login page
 * - /dashboard: Protected main dashboard with agent management
 * - /metrics: Protected metrics visualization page
 * - /: Redirects to dashboard (authenticated) or login (unauthenticated)
 */
function App() {
  const { isAuthenticated, loading, checkAuth } = useAuthStore();

  // Check authentication status on app mount
  useEffect(() => {
    checkAuth();
  }, []);

  // Show loading state while checking authentication
  if (loading) {
    return <div className="loading">Loading...</div>;
  }

  return (
    <Router>
      <Routes>
        {/* Public route - Login page */}
        <Route path="/login" element={<Login />} />
        
        {/* Protected route - Dashboard */}
        <Route
          path="/dashboard"
          element={
            <ProtectedRoute isAuthenticated={isAuthenticated} loading={loading}>
              <Dashboard />
            </ProtectedRoute>
          }
        />
        
        {/* Protected route - Metrics visualization */}
        <Route
          path="/metrics"
          element={
            <ProtectedRoute isAuthenticated={isAuthenticated} loading={loading}>
              <Metrics />
            </ProtectedRoute>
          }
        />
        
        {/* Root redirect - Go to dashboard if authenticated, otherwise login */}
        <Route 
          path="/" 
          element={loading ? <div className="loading">Loading...</div> : <Navigate to={isAuthenticated ? '/dashboard' : '/login'} replace />} 
        />
      </Routes>
    </Router>
  );
}

export default App;
