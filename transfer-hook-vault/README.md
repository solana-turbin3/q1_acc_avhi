# Transfer Hook Vault

This program implements a whitelisted token vault on Solana using the Token-2022 Transfer Hook extension.

Only admin-approved users can hold and transfer the vault's token. Every token transfer is validated on-chain by the transfer hook, which checks that the sender is whitelisted. Users deposit by pairing a ledger update with a `transfer_checked` in the same transaction; withdrawals follow the same pattern using a delegate approval.

This implementation uses **LiteSVM** for fast, in-process testing without requiring a local validator.

---

## Architecture

The program has 2 state accounts:

### Vault

A PDA that stores the admin and mint for the vault, and controls the vault's token account.

```rust
#[account]
pub struct Vault {
    pub admin: Pubkey,
    pub mint: Pubkey,
    pub bump: u8,
}
```

- **admin**: The public key of the vault creator and authority.
- **mint**: The Token-2022 mint associated with this vault.
- **bump**: The bump seed used to derive the vault PDA.

### UserAccount

A PDA per whitelisted user that tracks their deposited balance.

```rust
#[account]
pub struct UserAccount {
    pub account: Pubkey,
    pub amount: u64,
    pub bump: u8,
}
```

- **account**: The whitelisted user's public key.
- **amount**: The user's current balance recorded in the vault ledger.
- **bump**: The bump seed used to derive the user account PDA.

---

## Instructions

### 1. Initialize

Creates the vault config PDA and initializes a new Token-2022 mint with TransferHook, MetadataPointer, and TokenMetadata extensions.

```rust
#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(
        init,
        payer = admin,
        space = Vault::LEN,
        seeds = [VAULT_CONFIG.as_bytes(), admin.key().as_ref()],
        bump,
    )]
    pub vault: Account<'info, Vault>,
    /// CHECK: We will create and initialize this account manually
    #[account(mut, signer)]
    pub mint: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
    pub token_program: Interface<'info, TokenInterface>,
}
```

**Parameters:** `decimal`, `name`, `symbol`, `uri`

**Process:**
1. **Initialize vault PDA** with admin and mint public keys
2. **Create mint account** with space for TransferHook + MetadataPointer extensions, overfunded with lamports for the eventual metadata TLV
3. **Initialize TransferHook extension** pointing to this program
4. **Initialize MetadataPointer extension** pointing to the mint itself
5. **Initialize mint** via `initialize_mint2`
6. **Initialize TokenMetadata** — Token-2022 reallocs the account to fit the metadata TLV

---

### 2. AddUser / RemoveUser

Admin-only instructions to manage the whitelist. Each whitelisted user gets a `UserAccount` PDA.

```rust
#[derive(Accounts)]
#[instruction(address: Pubkey)]
pub struct AddUser<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(has_one = admin @ ErrorCode::Unauthorized)]
    pub vault: Account<'info, Vault>,
    #[account(
        init,
        payer = admin,
        seeds = [WHITELIST_ENTRY.as_bytes(), address.as_ref()],
        bump,
        space = UserAccount::LEN,
    )]
    pub user_account: Account<'info, UserAccount>,
    pub system_program: Program<'info, System>,
}
```

**Parameters:** `address` — the user's public key to whitelist

The `has_one = admin` constraint ensures only the vault admin can add or remove users.

---

### 3. InitExtraAccMeta

Initializes the `ExtraAccountMetaList` PDA required by the Transfer Hook interface. This account tells Token-2022 which additional accounts to forward to the hook program during every transfer.

```rust
#[derive(Accounts)]
pub struct InitExtraAccountMeta<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    /// CHECK: We create and initialize this account manually
    #[account(
        mut,
        seeds = [EXTRA_ACCOUNT_METAS.as_bytes(), mint.key().as_ref()],
        bump
    )]
    pub extra_acc_meta_list: AccountInfo<'info>,
    pub mint: InterfaceAccount<'info, Mint>,
    pub system_program: Program<'info, System>,
    pub token_program: Interface<'info, TokenInterface>,
}
```

**Extra accounts registered:**

| Index | Account | Derivation |
|-------|---------|------------|
| 5 | `whitelist` PDA | `["whitelist", owner_key]` where `owner` is account at index 3 (transfer authority) |

The `Seed::AccountKey { index: 3 }` resolves to the transfer owner/authority, so the whitelist check is always performed against the actual sender.

---

### 4. TransferHook

Called automatically by Token-2022 on every `transfer_checked`. Validates that the transfer authority is whitelisted.

```rust
#[derive(Accounts)]
pub struct TransferHook<'info> {
    pub source_token: InterfaceAccount<'info, TokenAccount>,
    pub mint: InterfaceAccount<'info, Mint>,
    pub destination_token: InterfaceAccount<'info, TokenAccount>,
    /// CHECK: Owner of source token, passed by Token-2022
    pub owner: AccountInfo<'info>,
    /// CHECK: The extra account meta list PDA
    #[account(seeds = [EXTRA_ACCOUNT_METAS.as_bytes(), mint.key().as_ref()], bump)]
    pub extra_account_meta_list: AccountInfo<'info>,
    #[account(seeds = [WHITELIST_ENTRY.as_bytes(), owner.key().as_ref()], bump = whitelist.bump)]
    pub whitelist: Account<'info, UserAccount>,
}
```

**Validation:**
1. **Check `transferring` flag** — ensures this is a real Token-2022 transfer (not a direct program call)
2. **Verify whitelist PDA** — `whitelist.account == owner.key()` confirms the owner is whitelisted

---

### 5. Deposit

Updates the user's ledger balance. Must be paired with a `transfer_checked` in the same transaction to actually move tokens into the vault.

> **Why not CPI?** Calling `transfer_checked` inside the deposit instruction would cause Token-2022 to CPI back into our `transfer_hook`, triggering Solana's `ReentrancyNotAllowed` error.

```rust
#[derive(Accounts)]
pub struct Deposit<'info> {
    pub user: Signer<'info>,
    #[account(seeds = [VAULT_CONFIG.as_bytes(), vault.admin.as_ref()], bump = vault.bump)]
    pub vault: Account<'info, Vault>,
    #[account(mut, seeds = [WHITELIST_ENTRY.as_bytes(), user.key().as_ref()], bump = user_account.bump)]
    pub user_account: Account<'info, UserAccount>,
}
```

**Client transaction:**
```
[deposit ix, transfer_checked ix (user → vault)]
```

---

### 6. Withdraw

Approves the user as a delegate on the vault's token account, then updates the ledger. The client completes the withdrawal with `transfer_checked` using the user as delegate authority — so the transfer hook checks the user's whitelist, not the vault PDA's.

```rust
#[derive(Accounts)]
pub struct Withdraw<'info> {
    pub user: Signer<'info>,
    #[account(seeds = [VAULT_CONFIG.as_bytes(), vault.admin.as_ref()], bump = vault.bump)]
    pub vault: Account<'info, Vault>,
    #[account(mut, seeds = [WHITELIST_ENTRY.as_bytes(), user.key().as_ref()], bump = user_account.bump)]
    pub user_account: Account<'info, UserAccount>,
    #[account(mut, token::mint = vault.mint, token::authority = vault, token::token_program = token_program)]
    pub vault_token_account: InterfaceAccount<'info, TokenAccount>,
    pub token_program: Interface<'info, TokenInterface>,
}
```

**Client transaction:**
```
[withdraw ix (approve delegate), transfer_checked ix (vault_ata → user_ata, authority = user)]
```

---

## LiteSVM Testing

This project uses **LiteSVM** for testing, which provides a lightweight, in-process Solana VM. Token-2022 and the Associated Token Program are included as built-ins in LiteSVM 0.9.1 — no `.so` fixture files needed.

### Setup

```rust
fn setup() -> (LiteSVM, Keypair) {
    let mut svm = LiteSVM::new();
    let payer = Keypair::new();

    svm.airdrop(&payer.pubkey(), 10 * LAMPORTS_PER_SOL)
        .expect("Failed to airdrop SOL to payer");

    let program_so = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/deploy/transfer_hook_vault.so");
    let program_data = std::fs::read(&program_so)
        .expect("Failed to read program SO file. Run `anchor build` first.");
    let _ = svm.add_program(pubkey_to_addr(&PROGRAM_ID), &program_data);

    (svm, payer)
}
```

### Test: Initialize

Verifies the vault PDA and mint are created correctly with the TransferHook, MetadataPointer, and TokenMetadata extensions.

### Test: AddUser / RemoveUser

Verifies that whitelisted user accounts are created and closed, and that only the admin can perform these actions.

### Test: InitExtraAccMeta

Verifies the `ExtraAccountMetaList` PDA is created and the whitelist seed resolution is registered correctly.

### Test: Deposit

Pairs the `deposit` instruction with `transfer_checked` in one transaction and verifies the user's ledger balance increases.

```rust
#[test]
fn test_deposit() {
    // 1. Initialize vault, whitelist user, init extra acc meta, create ATAs
    // 2. Mint tokens to user ATA
    // 3. Send [deposit ix, transfer_checked ix] in one tx
    // 4. Verify user_account.amount increased
    // 5. Verify vault ATA received the tokens
}
```

### Test: Withdraw

Pairs the `withdraw` instruction (delegate approval) with `transfer_checked` and verifies tokens return to the user.

```rust
#[test]
fn test_withdraw() {
    // 1. Setup and deposit tokens
    // 2. Send [withdraw ix (approve delegate), transfer_checked ix (user as delegate)] in one tx
    // 3. Verify user_account.amount decreased
    // 4. Verify user ATA received the tokens back
}
```

### Test: Withdraw Insufficient Funds

Verifies the `InsufficientFunds` error is returned when a user tries to withdraw more than their balance.

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
INITIALIZE:
1. initialize(decimal, name, symbol, uri)  -> Creates Vault PDA
                                           -> Creates Token-2022 mint with
                                              TransferHook + MetadataPointer + TokenMetadata

WHITELIST MANAGEMENT:
2. add_user(address)                       -> Creates UserAccount PDA for address
3. remove_user(address)                    -> Closes UserAccount PDA

SETUP TRANSFER HOOK:
4. init_extra_acc_meta()                   -> Creates ExtraAccountMetaList PDA
                                           -> Registers whitelist PDA resolution
                                              (resolves from transfer owner, index 3)

DEPOSIT FLOW (atomic tx):
5a. deposit(amount)                        -> Increments user_account.amount on ledger
5b. transfer_checked(user → vault, amount) -> Moves tokens; hook validates user is whitelisted

WITHDRAW FLOW (atomic tx):
6a. withdraw(amount)                       -> Decrements user_account.amount, approves user as delegate
6b. transfer_checked(vault → user, amount) -> Moves tokens; hook validates user (delegate) is whitelisted
```

---
