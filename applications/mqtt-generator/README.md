# MQTT Data Generator

Generates random values and publishes them to MQTT topics. Useful for testing and development.

## Features

- Multiple data streams with different intervals
- Configurable value ranges and decimals
- JSON-based configuration
- Health metrics and logging

## Configuration

Configure via environment variables:

### Basic Configuration

- `MQTT_BROKER`: MQTT broker hostname (default: `localhost`)
- `MQTT_PORT`: MQTT broker port (default: `1883`)
- `MQTT_CLIENT_ID`: Client ID (default: random)

### Data Streams Configuration

Set `GENERATOR_CONFIG` with a JSON configuration:

```json
{
  "streams": [
    {
      "topic": "homelab/energy",
      "interval": 1.0,
      "values": [
        {"name": "power", "min": 0, "max": 5000, "decimals": 2},
        {"name": "voltage", "min": 220, "max": 240, "decimals": 1}
      ]
    },
    {
      "topic": "homelab/temperature",
      "interval": 10.0,
      "values": [
        {"name": "indoor", "min": 18, "max": 24, "decimals": 1},
        {"name": "outdoor", "min": -10, "max": 30, "decimals": 1}
      ]
    }
  ]
}
```

**Fields**:
- `topic`: MQTT topic to publish to
- `interval`: Seconds between publishes
- `values`: Array of value generators
  - `name`: Field name in JSON payload
  - `min`: Minimum value
  - `max`: Maximum value
  - `decimals`: Number of decimal places (default: 2)

## Running Locally

### With Docker

```bash
# Build
docker build -t mqtt-generator .

# Run with default config
docker run --rm mqtt-generator

# Run with custom config
docker run --rm \
  -e MQTT_BROKER=mosquitto \
  -e GENERATOR_CONFIG='{"streams":[{"topic":"test","interval":1,"values":[{"name":"value","min":0,"max":100}]}]}' \
  mqtt-generator
```

### With Python

```bash
pip install -r requirements.txt
python main.py
```

## Testing

Subscribe to topics:

```bash
mosquitto_sub -h localhost -t 'homelab/#' -v
```

## Example Output

```json
{
  "timestamp": "2025-12-19T10:30:45.123456+00:00",
  "power": 2543.76,
  "voltage": 234.2
}
```

## Use Cases

- **Energy monitoring simulation**: Generate realistic power consumption data
- **Temperature sensors**: Simulate indoor/outdoor temperatures
- **Load testing**: Test MQTT consumers with high-frequency data
- **Development**: Mock sensor data without hardware
