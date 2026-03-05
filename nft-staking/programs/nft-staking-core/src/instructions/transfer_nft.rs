use anchor_lang::prelude::*;
use mpl_core::{instructions::TransferV1CpiBuilder, ID as MPL_CORE_ID};

use crate::state::Oracle;

#[derive(Accounts)]
pub struct TransferNft<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    /// CHECK: new owner of the NFT
    pub new_owner: UncheckedAccount<'info>,
    /// CHECK: validated by mpl-core
    #[account(mut)]
    pub nft: UncheckedAccount<'info>,
    /// CHECK: validated by mpl-core
    #[account(mut)]
    pub collection: UncheckedAccount<'info>,
    /// The global oracle account — passed as remaining account so mpl-core can
    /// evaluate the Oracle plugin's Transfer lifecycle check.
    #[account(
        seeds = [b"oracle"],
        bump = oracle.bump,
    )]
    pub oracle: Account<'info, Oracle>,
    /// CHECK: Metaplex Core program ID
    #[account(address = MPL_CORE_ID)]
    pub mpl_core_program: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
}

impl<'info> TransferNft<'info> {
    pub fn transfer_nft(&mut self) -> Result<()> {
        TransferV1CpiBuilder::new(&self.mpl_core_program.to_account_info())
            .asset(&self.nft.to_account_info())
            .collection(Some(&self.collection.to_account_info()))
            .payer(&self.user.to_account_info())
            .authority(Some(&self.user.to_account_info()))
            .new_owner(&self.new_owner.to_account_info())
            .system_program(Some(&self.system_program.to_account_info()))
            // Oracle account must be passed as a remaining account so mpl-core
            // can locate it by base_address and read the transfer validation byte.
            .add_remaining_account(&self.oracle.to_account_info(), false, false)
            .invoke()?;

        Ok(())
    }
}
