# syntax=docker/dockerfile:1.7

FROM homebridge/homebridge:2024-12-23 AS base

ARG MQTTTHING_VERSION=1.1.10

RUN npm install -g --omit=dev "homebridge-mqttthing@${MQTTTHING_VERSION}"

EXPOSE 8581 51826
