use pinocchio::{
    AccountView, Address, ProgramResult,
    cpi::{Seed, Signer},
    error::ProgramError,
    sysvars::{Sysvar, clock::Clock},
};
use pinocchio_pubkey::derive_address;
use pinocchio_token::{ID as TOKEN_ID, instructions::Transfer};

use crate::{
    helper::{check_signer, validate_eq},
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
        token_program,
        _remaining @ ..,
    ] = accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    check_signer(contributor, ProgramError::IncorrectAuthority)?;

    validate_eq(
        token_program.address(),
        &TOKEN_ID,
        ProgramError::IncorrectProgramId,
    )?;

    let (amount_to_raise, current_amount, time_started, duration, maker, fundraiser_bump) = {
        let fundraiser_data = unsafe { fundraiser.borrow_unchecked() };
        let state = Fundraiser::load(fundraiser_data)?;

        let seeds: [&[u8]; 2] = [b"fundraiser", &state.maker];
        let expected = derive_address(&seeds, Some(state.bump), program_id.as_array());

        validate_eq(
            fundraiser.address().as_array(),
            &expected,
            ProgramError::InvalidAccountData,
        )?;

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

    let refund_amount = {
        let contributor_data = unsafe { contributor_state.borrow_unchecked() };
        let state = Contributor::load(contributor_data)?;

        let seeds: [&[u8]; 3] = [
            b"contributor",
            fundraiser.address().as_array(),
            contributor.address().as_array(),
        ];
        let expected = derive_address(&seeds, Some(state.bump), program_id.as_array());

        validate_eq(
            contributor_state.address().as_array(),
            &expected,
            ProgramError::InvalidAccountData,
        )?;

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
