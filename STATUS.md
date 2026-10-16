# intertrain-wallet -- Integration Status

**Date:** 2026-10-16
**Build:** cargo build -- PASS (0 errors, 44 dead-code warnings)
**Server boot:** cargo run --bin generated-site -- PASS (port 8787)
**Smoke test:** 28/28 passed, 0 failed

## Endpoints

### GET endpoints

| Endpoint         | Status | Notes                                |
|------------------|--------|--------------------------------------|
| GET /healthz     | PASS   | Returns "ok" (text/plain)            |
| GET /metrics     | PASS   | Prometheus format, reports data_mode |
| GET /api/_health | PASS   | JSON: ok, dataMode, clickhouse       |
| GET /api/_branding | PASS | JSON: name, ticker, socials, theme   |
| GET /            | PASS   | Serves frontend (redirect to wallet) |

### JSON-RPC methods (POST /api/rpc and /api-mainnet/rpc)

All 21 RPC methods return valid JSON-RPC 2.0 responses.

| RPC Method                | Status | Data Source | Notes                              |
|---------------------------|--------|-------------|------------------------------------|
| chain_info                | PASS   | seed        | Chain ID, block height, native asset |
| validator_set             | PASS   | seed        | 4 validators                       |
| bridge_status             | PASS   | seed        | Ethereum + Solana lanes            |
| account_get               | PASS   | seed/rocks  | Falls back to seed, per-account state in rocks |
| transaction_broadcast     | PASS   | simulated   | Generates hash, persists to rocks  |
| transaction_status        | PASS   | rocks       | Looks up by hash, returns not_found if unknown |
| token_list                | PASS   | seed        | 3 wrapped tokens (wUSDC, wETH, wSOL) |
| token_evm_address         | PASS   | seed        | Derives EVM address for a token    |
| token_operation_prepare   | PASS   | seed        | Returns signing bytes              |
| token_operation_broadcast | PASS   | simulated   | Generates op_id, persists to rocks |
| token_operation_status    | PASS   | rocks       | Looks up by hash                   |
| mna_quote                 | PASS   | computed    | 1:1 USDC->WSK rate if param given, else seed |
| mna_swap_prepare          | PASS   | seed        | Returns signing bytes              |
| mna_swap_broadcast        | PASS   | simulated   | Generates op_id, persists to rocks |
| mna_swap_status           | PASS   | rocks       | Looks up by hash                   |
| mna_reserve_status        | PASS   | seed        | Fixed rate + collateralized flag   |
| asset_list                | PASS   | seed        | 4 protocol assets                  |
| auth_challenge            | PASS   | computed    | Generates nonce + challenge message |
| auth_verify               | PASS   | simulated   | Always returns session token       |
| devnet_faucet             | PASS   | simulated   | Credits 100 WSK to account in rocks |
| sol_mna_quote             | PASS   | seed/computed | SOL->WSK bridge lane quote       |

### Error handling
| Scenario             | Status | Notes                              |
|----------------------|--------|------------------------------------|
| Unknown RPC method   | PASS   | Returns -32601 "Method not found"  |
| Unknown API path     | PASS   | Returns 404 JSON for /api/*, 404 text otherwise |

## Rebrand

Rebrand binary (`cargo run --bin rebrand`) applied 7 replacement rules across 3 frontend files with 13 total substitutions. Original "Intertrain" / "WSK" references replaced with placeholder branding ("My Wallet" / "TICKER").

## Architecture

- **Runtime:** Axum 0.8 on Tokio, single binary
- **Data:** RocksDB (embedded, always available) + ClickHouse (optional, analytics)
- **22 seed collections** loaded from seed/*.json into RocksDB on boot
- **6 route groups** mounted via inventory (meta, health, rpc x2 paths, branding)
- **Frontend:** Cloned Next.js app served as static files via fallback handler
- **SPA detection:** Reads site.json strategy field; falls back to directory-style

## Known Gaps

1. **ClickHouse not connected** -- expected in local dev (no Docker). Falls back to seed data. No impact on functionality.
2. **Services (chain, auth, bridge, solana, jupiter, dexscreener, ipfs)** are defined but not called from routes. The RPC dispatch in rpc.rs handles everything inline via seed data + RocksDB state. The services exist as scaffolding for a future "live" data mode.
3. **example_coins.rs** exists but is not wired into routes/mod.rs -- dead code from the template. No impact.
4. **Frontend redirect:** Root / serves a meta-refresh redirect to /mirror/wallet/index.html rather than serving the SPA directly. This matches the cloned site structure.
5. **No HTTPS locally** -- expected. Railway provides TLS termination in production.
