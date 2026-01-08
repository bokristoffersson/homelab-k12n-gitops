# Usage Guide

## Connecting to the Stream

### Prerequisites

- Valid JWT token from Authentik
- WebSocket-capable client (browser, wscat, etc.)

### Web Browser

```javascript
// Get JWT token from Authentik OAuth flow
const token = localStorage.getItem('auth_token');

// Connect with token in query string
const ws = new WebSocket(`wss://energy-ws.k12n.com/ws?token=${token}`);

ws.addEventListener('open', () => {
  console.log('Connected to energy stream');
});

ws.addEventListener('message', (event) => {
  const energyData = JSON.parse(event.data);
  updateDashboard(energyData);
});

ws.addEventListener('error', (error) => {
  console.error('Connection error:', error);
});

ws.addEventListener('close', () => {
  console.log('Connection closed');
  // Implement exponential backoff reconnect
  setTimeout(() => connect(), 5000);
});
```

### Command Line (wscat)

```bash
# Install wscat
npm install -g wscat

# Connect with JWT token
wscat -c "wss://energy-ws.k12n.com/ws?token=YOUR_JWT_TOKEN"
```

### Python Client

```python
import websocket
import json

def on_message(ws, message):
    data = json.loads(message)
    print(f"Power: {data['power_w']}W")

def on_error(ws, error):
    print(f"Error: {error}")

def on_close(ws, close_status_code, close_msg):
    print("Connection closed")

def on_open(ws):
    print("Connected to energy stream")

token = "your-jwt-token"
ws = websocket.WebSocketApp(
    f"wss://energy-ws.k12n.com/ws?token={token}",
    on_open=on_open,
    on_message=on_message,
    on_error=on_error,
    on_close=on_close
)

ws.run_forever()
```

## Message Format

Each message contains the latest power reading:

```json
{
  "timestamp": "2026-01-08T19:30:00.123Z",
  "power_w": 2450.5,
  "voltage": 230.2,
  "current": 10.64,
  "energy_kwh": 145.3
}
```

**Fields**:
- `timestamp`: ISO 8601 timestamp with milliseconds
- `power_w`: Instantaneous power in watts
- `voltage`: Line voltage in volts
- `current`: Current draw in amperes
- `energy_kwh`: Cumulative energy consumption in kilowatt-hours

## Update Frequency

Messages arrive in real-time as Shelly EM publishes them:
- **Normal**: Every 1-2 seconds
- **Significant change**: Immediate (>10W change)
- **No data**: Connection remains open, waits for next reading

## Connection Management

### Heartbeat

The server sends ping frames every 30 seconds. Clients should respond with pong frames (most WebSocket libraries handle this automatically).

### Reconnection Strategy

Implement exponential backoff for reconnections:

```javascript
let reconnectDelay = 1000; // Start with 1 second
const maxDelay = 30000; // Max 30 seconds

function connect() {
  const ws = new WebSocket(url);

  ws.onclose = () => {
    console.log(`Reconnecting in ${reconnectDelay}ms`);
    setTimeout(() => {
      reconnectDelay = Math.min(reconnectDelay * 2, maxDelay);
      connect();
    }, reconnectDelay);
  };

  ws.onopen = () => {
    reconnectDelay = 1000; // Reset on successful connection
  };
}
```

### Connection Limits

- **Max concurrent connections**: 100 per pod
- **Max message size**: 1MB (not enforced, typical messages <1KB)
- **Idle timeout**: None (connection stays open)

## Error Handling

### Authentication Errors

**Close code 401**: Invalid or expired JWT token

```javascript
ws.onclose = (event) => {
  if (event.code === 401) {
    console.log('Token expired, refreshing...');
    refreshToken().then(newToken => {
      // Reconnect with new token
      connect(newToken);
    });
  }
};
```

### Connection Errors

**Close code 1006**: Abnormal closure (network issue, server restart)

Implement retry logic with exponential backoff.

## Integration Examples

### React Hook

```typescript
import { useEffect, useState } from 'react';

export function useEnergyStream(token: string) {
  const [energyData, setEnergyData] = useState(null);
  const [connected, setConnected] = useState(false);

  useEffect(() => {
    const ws = new WebSocket(
      `wss://energy-ws.k12n.com/ws?token=${token}`
    );

    ws.onopen = () => setConnected(true);
    ws.onclose = () => setConnected(false);
    ws.onmessage = (event) => {
      setEnergyData(JSON.parse(event.data));
    };

    return () => ws.close();
  }, [token]);

  return { energyData, connected };
}
```

### Vue Composable

```typescript
import { ref, onMounted, onUnmounted } from 'vue';

export function useEnergyStream(token: string) {
  const energyData = ref(null);
  const connected = ref(false);
  let ws: WebSocket;

  onMounted(() => {
    ws = new WebSocket(`wss://energy-ws.k12n.com/ws?token=${token}`);
    ws.onopen = () => connected.value = true;
    ws.onclose = () => connected.value = false;
    ws.onmessage = (event) => {
      energyData.value = JSON.parse(event.data);
    };
  });

  onUnmounted(() => ws?.close());

  return { energyData, connected };
}
```

## Troubleshooting

### Connection Refused

1. Check JWT token is valid and not expired
2. Verify token is properly URL-encoded in query string
3. Ensure WebSocket URL uses `wss://` not `ws://`

### No Messages Received

1. Check Shelly EM is publishing to MQTT
2. Verify mqtt-kafka-bridge is running
3. Check Redpanda consumer lag: `kubectl logs -n energy-ws -l app=energy-ws`

### Frequent Disconnects

1. Check network stability
2. Verify Traefik WebSocket timeout configuration
3. Review energy-ws pod logs for errors

## Performance Tips

1. **Debounce updates**: Don't update UI on every message
   ```javascript
   const debouncedUpdate = debounce(updateUI, 100);
   ws.onmessage = (event) => debouncedUpdate(JSON.parse(event.data));
   ```

2. **Batch DOM updates**: Use requestAnimationFrame for smooth rendering

3. **Limit history**: Keep only last N messages in memory

4. **Close when hidden**: Close WebSocket when tab is hidden
   ```javascript
   document.addEventListener('visibilitychange', () => {
     if (document.hidden) {
       ws.close();
     } else {
       connect();
     }
   });
   ```
