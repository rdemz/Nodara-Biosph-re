#!/bin/bash
# monitor.sh - Legendary Monitoring Script for Nodara BIOSPHÈRE QUANTIC

echo "---------------------------------------------------"
echo "Starting Nodara BIOSPHÈRE QUANTIC Monitoring Services"
echo "---------------------------------------------------"

# Start Prometheus
echo "Launching Prometheus..."
docker run -d --name prometheus -p 9090:9090 -v $(pwd)/prometheus.yml:/etc/prometheus/prometheus.yml prom/prometheus

# Start Grafana
echo "Launching Grafana..."
docker run -d --name grafana -p 3000:3000 grafana/grafana

# Wait a few seconds for services to initialize
sleep 10

echo "Monitoring services are now running."
echo "Access Prometheus at: http://localhost:9090"
echo "Access Grafana at: http://localhost:3000 (default credentials: admin/admin)"

echo "---------------------------------------------------"
echo "Monitoring setup complete."
