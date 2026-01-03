# Heatpump Settings Service

## Overview

The Heatpump Settings Service processes configuration changes and settings updates for heat pump devices via the Kafka streaming platform.

## Purpose

- **Consume** heat pump setting change events from Kafka
- **Validate** settings before application
- **Apply** configuration changes to heat pump devices
- **Track** setting history and changes

## Architecture

```
User Interface → API → Kafka (settings topic) → Redpanda Connect → Heat Pump Device
```

## Components

### Redpanda Connect Processor

Processes messages from the `heatpump-settings` Kafka topic and forwards them to the heat pump control system.

- **Consumer Group**: `heatpump-settings`
- **Topic**: `homelab.heatpump.settings`
- **Processing**: Validates and transforms settings

## Data Flow

1. User modifies heat pump settings via web interface
2. API publishes setting change to Kafka topic
3. Redpanda Connect consumes the message
4. Settings are validated and transformed
5. Commands are sent to heat pump controller
6. Confirmation is published back to Kafka

## Settings Types

| Setting | Range | Description |
|---------|-------|-------------|
| Target Temperature | 18-28°C | Desired room temperature |
| Mode | heating/cooling/auto | Operating mode |
| Schedule | 24h format | Temperature schedule |
| Eco Mode | on/off | Energy saving mode |

## Related Components

- **Homelab API**: Provides settings management API
- **Heat Pump Control**: Communicates with physical device
- **TimescaleDB**: Stores settings history
