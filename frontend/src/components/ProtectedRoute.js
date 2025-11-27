import React from 'react';
import { Navigate } from 'react-router-dom';

/**
 * ProtectedRoute Component
 * A wrapper component that protects routes from unauthenticated access.
 * Redirects to login page if user is not authenticated.
 * Shows loading state while authentication is being checked.
 * 
 * @param {boolean} isAuthenticated - Whether the user is currently authenticated
 * @param {boolean} loading - Whether authentication check is in progress
 * @param {React.ReactNode} children - The protected content to render
 */
const ProtectedRoute = ({ isAuthenticated, loading, children }) => {
  // Show loading indicator while checking authentication
  if (loading) {
    return <div className="loading">Loading...</div>;
  }

  // Redirect to login if not authenticated
  if (!isAuthenticated) {
    return <Navigate to="/login" replace />;
  }

  // Render protected content if authenticated
  return children;
};

export default ProtectedRoute;
