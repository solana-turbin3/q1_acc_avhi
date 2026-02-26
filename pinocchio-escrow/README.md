# Pinocchio Escrow

This program implements a trustless escrow mechanism for secure token swaps between two parties on Solana.

Alice (maker) can offer to exchange Token A for Token B, and Bob (taker) can accept the offer to complete the swap atomically. The escrow ensures neither party can cheat -- tokens are locked in a program-controlled vault until both sides of the trade are fulfilled or the maker cancels.

This implementation uses **Pinocchio** for zero-dependency, no-allocator on-chain code and **LiteSVM** for fast, in-process testing without requiring a local validator. Two deserialization strategies are implemented side by side for comparison: manual byte parsing (`unsafe`) and schema-driven deserialization (`wincode`).

---

## Architecture

The program has 1 state account:

### Escrow

A PDA that stores the details of an active offer and controls the vault holding the maker's tokens.

```rust
#[repr(C)]
#[derive(Clone, Copy, SchemaRead)]
pub struct Escrow {
    pub maker: [u8; 32],
    pub mint_a: [u8; 32],
    pub mint_b: [u8; 32],
    pub amount_to_receive: u64,
    pub amount_to_give: u64,
    pub bump: u8,
    pub _padding: [u8; 7],
}
```

- **maker**: The public key of the user who created the offer.
- **mint_a**: The token mint that the maker is offering (deposited in vault).
- **mint_b**: The token mint that the maker wants to receive.
- **amount_to_receive**: The amount of mint_b tokens the maker expects in exchange.
- **amount_to_give**: The amount of mint_a tokens locked in the vault.
- **bump**: The bump seed used to derive the escrow PDA.
- **_padding**: Alignment padding to keep the struct `#[repr(C)]` compatible.

The escrow PDA is derived from `["escrow", maker]`.

---

## Instruction Variants

Each instruction is implemented twice under two strategies with different discriminator bytes:

| Instruction | unsafe disc | wincode disc |
|-------------|-------------|--------------|
| Make        | `0`         | `3`          |
| Take        | `1`         | `4`          |
| Cancel      | `2`         | `5`          |

### unsafe

Manual deserialization using `u64::from_le_bytes` with explicit byte indexing. State is read via `borrow_unchecked` with pointer casting.

```rust
let amount_to_receive = u64::from_le_bytes([
    data[0], data[1], data[2], data[3], data[4], data[5], data[6], data[7],
]);
let amount_to_give = u64::from_le_bytes([
    data[8], data[9], data[10], data[11], data[12], data[13], data[14], data[15],
]);
let bump = data[16];
```

### wincode

Schema-driven deserialization using `#[derive(SchemaRead)]` from the `wincode` crate. Structs declare their layout and `wincode::deserialize` handles parsing.

```rust
#[derive(SchemaRead)]
pub struct MakeInstructionData {
    pub amount_to_receive: u64,
    pub amount_to_give: u64,
    pub bump: u8,
}

let ix_data = ::wincode::deserialize::<MakeInstructionData>(data)
    .map_err(|_| ProgramError::InvalidInstructionData)?;
```

---

## Instructions

### 1. Make

Creates an escrow offer by locking the maker's tokens in a vault.

**Accounts:**

| Index | Account         | Writable | Signer | Description                              |
|-------|-----------------|----------|--------|------------------------------------------|
| 0     | maker           | yes      | yes    | The maker creating the offer             |
| 1     | mint_a          | yes      | no     | Mint of the token being offered          |
| 2     | mint_b          | yes      | no     | Mint of the token wanted in return       |
| 3     | escrow_account  | yes      | no     | Escrow PDA (created by this instruction) |
| 4     | maker_ata       | yes      | no     | Maker's associated token account (A)     |
| 5     | escrow_ata      | yes      | no     | Vault ATA owned by the escrow PDA        |
| 6     | system_program  | no       | no     | System program                           |
| 7     | token_program   | no       | no     | SPL Token program                        |
| 8     | associated_token_program | no | no  | Associated Token program                 |

**Instruction data** (after discriminator byte, 17 bytes total):

| Bytes  | Field              | Type |
|--------|--------------------|------|
| 0..8   | amount_to_receive  | u64 LE |
| 8..16  | amount_to_give     | u64 LE |
| 16     | bump               | u8   |

**Validation:**
- `maker` must be a signer
- `maker_ata` owner must be `maker` and mint must be `mint_a`
- `escrow_account` address must match PDA derived from `["escrow", maker, bump]`

**Process:**
1. Derive and verify the escrow PDA address
2. Create the escrow account via CPI to system program (funded by maker)
3. Write escrow state fields into the new account
4. Create the vault ATA owned by the escrow PDA via CPI to associated token program
5. Transfer `amount_to_give` tokens from maker to vault

---

### 2. Take

Accepts an escrow offer by exchanging the requested tokens and completing the swap.

**Accounts:**

| Index | Account        | Writable | Signer | Description                              |
|-------|----------------|----------|--------|------------------------------------------|
| 0     | taker          | yes      | yes    | The taker accepting the offer            |
| 1     | maker          | yes      | no     | Original maker of the escrow             |
| 2     | escrow_account | yes      | no     | Escrow PDA                               |
| 3     | taker_ata_a    | yes      | no     | Taker's ATA for mint_a (receives tokens) |
| 4     | taker_ata_b    | yes      | no     | Taker's ATA for mint_b (sends tokens)    |
| 5     | maker_ata_b    | yes      | no     | Maker's ATA for mint_b (receives tokens) |
| 6     | escrow_ata     | yes      | no     | Vault ATA owned by escrow PDA            |
| 7     | token_program  | no       | no     | SPL Token program                        |

**Validation:**
- `taker` must be a signer
- `escrow_account.maker` must match `maker`
- `maker_ata_b` owner must be `maker` and mint must be `escrow.mint_b`
- `escrow_account` address must match the expected PDA

**Process:**
1. Read escrow state and verify maker
2. Verify `maker_ata_b` ownership and mint
3. Verify escrow PDA derivation
4. Transfer `amount_to_receive` from taker to maker (mint_b)
5. Transfer `amount_to_give` from vault to taker (mint_a), signed by escrow PDA
6. Close vault ATA, returning rent to maker
7. Close escrow account, returning lamports to maker

---

### 3. Cancel

Cancels the escrow and returns the locked tokens to the maker.

**Accounts:**

| Index | Account        | Writable | Signer | Description                          |
|-------|----------------|----------|--------|--------------------------------------|
| 0     | maker          | yes      | yes    | The original maker canceling the offer |
| 1     | escrow_account | yes      | no     | Escrow PDA                           |
| 2     | maker_ata_a    | yes      | no     | Maker's ATA for mint_a (refund dest) |
| 3     | escrow_ata     | yes      | no     | Vault ATA owned by escrow PDA        |
| 4     | token_program  | no       | no     | SPL Token program                    |

**Validation:**
- `maker` must be a signer
- `escrow_account.maker` must match `maker`
- `escrow_account` address must match the expected PDA

**Process:**
1. Read escrow state, verify maker, extract bump and amount
2. Verify escrow PDA derivation
3. Transfer all tokens from vault back to `maker_ata_a`, signed by escrow PDA
4. Close vault ATA, returning rent to maker
5. Close escrow account, returning lamports to maker

---

## LiteSVM Testing

This project uses **LiteSVM** for testing, which provides a lightweight, in-process Solana VM without needing a local validator.

### Setup

```rust
fn load_svm() -> (LiteSVM, Keypair) {
    let mut svm = LiteSVM::new();
    let payer = Keypair::new();

    svm.airdrop(&payer.pubkey(), 10 * LAMPORTS_PER_SOL).unwrap();

    let so_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target/sbpf-solana-solana/release/escrow.so");

    let program_data = std::fs::read(so_path).expect("Failed to read program SO file");
    svm.add_program(program_id(), &program_data).unwrap();

    (svm, payer)
}
```

### Key LiteSVM Features Used

1. **Mint Creation**: `CreateMint::new(&mut svm, &payer).decimals(6).authority(&maker).send()`
2. **ATA Creation**: `CreateAssociatedTokenAccount::new(&mut svm, &payer, &mint).owner(&owner).send()`
3. **Minting Tokens**: `MintTo::new(&mut svm, &payer, &mint, &ata, amount).send()`
4. **Transaction Sending**: `svm.send_transaction(transaction)`
5. **CU Measurement**: `tx.compute_units_consumed` from the returned transaction metadata

### Test: Make

Tests creating an escrow offer:

```rust
#[test]
fn test_make() {
    // 1. Create mints and maker ATA
    // 2. Mint amount_to_give tokens to maker_ata_a
    // 3. Derive escrow PDA and vault ATA addresses
    // 4. Build and send make instruction
    // 5. Print compute units consumed
}
```

### Test: Take

Tests accepting an escrow offer:

```rust
#[test]
fn test_take() {
    // 1. Setup: run make via setup_make_v2
    // 2. Airdrop SOL to taker
    // 3. Create taker ATAs for mint_a and mint_b
    // 4. Create maker ATA for mint_b
    // 5. Mint amount_to_receive tokens of mint_b to taker
    // 6. Build and send take instruction
    // 7. Print compute units consumed
}
```

### Test: Cancel

Tests canceling an escrow:

```rust
#[test]
fn test_cancel() {
    // 1. Setup: run make via setup_make_v2
    // 2. Build and send cancel instruction
    // 3. Print compute units consumed
}
```

### CU Comparison Table

`cu_table_test` runs all 6 instructions and prints a side-by-side CU breakdown:

```
+-------------+----------+----------+-------+
| instruction |   unsafe |  wincode |  diff |
+-------------+----------+----------+-------+
| make        |    30443 |    31940 | +1497 |
| take        |    16652 |    16666 |   +14 |
| cancel      |    10577 |    10599 |   +22 |
+-------------+----------+----------+-------+
```

The `unsafe` variant is cheaper on `make` because explicit `u64::from_le_bytes` byte indexing generates native `ldxb` load instructions in SBPF, matching what `wincode` generates internally. The small overhead in `take` and `cancel` comes from `wincode::deserialize` parsing the `Escrow` state struct.

---

## Build & Test

```bash
# Build the on-chain program (required before running tests)
cargo build-sbf

# Run all tests
cargo test

# Run only the CU comparison table
cargo test test_cu_table -- --nocapture

# Run unsafe instruction tests
cargo test unsafe -- --nocapture

# Run wincode instruction tests
cargo test wincode -- --nocapture
```

---

## Flow

```
MAKE FLOW:
1. make(amount_to_receive, amount_to_give, bump)
                              -> Creates Escrow PDA account
                              -> Creates vault ATA owned by escrow PDA
                              -> Transfers amount_to_give tokens from maker to vault
                              -> Stores mint_a, mint_b, amounts, bump in escrow

TAKE FLOW:
2. take()                     -> Verifies escrow maker and PDA derivation
                              -> Verifies maker_ata_b ownership and mint
                              -> Taker sends amount_to_receive of mint_b to maker
                              -> Vault sends amount_to_give of mint_a to taker
                              -> Closes vault and escrow, refunds rent to maker

CANCEL FLOW:
3. cancel()                   -> Verifies escrow maker and PDA derivation
                              -> Vault returns amount_to_give of mint_a to maker
                              -> Closes vault and escrow, refunds rent to maker
```

---
