# GPT Oracle

A Solana program that queries an on-chain **AI oracle** (powered by [MagicBlock](https://magicblock.gg)) and schedules those queries automatically using [TukTuk](https://www.tuktuk.fun) - a permissionless crank scheduler.

You initialize an AI agent with a system prompt, and TukTuk's crankers automatically fire `interact_with_llm` on a schedule. The oracle processes the query off-chain via GPT and calls back into your program with the response.

---

## Architecture

The program has 1 state account:

### Agent

A PDA that stores a reference to the LLM context created by the oracle program.

```rust
#[account]
pub struct Agent {
    pub context: Pubkey,
    pub bump: u8,
}
```

- **context**: Public key of the `ContextAccount` owned by the oracle program - holds the agent's system prompt.
- **bump**: Bump seed used to derive the agent PDA.

---

## Instructions

### 1. Initialize

Creates the `Agent` PDA and registers a system prompt with the oracle via CPI to `create_llm_context`. Must be called once before any interactions.

```rust
#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(
        init, payer = payer,
        space = 8 + 32 + 1,
        seeds = [b"agent", payer.key().as_ref()],
        bump
    )]
    pub agent: Account<'info, Agent>,
    #[account(mut)]
    pub counter: Account<'info, Counter>,
    /// CHECK: Checked in oracle program
    #[account(mut)]
    pub llm_context: AccountInfo<'info>,
    /// CHECK: Checked oracle id
    #[account(address = solana_gpt_oracle::ID)]
    pub oracle_program: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
}
```

**Process:**
1. Store `llm_context.key()` and bump in the `Agent` account
2. CPI into `solana_gpt_oracle::create_llm_context` with `AGENT_DESC` as the system prompt

---

### 2. Interact With LLM

Sends a query to the oracle. The oracle's off-chain service processes the request via GPT and fires the callback instruction with the response.

```rust
#[derive(Accounts)]
pub struct Interact<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    /// CHECK: Checked in oracle program
    #[account(mut)]
    pub interaction: AccountInfo<'info>,
    #[account(seeds = [b"agent", payer.key().as_ref()], bump)]
    pub agent: Account<'info, Agent>,
    #[account(address = agent.context)]
    pub context_account: Account<'info, ContextAccount>,
    /// CHECK: Checked oracle id
    #[account(address = solana_gpt_oracle::ID)]
    pub oracle_program: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
}
```

**Process:**
1. CPI into `solana_gpt_oracle::interact_with_llm` with `AGENT_DESC` as the prompt, your program ID as the callback target, and the `CallbackFromLlm` discriminator

---

### 3. Callback From LLM

Called by the oracle program after GPT responds. Verifies the `identity` account is a signer (proving the call came from the oracle) and logs the response.

```rust
#[derive(Accounts)]
pub struct Callback<'info> {
    pub identity: Account<'info, Identity>,
}
```

**Process:**
1. Verify `identity.to_account_info().is_signer`
2. `msg!("Response: {:?}", response)`

---

### 4. Schedule

Registers a TukTuk task that fires `interact_with_llm` with `TriggerV0::Now`. When TukTuk's crankers pick up the task, the interaction is triggered automatically - no manual transaction needed.

```rust
#[derive(Accounts)]
pub struct Schedule<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    /// CHECK: Checked in oracle program
    #[account(mut)]
    pub interaction: AccountInfo<'info>,
    #[account(seeds = [b"agent", payer.key().as_ref()], bump)]
    pub agent: Account<'info, Agent>,
    #[account(address = agent.context)]
    pub context_account: Account<'info, ContextAccount>,
    /// CHECK: Passed through to TukTuk CPI
    #[account(mut)]
    pub task_queue: UncheckedAccount<'info>,
    /// CHECK: Derived and verified by TukTuk program
    #[account(mut)]
    pub task_queue_authority: UncheckedAccount<'info>,
    /// CHECK: Initialized in CPI
    #[account(mut)]
    pub task: UncheckedAccount<'info>,
    /// CHECK: PDA signer
    #[account(mut, seeds = [b"queue_authority"], bump)]
    pub queue_authority: AccountInfo<'info>,
    pub tuktuk_program: Program<'info, Tuktuk>,
    pub system_program: Program<'info, System>,
}
```

**Parameters:**
- `task_id`: Available slot in the TukTuk task queue bitmap (0–capacity)

**Process:**
1. Build `CompiledTransactionV0` encoding the `interact_with_llm` instruction with all required accounts
2. CPI into TukTuk's `queue_task_v0` with `TriggerV0::Now`, signing with the `queue_authority` PDA

---

## Oracle + TukTuk Flow

```
initialize()
  └─> CPI to oracle create_llm_context (stores system prompt)
        └─> Agent PDA stores context pubkey

schedule(task_id)
  └─> CPI to TukTuk queue_task_v0
        └─> Task stored on-chain with TriggerV0::Now
              └─> TukTuk cranker fires interact_with_llm
                    └─> CPI to oracle interact_with_llm
                          └─> Oracle sends prompt to GPT off-chain
                                └─> Oracle calls callback_from_llm
                                      └─> Response logged on-chain
```

---

## TukTuk Integration

### Key accounts

| Account | Description |
|---|---|
| `task_queue` | TukTuk task queue (`UwdRmurFA11isBpDNY9HNcoL95Pnt4zNYE2cd1SQwn2`) |
| `queue_authority` | Program PDA (`seeds = [b"queue_authority"]`) registered as a TukTuk queue authority |
| `task_queue_authority` | TukTuk PDA linking the task queue to the queue authority |
| `task` | TukTuk PDA storing the queued task (`seeds = [b"task", task_queue, task_id]`) |

> **Note:** TukTuk's published crate targets Anchor 0.31.1 and is incompatible with Anchor 0.32.1. `tuktuk-program` is pulled from a fork with the Anchor dependency bumped to 0.32.1: [`AvhiMaz/tuktuk @ chore/bump-versions`](https://github.com/AvhiMaz/tuktuk/tree/chore/bump-versions).

---

## Devnet Deployment

```
Program ID  : 8d6wKSQNNoqSu98EgLn5ZotmJMZHq8cgcfLGsiubUqZe
Oracle ID   : LLMrieZMpbJFwN52WgmBNMxYojrpRVYXdC1RCweEbab
Task queue  : UwdRmurFA11isBpDNY9HNcoL95Pnt4zNYE2cd1SQwn2
```

---

## Build & Test

```bash
# Build the program
anchor build

# Deploy to devnet
anchor deploy --provider.cluster devnet

# Run tests against devnet
anchor test --skip-local-validator --provider.cluster devnet
```

The test suite does the following in order:
1. `initialize` - creates the agent and registers the system prompt with the oracle
2. `interact_with_llm` - sends a query to the oracle directly
3. `schedule` - registers a TukTuk task that will fire `interact_with_llm` automatically
