use pinocchio::{
    ProgramResult,
    cpi::{Seed, Signer},
    entrypoint::InstructionContext,
    sysvars::rent::ACCOUNT_STORAGE_OVERHEAD,
};

use crate::{
    entrypoint::PROGRAM_ADDRESS,
    raw_cpi,
    states::Fundraiser,
};

const DEFAULT_LAMPORTS_PER_BYTE: u64 = 6960;
const FUNDRAISER_RENT: u64 =
    (ACCOUNT_STORAGE_OVERHEAD + Fundraiser::LEN as u64) * DEFAULT_LAMPORTS_PER_BYTE;

#[inline(always)]
pub fn process_initialize(ctx: &mut InstructionContext) -> ProgramResult {
    let maker = unsafe { ctx.next_account_unchecked() }.assume_account();
    let fundraiser = unsafe { ctx.next_account_unchecked() }.assume_account();
    let mint = unsafe { ctx.next_account_unchecked() }.assume_account();
    let _ = unsafe { ctx.next_account_unchecked() }; // system_program

    let ix_data = unsafe { ctx.instruction_data_unchecked() };
    let (amount_to_raise, duration, bump) = unsafe {
        let ptr = ix_data.as_ptr().add(1);
        (*(ptr as *const u64), *ptr.add(8), *ptr.add(9))
    };

    let bump_arr = [bump];
    let seeds = [
        Seed::from(b"fundraiser"),
        Seed::from(maker.address().as_array()),
        Seed::from(&bump_arr),
    ];
    let signer_seeds = Signer::from(&seeds[..]);

    raw_cpi::raw_create_account_signed(
        &maker,
        &fundraiser,
        FUNDRAISER_RENT,
        Fundraiser::LEN as u64,
        &PROGRAM_ADDRESS,
        &[signer_seeds],
    )?;

    let fs_data = unsafe { fundraiser.borrow_unchecked_mut() };
    let fs = unsafe { &mut *(fs_data.as_mut_ptr() as *mut Fundraiser) };

    fs.maker = *maker.address().as_array();
    fs.mint_to_raise = *mint.address().as_array();
    fs.amount_to_raise = amount_to_raise;
    fs.current_amount = 0;
    fs.time_started = 0;
    fs.duration = duration;
    fs.bump = bump;

    Ok(())
}
