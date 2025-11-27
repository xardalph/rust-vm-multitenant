/**
 * Application Entry Point
 * This file initializes the React application and renders it to the DOM.
 * Uses React 18's createRoot API for concurrent rendering support.
 */
import React from 'react';
import ReactDOM from 'react-dom/client';
import './index.css';
import App from './App';

// Create root and render the App component with StrictMode enabled
// StrictMode helps detect potential problems during development
const root = ReactDOM.createRoot(document.getElementById('root'));
root.render(
  <React.StrictMode>
    <App />
  </React.StrictMode>
);
