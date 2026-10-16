// ─── JSON-RPC 2.0 dispatch for /api/rpc and /api-mainnet/rpc ─────────────────
// 21 RPC methods for the WorldStreet wallet backend.  Most return seed data
// from RocksDB; stateful methods (broadcast, faucet, account) read/write
// per-entity keys so subsequent status polls see realistic state.

use crate::db::rocks;
use crate::routes::RouteReg;
use crate::state::AppState;
use axum::{extract::State, routing::post, Json, Router};
use serde::Deserialize;
use serde_json::{json, Value};

// ─── JSON-RPC envelope ───────────────────────────────────────────────────

#[derive(Deserialize)]
struct RpcRequest {
    #[allow(dead_code)]
    jsonrpc: Option<String>,
    id: Value,
    method: String,
    #[serde(default)]
    params: Value,
}

fn rpc_ok(id: &Value, result: Value) -> Value {
    json!({ "jsonrpc": "2.0", "id": id, "result": result })
}

fn rpc_err(id: &Value, code: i64, message: &str) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "error": { "code": code, "message": message }
    })
}

/// Strip the `_note` documentation field that seed JSON files carry.
fn strip_note(mut v: Value) -> Value {
    if let Some(obj) = v.as_object_mut() {
        obj.remove("_note");
    }
    v
}

/// Generate a deterministic-looking 64-char hex hash from the current timestamp.
fn fake_hash() -> String {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    format!("{:0>64x}", nanos)
}

/// Load a seed collection from RocksDB and strip the `_note` field.
fn seed(st: &AppState, name: &str) -> Value {
    strip_note(rocks::seed_all(&st.rocks, name))
}

// ─── Top-level dispatch (shared by both mount paths) ─────────────────────

async fn handle_rpc(State(st): State<AppState>, Json(req): Json<RpcRequest>) -> Json<Value> {
    let id = &req.id;
    let p = &req.params;

    let result = match req.method.as_str() {
        // ── Chain / network ────────────────────────────────
        "chain_info"                => h_chain_info(&st),
        "validator_set"             => h_validator_set(&st),
        "bridge_status"             => h_bridge_status(&st),

        // ── Account ────────────────────────────────────────
        "account_get"               => h_account_get(&st, p),

        // ── Transactions ───────────────────────────────────
        "transaction_broadcast"     => h_transaction_broadcast(&st, p),
        "transaction_status"        => h_transaction_status(&st, p),

        // ── Tokens ─────────────────────────────────────────
        "token_list"                => h_token_list(&st),
        "token_evm_address"         => h_token_evm_address(&st, p),
        "token_operation_prepare"   => h_token_operation_prepare(&st),
        "token_operation_broadcast" => h_token_operation_broadcast(&st, p),
        "token_operation_status"    => h_token_operation_status(&st, p),

        // ── Reserve swap (MNA / WSK) ───────────────────────
        "mna_quote"                 => h_mna_quote(&st, p),
        "mna_swap_prepare"          => h_mna_swap_prepare(&st),
        "mna_swap_broadcast"        => h_mna_swap_broadcast(&st),
        "mna_swap_status"           => h_mna_swap_status(&st, p),
        "mna_reserve_status"        => h_mna_reserve_status(&st),

        // ── Assets ─────────────────────────────────────────
        "asset_list"                => h_asset_list(&st),

        // ── Auth ───────────────────────────────────────────
        "auth_challenge"            => h_auth_challenge(p),
        "auth_verify"               => h_auth_verify(),

        // ── Devnet ─────────────────────────────────────────
        "devnet_faucet"             => h_devnet_faucet(&st, p),

        // ── Solana bridge ──────────────────────────────────
        "sol_mna_quote"             => h_sol_mna_quote(&st, p),

        _ => return Json(rpc_err(id, -32601, "Method not found")),
    };

    Json(rpc_ok(id, result))
}

// ═══════════════════════════════════════════════════════════════════════════
//  Handler implementations — one per RPC method
// ═══════════════════════════════════════════════════════════════════════════

// ── Pure-seed handlers (stateless, return seed data unchanged) ────────────

fn h_chain_info(st: &AppState) -> Value {
    seed(st, "rpc.chain_info")
}

fn h_validator_set(st: &AppState) -> Value {
    seed(st, "rpc.validator_set")
}

fn h_bridge_status(st: &AppState) -> Value {
    seed(st, "rpc.bridge_status")
}

fn h_token_list(st: &AppState) -> Value {
    seed(st, "rpc.token_list")
}

fn h_asset_list(st: &AppState) -> Value {
    seed(st, "rpc.asset_list")
}

fn h_mna_reserve_status(st: &AppState) -> Value {
    seed(st, "rpc.mna_reserve_status")
}

// ── account_get: per-account state in rocks, fallback seed ───────────────

fn h_account_get(st: &AppState, params: &Value) -> Value {
    let addr = params
        .get("address")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    // Per-account state (written by faucet / broadcast side-effects).
    if !addr.is_empty() {
        if let Some(acct) = rocks::get_json(&st.rocks, &format!("account:{addr}")) {
            return acct;
        }
    }

    // Fall back to the seed demo account.
    seed(st, "rpc.account_get")
}

// ── transaction_broadcast: persist tx, return hash ───────────────────────

fn h_transaction_broadcast(st: &AppState, params: &Value) -> Value {
    let hash = fake_hash();

    let unsigned = params
        .get("transaction")
        .and_then(|t| t.get("unsigned"));

    let from = unsigned
        .and_then(|u| u.get("from"))
        .cloned()
        .unwrap_or_else(|| json!("unknown"));
    let to = unsigned
        .and_then(|u| u.get("to"))
        .cloned()
        .unwrap_or_else(|| json!("unknown"));
    let amount = unsigned
        .and_then(|u| u.get("amount"))
        .cloned()
        .unwrap_or_else(|| json!("0"));

    let record = json!({
        "status": "confirmed",
        "transaction": {
            "hash": &hash,
            "version": 1,
            "chain_id": "worldstreet-devnet-1",
            "from": from,
            "to": to,
            "amount": amount,
            "fee": "1",
            "block_height": 184720,
            "block_hash": "ff00ff00ff00ff00ff00ff00ff00ff00ff00ff00ff00ff00ff00ff00ff00ff00",
            "memo": ""
        }
    });
    let _ = rocks::put_json(&st.rocks, &format!("tx:{hash}"), &record);

    json!({ "hash": hash })
}

// ── transaction_status: look up by hash ──────────────────────────────────

fn h_transaction_status(st: &AppState, params: &Value) -> Value {
    let hash = params.get("hash").and_then(|v| v.as_str()).unwrap_or("");

    if !hash.is_empty() {
        if let Some(record) = rocks::get_json(&st.rocks, &format!("tx:{hash}")) {
            return record;
        }
    }

    // Unknown hash → not_found (not the seed "confirmed" record).
    json!({ "status": "not_found", "transaction": null })
}

// ── token_evm_address: derive EVM address for a token ────────────────────

fn h_token_evm_address(st: &AppState, params: &Value) -> Value {
    let _hash = params.get("hash").and_then(|v| v.as_str()).unwrap_or("");
    // In simulation mode, return the seed response for any token.
    seed(st, "rpc.token_evm_address")
}

// ── token_operation_prepare: return signing bytes ────────────────────────

fn h_token_operation_prepare(st: &AppState) -> Value {
    seed(st, "rpc.token_operation_prepare")
}

// ── token_operation_broadcast: persist, return ids ───────────────────────

fn h_token_operation_broadcast(st: &AppState, params: &Value) -> Value {
    let op_id = fake_hash();

    let token_id = params
        .get("operation")
        .and_then(|o| o.get("unsigned"))
        .and_then(|u| u.get("token_id"))
        .and_then(|v| v.as_str())
        .unwrap_or("aa00000000000000000000000000000000000000000000000000000000000001");

    let record = json!({
        "status": "confirmed",
        "operation": {
            "operation_id": &op_id,
            "kind": "Transfer",
            "token_id": token_id,
            "from": params.get("operation")
                .and_then(|o| o.get("unsigned"))
                .and_then(|u| u.get("from"))
                .cloned()
                .unwrap_or_else(|| json!("unknown")),
            "block_height": 184720
        }
    });
    let _ = rocks::put_json(&st.rocks, &format!("token_op:{op_id}"), &record);

    json!({ "operation_id": op_id, "token_id": token_id })
}

// ── token_operation_status ───────────────────────────────────────────────

fn h_token_operation_status(st: &AppState, params: &Value) -> Value {
    let hash = params.get("hash").and_then(|v| v.as_str()).unwrap_or("");

    if !hash.is_empty() {
        if let Some(record) = rocks::get_json(&st.rocks, &format!("token_op:{hash}")) {
            return record;
        }
    }

    json!({ "status": "not_found", "operation": null })
}

// ── mna_quote: fixed 1:1 USDC→WSK rate ──────────────────────────────────

fn h_mna_quote(st: &AppState, params: &Value) -> Value {
    if let Some(amount) = params.get("amount_usdc").and_then(|v| v.as_str()) {
        json!({
            "amount_usdc": amount,
            "amount_mna": amount,
            "rate": "1 USDC = 1 WSK"
        })
    } else {
        seed(st, "rpc.mna_quote")
    }
}

// ── mna_swap_prepare: return signing bytes ───────────────────────────────

fn h_mna_swap_prepare(st: &AppState) -> Value {
    seed(st, "rpc.mna_swap_prepare")
}

// ── mna_swap_broadcast: persist, return operation_id ─────────────────────

fn h_mna_swap_broadcast(st: &AppState) -> Value {
    let op_id = fake_hash();

    let record = json!({
        "status": "confirmed",
        "operation": {
            "operation_id": &op_id,
            "kind": "MintMna",
            "block_height": 184720
        }
    });
    let _ = rocks::put_json(&st.rocks, &format!("mna_swap:{op_id}"), &record);

    json!({ "operation_id": op_id })
}

// ── mna_swap_status ──────────────────────────────────────────────────────

fn h_mna_swap_status(st: &AppState, params: &Value) -> Value {
    let hash = params.get("hash").and_then(|v| v.as_str()).unwrap_or("");

    if !hash.is_empty() {
        if let Some(record) = rocks::get_json(&st.rocks, &format!("mna_swap:{hash}")) {
            return record;
        }
    }

    json!({ "status": "not_found", "operation": null })
}

// ── auth_challenge: generate a one-time challenge ────────────────────────

fn h_auth_challenge(params: &Value) -> Value {
    let address = params
        .get("address")
        .and_then(|v| v.as_str())
        .unwrap_or("wsk1unknown");
    let domain = params
        .get("domain")
        .and_then(|v| v.as_str())
        .unwrap_or("localhost");
    let full = fake_hash();
    let nonce = &full[..12];

    let message = format!(
        "Sign this message to prove you control this address.\n\n\
         Domain: {domain}\nAddress: {address}\nNonce: {nonce}\n\
         Timestamp: 2026-10-16T10:00:00Z"
    );

    json!({ "message": message, "nonce": nonce })
}

// ── auth_verify: always accept in simulated mode ─────────────────────────

fn h_auth_verify() -> Value {
    json!({
        "session_token": "eyJhbGciOiJFZDI1NTE5IiwidHlwIjoiSldUIn0.SIM.PLACEHOLDER_SIGNATURE"
    })
}

// ── devnet_faucet: credit 100 WSK to the given address ───────────────────

fn h_devnet_faucet(st: &AppState, params: &Value) -> Value {
    let address = params
        .get("address")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let hash = fake_hash();
    let credit: u64 = 100_000_000; // 100 WSK in microWSK

    if !address.is_empty() {
        let key = format!("account:{address}");
        let mut acct = rocks::get_json(&st.rocks, &key).unwrap_or_else(|| {
            json!({
                "address": address,
                "balance": "0",
                "nonce": 0,
                "assets": {}
            })
        });

        let current: u64 = acct
            .get("balance")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);
        acct["balance"] = json!((current + credit).to_string());

        let _ = rocks::put_json(&st.rocks, &key, &acct);
    }

    json!({ "hash": hash, "amount": credit.to_string() })
}

// ── sol_mna_quote: oracle quote for SOL→WSK ──────────────────────────────

fn h_sol_mna_quote(st: &AppState, params: &Value) -> Value {
    let s = seed(st, "rpc.sol_mna_quote");

    let amount_lamports = params
        .get("amount_lamports")
        .and_then(|v| v.as_str())
        .unwrap_or(
            s.get("amount_lamports")
                .and_then(|v| v.as_str())
                .unwrap_or("0"),
        );

    json!({
        "amount_mna":       s.get("amount_mna").cloned().unwrap_or(json!("25000000")),
        "fee_bps":          s.get("fee_bps").cloned().unwrap_or(json!(50)),
        "sol_price_usd":    s.get("sol_price_usd").cloned().unwrap_or(json!("25.00")),
        "amount_lamports":  amount_lamports,
    })
}

// ─── Mount at both RPC paths ─────────────────────────────────────────────

inventory::submit! {
    RouteReg {
        mount: |r: Router<AppState>| r.route("/api/rpc", post(handle_rpc))
    }
}

inventory::submit! {
    RouteReg {
        mount: |r: Router<AppState>| r.route("/api-mainnet/rpc", post(handle_rpc))
    }
}
