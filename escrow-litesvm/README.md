# Escrow with LiteSVM Testing

This program implements a trustless escrow mechanism for secure token swaps between two parties on Solana.

Alice (maker) can offer to exchange Token A for Token B, and Bob (taker) can accept the offer to complete the swap atomically. The escrow ensures neither party can cheat - tokens are locked in program-controlled vaults until both sides of the trade are fulfilled or the maker refunds.

This implementation uses **LiteSVM** for fast, in-process testing without requiring a local validator.

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
    pub bump: u8,
}
```

- **seed**: A unique identifier for this escrow offer, allowing multiple offers from the same maker.
- **maker**: The public key of the user who created the offer.
- **mint_a**: The token mint that the maker is offering (deposited in vault).
- **mint_b**: The token mint that the maker wants to receive.
- **receive**: The amount of mint_b tokens the maker expects to receive.
- **created_at**: Unix timestamp when the escrow was created (for time-based constraints).
- **bump**: The bump seed used to derive the escrow PDA.

---

## Instructions

### 1. Make

Creates an escrow offer by locking the maker's tokens in a vault.

```rust
#[derive(Accounts)]
#[instruction(seed: u64)]
pub struct Make<'info> {
    #[account(mut)]
    pub maker: Signer<'info>,
    pub mint_a: InterfaceAccount<'info, Mint>,
    pub mint_b: InterfaceAccount<'info, Mint>,
    #[account(
        mut,
        associated_token::mint = mint_a,
        associated_token::authority = maker,
    )]
    pub maker_ata_a: InterfaceAccount<'info, TokenAccount>,
    #[account(
        init,
        payer = maker,
        seeds = [b"escrow", maker.key().as_ref(), seed.to_le_bytes().as_ref()],
        bump,
        space = 8 + Escrow::INIT_SPACE,
    )]
    pub escrow: Account<'info, Escrow>,
    #[account(
        init,
        payer = maker,
        associated_token::mint = mint_a,
        associated_token::authority = escrow,
    )]
    pub vault: InterfaceAccount<'info, TokenAccount>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}
```

**Parameters:**
- `seed`: Unique identifier for this escrow
- `deposit`: Amount of mint_a tokens to lock
- `receive`: Amount of mint_b tokens to receive in exchange

The escrow PDA is derived from `[b"escrow", maker.key(), seed.to_le_bytes()]`. The vault is an associated token account owned by the escrow PDA.

**Process:**
1. **Initialize the escrow account** with offer details and current timestamp
2. **Transfer tokens from maker to vault** using `transfer_checked` for safety

---

### 2. Take

Accepts an escrow offer by exchanging the requested tokens and completing the swap.

```rust
#[derive(Accounts)]
pub struct Take<'info> {
    #[account(mut)]
    pub taker: Signer<'info>,
    #[account(mut)]
    pub maker: SystemAccount<'info>,
    pub mint_a: Account<'info, Mint>,
    pub mint_b: Account<'info, Mint>,
    #[account(mut)]
    pub taker_ata_a: Account<'info, TokenAccount>,
    #[account(mut)]
    pub taker_ata_b: Account<'info, TokenAccount>,
    #[account(mut)]
    pub maker_ata_b: Account<'info, TokenAccount>,
    #[account(
        mut,
        close = maker,
        has_one = maker,
        has_one = mint_a,
        has_one = mint_b,
        seeds = [b"escrow", maker.key().as_ref(), escrow.seed.to_le_bytes().as_ref()],
        bump = escrow.bump,
    )]
    pub escrow: Account<'info, Escrow>,
    #[account(mut)]
    pub vault: Account<'info, TokenAccount>,
    pub clock: Sysvar<'info, Clock>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}
```

**Time constraint:** The escrow must be at least 5 days old before it can be taken. This prevents instant trades and adds a security delay.

**Validation steps:**
1. **Verify time constraint** - ensures `current_time >= escrow.created_at + 5 days`
2. **Verify taker_ata_a** - belongs to taker and uses mint_a
3. **Verify taker_ata_b** - belongs to taker and uses mint_b
4. **Verify maker_ata_b** - belongs to maker and uses mint_b
5. **Verify vault** - belongs to escrow PDA and uses mint_a

**Process:**
1. **Taker pays maker** - transfers `escrow.receive` amount of mint_b tokens to maker_ata_b
2. **Vault pays taker** - transfers all vault tokens (mint_a) to taker_ata_a using escrow PDA as signer
3. **Close vault** - returns rent to maker
4. **Close escrow** - returns rent to maker (via `close = maker` constraint)

The escrow PDA signs using seeds: `[b"escrow", maker.key(), escrow.seed.to_le_bytes(), bump]`.

---

### 3. Refund

Cancels the escrow and returns the locked tokens to the maker.

```rust
#[derive(Accounts)]
pub struct Refund<'info> {
    #[account(mut)]
    maker: Signer<'info>,
    mint_a: InterfaceAccount<'info, Mint>,
    #[account(
        mut,
        associated_token::mint = mint_a,
        associated_token::authority = maker,
    )]
    maker_ata_a: InterfaceAccount<'info, TokenAccount>,
    #[account(
        mut,
        close = maker,
        has_one = mint_a,
        has_one = maker,
        seeds = [b"escrow", maker.key().as_ref(), escrow.seed.to_le_bytes().as_ref()],
        bump = escrow.bump,
    )]
    pub escrow: Account<'info, Escrow>,
    #[account(
        mut,
        associated_token::mint = mint_a,
        associated_token::authority = escrow,
    )]
    vault: InterfaceAccount<'info, TokenAccount>,
    token_program: Interface<'info, TokenInterface>,
    system_program: Program<'info, System>,
}
```

**Process:**
1. **Transfer all tokens from vault to maker** using `transfer_checked` with escrow PDA as signer
2. **Close vault** - returns rent to maker
3. **Close escrow** - returns rent to maker (via `close = maker` constraint)

Only the original maker can call this instruction, verified by the `has_one = maker` constraint and the `maker: Signer` requirement.

---

## LiteSVM Testing

This project uses **LiteSVM** for testing, which provides a lightweight, in-process Solana VM without needing a local validator.

### Setup

```rust
fn setup() -> (LiteSVM, Keypair) {
    let mut program = LiteSVM::new();
    let payer = Keypair::new();

    // Airdrop SOL to payer
    program.airdrop(&payer.pubkey(), 10 * LAMPORTS_PER_SOL)
        .expect("Failed to airdrop SOL to payer");

    // Load program binary
    let so_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/deploy/anchor_escrow.so");
    let program_data = std::fs::read(so_path)
        .expect("Failed to read program SO file");

    // Add program to LiteSVM
    let _ = program.add_program(pubkey_to_addr(&PROGRAM_ID), &program_data);

    (program, payer)
}
```

### Key LiteSVM Features Used

1. **Mint Creation**: `CreateMint::new(&mut program, &payer).decimals(6).authority(&maker).send()`
2. **ATA Creation**: `CreateAssociatedTokenAccount::new(&mut program, &payer, &mint).owner(&owner).send()`
3. **Minting Tokens**: `MintTo::new(&mut program, &payer, &mint, &ata, amount).send()`
4. **Transaction Sending**: `program.send_transaction(transaction)`
5. **Account Retrieval**: `program.get_account(&address)`
6. **Sysvar Manipulation**: `program.set_sysvar(&clock)` - allows time travel for testing time-based constraints

### Test: Make

Tests creating an escrow offer:

```rust
#[test]
fn test_make() {
    // 1. Create mints and ATAs
    // 2. Mint tokens to maker_ata_a
    // 3. Derive escrow and vault PDAs
    // 4. Call make instruction
    // 5. Verify vault has correct amount
    // 6. Verify escrow state matches parameters
}
```

### Test: Take

Tests accepting an escrow offer:

```rust
#[test]
fn test_take() {
    // 1. Setup: create mints, ATAs for maker and taker
    // 2. Mint tokens to both parties
    // 3. Call make instruction
    // 4. Fast-forward time by 5 days using set_sysvar
    // 5. Call take instruction
    // 6. Verify token balances changed correctly
    // 7. Verify vault and escrow are closed
}
```

**Time manipulation:**
```rust
let mut clock: Clock = program.get_sysvar();
clock.unix_timestamp += 5 * 24 * 60 * 60; // Add 5 days
program.set_sysvar(&clock);
```

This bypasses the 5-day waiting period, allowing instant testing of the take instruction.

### Test: Take Too Early

Tests that the 5-day time constraint is enforced:

```rust
#[test]
fn test_take_too_early() {
    // 1. Setup and create escrow (same as test_take)
    // 2. Attempt take WITHOUT fast-forwarding time
    // 3. Assert transaction fails with TooEarlyToTake error
}
```

### Test: Refund

Tests canceling an escrow:

```rust
#[test]
fn test_refund() {
    // 1. Setup and create escrow
    // 2. Call refund instruction
    // 3. Verify maker received all tokens back
    // 4. Verify vault and escrow are closed
}
```

---

## Build & Test

```bash
# Build the program
make build

# Build and run all tests
make test

# Format code (requires nightly)
make format

# Run clippy lints
make check

# Clean, build, and test
make all
```

---

## Flow

```
MAKE FLOW:
1. make(seed, deposit, receive)   -> Creates Escrow PDA, creates Vault ATA
                                  -> Transfers deposit amount to vault
                                  -> Stores mint_a, mint_b, receive amount

TAKE FLOW (after 5+ days):
2. take()                         -> Validates time constraint (5 days)
                                  -> Taker pays maker with mint_b tokens
                                  -> Vault pays taker with mint_a tokens
                                  -> Closes vault and escrow, refunds rent

REFUND FLOW (anytime):
3. refund()                       -> Vault returns tokens to maker
                                  -> Closes vault and escrow, refunds rent
```

---