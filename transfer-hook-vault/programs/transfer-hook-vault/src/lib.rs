pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;
mod tests;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;

use spl_discriminator::SplDiscriminate;

declare_id!("3n16mCbPsep8awDkznTGPNDFnJAKhgGRDEcsExX7G33S");

#[program]
pub mod transfer_hook_vault {

    use super::*;

    pub fn initialize(
        ctx: Context<Initialize>,
        decimal: u8,
        name: String,
        symbol: String,
        uri: String,
    ) -> Result<()> {
        ctx.accounts
            .initialize(decimal, name, symbol, uri, &ctx.bumps)
    }

    pub fn add_user(ctx: Context<AddUser>, address: Pubkey) -> Result<()> {
        ctx.accounts.add_user(address, &ctx.bumps)
    }

    pub fn remove_user(ctx: Context<RemoveUser>, address: Pubkey) -> Result<()> {
        ctx.accounts.remove_user(address)
    }

    pub fn init_extra_acc_meta(ctx: Context<InitExtraAccountMeta>) -> Result<()> {
        ctx.accounts.init_extra_account_meta(&ctx.bumps)
    }

    #[instruction(discriminator = spl_transfer_hook_interface::instruction::ExecuteInstruction::SPL_DISCRIMINATOR_SLICE)]
    pub fn transfer_hook(ctx: Context<TransferHook>, amount: u64) -> Result<()> {
        ctx.accounts.transfer_hook(amount)
    }

    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        ctx.accounts.deposit(amount)
    }

    pub fn withdraw(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
        ctx.accounts.withdraw(amount)
    }
}
