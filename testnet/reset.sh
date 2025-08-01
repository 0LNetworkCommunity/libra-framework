#!/bin/bash

echo "Resetting containers..."
docker compose down -v

echo "Starting containers..."
docker compose up -d