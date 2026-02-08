use anchor_lang::prelude::*;
use anchor_spl::token_interface::Mint;
use spl_token_2022::extension::BaseStateWithExtensions;
use spl_token_2022::extension::{transfer_hook::TransferHookAccount, PodStateWithExtensionsMut};
use spl_token_2022::pod::PodAccount;

use crate::{UserAccount, EXTRA_ACCOUNT_METAS, WHITELIST_ENTRY};

#[derive(Accounts)]
pub struct TransferHook<'info> {
    #[account(
          token::mint = mint,
          token::token_program = anchor_spl::token_2022::ID,
      )]
    pub source_token: InterfaceAccount<'info, anchor_spl::token_interface::TokenAccount>,

    pub mint: InterfaceAccount<'info, Mint>,

    #[account(
          token::mint = mint,
          token::token_program = anchor_spl::token_2022::ID,
      )]
    pub destination_token: InterfaceAccount<'info, anchor_spl::token_interface::TokenAccount>,

    /// CHECK: Owner of source token, Token-2022 passes this
    pub owner: AccountInfo<'info>,

    /// CHECK: The extra account meta list PDA
    #[account(
          seeds = [EXTRA_ACCOUNT_METAS.as_bytes(), mint.key().as_ref()],
          bump,
      )]
    pub extra_account_meta_list: AccountInfo<'info>,

    #[account(
          seeds = [WHITELIST_ENTRY.as_bytes(), owner.key().as_ref()],
          bump = whitelist.bump,
      )]
    pub whitelist: Account<'info, UserAccount>,
}

impl<'info> TransferHook<'info> {
    pub fn transfer_hook(&mut self, _amount: u64) -> Result<()> {
        self.check_is_transferring()?;

        require_keys_eq!(
            self.whitelist.account,
            self.owner.key(),
            anchor_lang::error::ErrorCode::ConstraintOwner
        );

        msg!("Transfer allowed: {} is whitelisted", self.owner.key());

        Ok(())
    }

    fn check_is_transferring(&self) -> Result<()> {
        let source_token_info = self.source_token.to_account_info();
        let mut account_data = source_token_info.try_borrow_mut_data()?;
        let account = PodStateWithExtensionsMut::<PodAccount>::unpack(&mut account_data)?;
        let transfer_hook = account.get_extension::<TransferHookAccount>()?;

        if !bool::from(transfer_hook.transferring) {
            return err!(anchor_lang::error::ErrorCode::AccountNotInitialized);
        }

        Ok(())
    }
}
