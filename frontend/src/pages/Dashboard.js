import React, { useEffect, useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { useAuthStore } from '../store/authStore';
import api from '../api/client';
import './Dashboard.css';

/**
 * Dashboard Component
 * Main dashboard page displaying agents list and providing CRUD operations.
 * Allows users to create, view, and delete monitoring agents.
 */
const Dashboard = () => {
  const navigate = useNavigate();
  const { user, logout } = useAuthStore();
  const [agents, setAgents] = useState([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState('');
  const [showCreateForm, setShowCreateForm] = useState(false);
  const [newAgentName, setNewAgentName] = useState('');
  const [newAgentToken, setNewAgentToken] = useState('');
  const [isCreating, setIsCreating] = useState(false);
  const [copiedToken, setCopiedToken] = useState(null);

  /**
   * Copies the agent token to clipboard and shows feedback.
   * @param {string} token - The agent token to copy
   * @param {string} agentId - The agent ID for tracking which token was copied
   */
  const copyTokenToClipboard = async (token, agentId) => {
    try {
      await navigator.clipboard.writeText(token);
      setCopiedToken(agentId);
      setTimeout(() => setCopiedToken(null), 2000);
    } catch (err) {
      console.error('Failed to copy token:', err);
    }
  };

  // Fetch agents on component mount
  useEffect(() => {
    fetchAgents();
  }, []);

  /**
   * Fetches the list of agents from the backend API.
   * Redirects to login page if authentication fails (307 or 401).
   */
  const fetchAgents = async () => {
    try {
      setLoading(true);
      const response = await api.get('/agent');
      // Check if response is an array, otherwise convert it
      const agentsList = Array.isArray(response.data) ? response.data : [];
      setAgents(agentsList);
      setError('');
    } catch (err) {
      console.error('Error fetching agents:', err);
      // If 307 or 401, redirect to login
      if (err.response?.status === 307 || err.response?.status === 401) {
        navigate('/login', { replace: true });
        return;
      }
      setError('Failed to load agents');
      setAgents([]);
    } finally {
      setLoading(false);
    }
  };

  /**
   * Handles the creation of a new agent.
   * Generates a random token if not provided by the user.
   * @param {Event} e - Form submit event
   */
  const handleCreateAgent = async (e) => {
    e.preventDefault();
    if (!newAgentName.trim()) return;

    setIsCreating(true);
    try {
      // Generate random token if not provided
      const token = newAgentToken.trim() || 'token_' + Math.random().toString(36).substr(2, 9);
      
      await api.post('/agent', { 
        name: newAgentName,
        token: token
      });
      setNewAgentName('');
      setNewAgentToken('');
      setShowCreateForm(false);
      fetchAgents();
    } catch (err) {
      setError(err.response?.data?.message || 'Failed to create agent');
    } finally {
      setIsCreating(false);
    }
  };

  /**
   * Handles agent deletion with confirmation dialog.
   * @param {string} agentId - The ID of the agent to delete
   * @param {string} agentName - The name of the agent (for confirmation message)
   */
  const handleDeleteAgent = async (agentId, agentName) => {
    if (!window.confirm(`Delete agent "${agentName}"?`)) return;

    try {
      await api.delete(`/agent/${agentId}`);
      fetchAgents();
    } catch (err) {
      setError(err.response?.data?.message || 'Failed to delete agent');
    }
  };

  /**
   * Handles user logout by calling the logout function from authStore.
   */
  const handleLogout = async () => {
    await logout();
  };

  return (
    <div className="dashboard">
      <nav className="navbar">
        <div className="navbar-content">
          <h1>NoSQL Rust Monitoring</h1>
          <div className="navbar-right">
            <span className="username">Welcome, {user?.username}</span>
            <button className="logout-btn" onClick={handleLogout}>
              Logout
            </button>
          </div>
        </div>
      </nav>

      <main className="dashboard-content">
        <div className="header">
          <h2>Dashboard</h2>
          <button
            className="create-btn"
            onClick={() => setShowCreateForm(!showCreateForm)}
          >
            {showCreateForm ? 'Cancel' : '+ Create Agent'}
          </button>
        </div>

        {error && (
          <div className="error-banner">
            <p>{error}</p>
            <button className="close-btn" onClick={() => setError('')}>×</button>
          </div>
        )}

        {showCreateForm && (
          <div className="create-form">
            <h3>Create New Agent</h3>
            <form onSubmit={handleCreateAgent}>
              <div className="form-group">
                <label htmlFor="agent-name">Agent Name</label>
                <input
                  id="agent-name"
                  type="text"
                  value={newAgentName}
                  onChange={(e) => setNewAgentName(e.target.value)}
                  placeholder="Enter agent name (e.g., server-01)"
                  disabled={isCreating}
                  required
                />
              </div>
              <div className="form-group">
                <label htmlFor="agent-token">Agent Token (optional)</label>
                <input
                  id="agent-token"
                  type="text"
                  value={newAgentToken}
                  onChange={(e) => setNewAgentToken(e.target.value)}
                  placeholder="Leave empty for auto-generated token"
                  disabled={isCreating}
                />
              </div>
              <button type="submit" className="submit-btn" disabled={isCreating}>
                {isCreating ? 'Creating...' : 'Create Agent'}
              </button>
            </form>
          </div>
        )}

        <div className="stats-grid">
          <div className="stat-card">
            <h3>Active Agents</h3>
            <p className="stat-value">{agents.length}</p>
          </div>
          <div className="stat-card">
            <h3>Status</h3>
            <p className="stat-value" style={{ color: '#28a745' }}>Online</p>
          </div>
        </div>

        <div className="agents-section">
          <h3>Agents</h3>
          {loading ? (
            <div className="loading-message">Loading agents...</div>
          ) : agents.length === 0 ? (
            <div className="empty-state">
              <p>No agents created yet</p>
              <p className="empty-hint">Create your first agent to start monitoring</p>
            </div>
          ) : (
            <div className="agents-list">
              {agents.map((agent) => (
                <div key={agent.id} className="agent-card">
                  <div className="agent-header">
                    <div className="agent-info">
                      <h4>{agent.name}</h4>
                      <p className="agent-id">ID: {agent.id}</p>
                    </div>
                    <div className="agent-actions">
                      <button
                        className="metrics-btn-small"
                        onClick={() => navigate(`/metrics?agent=${encodeURIComponent(agent.name)}&agentId=${agent.id}`)}
                      >
                        Metrics
                      </button>
                      <button
                        className={`token-badge ${copiedToken === agent.id ? 'copied' : ''}`}
                        onClick={() => copyTokenToClipboard(agent.token, agent.id)}
                        title="Click to copy full token"
                      >
                        {copiedToken === agent.id ? 'Copied!' : `${agent.token?.substring(0, 8)}...`}
                      </button>
                      <button
                        className="delete-btn"
                        onClick={() => handleDeleteAgent(agent.id, agent.name)}
                      >
                        Delete
                      </button>
                    </div>
                  </div>
                  <div className="agent-footer">
                    <small>Created: {new Date(agent.created_at).toLocaleString()}</small>
                  </div>
                </div>
              ))}
            </div>
          )}
        </div>
      </main>
    </div>
  );
};

export default Dashboard;
