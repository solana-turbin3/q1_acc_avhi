use anchor_lang::prelude::*;

mod state;
mod instructions;
mod errors;
mod helpers;
use instructions::*;

declare_id!("BLvzEjXLbhUWNMptcG33MAMMdd7myK69kbCHFQPmdXHk");

#[program]
pub mod nft_staking_core {
    use super::*;

    pub fn create_collection(ctx: Context<CreateCollection>, name: String, uri: String) -> Result<()> {
        ctx.accounts.create_collection(name, uri, &ctx.bumps)
    }

    pub fn mint_nft(ctx: Context<Mint>, name: String, uri: String) -> Result<()> {
        ctx.accounts.mint_nft(name, uri, &ctx.bumps)
    }

    pub fn initialize_config(ctx: Context<InitConfig>, points_per_stake: u32, freeze_period: u8) -> Result<()> {
        ctx.accounts.init_config(points_per_stake, freeze_period, &ctx.bumps)
    }

    pub fn stake(ctx: Context<Stake>) -> Result<()> {
        ctx.accounts.stake(&ctx.bumps)
    }

    pub fn unstake(ctx: Context<Unstake>) -> Result<()> {
        ctx.accounts.unstake(&ctx.bumps)
    }

    // Task 1.1 — claim accumulated rewards without unstaking
    pub fn claim_rewards(ctx: Context<ClaimRewards>) -> Result<()> {
        ctx.accounts.claim_rewards(&ctx.bumps)
    }

    // Task 1.2 — burn staked NFT for a massive one-time reward bonus
    pub fn burn_staked_nft(ctx: Context<BurnStakedNft>) -> Result<()> {
        ctx.accounts.burn_staked_nft(&ctx.bumps)
    }

    // Task 2 — Oracle: initialise the oracle account + vault PDA
    pub fn init_oracle(ctx: Context<InitOracle>) -> Result<()> {
        ctx.accounts.init_oracle(&ctx.bumps)
    }

    // Task 2 — Oracle: attach the Oracle external plugin adapter to the collection
    pub fn add_oracle_to_collection(ctx: Context<AddOracleToCollection>) -> Result<()> {
        ctx.accounts.add_oracle_to_collection(&ctx.bumps)
    }

    // Task 2 — Oracle: permissionless crank — update validation state + optional reward
    pub fn update_oracle(ctx: Context<UpdateOracle>) -> Result<()> {
        ctx.accounts.update_oracle()
    }

    // Task 2 — Oracle: fund the crank-reward vault
    pub fn fund_vault(ctx: Context<FundVault>, amount: u64) -> Result<()> {
        ctx.accounts.fund_vault(amount)
    }

    // Task 2 — Oracle: transfer NFT (validated by the time-gated oracle)
    pub fn transfer_nft(ctx: Context<TransferNft>) -> Result<()> {
        ctx.accounts.transfer_nft()
    }

}
