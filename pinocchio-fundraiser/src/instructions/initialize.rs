use pinocchio::{
    AccountView, Address, ProgramResult,
    cpi::{Seed, Signer},
    error::ProgramError,
    sysvars::{Sysvar, clock::Clock, rent::RENT_ID, rent::Rent},
};
use pinocchio_system::ID;
use pinocchio_system::instructions::CreateAccount;
use pinocchio_token::ID as TOKEN_ID;
use pinocchio_token::instructions::InitializeAccount;

use crate::{
    helper::{check_signer, validate_eq},
    states::Fundraiser,
    utils::{check_zero, impl_len, impl_load},
};

#[repr(C)]
#[derive(Clone, Copy)]
pub struct InitializeInstructionData {
    pub amount_to_raise: u64,
    pub duration: u8,
    pub bump: u8,
    pub _padding: [u8; 6],
}

impl_len!(InitializeInstructionData);
impl_load!(InitializeInstructionData);

pub fn process_initialize(
    program_id: &Address,
    accounts: &[AccountView],
    instruction_data: &[u8],
) -> ProgramResult {
    let [
        maker,
        fundraiser,
        mint,
        vault,
        system_program,
        token_program,
        rent_sysvar,
        _remaining @ ..,
    ] = accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    check_signer(maker, ProgramError::IncorrectAuthority)?;

    let data = InitializeInstructionData::load(instruction_data)?;

    check_zero!(== data.amount_to_raise, ProgramError::InvalidInstructionData);
    check_zero!(== data.duration, ProgramError::InvalidInstructionData);

    validate_eq(
        system_program.address(),
        &ID,
        ProgramError::IncorrectProgramId,
    )?;
    validate_eq(
        token_program.address(),
        &TOKEN_ID,
        ProgramError::IncorrectProgramId,
    )?;
    validate_eq(
        rent_sysvar.address(),
        &RENT_ID,
        ProgramError::InvalidAccountData,
    )?;

    if mint.is_data_empty() || mint.lamports() == 0 {
        return Err(ProgramError::UninitializedAccount);
    }

    let bump = [data.bump];
    let seeds = [
        Seed::from(b"fundraiser"),
        Seed::from(maker.address().as_array()),
        Seed::from(&bump),
    ];
    let signer_seeds = Signer::from(&seeds[..]);

    CreateAccount {
        from: maker,
        to: fundraiser,
        space: Fundraiser::LEN as u64,
        owner: program_id,
        lamports: Rent::get()?.minimum_balance_unchecked(Fundraiser::LEN),
    }
    .invoke_signed(&[signer_seeds])?;

    CreateAccount {
        from: maker,
        to: vault,
        owner: &TOKEN_ID,
        space: 165,
        lamports: Rent::get()?.minimum_balance_unchecked(165),
    }
    .invoke()?;

    InitializeAccount {
        account: vault,
        mint,
        owner: fundraiser,
        rent_sysvar,
    }
    .invoke()?;

    let fundraiser_data = unsafe { fundraiser.borrow_unchecked_mut() };
    let fundraiser_state = Fundraiser::load_mut(fundraiser_data)?;

    fundraiser_state.maker = *maker.address().as_array();
    fundraiser_state.mint_to_raise = *mint.address().as_array();
    fundraiser_state.amount_to_raise = data.amount_to_raise;
    fundraiser_state.current_amount = 0;
    fundraiser_state.time_started = Clock::get()?.unix_timestamp;
    fundraiser_state.duration = data.duration;
    fundraiser_state.bump = data.bump;

    Ok(())
}
