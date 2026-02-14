# TukTuk Escrow

A trustless token escrow program on Solana with **automated expiry refunds** powered by [TukTuk](https://www.tuktuk.fun) - a permissionless crank scheduler.

Alice (maker) deposits Token A and specifies how much Token B she wants in return. Bob (taker) can fulfill the trade before the escrow expires. If no taker shows up, the escrow automatically expires and TukTuk's crankers call `auto_refund` to return Alice's tokens - no manual intervention needed.

This is built on top of the standard escrow pattern with two additions: an `expires_at` timestamp on every escrow, and a `schedule` instruction that registers a TukTuk task to fire `auto_refund` at exactly that timestamp.

---

## Architecture

The program has 1 state account:

### Escrow

A PDA that stores the details of an active offer and controls the vault holding the maker's tokens.

```rust
#[account]
#[derive(InitSpace, Debug)]
pub struct Escrow {
    pub seed: u64,
    pub maker: Pubkey,
    pub mint_a: Pubkey,
    pub mint_b: Pubkey,
    pub receive: u64,
    pub created_at: i64,
    pub expires_at: i64,
    pub bump: u8,
}
```

- **seed**: Unique identifier allowing multiple offers from the same maker.
- **maker**: Public key of the user who created the offer.
- **mint_a**: The token the maker is offering (deposited into vault).
- **mint_b**: The token the maker wants in return.
- **receive**: Amount of mint_b tokens expected.
- **created_at**: Unix timestamp when the escrow was created.
- **expires_at**: Unix timestamp after which the escrow can be auto-refunded (`created_at + TIME`).
- **bump**: Bump seed used to derive the escrow PDA.

---

## Instructions

### 1. Make

Creates an escrow offer by locking the maker's tokens in a vault. Sets `expires_at = created_at + TIME`.

```rust
#[derive(Accounts)]
#[instruction(seed: u64)]
pub struct Make<'info> {
    #[account(mut)]
    pub maker: Signer<'info>,
    pub mint_a: Account<'info, Mint>,
    pub mint_b: Account<'info, Mint>,
    #[account(mut, associated_token::mint = mint_a, associated_token::authority = maker)]
    pub maker_ata_a: Account<'info, TokenAccount>,
    #[account(
        init, payer = maker,
        seeds = [b"escrow", maker.key().as_ref(), seed.to_le_bytes().as_ref()],
        bump, space = 8 + Escrow::INIT_SPACE,
    )]
    pub escrow: Account<'info, Escrow>,
    #[account(init, payer = maker, associated_token::mint = mint_a, associated_token::authority = escrow)]
    pub vault: Account<'info, TokenAccount>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}
```

**Parameters:**
- `seed`: Unique identifier for this escrow
- `deposit`: Amount of mint_a tokens to lock in the vault
- `receive`: Amount of mint_b tokens expected in return

**Process:**
1. Initialize the escrow account with offer details, `created_at`, and `expires_at`
2. Transfer tokens from `maker_ata_a` to the vault

---

### 2. Take

Taker fulfills the trade by sending mint_b and receiving mint_a. Can only be called **before** `expires_at`.

```rust
#[derive(Accounts)]
pub struct Take<'info> {
    #[account(mut)]
    pub taker: Signer<'info>,
    #[account(mut)]
    pub maker: AccountInfo<'info>,
    pub mint_a: Account<'info, Mint>,
    pub mint_b: Account<'info, Mint>,
    #[account(mut, token::mint = mint_a, token::authority = taker)]
    pub taker_ata_a: Box<Account<'info, TokenAccount>>,
    #[account(mut, token::mint = mint_b, token::authority = taker)]
    pub taker_ata_b: Box<Account<'info, TokenAccount>>,
    #[account(mut, token::mint = mint_b, token::authority = maker)]
    pub maker_ata_b: Box<Account<'info, TokenAccount>>,
    #[account(mut, close = maker, has_one = maker, has_one = mint_a, has_one = mint_b,
        seeds = [b"escrow", maker.key().as_ref(), escrow.seed.to_le_bytes().as_ref()],
        bump = escrow.bump,
    )]
    pub escrow: Box<Account<'info, Escrow>>,
    #[account(mut, token::mint = mint_a, token::authority = escrow)]
    pub vault: Box<Account<'info, TokenAccount>>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}
```

**Validation:**
- Rejects if `clock.unix_timestamp >= escrow.expires_at` (escrow already expired)

**Process:**
1. Taker sends `escrow.receive` amount of mint_b to maker
2. Vault sends all mint_a tokens to taker using escrow PDA as signer
3. Close vault - rent to maker
4. Close escrow - rent to maker (via `close = maker`)

---

### 3. Refund

Maker cancels the escrow at any time and reclaims their deposited tokens. Requires maker signature.

```rust
#[derive(Accounts)]
pub struct Refund<'info> {
    #[account(mut)]
    pub maker: Signer<'info>,
    pub mint_a: Account<'info, Mint>,
    #[account(mut, associated_token::mint = mint_a, associated_token::authority = maker)]
    pub maker_ata_a: Account<'info, TokenAccount>,
    #[account(mut, close = maker, has_one = mint_a, has_one = maker,
        seeds = [b"escrow", maker.key().as_ref(), escrow.seed.to_le_bytes().as_ref()],
        bump = escrow.bump,
    )]
    pub escrow: Account<'info, Escrow>,
    #[account(mut, associated_token::mint = mint_a, associated_token::authority = escrow)]
    pub vault: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}
```

**Process:**
1. Transfer all vault tokens back to `maker_ata_a` using escrow PDA as signer
2. Close vault - rent to maker
3. Close escrow - rent to maker

---

### 4. Auto Refund

Permissionless refund callable by **anyone** (including TukTuk crankers) once `expires_at` has passed. No maker signature required.

```rust
#[derive(Accounts)]
pub struct AutoRefund<'info> {
    /// CHECK: receives tokens and rent back — no signer needed
    #[account(mut, address = escrow.maker)]
    pub maker: AccountInfo<'info>,
    pub mint_a: Account<'info, Mint>,
    #[account(mut, associated_token::mint = mint_a, associated_token::authority = maker)]
    pub maker_ata_a: Account<'info, TokenAccount>,
    #[account(mut, close = maker, has_one = mint_a, has_one = maker,
        seeds = [b"escrow", maker.key().as_ref(), escrow.seed.to_le_bytes().as_ref()],
        bump = escrow.bump,
    )]
    pub escrow: Account<'info, Escrow>,
    #[account(mut, associated_token::mint = mint_a, associated_token::authority = escrow)]
    pub vault: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}
```

**Validation:**
- Rejects with `EscrowNotExpired` if `clock.unix_timestamp < escrow.expires_at`

**Process:**
1. Transfer all vault tokens back to `maker_ata_a` using escrow PDA as signer
2. Close vault - rent to maker
3. Close escrow - rent to maker

---

### 5. Schedule

Called by the maker after `make` to register a TukTuk task that will fire `auto_refund` at exactly `expires_at`. Uses a CPI to TukTuk's `queue_task_v0` with `TriggerV0::Timestamp(expires_at)`.

The `tuktuk-program` crate is referenced via a local path (`../../../../tuktuk/tuktuk-program`) and used to call `tuktuk_program::compile_transaction` which serializes the `auto_refund` instruction into TukTuk's `CompiledTransactionV0` format.

> **Note:** TukTuk's published crate targets Anchor 0.31.1 and is incompatible with Anchor 0.32.1. To work around this, `tuktuk-program` is pulled from a fork with the Anchor dependency manually bumped to 0.32.1: [`AvhiMaz/tuktuk @ chore/bump-versions`](https://github.com/AvhiMaz/tuktuk/tree/chore/bump-versions).

```rust
#[derive(Accounts)]
pub struct Schedule<'info> {
    #[account(mut)]
    pub maker: Signer<'info>,
    pub mint_a: Account<'info, Mint>,
    #[account(associated_token::mint = mint_a, associated_token::authority = maker)]
    pub maker_ata_a: Account<'info, TokenAccount>,
    #[account(has_one = mint_a, has_one = maker,
        seeds = [b"escrow", maker.key().as_ref(), escrow.seed.to_le_bytes().as_ref()],
        bump = escrow.bump,
    )]
    pub escrow: Account<'info, Escrow>,
    #[account(associated_token::mint = mint_a, associated_token::authority = escrow)]
    pub vault: Account<'info, TokenAccount>,
    #[account(mut)]
    pub task_queue: UncheckedAccount<'info>,
    #[account(mut)]
    pub task_queue_authority: UncheckedAccount<'info>,
    #[account(mut)]
    pub task: UncheckedAccount<'info>,
    #[account(mut, seeds = [b"queue_authority"], bump)]
    pub queue_authority: AccountInfo<'info>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub tuktuk_program: UncheckedAccount<'info>,
}
```

**Parameters:**
- `task_id`: Available slot in the TukTuk task queue bitmap (0–capacity)

**Process:**
1. Build `CompiledTransactionV0` encoding the `auto_refund` instruction with all required accounts
2. Serialize `QueueTaskArgsV0` with `TriggerV0::Timestamp(escrow.expires_at)`
3. CPI into TukTuk's `queue_task_v0`, signing with the `queue_authority` PDA

---

## TukTuk Integration

### How it works

TukTuk is a permissionless crank network on Solana. Anyone can run a cranker that monitors task queues and executes tasks when their trigger condition is met. Programs register tasks on-chain and fund them with SOL to reward crankers.

```
maker calls schedule()
  └─> CPI to TukTuk queue_task_v0
        └─> Task stored on-chain with TriggerV0::Timestamp(expires_at)
              └─> TukTuk cranker polls the queue
                    └─> When clock >= expires_at, cranker calls auto_refund
                          └─> Tokens returned to maker automatically
```

### Key accounts

| Account | Description |
|---|---|
| `task_queue` | TukTuk task queue (`UwdRmurFA11isBpDNY9HNcoL95Pnt4zNYE2cd1SQwn2`) |
| `queue_authority` | Program PDA (`seeds = [b"queue_authority"]`) registered as a TukTuk queue authority |
| `task_queue_authority` | TukTuk PDA linking the task queue to the queue authority |
| `task` | TukTuk PDA storing the queued task (`seeds = [b"task", task_queue, task_id]`) |

### Setup (one-time)

```bash
# Create a task queue
tuktuk -u https://api.devnet.solana.com -w ~/.config/solana/id.json task-queue create \
  --name "escrow-tuktuk" \
  --capacity 10 \
  --min-crank-reward 5000000 \
  --stale-task-age 604800 \
  --funding-amount 1000000000

# Register your wallet as a queue authority
tuktuk -u https://api.devnet.solana.com -w ~/.config/solana/id.json task-queue add-queue-authority \
  --task-queue-id <ID> \
  --queue-authority <WALLET_PUBKEY>

# Register the program PDA as a queue authority
tuktuk -u https://api.devnet.solana.com -w ~/.config/solana/id.json task-queue add-queue-authority \
  --task-queue-id <ID> \
  --queue-authority <QUEUE_AUTHORITY_PDA>
```

---

## Devnet Deployment

```
Program ID : 92t1k1s6XLTzrFzKvHFRHVX8At6DuzP9BSzkXT33pHjA
Task queue : UwdRmurFA11isBpDNY9HNcoL95Pnt4zNYE2cd1SQwn2
```

---

## Build & Test

```bash
# Build the program
anchor build

# Deploy to devnet
anchor deploy --provider.cluster devnet

# Run the end-to-end devnet test
yarn test:devnet
```

The `test:devnet` script (`scripts/setup-and-run.ts`) does everything in one shot:
1. Creates `mint_a` and `mint_b`
2. Mints tokens to the maker's ATA
3. Calls `make` - escrow opens, tokens locked in vault
4. Calls `schedule` - TukTuk one-shot task queued with `TriggerV0::Timestamp(expires_at)`
5. Polls every 5 seconds and confirms when TukTuk fires `auto_refund`

### Cron alternative

Instead of a one-shot timestamp task, `cron/cron.ts` sets up a **recurring cron job** (every minute) that calls `auto_refund`. The instruction is idempotent — it fails with `EscrowNotExpired` until the escrow actually expires, then succeeds once.

```bash
yarn cron \
  --cronName my-escrow-cron \
  --queueName escrow-tuktuk \
  --walletPath ~/.config/solana/id.json \
  --rpcUrl https://api.devnet.solana.com \
  --maker <MAKER_PUBKEY> \
  --mintA <MINT_A_PUBKEY> \
  --escrowSeed <SEED> \
  --fundingAmount 10000000
```

To stop the cron job once the escrow is settled, use the TukTuk CLI:

```bash
tuktuk -u https://api.devnet.solana.com -w ~/.config/solana/id.json cron close --name my-escrow-cron
```

---

## Flow

```
MAKE FLOW:
1. make(seed, deposit, receive)   -> Creates Escrow PDA + Vault ATA
                                  -> Transfers deposit to vault
                                  -> Sets expires_at = now + TIME 

SCHEDULE FLOW (after make):
2. schedule(task_id)              -> CPI to TukTuk queue_task_v0
                                  -> Task queued: TriggerV0::Timestamp(expires_at)
                                  -> TukTuk cranker will call auto_refund at expiry

TAKE FLOW (before expires_at):
3. take()                         -> Taker pays maker with mint_b
                                  -> Vault pays taker with mint_a
                                  -> Closes vault and escrow, refunds rent

AUTO REFUND FLOW (after expires_at — triggered by TukTuk):
4. auto_refund()                  -> Permissionless - no maker signature needed
                                  -> Vault returns tokens to maker
                                  -> Closes vault and escrow, refunds rent

MANUAL REFUND FLOW (anytime):
5. refund()                       -> Maker signs and cancels escrow
                                  -> Same outcome as auto_refund
```
