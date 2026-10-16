-- Wallet-specific analytics tables.
-- Extends 001_init.sql with structures for tracking simulated chain state.

-- Transactions submitted through the wallet (native WSK transfers).
CREATE TABLE IF NOT EXISTS app.transactions
(
    ts          DateTime64(3) DEFAULT now64(3),
    hash        String,
    chain_id    LowCardinality(String),
    from_addr   String,
    to_addr     String,
    amount      UInt64,          -- microWSK
    fee         UInt64,          -- microWSK
    nonce       UInt32,
    memo        String DEFAULT '',
    status      LowCardinality(String) DEFAULT 'pending',  -- pending | confirmed | not_found
    block_height UInt64 DEFAULT 0
)
ENGINE = MergeTree
ORDER BY (chain_id, ts);

-- Token operations (create, transfer, mint, burn, freeze).
CREATE TABLE IF NOT EXISTS app.token_operations
(
    ts             DateTime64(3) DEFAULT now64(3),
    operation_id   String,
    kind           LowCardinality(String),     -- Create | Transfer | Mint | Burn | Freeze
    token_id       String,
    from_addr      String,
    to_addr        String DEFAULT '',
    amount         String DEFAULT '0',
    fee            UInt64 DEFAULT 1,
    status         LowCardinality(String) DEFAULT 'pending',
    block_height   UInt64 DEFAULT 0,
    name           String DEFAULT '',
    symbol         LowCardinality(String) DEFAULT '',
    decimals       UInt8 DEFAULT 0
)
ENGINE = MergeTree
ORDER BY (kind, ts);

-- Reserve swap operations (USDC <-> WSK).
CREATE TABLE IF NOT EXISTS app.mna_swaps
(
    ts             DateTime64(3) DEFAULT now64(3),
    operation_id   String,
    kind           LowCardinality(String),     -- MintMna | RedeemMna
    from_addr      String,
    collateral_asset String,
    amount_usdc    UInt64,
    amount_mna     UInt64,
    fee            UInt64 DEFAULT 1,
    status         LowCardinality(String) DEFAULT 'pending',
    block_height   UInt64 DEFAULT 0
)
ENGINE = MergeTree
ORDER BY (kind, ts);

-- Bridge deposit and withdrawal events.
CREATE TABLE IF NOT EXISTS app.bridge_events
(
    ts              DateTime64(3) DEFAULT now64(3),
    lane            LowCardinality(String),     -- ethereum-sepolia | solana-devnet | arbitrum
    direction       LowCardinality(String),     -- deposit | withdrawal
    external_tx     String,                     -- tx hash on the external chain
    intertrain_addr String,
    amount          String,
    asset           LowCardinality(String),     -- USDC | wSOL | WETH
    status          LowCardinality(String) DEFAULT 'pending'
)
ENGINE = MergeTree
ORDER BY (lane, ts);

-- Auth challenge/verify audit log.
CREATE TABLE IF NOT EXISTS app.auth_events
(
    ts          DateTime64(3) DEFAULT now64(3),
    address     String,
    domain      String,
    action      LowCardinality(String),     -- challenge | verify
    nonce       String DEFAULT '',
    success     UInt8 DEFAULT 1
)
ENGINE = MergeTree
ORDER BY (address, ts);

-- Faucet drip log (devnet only).
CREATE TABLE IF NOT EXISTS app.faucet_events
(
    ts          DateTime64(3) DEFAULT now64(3),
    address     String,
    hash        String,
    amount      UInt64,           -- microWSK credited
    ip          String DEFAULT '' -- optional rate-limit key
)
ENGINE = MergeTree
ORDER BY (address, ts);

-- Account balance snapshots (periodic or on-change).
CREATE TABLE IF NOT EXISTS app.account_snapshots
(
    ts          DateTime64(3) DEFAULT now64(3),
    address     String,
    balance     UInt64,           -- microWSK
    nonce       UInt32,
    chain_id    LowCardinality(String)
)
ENGINE = MergeTree
ORDER BY (address, ts);

-- Sol bridge quotes and completions (SOL -> WSK lane).
CREATE TABLE IF NOT EXISTS app.sol_bridge_events
(
    ts              DateTime64(3) DEFAULT now64(3),
    sol_address     String,
    wsk_address     String,
    amount_lamports UInt64,
    amount_mna      UInt64,
    fee_bps         UInt16,
    sol_tx          String DEFAULT '',
    wsk_tx          String DEFAULT '',
    status          LowCardinality(String) DEFAULT 'pending'
)
ENGINE = MergeTree
ORDER BY (sol_address, ts);
