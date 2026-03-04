use pinocchio::{
    AccountView, Address, ProgramResult,
    cpi::{Seed, Signer},
    error::ProgramError,
};
use pinocchio_token::instructions::{CloseAccount, Transfer};

use crate::{helper::check_signer, states::Fundraiser};

pub fn process_checker(
    _program_id: &Address,
    accounts: &[AccountView],
    _instruction_data: &[u8],
) -> ProgramResult {
    let [
        maker,
        fundraiser,
        vault,
        maker_ata,
        _token_program,
        _remaining @ ..,
    ] = accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    check_signer(maker, ProgramError::IncorrectAuthority)?;

    let (amount_to_raise, current_amount, bump) = {
        let fundraiser_data = unsafe { fundraiser.borrow_unchecked() };
        let state = Fundraiser::load(fundraiser_data)?;

        if &state.maker != maker.address().as_array() {
            return Err(ProgramError::IncorrectAuthority);
        }

        (state.amount_to_raise, state.current_amount, state.bump)
    };

    if current_amount < amount_to_raise {
        return Err(ProgramError::InvalidAccountData);
    }

    let bump_arr = [bump];
    let signer_seeds = [
        Seed::from(b"fundraiser"),
        Seed::from(maker.address().as_array()),
        Seed::from(&bump_arr),
    ];
    let signer = Signer::from(&signer_seeds[..]);

    Transfer {
        from: vault,
        to: maker_ata,
        authority: fundraiser,
        amount: current_amount,
    }
    .invoke_signed(&[signer])?;

    CloseAccount {
        account: vault,
        destination: maker,
        authority: fundraiser,
    }
    .invoke_signed(&[Signer::from(&signer_seeds[..])])?;

    maker.set_lamports(maker.lamports() + fundraiser.lamports());
    fundraiser.set_lamports(0);

    let data = unsafe { fundraiser.borrow_unchecked_mut() };
    data.fill(0);

    Ok(())
}
