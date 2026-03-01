use pinocchio::{
    AccountView, Address, ProgramResult,
    cpi::{Seed, Signer},
    error::ProgramError,
    sysvars::{Sysvar, clock::Clock, rent::Rent},
};
use pinocchio_pubkey::derive_address;
use pinocchio_system::instructions::CreateAccount;
use pinocchio_token::{ID as TOKEN_ID, instructions::Transfer};

use crate::{
    helper::{check_signer, validate_eq},
    states::{Contributor, Fundraiser},
    utils::{check_zero, impl_len, impl_load},
};

#[repr(C)]
#[derive(Clone, Copy)]
pub struct ContributeInstructionData {
    pub amount: u64,
    pub bump: u8,
    pub _padding: [u8; 7],
}

impl_len!(ContributeInstructionData);
impl_load!(ContributeInstructionData);

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
        system_program,
        token_program,
        _remaining @ ..,
    ] = accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    check_signer(contributor, ProgramError::IncorrectAuthority)?;

    let data = ContributeInstructionData::load(instruction_data)?;

    check_zero!(== data.amount, ProgramError::InvalidInstructionData);

    validate_eq(
        system_program.address(),
        &pinocchio_system::ID,
        ProgramError::IncorrectProgramId,
    )?;

    validate_eq(
        token_program.address(),
        &TOKEN_ID,
        ProgramError::IncorrectProgramId,
    )?;

    let (expected_fundraiser, mint_to_raise, time_started, duration, fundraiser_bump) = {
        let fundraiser_data = unsafe { fundraiser.borrow_unchecked() };
        let state = Fundraiser::load(fundraiser_data)?;

        let seeds: [&[u8]; 2] = [b"fundraiser", &state.maker];
        let expected = derive_address(&seeds, Some(state.bump), program_id.as_array());

        (
            expected,
            state.mint_to_raise,
            state.time_started,
            state.duration,
            state.bump,
        )
    };

    validate_eq(
        fundraiser.address().as_array(),
        &expected_fundraiser,
        ProgramError::InvalidAccountData,
    )?;

    validate_eq(
        mint.address().as_array(),
        &mint_to_raise,
        ProgramError::InvalidAccountData,
    )?;

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
        lamports: Rent::get()?.minimum_balance_unchecked(Contributor::LEN),
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
    contributor_st.amount = data.amount;
    contributor_st.bump = data.bump;

    let fundraiser_data_mut = unsafe { fundraiser.borrow_unchecked_mut() };
    let fundraiser_st = Fundraiser::load_mut(fundraiser_data_mut)?;
    fundraiser_st.current_amount = fundraiser_st
        .current_amount
        .checked_add(data.amount)
        .ok_or(ProgramError::ArithmeticOverflow)?;

    let _ = fundraiser_bump;

    Ok(())
}
