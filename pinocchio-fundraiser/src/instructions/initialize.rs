use pinocchio::{
    AccountView, Address, ProgramResult,
    cpi::{Seed, Signer},
    error::ProgramError,
    sysvars::{Sysvar, clock::Clock, rent::ACCOUNT_STORAGE_OVERHEAD},
};
use pinocchio_system::instructions::CreateAccount;
use pinocchio_token::ID as TOKEN_ID;
use pinocchio_token::instructions::InitializeAccount3;

use crate::{
    helper::check_signer,
    states::Fundraiser,
    utils::{check_zero, impl_len, impl_load_ix},
};

const DEFAULT_LAMPORTS_PER_BYTE: u64 = 6960;
const FUNDRAISER_RENT: u64 =
    (ACCOUNT_STORAGE_OVERHEAD + Fundraiser::LEN as u64) * DEFAULT_LAMPORTS_PER_BYTE;
const VAULT_RENT: u64 = (ACCOUNT_STORAGE_OVERHEAD + 165) * DEFAULT_LAMPORTS_PER_BYTE;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct InitializeInstructionData {
    pub amount_to_raise: u64,
    pub duration: u8,
    pub bump: u8,
    pub _padding: [u8; 6],
}

impl_len!(InitializeInstructionData);
impl_load_ix!(InitializeInstructionData);

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
        _system_program,
        _token_program,
        _remaining @ ..,
    ] = accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    check_signer(maker, ProgramError::IncorrectAuthority)?;

    let data = InitializeInstructionData::load(instruction_data)?;

    check_zero!(== data.amount_to_raise, ProgramError::InvalidInstructionData);
    check_zero!(== data.duration, ProgramError::InvalidInstructionData);

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
        lamports: FUNDRAISER_RENT,
    }
    .invoke_signed(&[signer_seeds])?;

    CreateAccount {
        from: maker,
        to: vault,
        owner: &TOKEN_ID,
        space: 165,
        lamports: VAULT_RENT,
    }
    .invoke()?;

    InitializeAccount3 {
        account: vault,
        mint,
        owner: fundraiser.address(),
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
