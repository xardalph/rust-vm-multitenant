import React, { useEffect, useState, useCallback } from 'react';
import { useNavigate, useSearchParams } from 'react-router-dom';
import { useAuthStore } from '../store/authStore';
import api from '../api/client';
import {
  Chart as ChartJS,
  CategoryScale,
  LinearScale,
  PointElement,
  LineElement,
  Title,
  Tooltip,
  Legend,
  TimeScale,
} from 'chart.js';
import { Line } from 'react-chartjs-2';
import 'chartjs-adapter-date-fns';
import './Metrics.css';

// Register Chart.js components for time-series line charts
ChartJS.register(
  CategoryScale,
  LinearScale,
  PointElement,
  LineElement,
  Title,
  Tooltip,
  Legend,
  TimeScale
);

/**
 * Metrics Component
 * Displays metrics visualization with interactive charts from VictoriaMetrics.
 * Allows filtering by job, selecting multiple metrics, and adjusting time range.
 * Can be accessed for a specific agent or for all agents.
 */
const Metrics = () => {
  const navigate = useNavigate();
  const [searchParams] = useSearchParams();
  const { user, logout } = useAuthStore();
  
  // Get agent info from URL params (optional - for agent-specific views)
  const agentName = searchParams.get('agent');
  const agentId = searchParams.get('agentId');
  
  // State management
  const [availableMetrics, setAvailableMetrics] = useState([]);
  const [availableJobs, setAvailableJobs] = useState([]);
  const [selectedJob, setSelectedJob] = useState('');
  const [selectedMetrics, setSelectedMetrics] = useState([]);
  const [metricsData, setMetricsData] = useState({});
  const [loading, setLoading] = useState(false);
  const [loadingMetrics, setLoadingMetrics] = useState(true);
  const [error, setError] = useState('');
  const [timeRange, setTimeRange] = useState('1h');

  // Fetch available metrics and jobs on component mount or agent change
  useEffect(() => {
    fetchAvailableMetrics();
    fetchAvailableJobs();
  }, [agentId]);

  /**
   * Fetches metrics data from VictoriaMetrics export API.
   * Builds a PromQL query based on selected metrics and optional job filter.
   * Parses JSONL response and transforms it into chart-ready data.
   */
  const fetchMetricsData = useCallback(async () => {
    if (selectedMetrics.length === 0) {
      setMetricsData({});
      return;
    }

    setLoading(true);
    setError('');

    try {
      // Build PromQL query with optional job filter
      const metricsQuery = selectedMetrics.join('|');
      const query = selectedJob 
        ? `{__name__=~"${metricsQuery}",job="${selectedJob}"}`
        : `{__name__=~"${metricsQuery}"}`;
      
      // Use POST with form-urlencoded data (VictoriaMetrics format)
      const response = await api.post('/vm/export', 
        `match[]=${encodeURIComponent(query)}`,
        {
          headers: {
            'Content-Type': 'application/x-www-form-urlencoded',
          },
          transformResponse: [(data) => data], 
        }
      );

      // Parse JSONL response (one JSON object per line)
      const text = response.data?.trim() || '';
      if (!text) {
        setMetricsData({});
        return;
      }
      
      const lines = text.split('\n').filter(line => line.length > 0);
      const parsedData = {};

      // Transform each line into chart data format
      lines.forEach(line => {
        try {
          const data = JSON.parse(line);
          const metricName = data.metric.__name__;
          const instance = data.metric.instance || data.metric.job || 'unknown';
          const key = `${metricName}_${instance}`;
          
          parsedData[key] = {
            label: `${metricName} (${instance})`,
            metricName,
            instance,
            timestamps: data.timestamps || [],
            values: data.values || [],
          };
        } catch (e) {
          console.error('Error parsing line:', line, e);
        }
      });

      setMetricsData(parsedData);
    } catch (err) {
      console.error('Error fetching metrics data:', err);
      setError('Failed to fetch metrics data: ' + (err.response?.data || err.message));
    } finally {
      setLoading(false);
    }
  }, [selectedJob, selectedMetrics, timeRange]);

  // Auto-refresh metrics data every 10 seconds when metrics are selected
  useEffect(() => {
    if (selectedMetrics.length > 0) {
      fetchMetricsData();
      const interval = setInterval(fetchMetricsData, 10000);
      return () => clearInterval(interval);
    } else {
      setMetricsData({});
    }
  }, [selectedMetrics, selectedJob, timeRange, fetchMetricsData]);

  /**
   * Fetches available job labels from VictoriaMetrics.
   * Jobs represent different data sources (vmagent, vmselect, etc.).
   */
  const fetchAvailableJobs = async () => {
    try {
      const response = await api.get('/vm/label/job/values');
      if (response.data?.status === 'success' && response.data?.data) {
        setAvailableJobs(response.data.data);
      }
    } catch (err) {
      console.error('Error fetching jobs:', err);
    }
  };

  /**
   * Fetches available metric names from VictoriaMetrics.
   * These are the __name__ labels representing different metric types.
   */
  const fetchAvailableMetrics = async () => {
    setLoadingMetrics(true);
    try {
      const response = await api.get('/vm/label/__name__/values');
      console.log('Metrics response:', response.data);
      if (response.data?.status === 'success' && response.data?.data) {
        const metrics = response.data.data;
        console.log('Loaded metrics:', metrics.length);
        setAvailableMetrics(metrics);
      } else if (Array.isArray(response.data)) {
        setAvailableMetrics(response.data);
      }
    } catch (err) {
      console.error('Error fetching metrics:', err);
      setError('Failed to load metrics: ' + (err.message || 'Unknown error'));
    } finally {
      setLoadingMetrics(false);
    }
  };

  /**
   * Toggles a metric selection on/off.
   * @param {string} metric - The metric name to toggle
   */
  const toggleMetric = (metric) => {
    setSelectedMetrics(prev => 
      prev.includes(metric) 
        ? prev.filter(m => m !== metric)
        : [...prev, metric]
    );
  };

  /**
   * Transforms raw metric data into Chart.js compatible format.
   * @param {string} metricKey - The unique key for the metric
   * @returns {Object|null} - Chart.js data object or null if no data
   */
  const getChartData = (metricKey) => {
    const data = metricsData[metricKey];
    if (!data) return null;

    return {
      labels: data.timestamps.map(ts => new Date(ts)),
      datasets: [
        {
          label: data.label,
          data: data.values,
          borderColor: getRandomColor(metricKey),
          backgroundColor: getRandomColor(metricKey, 0.1),
          tension: 0.4,
          fill: true,
        },
      ],
    };
  };

  /**
   * Generates a consistent color based on a seed string.
   * Uses hash of the string to pick from a predefined color palette.
   * @param {string} seed - String to generate color from
   * @param {number} alpha - Opacity value (0-1)
   * @returns {string} - RGBA color string
   */
  const getRandomColor = (seed, alpha = 1) => {
    const colors = [
      `rgba(255, 99, 132, ${alpha})`,
      `rgba(54, 162, 235, ${alpha})`,
      `rgba(255, 206, 86, ${alpha})`,
      `rgba(75, 192, 192, ${alpha})`,
      `rgba(153, 102, 255, ${alpha})`,
      `rgba(255, 159, 64, ${alpha})`,
    ];
    const hash = seed.split('').reduce((acc, char) => acc + char.charCodeAt(0), 0);
    return colors[hash % colors.length];
  };

  // Chart.js configuration options
  const chartOptions = {
    responsive: true,
    maintainAspectRatio: false,
    plugins: {
      legend: {
        position: 'top',
      },
      title: {
        display: false,
      },
    },
    scales: {
      x: {
        type: 'time',
        time: {
          unit: 'minute',
        },
      },
      y: {
        beginAtZero: true,
      },
    },
  };

  /**
   * Handles user logout and redirects to login page.
   */
  const handleLogout = async () => {
    await logout();
    navigate('/login');
  };

  return (
    <div className="metrics-page">
      <nav className="navbar">
        <div className="navbar-content">
          <h1>Metrics: {agentName || 'All Agents'}</h1>
          <div className="navbar-right">
            <button className="nav-btn" onClick={() => navigate('/dashboard')}>
              ← Dashboard
            </button>
            <span className="username">Welcome, {user?.username}</span>
            <button className="logout-btn" onClick={handleLogout}>
              Logout
            </button>
          </div>
        </div>
      </nav>

      <div className="metrics-container">
        <div className="controls-panel">
          {agentName && (
            <div className="agent-info-box">
              <h3>Agent: {agentName}</h3>
              <p className="agent-id-small">ID: {agentId}</p>
            </div>
          )}

          <div className="control-group">
            <label>Filter by Job:</label>
            <select 
              value={selectedJob} 
              onChange={(e) => setSelectedJob(e.target.value)}
              className="select-input"
            >
              <option value="">All Jobs</option>
              {availableJobs.map(job => (
                <option key={job} value={job}>{job}</option>
              ))}
            </select>
          </div>

          <div className="control-group">
            <label>Time Range:</label>
            <div className="time-buttons">
              {['1h', '6h', '24h', '7d'].map(range => (
                <button
                  key={range}
                  className={`time-btn ${timeRange === range ? 'active' : ''}`}
                  onClick={() => setTimeRange(range)}
                >
                  {range}
                </button>
              ))}
            </div>
          </div>

          <div className="control-group">
            <label>Select Metrics ({availableMetrics.length} available):</label>
            {selectedMetrics.length > 0 && (
              <button 
                className="clear-btn"
                onClick={() => setSelectedMetrics([])}
              >
                Clear all ({selectedMetrics.length})
              </button>
            )}
            <input
              type="text"
              placeholder="Filter metrics (e.g., cpu, memory)..."
              className="select-input"
              style={{ marginBottom: '10px' }}
              onChange={(e) => {
                const filter = e.target.value.toLowerCase();
                document.querySelectorAll('.metric-checkbox').forEach(el => {
                  const label = el.querySelector('label').textContent.toLowerCase();
                  el.style.display = label.includes(filter) ? 'flex' : 'none';
                });
              }}
            />
            {loadingMetrics ? (
              <div className="loading-indicator">Loading available metrics...</div>
            ) : availableMetrics.length === 0 ? (
              <div className="no-metrics-msg">
                <p>No metrics available yet.</p>
                <p className="hint">Metrics will appear once your agent starts sending data.</p>
              </div>
            ) : (
              <div className="metrics-selector">
                {availableMetrics.map(metric => (
                  <div key={metric} className="metric-checkbox">
                    <input
                      type="checkbox"
                      id={metric}
                      checked={selectedMetrics.includes(metric)}
                      onChange={() => toggleMetric(metric)}
                    />
                    <label htmlFor={metric}>{metric}</label>
                  </div>
                ))}
              </div>
            )}
          </div>

          {loading && <div className="loading-indicator">Loading data...</div>}
          {error && <div className="error-message">{error}</div>}
        </div>

        <div className="charts-panel">
          {Object.keys(metricsData).length === 0 && !loading && selectedMetrics.length === 0 && (
            <div className="no-data">
              <p>Select metrics from the left panel to visualize data</p>
              <p className="hint">Filter by typing "cpu", "memory", or "go_" to find common metrics</p>
            </div>
          )}
          
          {Object.keys(metricsData).length === 0 && !loading && selectedMetrics.length > 0 && (
            <div className="no-data">
              <p>No data found for selected metrics</p>
              <p className="hint">This agent may not have sent any data yet, or try different metrics</p>
            </div>
          )}

          {Object.entries(metricsData).map(([key, data]) => {
            const chartData = getChartData(key);
            if (!chartData) return null;
            return (
              <div key={key} className="chart-container">
                <h3>{data.label}</h3>
                <div className="chart-wrapper">
                  <Line data={chartData} options={chartOptions} />
                </div>
              </div>
            );
          })}
        </div>
      </div>
    </div>
  );
};

export default Metrics;
