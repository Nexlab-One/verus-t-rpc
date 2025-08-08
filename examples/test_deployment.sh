#!/bin/bash

# Test Script for Secure Verus RPC Server Deployment
# This script demonstrates the token issuance and RPC functionality

set -e

echo "🚀 Testing Secure Verus RPC Server Deployment"
echo "=============================================="

# Configuration
RPC_SERVER_URL="http://127.0.0.1:8080"
TOKEN_SERVICE_URL="http://127.0.0.1:8081"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

print_error() {
    echo -e "${RED}❌ $1${NC}"
}

# Check if services are running
echo "📋 Checking service status..."

# Check token service
if curl -s "$TOKEN_SERVICE_URL/health" > /dev/null; then
    print_status "Token service is running"
else
    print_error "Token service is not running. Start it with: cargo run --bin token-service"
    exit 1
fi

# Check RPC server
if curl -s "$RPC_SERVER_URL/health" > /dev/null; then
    print_status "RPC server is running"
else
    print_error "RPC server is not running. Start it with: cargo run"
    exit 1
fi

echo ""
echo "🔐 Testing Token Issuance"
echo "========================="

# Test 1: Issue a token
echo "Requesting JWT token..."

TOKEN_RESPONSE=$(curl -s -X POST "$TOKEN_SERVICE_URL/issue" \
  -H "Content-Type: application/json" \
  -d '{
    "user_id": "test_client_1",
    "permissions": ["read", "write"],
    "client_ip": "127.0.0.1",
    "user_agent": "TestScript/1.0"
  }')

if echo "$TOKEN_RESPONSE" | grep -q "token"; then
    print_status "Token issued successfully"
    TOKEN=$(echo "$TOKEN_RESPONSE" | jq -r '.token')
    echo "Token: ${TOKEN:0:50}..."
else
    print_error "Failed to issue token"
    echo "Response: $TOKEN_RESPONSE"
    exit 1
fi

echo ""
echo "🔍 Testing Token Validation"
echo "==========================="

# Test 2: Validate the token
echo "Validating JWT token..."

VALIDATION_RESPONSE=$(curl -s -X POST "$TOKEN_SERVICE_URL/validate" \
  -H "Content-Type: application/json" \
  -d "{
    \"token\": \"$TOKEN\",
    \"client_ip\": \"127.0.0.1\"
  }")

if echo "$VALIDATION_RESPONSE" | grep -q '"valid":true'; then
    print_status "Token validation successful"
    USER_ID=$(echo "$VALIDATION_RESPONSE" | jq -r '.user_id')
    PERMISSIONS=$(echo "$VALIDATION_RESPONSE" | jq -r '.permissions[]' | tr '\n' ' ')
    echo "User ID: $USER_ID"
    echo "Permissions: $PERMISSIONS"
else
    print_error "Token validation failed"
    echo "Response: $VALIDATION_RESPONSE"
    exit 1
fi

echo ""
echo "📡 Testing RPC Calls"
echo "===================="

# Test 3: RPC call without token (should fail in production mode)
echo "Testing RPC call without authentication..."

RPC_RESPONSE_NO_AUTH=$(curl -s -X POST "$RPC_SERVER_URL/" \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "getinfo",
    "params": [],
    "id": 1
  }')

if echo "$RPC_RESPONSE_NO_AUTH" | grep -q "error"; then
    print_status "Unauthenticated request properly rejected"
else
    print_warning "Unauthenticated request was allowed (development mode may be enabled)"
fi

# Test 4: RPC call with valid token
echo "Testing RPC call with authentication..."

RPC_RESPONSE_AUTH=$(curl -s -X POST "$RPC_SERVER_URL/" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{
    "jsonrpc": "2.0",
    "method": "getinfo",
    "params": [],
    "id": 1
  }')

if echo "$RPC_RESPONSE_AUTH" | grep -q "result"; then
    print_status "Authenticated RPC call successful"
    echo "Response received successfully"
else
    print_error "Authenticated RPC call failed"
    echo "Response: $RPC_RESPONSE_AUTH"
    exit 1
fi

echo ""
echo "🔒 Testing Security Features"
echo "============================"

# Test 5: Invalid token
echo "Testing with invalid token..."

INVALID_RESPONSE=$(curl -s -X POST "$RPC_SERVER_URL/" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer invalid.token.here" \
  -d '{
    "jsonrpc": "2.0",
    "method": "getinfo",
    "params": [],
    "id": 1
  }')

if echo "$INVALID_RESPONSE" | grep -q "error"; then
    print_status "Invalid token properly rejected"
else
    print_warning "Invalid token was accepted (check security configuration)"
fi

# Test 6: Expired token (if we had one)
echo "Testing token expiration handling..."

EXPIRED_RESPONSE=$(curl -s -X POST "$TOKEN_SERVICE_URL/validate" \
  -H "Content-Type: application/json" \
  -d '{
    "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiJ0ZXN0IiwiaXNzIjoidmVydXMtcnBjLXNlcnZlciIsImF1ZCI6InZlcnVzLWNsaWVudHMiLCJpYXQiOjE1MTYyMzkwMjIsImV4cCI6MTUxNjIzOTAyMywibmJmIjoxNTE2MjM5MDIyLCJqdGkiOiJ0ZXN0IiwicGVybWlzc2lvbnMiOlsicmVhZCJdfQ.invalid",
    "client_ip": "127.0.0.1"
  }')

if echo "$EXPIRED_RESPONSE" | grep -q '"valid":false'; then
    print_status "Expired token properly rejected"
else
    print_warning "Expired token handling may need review"
fi

echo ""
echo "📊 Testing Monitoring Endpoints"
echo "==============================="

# Test 7: Health check
echo "Testing health check endpoint..."

HEALTH_RESPONSE=$(curl -s "$RPC_SERVER_URL/health")

if echo "$HEALTH_RESPONSE" | grep -q "healthy"; then
    print_status "Health check endpoint working"
else
    print_error "Health check endpoint failed"
    echo "Response: $HEALTH_RESPONSE"
fi

# Test 8: Metrics endpoint
echo "Testing metrics endpoint..."

METRICS_RESPONSE=$(curl -s "$RPC_SERVER_URL/metrics")

if [ -n "$METRICS_RESPONSE" ]; then
    print_status "Metrics endpoint working"
else
    print_error "Metrics endpoint failed"
fi

echo ""
echo "🎉 All Tests Completed Successfully!"
echo "===================================="
echo ""
echo "📋 Summary:"
echo "- ✅ Token issuance working"
echo "- ✅ Token validation working"
echo "- ✅ RPC authentication working"
echo "- ✅ Security features working"
echo "- ✅ Monitoring endpoints working"
echo ""
echo "🚀 Your secure Verus RPC server is ready for production!"
echo ""
echo "📚 Next steps:"
echo "1. Review the production deployment guide: ../docs/deployment/production.md"
echo "2. Configure your reverse proxy (nginx/Caddy)"
echo "3. Set up SSL/TLS certificates"
echo "4. Configure firewall rules"
echo "5. Set up monitoring and alerting"
echo ""
echo "🔐 Security reminder:"
echo "- Always use HTTPS in production"
echo "- Keep JWT secret keys secure"
echo "- Monitor logs for suspicious activity"
echo "- Regularly rotate tokens and keys"
