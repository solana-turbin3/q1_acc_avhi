# Pyth Scheduler

A Solana program that fetches the **SOL/USD price from Pyth** and stores it on-chain, with **automated recurring updates** powered by [TukTuk](https://www.tuktuk.fun) - a permissionless crank scheduler.

The program uses Pyth's **pull oracle** model - price data is fetched from [Hermes](https://hermes.pyth.network) and posted on-chain before being read. TukTuk schedules the `update_price` instruction to keep the stored price fresh automatically.

---

## Architecture

The program has 1 state account:

### PriceStore

A PDA that stores the latest SOL/USD price data fetched from Pyth.

```rust
#[account]
#[derive(InitSpace)]
pub struct PriceStore {
    pub price: i64,        // the price value
    pub exponent: i32,     // price * 10^exponent = actual price
    pub confidence: u64,   // uncertainty range
    pub published_at: i64, // unix timestamp from pyth
    pub bump: u8,
}
```

- **price**: Raw price integer from Pyth (e.g. `8787000000`)
- **exponent**: Scaling exponent (e.g. `-8`), so actual price = `price * 10^exponent`
- **confidence**: Half-width of the confidence interval (same exponent as price)
- **published_at**: Unix timestamp when Pyth published this price
- **bump**: Bump seed used to derive the PDA

---

## Instructions

### 1. Update Price

Reads a fresh `PriceUpdateV2` account (posted via the Pyth pull oracle) and stores the SOL/USD price in `PriceStore`. Verifies the feed ID matches SOL/USD and that the price is no older than 300 seconds.

```rust
#[derive(Accounts)]
pub struct UpdatePrice<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        init_if_needed,
        payer = payer,
        seeds = [PRICE.as_bytes()],
        bump,
        space = 8 + PriceStore::INIT_SPACE
    )]
    pub price_store: Account<'info, PriceStore>,

    pub price_feed: Account<'info, PriceUpdateV2>,

    pub system_program: Program<'info, System>,
}
```

**Process:**
1. Verify the `PriceUpdateV2` account contains the SOL/USD feed ID
2. Check the price is no older than 300 seconds
3. Store `price`, `exponent`, `confidence`, and `published_at` into `PriceStore`

---

### 2. Schedule

Registers a TukTuk task that will call `update_price` via a CPI. Uses `TriggerV0::Now` so it fires immediately - useful for testing the full scheduling flow.

```rust
#[derive(Accounts)]
pub struct Schedule<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(seeds = [PRICE.as_bytes()], bump = price_store.bump)]
    pub price_store: Account<'info, PriceStore>,

    /// CHECK: Pyth PriceUpdateV2 account, passed through to compiled TukTuk task
    pub price_feed: UncheckedAccount<'info>,

    /// CHECK: Passed through to TukTuk CPI
    #[account(mut)]
    pub task_queue: UncheckedAccount<'info>,

    /// CHECK: Derived and verified by TukTuk program
    #[account(mut)]
    pub task_queue_authority: UncheckedAccount<'info>,

    /// CHECK: Initialized in CPI
    #[account(mut)]
    pub task: UncheckedAccount<'info>,

    #[account(mut, seeds = [QUEUE_AUTHORITY_SEED], bump)]
    pub queue_authority: AccountInfo<'info>,

    pub tuktuk_program: Program<'info, Tuktuk>,
    pub system_program: Program<'info, System>,
}
```

**Parameters:**
- `task_id`: Available slot in the TukTuk task queue bitmap (0–capacity)

**Process:**
1. Build `CompiledTransactionV0` encoding the `update_price` instruction with all required accounts
2. CPI into TukTuk's `queue_task_v0`, signing with the `queue_authority` PDA

---

## Pyth Pull Oracle

This program uses Pyth's **pull oracle** model via `pyth-solana-receiver-sdk = "1.1.0"`. Unlike the legacy push oracle (which continuously posts to a fixed account), the pull oracle requires explicitly fetching the latest signed price from Hermes and posting it on-chain before reading.

### Flow

```
1. Fetch signed price update from Hermes (off-chain)
      └─> GET https://hermes.pyth.network/v2/updates/price/latest?ids[]=<FEED_ID>

2. Post to Pyth Receiver program (on-chain)
      └─> Creates / updates PriceUpdateV2 account at deterministic address
            └─> seeds = [b"price_update", shard_id, feed_id]
                  on program rec5EKMGg6MxZYaMdyBfgwp4d5rB9T1VQH5pJv5LtFJ

3. Call update_price with the PriceUpdateV2 account
      └─> Reads price, verifies feed ID + freshness
            └─> Stores result in PriceStore PDA
```

### SOL/USD Feed

| | |
|---|---|
| Feed ID | `0xef0d8b6fda2ceba41da15d4095d1da392a0d2f8ed0c6c7bc0f4cfac8c280b56d` |
| PriceUpdateV2 account (shard 0) | `7UVimffxr9ow1uXYxsr4LHAcV58mLzhmwaeKvJ1pjLiE` |

---

## TukTuk Integration

### How it works

TukTuk is a permissionless crank network on Solana. Anyone can run a cranker that monitors task queues and executes tasks when their trigger condition is met.

```
caller calls schedule()
  └─> CPI to TukTuk queue_task_v0
        └─> Task stored on-chain with TriggerV0::Now
              └─> TukTuk cranker picks up the task
                    └─> Calls update_price with fresh PriceUpdateV2 account
                          └─> SOL/USD price updated on-chain automatically
```

### Key accounts

| Account | Description |
|---|---|
| `task_queue` | TukTuk task queue (`UwdRmurFA11isBpDNY9HNcoL95Pnt4zNYE2cd1SQwn2`) |
| `queue_authority` | Program PDA (`seeds = [b"queue_authority"]`) registered as a TukTuk queue authority |
| `task_queue_authority` | TukTuk PDA linking the task queue to the queue authority |
| `task` | TukTuk PDA storing the queued task (`seeds = [b"task", task_queue, task_id]`) |

> **Note:** TukTuk's published crate targets Anchor 0.31.1 and is incompatible with Anchor 0.32.1. `tuktuk-program` is pulled from a fork with the Anchor dependency bumped: [`AvhiMaz/tuktuk @ chore/bump-versions`](https://github.com/AvhiMaz/tuktuk/tree/chore/bump-versions).

---

## Devnet Deployment

```
Program ID : 8iCfkBsLg5G7HkJZt95KnutdyqnpFuUdAskwMjiupDzu
Task queue : UwdRmurFA11isBpDNY9HNcoL95Pnt4zNYE2cd1SQwn2
```

---

## Build & Test

```bash
# Build the program
anchor build

# Deploy to devnet
anchor deploy --provider.cluster devnet

# Run tests
anchor test --skip-build --skip-deploy
```

The test suite does the following:

**Update Price test:**
1. Fetches the latest SOL/USD price update from Hermes
2. Posts it to the Pyth Receiver program on-chain (creates / refreshes the `PriceUpdateV2` account)
3. Calls `update_price` - reads the price and stores it in `PriceStore`
4. Logs the current SOL/USD price, confidence interval, and publish timestamp

**Schedule test:**
1. Registers the program's `queue_authority` PDA with the TukTuk task queue (if not already)
2. Finds the next available task slot in the queue bitmap
3. Calls `schedule` - CPIs into TukTuk to queue an `update_price` task with `TriggerV0::Now`

---

## Flow

```
UPDATE PRICE FLOW:
1. (off-chain) Fetch latest price from Hermes
2. (on-chain)  Post signed update to Pyth Receiver program
                   -> PriceUpdateV2 account refreshed at deterministic address
3. update_price()  -> Verifies feed ID + freshness (< 300s)
                   -> Writes price/exponent/confidence/published_at to PriceStore

SCHEDULE FLOW:
4. schedule(task_id) -> CPI to TukTuk queue_task_v0
                     -> Task queued: TriggerV0::Now
                     -> TukTuk cranker calls update_price automatically
```
