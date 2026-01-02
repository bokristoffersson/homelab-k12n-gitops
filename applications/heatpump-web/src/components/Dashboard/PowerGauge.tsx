import { useEffect, useState, useRef } from 'react';

interface PowerData {
  // Current energy message structure from Kafka
  activeActualConsumption?: {
    total?: number;
    L1?: number;
    L2?: number;
    L3?: number;
  };
  fields?: {
    active_power_total?: number;
  };
  timestamp?: string;
}

interface WebSocketMessage {
  type: string;
  stream?: string;
  timestamp?: string;
  data?: PowerData;
  message?: string;
}

export default function PowerGauge() {
  const [power, setPower] = useState<number>(0);
  const [connected, setConnected] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const wsRef = useRef<WebSocket | null>(null);
  const reconnectTimeoutRef = useRef<number | null>(null);

  useEffect(() => {
    const connectWebSocket = async () => {
      try {
        // Get JWT token from the user info endpoint (authenticated via oauth2-proxy)
        const apiBaseUrl = import.meta.env.DEV
          ? 'http://localhost:8000/api/v1'
          : `${window.ENV?.API_URL}/api/v1`;

        const response = await fetch(`${apiBaseUrl}/user/info`);
        if (!response.ok) {
          throw new Error('Failed to get authentication token');
        }

        const userInfo = await response.json();
        const token = userInfo.token;

        // Determine WebSocket URL based on environment
        const wsUrl = import.meta.env.DEV
          ? `ws://localhost:8080/ws/energy?token=${token}`  // Development
          : `wss://${new URL(window.ENV?.API_URL || '').host}/ws/energy?token=${token}`;  // Production

        console.log('Connecting to WebSocket:', wsUrl.replace(/token=[^&]+/, 'token=***'));
        const ws = new WebSocket(wsUrl);
        wsRef.current = ws;

        ws.onopen = () => {
          console.log('WebSocket connected');
          setConnected(true);
          setError(null);

          // Subscribe to energy stream
          const subscribeMessage = {
            type: 'subscribe',
            streams: ['energy']
          };
          ws.send(JSON.stringify(subscribeMessage));
          console.log('Subscribed to energy stream');
        };

        ws.onmessage = (event) => {
          try {
            const message: WebSocketMessage = JSON.parse(event.data);

            if (message.type === 'data' && message.stream === 'energy' && message.data) {
              // Extract power from the energy message structure
              const powerW = message.data.activeActualConsumption?.total
                || message.data.fields?.active_power_total
                || 0;
              setPower(powerW);
            } else if (message.type === 'error') {
              console.error('WebSocket error message:', message.message);
              setError(message.message || 'Unknown error');
            }
          } catch (err) {
            console.error('Failed to parse WebSocket message:', err);
          }
        };

        ws.onerror = (event) => {
          console.error('WebSocket error:', event);
          setError('Connection error');
        };

        ws.onclose = () => {
          console.log('WebSocket closed, attempting to reconnect...');
          setConnected(false);

          // Attempt to reconnect after 5 seconds
          reconnectTimeoutRef.current = setTimeout(() => {
            connectWebSocket();
          }, 5000);
        };

      } catch (err) {
        console.error('Failed to create WebSocket:', err);
        setError('Failed to connect');
      }
    };

    connectWebSocket();

    // Cleanup on unmount
    return () => {
      if (reconnectTimeoutRef.current) {
        clearTimeout(reconnectTimeoutRef.current);
      }
      if (wsRef.current) {
        wsRef.current.close();
      }
    };
  }, []);

  const powerKw = power / 1000;
  const maxPower = 17.5; // Maximum power for gauge (17.5 kW)
  const percentage = Math.min((powerKw / maxPower) * 100, 100);

  // Calculate stroke dash offset for progress ring
  const radius = 80;
  const circumference = 2 * Math.PI * radius;
  const strokeDashoffset = circumference - (percentage / 100) * circumference;

  // Color based on power level
  const getColor = () => {
    if (powerKw < 4) return '#4ade80'; // Green
    if (powerKw < 9) return '#facc15'; // Yellow
    if (powerKw < 14) return '#fb923c'; // Orange
    return '#ef4444'; // Red
  };

  if (error) {
    return (
      <div className="card card-error">
        <h3>Live Power Monitor</h3>
        <div className="error-message">
          Error: {error}
        </div>
      </div>
    );
  }

  return (
    <div className="card">
      <h3>Live Power Monitor</h3>
      <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'center', padding: '1rem' }}>
        {/* Progress Ring */}
        <div style={{ position: 'relative', width: '200px', height: '200px' }}>
          <svg width="200" height="200" style={{ transform: 'rotate(-90deg)' }}>
            {/* Background ring */}
            <circle
              cx="100"
              cy="100"
              r={radius}
              fill="none"
              stroke="#e5e7eb"
              strokeWidth="20"
            />
            {/* Progress ring */}
            <circle
              cx="100"
              cy="100"
              r={radius}
              fill="none"
              stroke={getColor()}
              strokeWidth="20"
              strokeDasharray={circumference}
              strokeDashoffset={strokeDashoffset}
              strokeLinecap="round"
              style={{ transition: 'stroke-dashoffset 0.3s ease, stroke 0.3s ease' }}
            />
          </svg>

          {/* Center value */}
          <div style={{
            position: 'absolute',
            top: '50%',
            left: '50%',
            transform: 'translate(-50%, -50%)',
            textAlign: 'center'
          }}>
            <div style={{ fontSize: '2.5rem', fontWeight: 'bold', color: getColor() }}>
              {powerKw.toFixed(1)}
            </div>
            <div style={{ fontSize: '1rem', color: '#6b7280', marginTop: '0.25rem' }}>
              kW
            </div>
            <div style={{ fontSize: '0.75rem', color: '#9ca3af', marginTop: '0.5rem' }}>
              {percentage.toFixed(0)}% of {maxPower}
            </div>
          </div>
        </div>

        {/* Connection status */}
        <div className="live-indicator" style={{ marginTop: '1rem' }}>
          <span className="dot" style={{
            background: connected ? '#22c55e' : '#ef4444'
          }}></span>
          {connected ? 'Live' : 'Connecting...'}
        </div>
      </div>
    </div>
  );
}
