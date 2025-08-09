#!/bin/bash
# Wait for Verus daemon to be ready before starting the RPC server

set -e

VERUS_HOST="${VERUS_HOST:-verus-daemon-test}"
VERUS_PORT="${VERUS_PORT:-27486}"
VERUS_USER="${VERUS_USER:-verus_rpc_user}"
VERUS_PASSWORD="${VERUS_PASSWORD:-verus_rpc_password}"
MAX_ATTEMPTS="${MAX_ATTEMPTS:-60}"
ATTEMPT_INTERVAL="${ATTEMPT_INTERVAL:-10}"

echo "Waiting for Verus daemon to be ready..."
echo "Host: $VERUS_HOST:$VERUS_PORT"
echo "Max attempts: $MAX_ATTEMPTS"
echo "Attempt interval: ${ATTEMPT_INTERVAL}s"

for attempt in $(seq 1 $MAX_ATTEMPTS); do
    echo "Attempt $attempt/$MAX_ATTEMPTS: Checking Verus daemon..."
    
    # Try to connect to the Verus daemon
    if curl -s -f -X POST \
        -H "Content-Type: application/json" \
        -u "$VERUS_USER:$VERUS_PASSWORD" \
        -d '{"jsonrpc":"1.0","id":"healthcheck","method":"getinfo","params":[]}' \
        "http://$VERUS_HOST:$VERUS_PORT" > /dev/null 2>&1; then
        
        echo "✅ Verus daemon is ready!"
        exit 0
    fi
    
    echo "⏳ Verus daemon not ready yet, waiting ${ATTEMPT_INTERVAL}s..."
    sleep $ATTEMPT_INTERVAL
done

echo "❌ Verus daemon failed to become ready after $MAX_ATTEMPTS attempts"
echo "Starting RPC server anyway (it will retry connections on requests)..."
exit 0
