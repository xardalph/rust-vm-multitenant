/**
 * Development Proxy Configuration
 * This file configures the development server to proxy API requests
 * to the backend server at localhost:3000.
 * 
 * In production, nginx handles these proxies instead.
 * This setup avoids CORS issues during local development.
 */
const { createProxyMiddleware } = require('http-proxy-middleware');

/**
 * Configure proxy middleware for development server.
 * Routes /login, /logout, /agent, and /select to backend API.
 * @param {Express} app - Express application instance
 */
module.exports = function(app) {
  // Proxy authentication login endpoint
  app.use(
    '/login',
    createProxyMiddleware({
      target: 'http://localhost:3000',
      changeOrigin: true,
      pathRewrite: {
        '^/login': '/login',
      },
    })
  );

  // Proxy logout endpoint
  app.use(
    '/logout',
    createProxyMiddleware({
      target: 'http://localhost:3000',
      changeOrigin: true,
      pathRewrite: {
        '^/logout': '/logout',
      },
    })
  );

  // Proxy agent management endpoints
  app.use(
    '/agent',
    createProxyMiddleware({
      target: 'http://localhost:3000',
      changeOrigin: true,
      pathRewrite: {
        '^/agent': '/agent',
      },
    })
  );

  // Proxy VictoriaMetrics query endpoints
  app.use(
    '/select',
    createProxyMiddleware({
      target: 'http://localhost:3000',
      changeOrigin: true,
      pathRewrite: {
        '^/select': '/select',
      },
    })
  );
};
