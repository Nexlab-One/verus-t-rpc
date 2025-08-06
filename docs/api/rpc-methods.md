# RPC Methods Reference

Reference for all supported RPC methods in the Verus RPC Server.

## 📋 Method Categories

The server supports **60+ Verus RPC methods** organized into the following categories:

- **Blockchain Information** - General blockchain data
- **Block Operations** - Block retrieval and analysis
- **Transaction Operations** - Transaction management
- **Address Operations** - Address-related queries
- **Identity Operations** - Verus identity management
- **Currency Operations** - Currency and token management
- **Mining Operations** - Mining-related functions
- **Network Operations** - Network information and management

## 🔗 Blockchain Information Methods

### getinfo

Get general information about the Verus daemon.

**Parameters:** `[]`

**Returns:**
```json
{
  "version": 123456,
  "protocolversion": 123456,
  "walletversion": 123456,
  "balance": 0.0,
  "blocks": 123456,
  "timeoffset": 0,
  "connections": 8,
  "proxy": "",
  "difficulty": 123456.789,
  "testnet": false,
  "keypoololdest": 1234567890,
  "keypoolsize": 100,
  "unlocked_until": 0,
  "paytxfee": 0.0001,
  "relayfee": 0.00001,
  "errors": ""
}
```

### getblockchaininfo

Get detailed blockchain information.

**Parameters:** `[]`

**Returns:**
```json
{
  "chain": "main",
  "blocks": 123456,
  "headers": 123456,
  "bestblockhash": "0000000000000000000000000000000000000000000000000000000000000000",
  "difficulty": 123456.789,
  "mediantime": 1234567890,
  "verificationprogress": 0.9999,
  "initialblockdownload": false,
  "chainwork": "0000000000000000000000000000000000000000000000000000000000000000",
  "size_on_disk": 123456789,
  "pruned": false,
  "pruneheight": 0,
  "automatic_pruning": false,
  "prune_target_size": 0,
  "softforks": [],
  "bip9_softforks": {},
  "warnings": ""
}
```

### getblockcount

Get the current block count.

**Parameters:** `[]`

**Returns:** `number`

### getdifficulty

Get the current difficulty.

**Parameters:** `[]`

**Returns:** `number`

### getconnectioncount

Get the number of connections to other nodes.

**Parameters:** `[]`

**Returns:** `number`

## 🧱 Block Operations Methods

### getblock

Get block information by hash.

**Parameters:** `[string block_hash, boolean verbose]`

**Returns:**
```json
{
  "hash": "0000000000000000000000000000000000000000000000000000000000000000",
  "confirmations": 123456,
  "size": 1234,
  "height": 123456,
  "version": 1,
  "merkleroot": "0000000000000000000000000000000000000000000000000000000000000000",
  "tx": ["txid1", "txid2"],
  "time": 1234567890,
  "mediantime": 1234567890,
  "nonce": 1234567890,
  "bits": "1d00ffff",
  "difficulty": 123456.789,
  "chainwork": "0000000000000000000000000000000000000000000000000000000000000000",
  "previousblockhash": "0000000000000000000000000000000000000000000000000000000000000000",
  "nextblockhash": "0000000000000000000000000000000000000000000000000000000000000000"
}
```

### getblockhash

Get block hash by height.

**Parameters:** `[number height]`

**Returns:** `string`

### getblockheader

Get block header information.

**Parameters:** `[string block_hash, boolean verbose]`

**Returns:**
```json
{
  "hash": "0000000000000000000000000000000000000000000000000000000000000000",
  "confirmations": 123456,
  "height": 123456,
  "version": 1,
  "merkleroot": "0000000000000000000000000000000000000000000000000000000000000000",
  "time": 1234567890,
  "mediantime": 1234567890,
  "nonce": 1234567890,
  "bits": "1d00ffff",
  "difficulty": 123456.789,
  "chainwork": "0000000000000000000000000000000000000000000000000000000000000000",
  "previousblockhash": "0000000000000000000000000000000000000000000000000000000000000000",
  "nextblockhash": "0000000000000000000000000000000000000000000000000000000000000000"
}
```

### getblockstats

Get block statistics.

**Parameters:** `[string block_hash]`

**Returns:**
```json
{
  "avgfee": 1234,
  "avgfeerate": 123.45,
  "avgtxsize": 1234,
  "blockhash": "0000000000000000000000000000000000000000000000000000000000000000",
  "feerate_percentiles": [1.0, 2.0, 3.0, 4.0, 5.0],
  "height": 123456,
  "ins": 123,
  "maxfee": 12345,
  "maxfeerate": 1234.56,
  "maxtxsize": 12345,
  "medianfee": 1234,
  "mediantime": 1234567890,
  "mediantxsize": 1234,
  "minfee": 123,
  "minfeerate": 12.34,
  "mintxsize": 123,
  "outs": 456,
  "subsidy": 1234567890,
  "swtotal_size": 12345,
  "swtotal_weight": 12345,
  "swtxs": 123,
  "time": 1234567890,
  "total_out": 1234567890,
  "total_size": 12345,
  "total_weight": 12345,
  "totalfee": 12345,
  "txs": 123,
  "utxo_increase": 123,
  "utxo_size_inc": 12345
}
```

## 💰 Transaction Operations Methods

### getrawtransaction

Get raw transaction data.

**Parameters:** `[string txid, boolean verbose]`

**Returns:**
```json
{
  "txid": "0000000000000000000000000000000000000000000000000000000000000000",
  "hash": "0000000000000000000000000000000000000000000000000000000000000000",
  "version": 1,
  "size": 1234,
  "vsize": 1234,
  "weight": 1234,
  "locktime": 0,
  "vin": [
    {
      "txid": "0000000000000000000000000000000000000000000000000000000000000000",
      "vout": 0,
      "scriptSig": {
        "asm": "script",
        "hex": "hexstring"
      },
      "sequence": 4294967295
    }
  ],
  "vout": [
    {
      "value": 0.12345678,
      "n": 0,
      "scriptPubKey": {
        "asm": "script",
        "hex": "hexstring",
        "reqSigs": 1,
        "type": "pubkeyhash",
        "addresses": ["address"]
      }
    }
  ],
  "blockhash": "0000000000000000000000000000000000000000000000000000000000000000",
  "confirmations": 123456,
  "time": 1234567890,
  "blocktime": 1234567890
}
```

### sendrawtransaction

Send a raw transaction.

**Parameters:** `[string hexstring, boolean allowhighfees]`

**Returns:** `string` (transaction hash)

### createrawtransaction

Create a raw transaction.

**Parameters:** `[array inputs, object outputs]`

**Returns:** `string` (hex-encoded transaction)

### decoderawtransaction

Decode a raw transaction.

**Parameters:** `[string hexstring]`

**Returns:**
```json
{
  "txid": "0000000000000000000000000000000000000000000000000000000000000000",
  "hash": "0000000000000000000000000000000000000000000000000000000000000000",
  "version": 1,
  "size": 1234,
  "vsize": 1234,
  "weight": 1234,
  "locktime": 0,
  "vin": [...],
  "vout": [...]
}
```

### fundrawtransaction

Fund a raw transaction.

**Parameters:** `[string hexstring, object options]`

**Returns:**
```json
{
  "hex": "hexstring",
  "fee": 0.0001,
  "changepos": 1
}
```

### signdata

Sign data with a private key.

**Parameters:** `[string address, string data]`

**Returns:** `string` (signature)

## 🏠 Address Operations Methods

### getaddressbalance

Get balance for an address.

**Parameters:** `[string address]`

**Returns:**
```json
{
  "balance": 123.456789,
  "received": 456.789012,
  "currency": "VRSC"
}
```

### getaddressutxos

Get UTXOs for an address.

**Parameters:** `[string address]`

**Returns:**
```json
[
  {
    "address": "address",
    "txid": "0000000000000000000000000000000000000000000000000000000000000000",
    "outputIndex": 0,
    "script": "script",
    "satoshis": 1234567890,
    "height": 123456
  }
]
```

### getaddressmempool

Get mempool transactions for an address.

**Parameters:** `[string address]`

**Returns:**
```json
[
  {
    "address": "address",
    "txid": "0000000000000000000000000000000000000000000000000000000000000000",
    "index": 0,
    "satoshis": 1234567890,
    "timestamp": 1234567890,
    "prevtxid": "0000000000000000000000000000000000000000000000000000000000000000",
    "prevout": 0
  }
]
```

### getaddressdeltas

Get address deltas.

**Parameters:** `[array addresses, number start, number end]`

**Returns:**
```json
[
  {
    "satoshis": 1234567890,
    "txid": "0000000000000000000000000000000000000000000000000000000000000000",
    "index": 0,
    "height": 123456,
    "address": "address"
  }
]
```

## 🆔 Identity Operations Methods

### getidentity

Get identity information.

**Parameters:** `[string identity_name]`

**Returns:**
```json
{
  "identity": {
    "name": "identity_name",
    "parent": "parent_identity",
    "systemid": "system_id",
    "contentmap": {},
    "privateaddress": "private_address",
    "revocationauthority": "revocation_authority",
    "recoveryauthority": "recovery_authority",
    "timelock": 0,
    "flags": 0,
    "primaryaddresses": ["address1", "address2"],
    "minimumsignatures": 1,
    "identityaddress": "identity_address"
  },
  "status": "active"
}
```

### registeridentity

Register a new identity.

**Parameters:** `[string identity_name, object identity_data]`

**Returns:** `string` (transaction hash)

### updateidentity

Update an existing identity.

**Parameters:** `[string identity_name, object identity_data]`

**Returns:** `string` (transaction hash)

### revokeidentity

Revoke an identity.

**Parameters:** `[string identity_name]`

**Returns:** `string` (transaction hash)

### recoveridentity

Recover a revoked identity.

**Parameters:** `[string identity_name]`

**Returns:** `string` (transaction hash)

### setidentitytimelock

Set identity timelock.

**Parameters:** `[string identity_name, number timelock]`

**Returns:** `string` (transaction hash)

## 💱 Currency Operations Methods

### getcurrency

Get currency information.

**Parameters:** `[string currency_name]`

**Returns:**
```json
{
  "currencyid": "0000000000000000000000000000000000000000000000000000000000000000",
  "parent": "parent_currency",
  "name": "currency_name",
  "systemid": "system_id",
  "notarizationprotocol": 1,
  "proofprotocol": 1,
  "startblock": 123456,
  "endblock": 0,
  "currencies": ["currency1", "currency2"],
  "weights": [1.0, 1.0],
  "conversions": [],
  "minpreconversion": [],
  "currencystate": {
    "flags": 0,
    "version": 1,
    "currencyid": "0000000000000000000000000000000000000000000000000000000000000000",
    "reservecurrencies": ["currency1"],
    "initialsupply": 1000000,
    "emitted": 0,
    "supply": 1000000,
    "currencies": {"currency1": {"reserves": 1000000, "price": 1.0}},
    "primarycurrencyfees": 0,
    "primarycurrencyconversionfees": 0,
    "primarycurrencyout": 0,
    "primarycurrencyin": 0,
    "preconvertedout": 0,
    "preconvertedin": 0,
    "initialweights": [1.0],
    "launchcleared": true,
    "options": 0
  }
}
```

### sendcurrency

Send currency to an address.

**Parameters:** `[string address, object currency_data]`

**Returns:** `string` (transaction hash)

### listcurrencies

List all currencies.

**Parameters:** `[boolean verbose]`

**Returns:**
```json
[
  {
    "currencyid": "0000000000000000000000000000000000000000000000000000000000000000",
    "name": "currency_name",
    "systemid": "system_id",
    "notarizationprotocol": 1,
    "proofprotocol": 1,
    "startblock": 123456,
    "endblock": 0
  }
]
```

### getcurrencystate

Get currency state.

**Parameters:** `[string currency_name]`

**Returns:**
```json
{
  "flags": 0,
  "version": 1,
  "currencyid": "0000000000000000000000000000000000000000000000000000000000000000",
  "reservecurrencies": ["currency1"],
  "initialsupply": 1000000,
  "emitted": 0,
  "supply": 1000000,
  "currencies": {"currency1": {"reserves": 1000000, "price": 1.0}},
  "primarycurrencyfees": 0,
  "primarycurrencyconversionfees": 0,
  "primarycurrencyout": 0,
  "primarycurrencyin": 0,
  "preconvertedout": 0,
  "preconvertedin": 0,
  "initialweights": [1.0],
  "launchcleared": true,
  "options": 0
}
```

### getcurrencyconverters

Get currency converters.

**Parameters:** `[string currency_name]`

**Returns:**
```json
[
  {
    "currencyid": "0000000000000000000000000000000000000000000000000000000000000000",
    "name": "converter_name",
    "systemid": "system_id",
    "notarizationprotocol": 1,
    "proofprotocol": 1,
    "startblock": 123456,
    "endblock": 0
  }
]
```

## ⛏️ Mining Operations Methods

### getblocktemplate

Get block template for mining.

**Parameters:** `[object template_request]`

**Returns:**
```json
{
  "version": 1,
  "previousblockhash": "0000000000000000000000000000000000000000000000000000000000000000",
  "transactions": [...],
  "coinbaseaux": {...},
  "coinbasevalue": 1234567890,
  "target": "0000000000000000000000000000000000000000000000000000000000000000",
  "mintime": 1234567890,
  "mutable": [...],
  "noncerange": "00000000ffffffff",
  "sigoplimit": 20000,
  "sizelimit": 1000000,
  "curtime": 1234567890,
  "bits": "1d00ffff",
  "height": 123456
}
```

### submitblock

Submit a mined block.

**Parameters:** `[string hexdata, string hash]`

**Returns:** `string` (result)

### getmininginfo

Get mining information.

**Parameters:** `[]`

**Returns:**
```json
{
  "blocks": 123456,
  "currentblocksize": 1234,
  "currentblocktx": 123,
  "difficulty": 123456.789,
  "errors": "",
  "genproclimit": 1,
  "networkhashps": 1234567890,
  "pooledtx": 123,
  "testnet": false,
  "chain": "main",
  "generate": false,
  "hashespersec": 123456
}
```

### getnetworkhashps

Get network hash rate.

**Parameters:** `[number blocks, number height]`

**Returns:** `number`

## 🌐 Network Operations Methods

### getnetworkinfo

Get network information.

**Parameters:** `[]`

**Returns:**
```json
{
  "version": 123456,
  "subversion": "/Verus:0.8.0/",
  "protocolversion": 123456,
  "localservices": "0000000000000001",
  "localservicesnames": ["NETWORK"],
  "localrelay": true,
  "timeoffset": 0,
  "networkactive": true,
  "connections": 8,
  "networks": [
    {
      "name": "ipv4",
      "limited": false,
      "reachable": true,
      "proxy": "",
      "proxy_randomize_credentials": false
    }
  ],
  "relayfee": 0.00001,
  "incrementalfee": 0.00001,
  "localaddresses": [],
  "warnings": ""
}
```

### getpeerinfo

Get peer information.

**Parameters:** `[]`

**Returns:**
```json
[
  {
    "id": 1,
    "addr": "192.168.1.100:27486",
    "addrlocal": "192.168.1.50:12345",
    "addrbind": "192.168.1.50:12345",
    "services": "0000000000000001",
    "servicesnames": ["NETWORK"],
    "relaytxes": true,
    "lastsend": 1234567890,
    "lastrecv": 1234567890,
    "bytessent": 123456,
    "bytesrecv": 123456,
    "conntime": 1234567890,
    "timeoffset": 0,
    "pingtime": 0.05,
    "minping": 0.05,
    "version": 123456,
    "subver": "/Verus:0.8.0/",
    "inbound": false,
    "addnode": false,
    "startingheight": 123456,
    "banscore": 0,
    "synced_headers": 123456,
    "synced_blocks": 123456,
    "inflight": [],
    "whitelisted": false,
    "minfeefilter": 0.00001,
    "bytessent_per_msg": {...},
    "bytesrecv_per_msg": {...}
  }
]
```

## 🔧 Advanced Methods

### estimatefee

Estimate transaction fee.

**Parameters:** `[number conf_target]`

**Returns:** `number`

### estimatepriority

Estimate transaction priority.

**Parameters:** `[number conf_target]`

**Returns:** `number`

### validateaddress

Validate an address.

**Parameters:** `[string address]`

**Returns:**
```json
{
  "isvalid": true,
  "address": "address",
  "scriptPubKey": "script",
  "ismine": false,
  "iswatchonly": false,
  "isscript": false,
  "pubkey": "pubkey",
  "iscompressed": true,
  "account": ""
}
```

### verifymessage

Verify a signed message.

**Parameters:** `[string address, string signature, string message]`

**Returns:** `boolean`

## 📊 Method Statistics

### Usage Statistics

The server tracks method usage for monitoring and optimization:

```json
{
  "method_stats": {
    "getinfo": {
      "total_requests": 1234,
      "avg_response_time": 0.05,
      "error_rate": 0.001
    },
    "getblock": {
      "total_requests": 567,
      "avg_response_time": 0.12,
      "error_rate": 0.002
    }
  }
}
```

### Performance Metrics

Each method includes performance tracking:

- **Response Time**: Average, 95th percentile, 99th percentile
- **Error Rate**: Percentage of failed requests
- **Throughput**: Requests per second
- **Cache Hit Ratio**: For cacheable methods

## 🔗 Related Documentation

- [Request/Response Format](./request-response.md) - API request/response format
- [Authentication](./authentication.md) - Authentication requirements
- [Error Handling](./error-handling.md) - Error codes and handling
- [Rate Limiting](./rate-limiting.md) - Rate limiting by method
