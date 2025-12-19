#!/usr/bin/env python3
"""
MQTT Random Data Generator

Generates random values and publishes them to MQTT topics.
Configurable for different value types (energy, temperature, etc.)
with different intervals and ranges.
"""

import os
import sys
import time
import json
import random
import logging
from datetime import datetime, timezone
from typing import Dict, List, Any
import paho.mqtt.client as mqtt

# Configure logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger(__name__)


class ValueGenerator:
    """Generates random values within a specified range."""

    def __init__(self, name: str, min_val: float, max_val: float, decimals: int = 2):
        self.name = name
        self.min_val = min_val
        self.max_val = max_val
        self.decimals = decimals

    def generate(self) -> float:
        """Generate a random value within the range."""
        value = random.uniform(self.min_val, self.max_val)
        return round(value, self.decimals)


class DataStream:
    """Represents a data stream with topic, generators, and interval."""

    def __init__(self, config: Dict[str, Any]):
        self.topic = config['topic']
        self.interval = config['interval']
        self.generators = []

        for gen_config in config['values']:
            self.generators.append(ValueGenerator(
                name=gen_config['name'],
                min_val=gen_config['min'],
                max_val=gen_config['max'],
                decimals=gen_config.get('decimals', 2)
            ))

        self.last_publish = 0

    def should_publish(self, current_time: float) -> bool:
        """Check if it's time to publish based on interval."""
        return current_time - self.last_publish >= self.interval

    def generate_data(self) -> Dict[str, Any]:
        """Generate data payload."""
        data = {
            'timestamp': datetime.now(timezone.utc).isoformat(),
        }

        for generator in self.generators:
            data[generator.name] = generator.generate()

        return data

    def update_last_publish(self, current_time: float):
        """Update the last publish time."""
        self.last_publish = current_time


class MQTTDataGenerator:
    """Main application class."""

    def __init__(self):
        self.mqtt_broker = os.getenv('MQTT_BROKER', 'localhost')
        self.mqtt_port = int(os.getenv('MQTT_PORT', '1883'))
        self.mqtt_client_id = os.getenv('MQTT_CLIENT_ID', f'mqtt-generator-{random.randint(0, 1000)}')

        # Parse configuration from environment
        self.streams = self._parse_config()

        # MQTT client setup
        self.client = mqtt.Client(client_id=self.mqtt_client_id)
        self.client.on_connect = self._on_connect
        self.client.on_disconnect = self._on_disconnect
        self.connected = False

    def _parse_config(self) -> List[DataStream]:
        """Parse configuration from environment variables."""
        config_json = os.getenv('GENERATOR_CONFIG')

        if not config_json:
            # Default configuration
            logger.info("Using default configuration")
            return [
                DataStream({
                    'topic': 'homelab/energy',
                    'interval': 1.0,
                    'values': [
                        {'name': 'power', 'min': 0, 'max': 5000, 'decimals': 2},
                        {'name': 'voltage', 'min': 220, 'max': 240, 'decimals': 1}
                    ]
                }),
                DataStream({
                    'topic': 'homelab/temperature',
                    'interval': 10.0,
                    'values': [
                        {'name': 'indoor', 'min': 18, 'max': 24, 'decimals': 1},
                        {'name': 'outdoor', 'min': -10, 'max': 30, 'decimals': 1}
                    ]
                })
            ]

        try:
            config = json.loads(config_json)
            return [DataStream(stream_config) for stream_config in config['streams']]
        except json.JSONDecodeError as e:
            logger.error(f"Failed to parse GENERATOR_CONFIG: {e}")
            sys.exit(1)

    def _on_connect(self, client, userdata, flags, rc):
        """Callback for when the client connects to the broker."""
        if rc == 0:
            self.connected = True
            logger.info(f"Connected to MQTT broker at {self.mqtt_broker}:{self.mqtt_port}")
        else:
            logger.error(f"Failed to connect to MQTT broker, return code {rc}")

    def _on_disconnect(self, client, userdata, rc):
        """Callback for when the client disconnects from the broker."""
        self.connected = False
        if rc != 0:
            logger.warning(f"Unexpected disconnect from MQTT broker, return code {rc}")
        else:
            logger.info("Disconnected from MQTT broker")

    def connect(self):
        """Connect to MQTT broker."""
        logger.info(f"Connecting to MQTT broker at {self.mqtt_broker}:{self.mqtt_port}")
        try:
            self.client.connect(self.mqtt_broker, self.mqtt_port, 60)
            self.client.loop_start()

            # Wait for connection
            timeout = 10
            start = time.time()
            while not self.connected and time.time() - start < timeout:
                time.sleep(0.1)

            if not self.connected:
                logger.error("Connection timeout")
                sys.exit(1)

        except Exception as e:
            logger.error(f"Failed to connect to MQTT broker: {e}")
            sys.exit(1)

    def run(self):
        """Main run loop."""
        logger.info(f"Starting MQTT data generator with {len(self.streams)} streams")
        for stream in self.streams:
            logger.info(f"  - Topic: {stream.topic}, Interval: {stream.interval}s, Values: {[g.name for g in stream.generators]}")

        try:
            while True:
                current_time = time.time()

                for stream in self.streams:
                    if stream.should_publish(current_time):
                        data = stream.generate_data()
                        payload = json.dumps(data)

                        result = self.client.publish(stream.topic, payload, qos=0)
                        if result.rc == mqtt.MQTT_ERR_SUCCESS:
                            logger.debug(f"Published to {stream.topic}: {payload}")
                        else:
                            logger.error(f"Failed to publish to {stream.topic}")

                        stream.update_last_publish(current_time)

                # Sleep a short time to avoid busy waiting
                time.sleep(0.1)

        except KeyboardInterrupt:
            logger.info("Shutting down...")
        finally:
            self.client.loop_stop()
            self.client.disconnect()


def main():
    """Entry point."""
    generator = MQTTDataGenerator()
    generator.connect()
    generator.run()


if __name__ == '__main__':
    main()
