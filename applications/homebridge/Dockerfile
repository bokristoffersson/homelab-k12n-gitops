# syntax=docker/dockerfile:1.7

# Extends the official Homebridge image with the MQTT Thing plugin.
# A multi-stage build adds nothing here: there is no own source code to
# compile, and the single npm-install layer is already cache-keyed by
# MQTTTHING_VERSION below — it only re-runs when the version bumps.
#
# Base image is pinned to a dated tag for reproducibility. Bump it
# quarterly or when upstream publishes a security release:
#   https://github.com/homebridge/docker-homebridge/releases
FROM homebridge/homebridge:2024-12-23 AS base

ARG MQTTTHING_VERSION=1.1.10

RUN npm install -g --omit=dev "homebridge-mqttthing@${MQTTTHING_VERSION}"

EXPOSE 8581 51826
