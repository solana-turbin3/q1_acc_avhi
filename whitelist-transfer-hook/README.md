# Whitelist Transfer Hook

This program implements a transfer hook using the SPL Token 2022 Transfer Hook interface to enforce whitelist restrictions on token transfers.

Only whitelisted addresses can transfer tokens that have this transfer hook enabled, providing fine-grained access control over token movements.

---

## Architecture

The program has 2 state accounts:

### Config

A singleton PDA that stores the admin who controls whitelist operations.

```rust
#[account]
#[derive(InitSpace)]
pub struct Config {
    pub admin: Pubkey,
    pub bump: u8,
}
```

- **admin**: The public key authorized to add/remove whitelist entries.
- **bump**: The bump seed used to derive the config PDA.

### Whitelist

A separate PDA account is created for each whitelisted address. This gives O(1) lookups during transfer validation and avoids vector resizing costs.

```rust
#[account]
pub struct Whitelist {
    pub address: Pubkey,
    pub bump: u8,
}
```

- **address**: The whitelisted public key.
- **bump**: The bump seed used to derive this whitelist entry PDA.

---

## Instructions

### 1. Initialize Config

Creates the Config PDA and sets the caller as the admin.

```rust
#[derive(Accounts)]
pub struct InitializeConfig<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(
        init,
        payer = admin,
        space = Config::LEN,
        seeds = [b"config"],
        bump,
    )]
    pub config: Account<'info, Config>,
    pub system_program: Program<'info, System>,
}
```

The admin who calls this instruction becomes the sole authority for managing the whitelist. The config PDA is derived from the seed `[b"config"]`.

---

### 2. Add to Whitelist

Creates a new PDA account for the whitelisted address. Only the admin (verified via the Config account) can call this.

```rust
#[derive(Accounts)]
#[instruction(address: Pubkey)]
pub struct AddToWhiteList<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(
        seeds = [b"config"],
        bump = config.bump,
        constraint = config.admin == admin.key() @ ErrorCode::Unauthorized,
    )]
    pub config: Account<'info, Config>,
    #[account(
        init,
        payer = admin,
        space = Whitelist::LEN,
        seeds = [b"whitelist-entry", address.as_ref()],
        bump
    )]
    pub whitelist_entry: Account<'info, Whitelist>,
    pub system_program: Program<'info, System>,
}
```

Each whitelist entry PDA is derived from `[b"whitelist-entry", address]`. The `constraint` on the config account ensures only the admin can add entries.

---

### 3. Remove from Whitelist

Closes the whitelist entry PDA and refunds rent to the admin.

```rust
#[derive(Accounts)]
#[instruction(address: Pubkey)]
pub struct RemoveFromWhiteList<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(
        seeds = [b"config"],
        bump = config.bump,
        constraint = config.admin == admin.key() @ ErrorCode::Unauthorized,
    )]
    pub config: Account<'info, Config>,
    #[account(
        mut,
        close = admin,
        seeds = [b"whitelist-entry", address.as_ref()],
        bump = whitelist_entry.bump
    )]
    pub whitelist_entry: Account<'info, Whitelist>,
    pub system_program: Program<'info, System>,
}
```

The `close = admin` constraint closes the account and returns the lamports to the admin.

---

### 4. Initialize Mint with Transfer Hook Extension

Creates a Token-2022 mint with the TransferHook extension enabled, pointing to this program.

```rust
#[derive(Accounts)]
pub struct TokenFactory<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    /// CHECK: We will create and initialize this account manually
    #[account(mut, signer)]
    pub mint: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token2022>,
}
```

The initialization happens in three steps:

1. **Create the account** with space calculated for the TransferHook extension using `ExtensionType::try_calculate_account_len`.
2. **Initialize the TransferHook extension** pointing to this program's ID, so all transfers invoke our `transfer_hook` instruction.
3. **Initialize the base mint** with `initialize_mint2` (extensions must be initialized before the base mint).

---

### 5. Initialize Extra Account Meta List

Sets up the extra accounts that Token-2022 will automatically resolve and pass to the transfer hook during transfers.

```rust
pub fn extra_account_metas() -> Result<Vec<ExtraAccountMeta>> {
    Ok(vec![
        ExtraAccountMeta::new_with_seeds(
            &[
                Seed::Literal {
                    bytes: b"whitelist-entry".to_vec(),
                },
                Seed::AccountKey {
                    index: 3, // source token owner's pubkey
                },
            ],
            false,
            false,
        )
        .map_err(|_| error!(ErrorCode::ExtraAccountMetaError))?,
    ])
}
```

This tells Token-2022 to derive the whitelist entry PDA using `[b"whitelist-entry", owner_pubkey]` at runtime. Index 3 refers to the owner account in the transfer instruction's account list.

---

### 6. Transfer Hook

Validates every token transfer by checking that the source token owner has a whitelist entry PDA.

```rust
#[derive(Accounts)]
pub struct TransferHook<'info> {
    #[account(token::mint = mint, token::authority = owner)]
    pub source_token: InterfaceAccount<'info, TokenAccount>,
    pub mint: InterfaceAccount<'info, Mint>,
    #[account(token::mint = mint)]
    pub destination_token: InterfaceAccount<'info, TokenAccount>,
    /// CHECK: source token account owner
    pub owner: UncheckedAccount<'info>,
    #[account(
        seeds = [b"extra-account-metas", mint.key().as_ref()],
        bump
    )]
    pub extra_account_meta_list: UncheckedAccount<'info>,
    #[account(
        seeds = [b"whitelist-entry", source_token.owner.key().as_ref()],
        bump = whitelist.bump,
    )]
    pub whitelist: Account<'info, Whitelist>,
}
```

The validation logic:

1. **Checks the transferring flag** on the source token account's TransferHookAccount extension to ensure this is being called during an actual transfer.
2. **Verifies the whitelist PDA exists** and that `whitelist.address == source_token.owner`. If the PDA doesn't exist or doesn't match, the transfer fails.

Since each whitelisted address has its own PDA, the lookup is O(1) — Token-2022 derives the PDA from the owner's pubkey and passes it in. If the account doesn't exist, the transaction fails automatically.

---

## Flow

```
1. initialize_config()          -> Creates Config PDA, sets admin
2. add_to_whitelist(address)    -> Creates Whitelist PDA for that address
3. init_mint(decimals)          -> Creates Token-2022 mint with TransferHook extension
4. initialize_transfer_hook()   -> Sets up ExtraAccountMetaList for the mint
5. transfer_hook(amount)        -> Auto-invoked on every transfer, validates whitelist
```
