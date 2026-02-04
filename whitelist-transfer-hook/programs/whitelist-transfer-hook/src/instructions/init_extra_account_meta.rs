use crate::constants::{EXTRA_ACCOUNT_METAS_SEED, WHITELIST_ENTRY_SEED};
use crate::error::ErrorCode;
use anchor_lang::prelude::*;
use anchor_lang::system_program::{create_account, CreateAccount};
use anchor_spl::token_interface::Mint;
use spl_tlv_account_resolution::{
    account::ExtraAccountMeta, seeds::Seed, state::ExtraAccountMetaList,
};
use spl_transfer_hook_interface::instruction::ExecuteInstruction;

#[derive(Accounts)]
pub struct InitializeExtraAccountMetaList<'info> {
    #[account(mut)]
    payer: Signer<'info>,

    /// CHECK: ExtraAccountMetaList Account, must use these seeds
    #[account(
        mut,
        seeds = [EXTRA_ACCOUNT_METAS_SEED, mint.key().as_ref()],
        bump,

    )]
    pub extra_account_meta_list: AccountInfo<'info>,
    pub mint: InterfaceAccount<'info, Mint>,
    pub system_program: Program<'info, System>,
}

impl<'info> InitializeExtraAccountMetaList<'info> {
    pub fn initialize_extra_account_meta_list(
        &mut self,
        bumps: &InitializeExtraAccountMetaListBumps,
    ) -> Result<()> {
        // Get the extra accounts needed for the transfer hook
        let extra_account_metas = Self::extra_account_metas()?;

        // Calculate the size needed for the extra account meta list
        let account_size = ExtraAccountMetaList::size_of(extra_account_metas.len())
            .map_err(|_| error!(ErrorCode::ExtraAccountMetaError))?;

        // Calculate rent
        let lamports = Rent::get()?.minimum_balance(account_size);

        // Get the seeds for the PDA
        let mint_key = self.mint.key();
        let signer_seeds: &[&[&[u8]]] = &[&[
            EXTRA_ACCOUNT_METAS_SEED,
            mint_key.as_ref(),
            &[bumps.extra_account_meta_list],
        ]];

        // Create the account
        create_account(
            CpiContext::new(
                self.system_program.to_account_info(),
                CreateAccount {
                    from: self.payer.to_account_info(),
                    to: self.extra_account_meta_list.to_account_info(),
                },
            )
            .with_signer(signer_seeds),
            lamports,
            account_size as u64,
            &crate::ID,
        )?;

        // Initialize the extra account meta list
        let mut data = self.extra_account_meta_list.try_borrow_mut_data()?;
        ExtraAccountMetaList::init::<ExecuteInstruction>(&mut data, &extra_account_metas)
            .map_err(|_| error!(ErrorCode::ExtraAccountMetaError))?;

        Ok(())
    }

    /// Now we need to include the whitelist_entry PDA
    /// But we need to derive it based on the source token owner at runtime
    /// So we use ExtraAccountMeta::new_with_seeds() to tell Token-2022
    /// "derive the PDA using these seeds + the owner pubkey"
    pub fn extra_account_metas() -> Result<Vec<ExtraAccountMeta>> {
        Ok(vec![
            // This tells Token-2022: "Derive a PDA with seeds [b"whitelist-entry", source_owner]"
            // The source_owner comes from the transfer instruction
            ExtraAccountMeta::new_with_seeds(
                &[
                    Seed::Literal {
                        bytes: WHITELIST_ENTRY_SEED.to_vec(),
                    },
                    Seed::AccountKey {
                        index: 3, // Use owner's pubkey as seed
                    },
                ],
                false,
                false,
            )
            .map_err(|_| error!(ErrorCode::ExtraAccountMetaError))?,
        ])
    }
}
