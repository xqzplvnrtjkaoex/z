#!/bin/bash
set -e

# Service listen addresses
export AUTH_LISTEN_ADDR="0.0.0.0:50051"
export CATALOG_LISTEN_ADDR="0.0.0.0:50052"
export USER_LISTEN_ADDR="0.0.0.0:50053"

# Gateway connects to services
export AUTH_GRPC_ADDR="http://127.0.0.1:50051"
export CATALOG_GRPC_ADDR="http://127.0.0.1:50052"
export USER_GRPC_ADDR="http://127.0.0.1:50053"
export GATEWAY_ADDR="0.0.0.0:3000"

echo "Starting auth service on :50051"
cargo run --bin auth &
AUTH_PID=$!

echo "Starting catalog service on :50052"
cargo run --bin catalog &
CATALOG_PID=$!

echo "Starting user service on :50053"
cargo run --bin user &
USER_PID=$!

echo "Waiting for gRPC services to start..."
sleep 3

echo "Starting gateway on :3000"
cargo run --bin gateway &
GATEWAY_PID=$!

# Trap to kill all services on exit
trap "kill $AUTH_PID $CATALOG_PID $USER_PID $GATEWAY_PID 2>/dev/null; exit" INT TERM EXIT

echo ""
echo "All services running. Gateway at http://127.0.0.1:3000"
echo "  GET http://127.0.0.1:3000/health"
echo "  GET http://127.0.0.1:3000/v1/health/services"
echo ""
echo "Press Ctrl+C to stop all services."

wait
