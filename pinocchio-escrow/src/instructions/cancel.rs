use crate::entrypoint::ID;
use pinocchio::{
    cpi::{Seed, Signer},
    error::ProgramError,
    AccountView, ProgramResult,
};
use pinocchio_pubkey::derive_address;
use pinocchio_token::instructions::{CloseAccount, Transfer};

use crate::state::Escrow;

pub fn process_cancel_instruction(accounts: &[AccountView], _data: &[u8]) -> ProgramResult {
    let [maker, escrow_account, maker_ata_a, escrow_ata, _remaining @ ..] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    if !maker.is_signer() {
        return Err(ProgramError::IncorrectAuthority);
    }

    let bump = {
        let escrow_data = unsafe { escrow_account.borrow_unchecked() };
        let escrow_state = Escrow::load(escrow_data)?;

        if escrow_state.maker != *maker.address().as_array() {
            return Err(ProgramError::InvalidAccountData);
        }

        escrow_state.bump
    };

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

    Transfer {
        from: escrow_ata,
        to: maker_ata_a,
        authority: escrow_account,
        amount: {
            let escrow_data = unsafe { escrow_account.borrow_unchecked() };
            Escrow::load(escrow_data)?.amount_to_give
        },
    }
    .invoke_signed(&[Signer::from(&signer_seeds[..])])?;

    CloseAccount {
        account: escrow_ata,
        destination: maker,
        authority: escrow_account,
    }
    .invoke_signed(&[Signer::from(&signer_seeds[..])])?;

    let escrow_lamports = escrow_account.lamports();
    maker.set_lamports(maker.lamports() + escrow_lamports);
    escrow_account.set_lamports(0);

    Ok(())
}
