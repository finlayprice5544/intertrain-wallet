// Simulated chain state for the wallet's native chain.
// Returns deterministic responses so the frontend works offline.
// Never touches a real chain — all values are synthetic.
use serde_json::{json, Value};
use std::time::{SystemTime, UNIX_EPOCH};

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn pseudo_hash(seed: u64) -> String {
    format!("{:016x}{:016x}{:016x}{:016x}", seed, seed ^ 0xdeadbeef, seed.wrapping_mul(7), seed ^ 0xcafebabe)
}

/// Simulated chain_info response.
pub fn chain_info() -> Value {
    let ts = now_secs();
    let height = 1_000_000 + (ts % 100_000);
    json!({
        "chain_id": "worldstreet-1",
        "block_height": height,
        "block_hash": pseudo_hash(height),
        "genesis_time": "2024-01-01T00:00:00Z",
        "native_asset": {
            "symbol": "WSK",
            "decimals": 6
        }
    })
}

/// Simulated validator_set response.
pub fn validator_set() -> Value {
    json!({
        "validators": [
            { "address": "wskvaloper1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqszukzjj", "voting_power": 1000000, "moniker": "genesis-validator-0" },
            { "address": "wskvaloper1qyqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqd3v4rz3", "voting_power": 900000, "moniker": "genesis-validator-1" },
            { "address": "wskvaloper1qgqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqas6g3gy", "voting_power": 850000, "moniker": "genesis-validator-2" },
            { "address": "wskvaloper1qfqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqamdl77j", "voting_power": 800000, "moniker": "genesis-validator-3" },
        ]
    })
}

/// Simulated account_get response for any address.
pub fn account_get(address: &str) -> Value {
    // Deterministic balance from the address bytes so the same address always
    // returns the same data within a session.
    let seed: u64 = address.bytes().map(|b| b as u64).sum::<u64>().wrapping_mul(997);
    let balance = 1_000_000_000 + (seed % 50_000_000_000); // 1000..51000 WSK in micro
    json!({
        "address": address,
        "balance": balance.to_string(),
        "nonce": (seed % 200) as u32,
        "assets": {}
    })
}

/// Simulated transaction_status response.
pub fn transaction_status(hash: &str) -> Value {
    let ts = now_secs();
    json!({
        "status": "confirmed",
        "transaction": {
            "hash": hash,
            "from": "wsk1sender0000000000000000000000000000000000",
            "to": "wsk1receiver000000000000000000000000000000000",
            "amount": "1000000",
            "fee": "1000",
            "block_height": 1_000_000 + (ts % 100_000),
            "timestamp": ts
        }
    })
}

/// Simulated transaction_broadcast — returns a fake hash.
pub fn transaction_broadcast() -> Value {
    let ts = now_secs();
    json!({
        "hash": pseudo_hash(ts)
    })
}

/// Simulated token_list response.
pub fn token_list() -> Value {
    json!({
        "tokens": [
            {
                "token_id": "tok_0000000000000000000000000000000000000001",
                "name": "Wrapped USDC",
                "symbol": "wUSDC",
                "decimals": 6,
                "display_name": "Wrapped USDC"
            },
            {
                "token_id": "tok_0000000000000000000000000000000000000002",
                "name": "Wrapped Ether",
                "symbol": "wETH",
                "decimals": 18,
                "display_name": "Wrapped Ether"
            },
            {
                "token_id": "tok_0000000000000000000000000000000000000003",
                "name": "Wrapped SOL",
                "symbol": "wSOL",
                "decimals": 9,
                "display_name": "Wrapped SOL"
            }
        ]
    })
}

/// Simulated token_evm_address response.
pub fn token_evm_address(token_id: &str) -> Value {
    // Deterministic EVM address from token_id
    let seed: u64 = token_id.bytes().map(|b| b as u64).sum::<u64>().wrapping_mul(31);
    json!({
        "token_id": token_id,
        "evm_address": format!("0x{:040x}", seed),
        "symbol": "wTOK",
        "decimals": 18,
        "image": ""
    })
}

/// Simulated token_operation_status.
pub fn token_operation_status(op_hash: &str) -> Value {
    json!({
        "status": "confirmed",
        "operation": {
            "operation_id": op_hash,
            "kind": "Transfer",
            "token_id": "tok_0000000000000000000000000000000000000001",
            "block_height": 1_000_000 + (now_secs() % 100_000)
        }
    })
}

/// Simulated token_operation_prepare — returns signing bytes.
pub fn token_operation_prepare() -> Value {
    let ts = now_secs();
    json!({
        "signing_bytes": format!("SIMULATED_SIGNING_BYTES_{:x}", ts)
    })
}

/// Simulated token_operation_broadcast.
pub fn token_operation_broadcast() -> Value {
    let ts = now_secs();
    json!({
        "operation_id": pseudo_hash(ts),
        "token_id": "tok_0000000000000000000000000000000000000001"
    })
}

/// Simulated mna_quote (USDC -> WSK reserve swap).
pub fn mna_quote(amount_usdc: &str) -> Value {
    let usdc: f64 = amount_usdc.parse().unwrap_or(1_000_000.0);
    let rate = 0.10; // 1 USDC = 10 WSK
    let mna = usdc * (1.0 / rate);
    json!({
        "amount_usdc": amount_usdc,
        "amount_mna": format!("{:.0}", mna),
        "rate": format!("{:.8}", rate)
    })
}

/// Simulated mna_swap_status.
pub fn mna_swap_status(op_hash: &str) -> Value {
    json!({
        "status": "confirmed",
        "operation": {
            "operation_id": op_hash,
            "kind": "MintMna",
            "block_height": 1_000_000 + (now_secs() % 100_000)
        }
    })
}

/// Simulated mna_swap_prepare.
pub fn mna_swap_prepare() -> Value {
    let ts = now_secs();
    json!({
        "signing_bytes": format!("SIMULATED_SWAP_SIGNING_{:x}", ts)
    })
}

/// Simulated mna_swap_broadcast.
pub fn mna_swap_broadcast() -> Value {
    let ts = now_secs();
    json!({
        "operation_id": pseudo_hash(ts)
    })
}

/// Simulated mna_reserve_status.
pub fn mna_reserve_status() -> Value {
    json!({
        "rate": "0.10000000",
        "collateralized": true
    })
}

/// Simulated asset_list.
pub fn asset_list() -> Value {
    json!({
        "assets": [
            { "id": "native", "symbol": "WSK", "display_name": "WSK", "name": "WSK", "enabled": true, "decimals": 6 },
            { "id": "usdc", "symbol": "wUSDC", "display_name": "Wrapped USDC", "name": "Wrapped USDC", "enabled": true, "decimals": 6, "token_id": "tok_0000000000000000000000000000000000000001" },
            { "id": "weth", "symbol": "wETH", "display_name": "Wrapped Ether", "name": "Wrapped Ether", "enabled": true, "decimals": 18, "token_id": "tok_0000000000000000000000000000000000000002" },
            { "id": "wsol", "symbol": "wSOL", "display_name": "Wrapped SOL", "name": "Wrapped SOL", "enabled": true, "decimals": 9, "token_id": "tok_0000000000000000000000000000000000000003" }
        ]
    })
}

/// Simulated bridge_status.
pub fn bridge_status() -> Value {
    json!({
        "ethereum": {
            "network": "sepolia",
            "connected": true
        },
        "solana": {
            "network": "devnet",
            "mode": "light",
            "connected": true
        }
    })
}

/// Simulated sol_mna_quote (SOL -> WSK bridge lane).
pub fn sol_mna_quote(amount_lamports: &str) -> Value {
    let lamports: f64 = amount_lamports.parse().unwrap_or(1_000_000_000.0);
    let sol = lamports / 1e9;
    let sol_price_usd = 150.0; // placeholder
    let usd = sol * sol_price_usd;
    let wsk_rate = 10.0; // 1 USD = 10 WSK
    let mna = usd * wsk_rate * 1_000_000.0; // in micro-WSK
    json!({
        "amount_mna": format!("{:.0}", mna),
        "fee_bps": 50
    })
}

/// Simulated devnet_faucet.
pub fn devnet_faucet(address: &str) -> Value {
    let ts = now_secs();
    json!({
        "hash": pseudo_hash(ts ^ address.len() as u64),
        "amount": "10000000000" // 10,000 WSK in micro
    })
}
