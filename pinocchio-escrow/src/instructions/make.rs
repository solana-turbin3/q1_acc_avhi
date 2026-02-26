use crate::entrypoint::ID;
use pinocchio::{
    cpi::{Seed, Signer},
    error::ProgramError,
    sysvars::{rent::Rent, Sysvar},
    AccountView, ProgramResult,
};
use pinocchio_associated_token_account::instructions::Create;
use pinocchio_pubkey::derive_address;
use pinocchio_system::instructions::CreateAccount;
use pinocchio_token::instructions::Transfer;

use crate::{
    state::Escrow,
    utils::{impl_len, impl_load},
};

#[repr(C)]
#[derive(Clone, Copy)]
pub struct MakeInstructionData {
    pub bump: u8,
    pub amount_to_receive: [u8; 8],
    pub amount_to_give: [u8; 8],
}

impl_len!(MakeInstructionData);
impl_load!(MakeInstructionData);

pub fn process_make_instruction(accounts: &[AccountView], data: &[u8]) -> ProgramResult {
    let [maker, mint_a, mint_b, escrow_account, maker_ata, escrow_ata, system_program, token_program, _remaining @ ..] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    if !maker.is_signer() {
        return Err(ProgramError::IncorrectAuthority);
    }

    if data.len() < MakeInstructionData::LEN {
        return Err(ProgramError::InvalidInstructionData);
    }

    let ix_data = MakeInstructionData::load(data)?;

    let bump = ix_data.bump;
    let amount_to_receive = u64::from_le_bytes(ix_data.amount_to_receive);
    let amount_to_give = u64::from_le_bytes(ix_data.amount_to_give);

    let maker_ata_state = pinocchio_token::state::TokenAccount::from_account_view(maker_ata)?;
    if maker_ata_state.owner() != maker.address() {
        return Err(ProgramError::IllegalOwner);
    }
    if maker_ata_state.mint() != mint_a.address() {
        return Err(ProgramError::InvalidAccountData);
    }

    let seeds: [&[u8]; 3] = [b"escrow", maker.address().as_array(), &[bump]];
    let expected_escrow = derive_address(&seeds, None, ID.as_array());

    if escrow_account.address().as_array() != &expected_escrow {
        return Err(ProgramError::InvalidAccountData);
    }

    let bump_seed = [bump];
    let signer_seeds = [
        Seed::from(b"escrow"),
        Seed::from(maker.address().as_array()),
        Seed::from(&bump_seed),
    ];
    let signer = Signer::from(&signer_seeds[..]);

    CreateAccount {
        from: maker,
        to: escrow_account,
        lamports: Rent::get()?.minimum_balance_unchecked(Escrow::LEN),
        space: Escrow::LEN as u64,
        owner: &ID,
    }
    .invoke_signed(&[signer])?;

    let escrow_data = unsafe { escrow_account.borrow_unchecked_mut() };
    let escrow_state = Escrow::load_mut(escrow_data)?;

    escrow_state.maker = *maker.address().as_array();
    escrow_state.mint_a = *mint_a.address().as_array();
    escrow_state.mint_b = *mint_b.address().as_array();
    escrow_state.amount_to_receive = amount_to_receive;
    escrow_state.amount_to_give = amount_to_give;
    escrow_state.bump = bump;

    Create {
        funding_account: maker,
        account: escrow_ata,
        wallet: escrow_account,
        mint: mint_a,
        token_program,
        system_program,
    }
    .invoke()?;

    Transfer {
        from: maker_ata,
        to: escrow_ata,
        authority: maker,
        amount: amount_to_give,
    }
    .invoke()?;

    Ok(())
}
