// `cargo run --bin rebrand` — push configured branding (CA / socials / name)
// into the cloned frontend in public/. Standalone (no crate deps).
use serde_json::Value;
use std::path::Path;

fn env_or_file<'a>(env_key: &str, file: &'a Value, path: &[&str]) -> String {
    if let Ok(v) = std::env::var(env_key) {
        let v = v.trim();
        if !v.is_empty() {
            return v.to_string();
        }
    }
    let mut cur = file;
    for p in path {
        cur = match cur.get(p) {
            Some(x) => x,
            None => return String::new(),
        };
    }
    cur.as_str().unwrap_or("").to_string()
}

fn main() {
    dotenvy::dotenv().ok();

    let file: Value = std::fs::read_to_string("config/branding.json")
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or(Value::Null);

    // resolved branding values keyed by the dotted keys used in replacements.json
    let site_name = env_or_file("SITE_NAME", &file, &["siteName"]);
    let resolved = serde_json::json!({
        "siteName": site_name,
        "siteName_lower": site_name.to_lowercase(),
        "ticker": env_or_file("TOKEN_TICKER", &file, &["ticker"]),
        "contractAddress": env_or_file("CONTRACT_ADDRESS", &file, &["contractAddress"]),
        "socials": {
            "twitter": env_or_file("TWITTER_URL", &file, &["socials","twitter"]),
            "github": env_or_file("GITHUB_URL", &file, &["socials","github"]),
            "discord": env_or_file("DISCORD_URL", &file, &["socials","discord"]),
            "telegram": env_or_file("TELEGRAM_URL", &file, &["socials","telegram"]),
            "website": env_or_file("WEBSITE_URL", &file, &["socials","website"]),
        },
        "urls": {
            "wallets": env_or_file("WALLETS_URL", &file, &["urls","wallets"]),
            "explorer": env_or_file("EXPLORER_URL", &file, &["urls","explorer"]),
            "rpc_worldstreet": env_or_file("CHAIN_RPC_URL", &file, &["urls","rpc_worldstreet"]),
            "rpc_worldstreet_healthz": env_or_file("CHAIN_RPC_HEALTHZ_URL", &file, &["urls","rpc_worldstreet_healthz"]),
            "rpc_worldstreet_metrics": env_or_file("CHAIN_RPC_METRICS_URL", &file, &["urls","rpc_worldstreet_metrics"]),
            "rpc_mainnet": env_or_file("CHAIN_RPC_MAINNET_URL", &file, &["urls","rpc_mainnet"]),
            "rpc_testnet": env_or_file("CHAIN_RPC_TESTNET_URL", &file, &["urls","rpc_testnet"]),
        },
        "domain": env_or_file("DOMAIN", &file, &["domain"]),
        "domain_wallets": env_or_file("DOMAIN_WALLETS", &file, &["domain_wallets"]),
        "chainRef": env_or_file("CHAIN_REF", &file, &["chainRef"]),
        "networkName": env_or_file("NETWORK_NAME", &file, &["networkName"]),
    });

    let repl: Value = std::fs::read_to_string("config/replacements.json")
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or(Value::Null);
    let map = repl.get("map").and_then(|m| m.as_object()).cloned().unwrap_or_default();

    let pairs: Vec<(String, String)> = map
        .iter()
        .filter_map(|(from, key)| {
            let key = key.as_str()?;
            let mut cur = &resolved;
            for p in key.split('.') {
                cur = cur.get(p)?;
            }
            let to = cur.as_str()?.to_string();
            if from.is_empty() || to.is_empty() || *from == to {
                None
            } else {
                Some((from.clone(), to))
            }
        })
        .collect();

    if pairs.is_empty() {
        println!("No replacements configured (config/replacements.json -> map empty). Nothing to do.");
        return;
    }

    let public_dir = std::env::var("PUBLIC_DIR")
        .ok()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "../frontend".into());

    let mut files = 0;
    let mut hits = 0;
    walk(Path::new(&public_dir), &pairs, &mut files, &mut hits);
    println!("rebrand: {} replacement(s), touched {files} file(s), {hits} substitution(s).", pairs.len());
    for (from, to) in &pairs {
        println!("  {from}  ->  {to}");
    }
}

fn is_text(name: &str) -> bool {
    matches!(
        name.rsplit('.').next().map(|e| e.to_ascii_lowercase()).as_deref(),
        Some("html") | Some("htm") | Some("js") | Some("mjs") | Some("css")
            | Some("json") | Some("txt") | Some("svg") | Some("xml") | Some("webmanifest")
    )
}

fn walk(dir: &Path, pairs: &[(String, String)], files: &mut usize, hits: &mut usize) {
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            walk(&path, pairs, files, hits);
        } else if path.file_name().and_then(|n| n.to_str()).map(is_text).unwrap_or(false) {
            let Ok(mut s) = std::fs::read_to_string(&path) else { continue };
            let mut changed = false;
            for (from, to) in pairs {
                if s.contains(from.as_str()) {
                    s = s.replace(from.as_str(), to);
                    changed = true;
                    *hits += 1;
                }
            }
            if changed && std::fs::write(&path, s).is_ok() {
                *files += 1;
            }
        }
    }
}
