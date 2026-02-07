use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{self, Mint, TokenAccount, TokenInterface, TransferChecked},
};

use crate::{
    error::ErrorCode, UserAccount, Vault, EXTRA_ACCOUNT_METAS, VAULT_CONFIG, WHITELIST_ENTRY,
};

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        seeds = [VAULT_CONFIG.as_bytes(), vault.admin.as_ref()],
        bump = vault.bump,
    )]
    pub vault: Account<'info, Vault>,

    #[account(
        mut,
        seeds = [WHITELIST_ENTRY.as_bytes(), user.key().as_ref()],
        bump = user_account.bump,
    )]
    pub user_account: Account<'info, UserAccount>,

    #[account(address = vault.mint)]
    pub mint: InterfaceAccount<'info, Mint>,

    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = user,
        associated_token::token_program = token_program,
    )]
    pub user_token_account: InterfaceAccount<'info, TokenAccount>,

    #[account(
        init_if_needed,
        payer = user,
        associated_token::mint = mint,
        associated_token::authority = vault,
        associated_token::token_program = token_program,
    )]
    pub vault_token_account: InterfaceAccount<'info, TokenAccount>,

    /// CHECK: ExtraAccountMetaList PDA
    #[account(
        seeds = [EXTRA_ACCOUNT_METAS.as_bytes(), mint.key().as_ref()],
        bump,
    )]
    pub extra_account_meta_list: AccountInfo<'info>,

    /// CHECK: Our program - Token-2022 needs it to CPI the hook
    #[account(address = crate::ID)]
    pub hook_program: AccountInfo<'info>,

    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

impl<'info> Deposit<'info> {
    pub fn deposit(&mut self, amount: u64) -> Result<()> {
        let cpi_ctx = CpiContext::new(
            self.token_program.to_account_info(),
            TransferChecked {
                from: self.user_token_account.to_account_info(),
                to: self.vault_token_account.to_account_info(),
                authority: self.user.to_account_info(),
                mint: self.mint.to_account_info(),
            },
        )
        .with_remaining_accounts(vec![
            self.extra_account_meta_list.to_account_info(),
            self.user_account.to_account_info(),
            self.hook_program.to_account_info(),
        ]);

        token_interface::transfer_checked(cpi_ctx, amount, self.mint.decimals)?;

        self.user_account.amount = self
            .user_account
            .amount
            .checked_add(amount)
            .ok_or(ErrorCode::Overflow)?;

        msg!("Deposited {} tokens", amount);
        Ok(())
    }
}
