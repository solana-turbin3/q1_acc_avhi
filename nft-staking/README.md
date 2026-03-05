# NFT Staking Core

This program implements an NFT staking contract built with Anchor, leveraging the Metaplex Core plugins `Attribute` and `FreezeDelegate`. This example showcases how to manage staking data directly on the asset, without storing it in PDAs.

---

## How it works

The program allows users to stake their Core NFTs in a non-custodial way and for a configurable minimum period. When unstaking, rewards are calculated based on how long the NFT was staked and minted to the user's token account.

The staking state is stored directly on the NFT using the **Attribute Plugin**, and the NFT is locked using the **FreezeDelegate Plugin**.

---

## Plugins

### FreezeDelegate Plugin

The **FreezeDelegate Plugin** is an **owner-managed plugin** that allows a delegate to freeze and thaw an asset. While frozen, the asset cannot be transferred. Freezing is as simple as toggling a boolean in the plugin data.

When staking, the plugin is added (or updated if it already exists from a previous stake cycle) and the asset is frozen. On unstake, the asset is unfreezed.

### Attribute Plugin

The **Attribute Plugin** is an **authority-managed plugin** that stores key-value pairs directly on the asset, on-chain. This allows programs to read and write traits without touching off-chain metadata.

This program uses two attributes on each NFT:
- `staked` — `"true"` when the NFT is currently staked, `"false"` otherwise
- `staked_at` — the Unix timestamp when the NFT was staked (reset to `"0"` on unstake)

---

## Program State

The `Config` account is a PDA scoped to each collection and holds the staking configuration:

```rust
#[account]
pub struct Config {
    pub points_per_stake: u32,  // reward tokens earned per day
    pub freeze_period: u8,      // minimum days before unstaking is allowed
    pub rewards_bump: u8,
    pub config_bump: u8,
}
```

The `Config` account also acts as the mint authority for the rewards token, which is a separate SPL token mint derived as a PDA from the config.

---

## Instructions

### `create_collection`

Creates a new Metaplex Core collection. The update authority is a PDA derived from the collection key, so the program is the sole authority over NFTs in the collection.

### `mint_nft`

Mints a new Core NFT into the collection. The update authority is the collection, ensuring the program can manage plugins.

### `initialize_config`

Sets up the staking configuration for a collection. Initializes the `Config` account and the rewards token mint. Only needs to be called once per collection.

### `stake`

Stakes an NFT from the collection.

- Verifies the user is the owner of the NFT and that the NFT belongs to the collection
- Adds (or updates) the Attribute Plugin with `staked = true` and `staked_at = <current timestamp>`
- Adds the FreezeDelegate Plugin if this is the first time staking; otherwise updates the existing one

### `unstake`

Unstakes the NFT and mints rewards to the user.

- Validates the NFT is currently staked
- Checks that the minimum freeze period has elapsed
- Calculates the number of full days staked and mints `days * points_per_stake` reward tokens to the user's ATA
- Resets the staking attributes (`staked = false`, `staked_at = 0`)
- Thaws the asset (FreezeDelegate plugin stays on the asset with `false` state)

---

## Architecture walkthrough

### Account constraints

The `Stake` and `Unstake` contexts both verify:
- The NFT's `owner` matches the signer
- The NFT's `update_authority` is the collection
- The collection's `update_authority` is the program's PDA

### PDA structure

| Account | Seeds |
|---|---|
| Update Authority | `["update_authority", collection]` |
| Config | `["config", collection]` |
| Rewards Mint | `["rewards", config]` |

---

## Dependencies

```toml
anchor-lang = "0.32.1"
anchor-spl = "0.32.1"
mpl-core = { version = "0.11.1", features = ["anchor"] }
```
