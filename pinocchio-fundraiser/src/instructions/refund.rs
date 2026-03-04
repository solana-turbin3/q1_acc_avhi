use pinocchio::{
    AccountView, Address, ProgramResult,
    cpi::{Seed, Signer},
    error::ProgramError,
    sysvars::{Sysvar, clock::Clock},
};
use pinocchio_token::instructions::Transfer;

use crate::{
    helper::check_signer,
    states::{Contributor, Fundraiser},
    utils::check_zero,
};

pub fn process_refund(
    program_id: &Address,
    accounts: &[AccountView],
    _instruction_data: &[u8],
) -> ProgramResult {
    let [
        contributor,
        fundraiser,
        vault,
        contributor_ata,
        contributor_state,
        _token_program,
        _remaining @ ..,
    ] = accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    check_signer(contributor, ProgramError::IncorrectAuthority)?;

    let (amount_to_raise, current_amount, time_started, duration, maker, fundraiser_bump) = {
        let fundraiser_data = unsafe { fundraiser.borrow_unchecked() };
        let state = Fundraiser::load(fundraiser_data)?;
        (
            state.amount_to_raise,
            state.current_amount,
            state.time_started,
            state.duration,
            state.maker,
            state.bump,
        )
    };

    let deadline = time_started + duration as i64 * 86400;
    if Clock::get()?.unix_timestamp <= deadline {
        return Err(ProgramError::InvalidAccountData);
    }

    if current_amount >= amount_to_raise {
        return Err(ProgramError::InvalidAccountData);
    }

    if !contributor_state.owned_by(program_id) {
        return Err(ProgramError::IllegalOwner);
    }

    let refund_amount = {
        let contributor_data = unsafe { contributor_state.borrow_unchecked() };
        let state = Contributor::load(contributor_data)?;

        if &state.contributor != contributor.address().as_array() {
            return Err(ProgramError::InvalidAccountData);
        }

        state.amount
    };

    check_zero!(== refund_amount, ProgramError::InvalidAccountData);

    let bump_arr = [fundraiser_bump];
    let signer_seeds = [
        Seed::from(b"fundraiser"),
        Seed::from(&maker),
        Seed::from(&bump_arr),
    ];

    Transfer {
        from: vault,
        to: contributor_ata,
        authority: fundraiser,
        amount: refund_amount,
    }
    .invoke_signed(&[Signer::from(&signer_seeds[..])])?;

    contributor.set_lamports(contributor.lamports() + contributor_state.lamports());
    contributor_state.set_lamports(0);

    let data = unsafe { contributor_state.borrow_unchecked_mut() };
    data.fill(0);

    Ok(())
}
