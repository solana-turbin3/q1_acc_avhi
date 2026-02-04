#![allow(unexpected_cfgs)]
#![allow(deprecated)]

use anchor_lang::prelude::*;

mod constants;
mod error;
mod instructions;
mod state;

use instructions::*;

use spl_discriminator::SplDiscriminate;
use spl_transfer_hook_interface::instruction::ExecuteInstruction;

declare_id!("DhzyDgCmmQzVC4vEcj2zRGUyN8Mt5JynfdGLKkBcRGaX");

#[program]
pub mod whitelist_transfer_hook {
    use super::*;

    pub fn initialize_config(ctx: Context<InitializeConfig>) -> Result<()> {
        ctx.accounts.initialize_config(&ctx.bumps)
    }

    pub fn add_to_whitelist(ctx: Context<AddToWhiteList>, address: Pubkey) -> Result<()> {
        ctx.accounts.add_to_whitelist(address, &ctx.bumps)
    }

    pub fn remove_from_whitelist(ctx: Context<RemoveFromWhiteList>, address: Pubkey) -> Result<()> {
        ctx.accounts.remove_from_whitelist(address)
    }

    pub fn init_mint(ctx: Context<TokenFactory>, decimals: u8) -> Result<()> {
        ctx.accounts.init_mint(decimals)
    }

    pub fn initialize_transfer_hook(ctx: Context<InitializeExtraAccountMetaList>) -> Result<()> {
        ctx.accounts.initialize_extra_account_meta_list(&ctx.bumps)
    }

    #[instruction(discriminator = ExecuteInstruction::SPL_DISCRIMINATOR_SLICE)]
    pub fn transfer_hook(ctx: Context<TransferHook>, amount: u64) -> Result<()> {
        // Call the transfer hook logic
        ctx.accounts.transfer_hook(amount)
    }
}
