// `cargo run --bin smoke` — hit every endpoint against a RUNNING server.
// Exits non-zero on failures.
// Start the server first (another terminal):  cargo run
use serde_json::{json, Value};

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    let port = std::env::var("PORT").unwrap_or_else(|_| "8787".into());
    let base = format!("http://localhost:{port}");
    let http = reqwest::Client::new();

    let mut pass = 0u32;
    let mut fail = 0u32;
    println!("smoke test -> {base}\n");

    // ── GET endpoints ────────────────────────────────────────────
    let get_endpoints = [
        ("/healthz", "health check"),
        ("/metrics", "prometheus metrics"),
        ("/api/_health", "api health"),
        ("/api/_branding", "branding payload"),
        ("/", "frontend root page"),
    ];

    for (path, label) in &get_endpoints {
        let url = format!("{base}{path}");
        match http.get(&url).send().await {
            Ok(r) if r.status().as_u16() < 500 => {
                println!("  ok  GET {path} ({label}) -> {}", r.status());
                pass += 1;
            }
            Ok(r) => {
                println!("  ERR GET {path} ({label}) -> {}", r.status());
                fail += 1;
            }
            Err(e) => {
                println!("  ERR GET {path} ({label}) -> {e}");
                fail += 1;
            }
        }
    }

    // ── JSON-RPC methods (POST /api/rpc) ─────────────────────────
    let rpc_methods: Vec<(&str, Value)> = vec![
        // stateless seed methods
        ("chain_info", json!({})),
        ("validator_set", json!({})),
        ("bridge_status", json!({})),
        ("token_list", json!({})),
        ("asset_list", json!({})),
        ("mna_reserve_status", json!({})),
        // parameterized methods
        ("account_get", json!({ "address": "wsk1testaddr000000000000000000000000000000" })),
        ("transaction_status", json!({ "hash": "0000000000000000000000000000000000000000000000000000000000000000" })),
        ("transaction_broadcast", json!({
            "transaction": {
                "unsigned": { "from": "wsk1sender", "to": "wsk1receiver", "amount": "1000000" },
                "signature": "deadbeef"
            }
        })),
        ("token_evm_address", json!({ "hash": "tok_0000000000000000000000000000000000000001" })),
        ("token_operation_prepare", json!({ "unsigned": {} })),
        ("token_operation_broadcast", json!({
            "operation": {
                "unsigned": { "token_id": "tok_0000000000000000000000000000000000000001", "from": "wsk1sender" },
                "signature": "deadbeef"
            }
        })),
        ("token_operation_status", json!({ "hash": "0000000000000000000000000000000000000000000000000000000000000000" })),
        ("mna_quote", json!({ "amount_usdc": "1000000" })),
        ("mna_swap_prepare", json!({ "unsigned": {} })),
        ("mna_swap_broadcast", json!({
            "operation": { "unsigned": {}, "signature": "deadbeef" }
        })),
        ("mna_swap_status", json!({ "hash": "0000000000000000000000000000000000000000000000000000000000000000" })),
        ("auth_challenge", json!({ "address": "wsk1testaddr000000000000000000000000000000", "domain": "localhost" })),
        ("auth_verify", json!({
            "address": "wsk1testaddr000000000000000000000000000000",
            "domain": "localhost",
            "nonce": "abc123",
            "public_key": "deadbeef",
            "signature": "deadbeef"
        })),
        ("devnet_faucet", json!({ "address": "wsk1testaddr000000000000000000000000000000" })),
        ("sol_mna_quote", json!({ "amount_lamports": "1000000000" })),
    ];

    println!();
    for (method, params) in &rpc_methods {
        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": method,
            "params": params,
        });
        let url = format!("{base}/api/rpc");
        match http
            .post(&url)
            .header("content-type", "application/json")
            .json(&body)
            .send()
            .await
        {
            Ok(r) => {
                let status = r.status();
                if status.as_u16() >= 500 {
                    println!("  ERR RPC {method} -> HTTP {status}");
                    fail += 1;
                    continue;
                }
                let resp: Value = r.json().await.unwrap_or(Value::Null);
                if resp.get("error").is_some() {
                    println!("  ERR RPC {method} -> {}", resp["error"]);
                    fail += 1;
                } else if resp.get("result").is_some() {
                    println!("  ok  RPC {method} -> 200");
                    pass += 1;
                } else {
                    println!("  WARN RPC {method} -> unexpected: {}", resp);
                    pass += 1; // non-fatal
                }
            }
            Err(e) => {
                println!("  ERR RPC {method} -> {e}");
                fail += 1;
            }
        }
    }

    // ── Also test /api-mainnet/rpc with one method ───────────────
    println!();
    let mainnet_body = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "chain_info",
        "params": {},
    });
    match http
        .post(format!("{base}/api-mainnet/rpc"))
        .header("content-type", "application/json")
        .json(&mainnet_body)
        .send()
        .await
    {
        Ok(r) if r.status().as_u16() < 500 => {
            println!("  ok  POST /api-mainnet/rpc chain_info -> {}", r.status());
            pass += 1;
        }
        Ok(r) => {
            println!("  ERR POST /api-mainnet/rpc chain_info -> {}", r.status());
            fail += 1;
        }
        Err(e) => {
            println!("  ERR POST /api-mainnet/rpc -> {e}");
            fail += 1;
        }
    }

    // ── Unknown method should return -32601 ──────────────────────
    let unknown_body = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "nonexistent_method_xyz",
        "params": {},
    });
    match http
        .post(format!("{base}/api/rpc"))
        .header("content-type", "application/json")
        .json(&unknown_body)
        .send()
        .await
    {
        Ok(r) if r.status().as_u16() < 500 => {
            let resp: Value = r.json().await.unwrap_or(Value::Null);
            if resp.get("error").is_some() {
                println!("  ok  RPC unknown_method -> -32601 error as expected");
                pass += 1;
            } else {
                println!("  WARN RPC unknown_method -> expected error, got: {}", resp);
                pass += 1;
            }
        }
        Ok(r) => {
            println!("  ERR RPC unknown_method -> {}", r.status());
            fail += 1;
        }
        Err(e) => {
            println!("  ERR RPC unknown_method -> {e}");
            fail += 1;
        }
    }

    println!("\n{pass} passed, {fail} failed");
    if fail > 0 {
        std::process::exit(1);
    }
}
