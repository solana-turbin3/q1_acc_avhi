use anchor_lang::prelude::{system_instruction::create_account, *};
use anchor_lang::solana_program::program::invoke_signed;
use anchor_spl::token_interface::{Mint, TokenInterface};
use spl_tlv_account_resolution::{
    account::ExtraAccountMeta, seeds::Seed, state::ExtraAccountMetaList,
};
use spl_transfer_hook_interface::instruction::ExecuteInstruction;

use crate::{error::ErrorCode, EXTRA_ACCOUNT_METAS, WHITELIST_ENTRY};

#[derive(Accounts)]
pub struct InitExtraAccountMeta<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    /// CHECK: We create and initialize this account manually
    #[account(
        mut,
        seeds = [EXTRA_ACCOUNT_METAS.as_bytes(), mint.key().as_ref()],
        bump
    )]
    pub extra_acc_meta_list: AccountInfo<'info>,

    pub mint: InterfaceAccount<'info, Mint>,

    pub system_program: Program<'info, System>,

    pub token_program: Interface<'info, TokenInterface>,
}

impl<'info> InitExtraAccountMeta<'info> {
    pub fn init_extra_account_meta(&mut self, bump: &InitExtraAccountMetaBumps) -> Result<()> {
        let get_acc_meta = Self::extra_acc_meta()?;

        let space = ExtraAccountMetaList::size_of(get_acc_meta.len())
            .map_err(|_| ErrorCode::InvalidAccountSize)?;

        let lamports = Rent::get()?.minimum_balance(space);

        let mint = self.mint.key();
        let signers_seeds: &[&[u8]] = &[
            EXTRA_ACCOUNT_METAS.as_bytes(),
            mint.as_ref(),
            &[bump.extra_acc_meta_list],
        ];

        let create_ix = create_account(
            &self.payer.key(),
            &self.extra_acc_meta_list.key(),
            lamports,
            space as u64,
            &crate::ID,
        );

        invoke_signed(
            &create_ix,
            &[
                self.payer.to_account_info(),
                self.extra_acc_meta_list.to_account_info(),
                self.system_program.to_account_info(),
            ],
            &[signers_seeds],
        )?;

        ExtraAccountMetaList::init::<ExecuteInstruction>(
            &mut self.extra_acc_meta_list.try_borrow_mut_data()?,
            &get_acc_meta,
        )
        .map_err(|_| ErrorCode::InvalidAccountSize)?;

        Ok(())
    }

    fn extra_acc_meta() -> Result<Vec<ExtraAccountMeta>> {
        Ok(vec![ExtraAccountMeta::new_with_seeds(
            &[
                Seed::Literal {
                    bytes: WHITELIST_ENTRY.as_bytes().to_vec(),
                },
                Seed::AccountKey { index: 3 },
            ],
            false,
            false,
        )
        .map_err(|_| ErrorCode::ExtraAccountMetaError)?])
    }
}
