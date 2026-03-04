# Pinocchio Fundraiser

A token fundraising program built with [pinocchio](https://github.com/anza-xyz/pinocchio) on Solana. A maker creates a fundraiser specifying a target token and amount, contributors deposit tokens into a shared vault, and the maker can claim the full vault once the goal is reached. Contributors can refund their share at any time.

This implementation uses a custom BPF entrypoint with a `peek_discriminator` function that traverses the raw account buffer to read the instruction discriminator before account parsing begins. All CPIs use `invoke_signed_unchecked` to bypass borrow validation overhead. Tests run with **LiteSVM** and a pinocchio-optimized token program fixture.

---

## Compute Units

| Instruction | CUs |
|---|---|
| initialize | 1,371 |
| create_contributor | 1,359 |
| contribute | 1,323 |
| checker | 1,311 |
| refund | 1,337 |

---

## Architecture

The program has 2 state accounts:

### Fundraiser

A PDA that stores the details of an active fundraiser and controls the vault holding contributed tokens.

```rust
#[repr(C)]
pub struct Fundraiser {
    pub maker: [u8; 32],
    pub mint_to_raise: [u8; 32],
    pub amount_to_raise: u64,
    pub current_amount: u64,
    pub time_started: i64,
    pub duration: u8,
    pub bump: u8,
    pub _padding: [u8; 6],
}
```

- `maker` - public key of the fundraiser creator
- `mint_to_raise` - the token mint being collected
- `amount_to_raise` - the target amount
- `current_amount` - total tokens deposited so far
- `time_started` - unix timestamp when the fundraiser began
- `duration` - how long (in days) contributors have to participate
- `bump` - PDA bump seed

The fundraiser PDA is derived from `[b"fundraiser", maker, bump]`.

### Contributor

A PDA per contributor per fundraiser, tracking how much that contributor has deposited.

```rust
#[repr(C)]
pub struct Contributor {
    pub contributor: [u8; 32],
    pub amount: u64,
    pub bump: u8,
    pub _padding: [u8; 7],
}
```

- `contributor` - public key of the contributor
- `amount` - total tokens this contributor has deposited
- `bump` - PDA bump seed

The contributor PDA is derived from `[b"contributor", fundraiser, contributor, bump]`.

---

## Instructions

### 0. Initialize

Creates a new fundraiser. The vault (an ATA owned by the fundraiser PDA) is pre-created by the client before calling this instruction.

**Accounts:** `[maker, fundraiser, mint, system_program]`

**Parameters:**
- `amount_to_raise` - target token amount
- `duration` - duration in days
- `bump` - fundraiser PDA bump

**Process:**
1. Create the fundraiser PDA account via raw system CPI
2. Write fundraiser state fields

---

### 1. Create Contributor

Initializes a contributor state account for a given (fundraiser, contributor) pair. Must be called before a contributor can deposit.

**Accounts:** `[contributor, fundraiser, contributor_state, system_program]`

**Parameters:**
- `bump` - contributor PDA bump

**Process:**
1. Create the contributor PDA account via raw system CPI
2. Write contributor state with `amount = 0`

---

### 2. Contribute

Transfers tokens from the contributor's ATA into the vault and updates both the contributor state and fundraiser totals.

**Accounts:** `[contributor, fundraiser, vault, contributor_ata, contributor_state, token_program]`

**Parameters:**
- `amount` - number of tokens to deposit

**Process:**
1. Token transfer from `contributor_ata` to `vault` via raw token CPI
2. Increment `contributor_state.amount`
3. Increment `fundraiser.current_amount`

---

### 3. Checker

Called by the maker to claim all tokens from the vault once the fundraiser concludes. Closes the fundraiser account and returns lamports to the maker.

**Accounts:** `[maker, fundraiser, vault, maker_ata, token_program]`

**Process:**
1. Transfer full vault balance to `maker_ata`, signed by fundraiser PDA
2. Transfer fundraiser lamports to maker and zero out fundraiser lamports

---

### 4. Refund

Returns a contributor's deposited tokens and closes their contributor state account.

**Accounts:** `[contributor, fundraiser, vault, contributor_ata, contributor_state]`

**Process:**
1. Transfer `contributor_state.amount` from vault to `contributor_ata`, signed by fundraiser PDA
2. Transfer contributor_state lamports to contributor and zero out contributor_state lamports

---

## Raw CPI

All cross-program invocations use `invoke_signed_unchecked` to skip per-account borrow validation.

`raw_create_account_signed` constructs a system program `CreateAccount` instruction directly:

```
[discriminator:4=0][lamports:8][space:8][owner:32] = 52 bytes
```

`raw_transfer_signed` constructs a token program `Transfer` instruction directly:

```
[discriminator:1=3][amount:8] = 9 bytes
```

---

## LiteSVM Testing

Tests use **LiteSVM** for in-process execution without a local validator. A pinocchio-optimized token program fixture (`src/tests/fixtures/pinocchio_token_program.so`) is loaded in place of the standard SPL Token program to keep CU counts minimal.

```rust
fn load_svm() -> LiteSVM {
    let mut svm = LiteSVM::new();
    // load optimized token program
    let p_token_data = std::fs::read("src/tests/fixtures/pinocchio_token_program.so").unwrap();
    svm.add_program(TOKEN_PROGRAM_ID, &p_token_data).unwrap();
    svm
}
```

The vault is pre-created as an ATA before the initialize instruction is sent, removing 2 CPIs from the program path.

### Test: Initialize

```rust
#[test]
fn test_initialize() {
    // 1. Create mint
    // 2. Derive fundraiser PDA
    // 3. Pre-create vault as ATA owned by fundraiser PDA
    // 4. Call initialize
    // 5. Verify fundraiser state fields
}
```

### Test: Create Contributor

```rust
#[test]
fn test_create_contributor() {
    // 1. Setup fundraiser
    // 2. Derive contributor PDA
    // 3. Call create_contributor
    // 4. Verify contributor_state fields
}
```

### Test: Contribute

```rust
#[test]
fn test_contribute() {
    // 1. Setup fundraiser and contributor
    // 2. Mint tokens to contributor ATA
    // 3. Call contribute
    // 4. Verify vault balance increased
    // 5. Verify contributor_state.amount updated
    // 6. Verify fundraiser.current_amount updated
}
```

### Test: Checker

```rust
#[test]
fn test_checker() {
    // 1. Setup fundraiser, contribute tokens
    // 2. Call checker as maker
    // 3. Verify maker_ata received all vault tokens
    // 4. Verify fundraiser account lamports zeroed
}
```

### Test: Refund

```rust
#[test]
fn test_refund() {
    // 1. Setup fundraiser, contribute tokens
    // 2. Call refund as contributor
    // 3. Verify contributor_ata received tokens back
    // 4. Verify contributor_state lamports returned
}
```

---

## Flow

```
INITIALIZE:
0. initialize(amount_to_raise, duration, bump)  -> Creates Fundraiser PDA
                                                -> Vault ATA pre-created by client

CREATE CONTRIBUTOR:
1. create_contributor(bump)                     -> Creates Contributor PDA
                                                -> Sets amount = 0

CONTRIBUTE FLOW:
2. contribute(amount)                           -> Transfers tokens contributor_ata -> vault
                                                -> Updates contributor and fundraiser totals

CHECKER FLOW (maker claims):
3. checker()                                    -> Transfers all vault tokens to maker_ata
                                                -> Closes fundraiser, returns rent to maker

REFUND FLOW (contributor exits):
4. refund()                                     -> Transfers contributor amount vault -> contributor_ata
                                                -> Closes contributor_state, returns rent
```

---

## Build & Test

```bash
cargo build-sbf

cargo test
```
