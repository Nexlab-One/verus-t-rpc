# Payments API

This document describes the REST endpoints that enable VRSC shielded payments for RPC access.

## Overview

- Flow:
  1. Client requests a quote and payment address
  2. Client sends a shielded payment on-chain and submits the raw tx to the server
  3. Server verifies payment (viewing key or wallet) and issues a provisional JWT after min confirmations, then a final JWT after deeper confirmations
- Address types: "orchard" (default) or "sapling"
- Chain: use `VRSCTEST` for development

## Endpoints

### POST /payments/request
Request a payment quote and receive a shielded address.

Request body:
```json
{
  "tier_id": "basic",
  "address_type": "orchard"
}
```
- `address_type` optional; defaults to configured `default_address_type`.

Response (200):
```json
{
  "payment_id": "b2c8e1d9-...",
  "tier_id": "basic",
  "amount_vrsc": 1.0,
  "address": "zs1...",
  "address_type": "orchard",
  "expires_at": "2025-01-01T12:00:00Z"
}
```

Errors: `unknown tier`, `unsupported address type`.

Notes:
- Viewing-key-only mode: selects an imported shielded address compatible with requested type
- Hot-wallet mode: requests a new z-address from the daemon

### POST /payments/submit
Submit the raw transaction (hex) after sending your on-chain payment.

Request body:
```json
{
  "payment_id": "b2c8e1d9-...",
  "rawtx_hex": "02000080..."
}
```

Response (200):
```json
{
  "txid": "9e7a..."
}
```

Errors: `unknown payment_id`, `payment session expired`, `invalid state for submission`, `invalid raw tx hex`.

### GET /payments/status/{payment_id}
Check payment status and retrieve tokens when available.

Response (200):
```json
{
  "status": "Confirmed1",
  "confirmations": 1,
  "amount_vrsc": 1.0,
  "address": "zs1...",
  "txid": "9e7a...",
  "provisional_token": "eyJhbGciOi...",
  "final_token": null
}
```

Possible statuses: `Pending`, `Submitted`, `Verified`, `Confirmed1`, `Finalized`, `Expired`, `Failed`.

Token policy:
- Provisional token at `min_confirmations` (default 1) with `permissions: ["provisional", ...]`
- Final token at deeper confirmations (≥ max(2, min_confirmations)) with `permissions: ["paid", ...]`
- If verification later fails or session expires, provisional tokens are revoked via the revocation store (Redis-backed when cache.enabled)

## Configuration
See configuration reference for `[payments]` options: address types, confirmations, session TTL, tiers, viewing keys, and revocation behavior. When `[cache].enabled = true`, sessions and revocations are persisted in Redis; otherwise, in-memory fallbacks are used.

## Examples

Quote (orchard):
```bash
curl -X POST http://127.0.0.1:8080/payments/request \
  -H "Content-Type: application/json" \
  -H "x-forwarded-for: 127.0.0.1" \
  -d '{"tier_id":"basic","address_type":"orchard"}'
```

Submit raw tx:
```bash
curl -X POST http://127.0.0.1:8080/payments/submit \
  -H "Content-Type: application/json" \
  -H "x-forwarded-for: 127.0.0.1" \
  -d '{"payment_id":"<id>","rawtx_hex":"<hex>"}'
```

Check status:
```bash
curl -X GET http://127.0.0.1:8080/payments/status/<payment_id>
```

## Security & Wallet Modes
 - Viewing-only mode (recommended): imports z-address viewing keys on startup and verifies with `z_viewtransaction`; no spending keys on the server
 - Hot-wallet mode: if no viewing keys are configured, requests new addresses; auto-sweep is intentionally not implemented
 - Payments can be disabled globally via `payments.enabled=false`
