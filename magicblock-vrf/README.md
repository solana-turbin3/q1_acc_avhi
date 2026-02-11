# MagicBlock VRF

This program demonstrates Verifiable Random Function (VRF) integration using the MagicBlock ephemeral rollup SDK to update on-chain user state with verifiable randomness.

Two implementations are covered: requesting randomness on the Solana base layer, and requesting randomness inside an ephemeral rollup for faster, cheaper execution.

---

## Architecture

The program has 1 state account:

### UserAccount

A PDA that stores the user's public key and their current data value, which is updated with a random number upon each VRF callback.

```rust
#[account]
pub struct UserAccount {
    pub user: Pubkey,
    pub data: u64,
    pub bump: u8,
}
```

- **user**: The public key of the account owner.
- **data**: The u64 value updated by the VRF callback with verifiable randomness.
- **bump**: The bump seed used to derive this PDA.

PDA seeds: `[b"user", user.key()]`

---

## Instructions

### 1. Initialize

Creates a new `UserAccount` PDA for the caller with `data` set to zero.

```rust
#[derive(Accounts)]
pub struct InitUser<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(
        init,
        payer = user,
        space = UserAccount::INIT_SPACE,
        seeds = [b"user", user.key().as_ref()],
        bump
    )]
    pub user_account: Account<'info, UserAccount>,
    pub system_program: Program<'info, System>,
}
```

---

### 2. Request Randomness

Submits a VRF randomness request via CPI to the VRF program. The seed is derived on-chain from `Clock::get()` (slot + unix_timestamp) and the payer's public key, ensuring uniqueness across multiple calls without requiring any client input.

```rust
#[vrf]
#[derive(Accounts)]
pub struct RequestRandomnessCtx<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(seeds = [b"user", payer.key().as_ref()], bump)]
    pub user_account: Account<'info, UserAccount>,
    /// CHECK: The oracle queue (base layer or ephemeral)
    #[account(mut)]
    pub oracle_queue: AccountInfo<'info>,
}
```

The `#[vrf]` macro injects `program_identity`, `vrf_program`, `slot_hashes`, and `system_program` accounts automatically. The handler calls `invoke_signed_vrf` which signs the CPI using the program identity PDA.

- **Task 1 (base layer)**: pass `DEFAULT_QUEUE` (`Cuj97ggrhhidhbu39TijNVqE74xvKJ69gDervRUXAxGh`)
- **Task 2 (inside ER)**: pass `DEFAULT_EPHEMERAL_QUEUE` (`5hBR571xnXppuCPveTrctfTU7tJLSN94nq7kv7FRK5Tc`)

---

### 3. Consume Randomness

Oracle callback invoked by the VRF program after processing the request. Receives the verified `[u8; 32]` randomness bytes and writes a derived `u64` into `user_account.data`.

```rust
#[derive(Accounts)]
pub struct ConsumeRandomnessCtx<'info> {
    #[account(address = ephemeral_vrf_sdk::consts::VRF_PROGRAM_IDENTITY)]
    pub vrf_program_identity: Signer<'info>,
    #[account(mut)]
    pub user_account: Account<'info, UserAccount>,
}
```

The `vrf_program_identity` constraint ensures only the VRF program can trigger this callback. The randomness bytes are converted to a `u64` using `ephemeral_vrf_sdk::rnd::random_u64`.

---

### 4. Delegate

Delegates the `UserAccount` to the ephemeral rollup validator, enabling high-frequency updates inside the ER.

```rust
#[delegate]
#[derive(Accounts)]
pub struct Delegate<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(mut, del, seeds = [b"user", user.key().as_ref()], bump)]
    pub user_account: Account<'info, UserAccount>,
}
```

---

### 5. Update Commit

Updates `user_account.data` with a manual value and commits the state back to the base layer from inside the ER.

---

### 6. Undelegate

Commits the final state and returns the `UserAccount` from the ephemeral rollup back to the base layer.

---

### 7. Close

Closes the `UserAccount` PDA and returns rent lamports to the user.

---

## Flow

### Task 1: VRF on base layer

```
1. initialize()                        -> Create UserAccount on base layer
2. request_randomness(queue)           -> CPI to VRF program, oracle queues request
3. [oracle callback]                   -> consume_randomness() writes random u64 to data
```

### Task 2: VRF inside ephemeral rollup

```
1. initialize()                        -> Create UserAccount on base layer
2. delegate()                          -> Delegate UserAccount to ephemeral rollup
3. request_randomness(er_queue)        -> CPI to VRF program inside ER
4. [oracle callback]                   -> consume_randomness() writes random u64 to data
5. undelegate()                        -> Commit final state back to base layer
```

---

## Key Concepts

**Two-transaction pattern**: VRF always requires two separate transactions. The first submits the request; the second is the oracle callback. You cannot request and consume randomness in the same transaction.

**On-chain seed derivation**: The request seed is derived inside the program using `Clock::get()` (slot + unix_timestamp) combined with the first 8 bytes of the payer's public key. This produces a unique `[u8; 32]` seed per call without any client input, preventing PDA collisions.

**Queue selection**: The base layer and ephemeral rollup use different oracle queues. Pass the correct queue from the client since the program accepts either.
