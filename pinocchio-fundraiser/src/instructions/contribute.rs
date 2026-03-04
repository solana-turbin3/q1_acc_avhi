use pinocchio::{
    AccountView, Address, ProgramResult,
    cpi::{Seed, Signer},
    error::ProgramError,
    sysvars::{Sysvar, clock::Clock, rent::ACCOUNT_STORAGE_OVERHEAD},
};
use pinocchio_system::instructions::CreateAccount;
use pinocchio_token::instructions::Transfer;

use crate::{
    helper::check_signer,
    states::{Contributor, Fundraiser},
    utils::{check_zero, impl_len, impl_load_ix},
};

const DEFAULT_LAMPORTS_PER_BYTE: u64 = 6960;
const CONTRIBUTOR_RENT: u64 =
    (ACCOUNT_STORAGE_OVERHEAD + Contributor::LEN as u64) * DEFAULT_LAMPORTS_PER_BYTE;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct ContributeInstructionData {
    pub amount: u64,
    pub bump: u8,
    pub _padding: [u8; 7],
}

impl_len!(ContributeInstructionData);
impl_load_ix!(ContributeInstructionData);

pub fn process_contribute(
    program_id: &Address,
    accounts: &[AccountView],
    instruction_data: &[u8],
) -> ProgramResult {
    let [
        contributor,
        fundraiser,
        mint,
        vault,
        contributor_ata,
        contributor_state,
        _system_program,
        _token_program,
        _remaining @ ..,
    ] = accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    check_signer(contributor, ProgramError::IncorrectAuthority)?;

    let data = ContributeInstructionData::load(instruction_data)?;

    check_zero!(== data.amount, ProgramError::InvalidInstructionData);

    if !fundraiser.owned_by(program_id) {
        return Err(ProgramError::IllegalOwner);
    }

    let (mint_to_raise, time_started, duration) = {
        let fundraiser_data = unsafe { fundraiser.borrow_unchecked() };
        let state = Fundraiser::load(fundraiser_data)?;
        (state.mint_to_raise, state.time_started, state.duration)
    };

    if mint.address().as_array() != &mint_to_raise {
        return Err(ProgramError::InvalidAccountData);
    }

    let deadline = time_started + duration as i64 * 86400;
    if Clock::get()?.unix_timestamp > deadline {
        return Err(ProgramError::InvalidAccountData);
    }

    let bump = [data.bump];
    let contributor_seeds = [
        Seed::from(b"contributor"),
        Seed::from(fundraiser.address().as_array()),
        Seed::from(contributor.address().as_array()),
        Seed::from(&bump),
    ];
    let contributor_signer = Signer::from(&contributor_seeds[..]);

    CreateAccount {
        from: contributor,
        to: contributor_state,
        space: Contributor::LEN as u64,
        owner: program_id,
        lamports: CONTRIBUTOR_RENT,
    }
    .invoke_signed(&[contributor_signer])?;

    Transfer {
        from: contributor_ata,
        to: vault,
        authority: contributor,
        amount: data.amount,
    }
    .invoke()?;

    let contributor_data = unsafe { contributor_state.borrow_unchecked_mut() };
    let contributor_st = Contributor::load_mut(contributor_data)?;
    contributor_st.contributor = *contributor.address().as_array();
    contributor_st.amount = data.amount;
    contributor_st.bump = data.bump;

    let fundraiser_data_mut = unsafe { fundraiser.borrow_unchecked_mut() };
    let fundraiser_st = Fundraiser::load_mut(fundraiser_data_mut)?;
    fundraiser_st.current_amount = fundraiser_st
        .current_amount
        .checked_add(data.amount)
        .ok_or(ProgramError::ArithmeticOverflow)?;

    Ok(())
}
