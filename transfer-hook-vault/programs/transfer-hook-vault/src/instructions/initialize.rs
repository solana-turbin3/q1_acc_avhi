use anchor_lang::prelude::{program::invoke, system_instruction::create_account, *};
use anchor_spl::token_interface::TokenInterface;
use spl_token_2022::{
    extension::{transfer_hook::instruction::initialize as init_transfer_hook, ExtensionType},
    instruction::initialize_mint2,
    state::Mint as Token2022Mint,
};

use crate::{error::ErrorCode, Vault, VAULT_CONFIG};

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

impl<'info> Initialize<'info> {
    pub fn initialize(&mut self, decimal: u8, bump: &InitializeBumps) -> Result<()> {
        self.vault.set_inner(Vault {
            admin: self.admin.key(),
            mint: self.mint.key(),
            bump: bump.vault,
        });

        let extension_type = vec![ExtensionType::TransferHook];
        let space = ExtensionType::try_calculate_account_len::<Token2022Mint>(&extension_type)
            .map_err(|_| ErrorCode::InvalidAccountSize)?;

        let lamport = Rent::get()?.minimum_balance(space);

        let create_ix = create_account(
            &self.admin.key(),
            &self.mint.key(),
            lamport,
            space as u64,
            &self.token_program.key(),
        );

        invoke(
            &create_ix,
            &[
                self.admin.to_account_info(),
                self.mint.to_account_info(),
                self.system_program.to_account_info(),
            ],
        )?;

        let init_hook_ix = init_transfer_hook(
            &self.token_program.key(),
            &self.mint.key(),
            Some(self.admin.key()),
            Some(crate::ID),
        )?;

        invoke(&init_hook_ix, &[self.mint.to_account_info()])?;

        let init_mint_ix = initialize_mint2(
            &self.token_program.key(),
            &self.mint.key(),
            &self.admin.key(),
            Some(&self.admin.key()),
            decimal,
        )?;

        invoke(&init_mint_ix, &[self.mint.to_account_info()])?;

        Ok(())
    }
}
