use core::mem::MaybeUninit;
use core::slice::from_raw_parts;

use pinocchio::{
    AccountView, Address, ProgramResult,
    cpi::{CpiAccount, Signer, invoke_signed_unchecked},
    instruction::{InstructionAccount, InstructionView},
};

const UNINIT_BYTE: MaybeUninit<u8> = MaybeUninit::<u8>::uninit();

#[inline(always)]
pub fn raw_create_account_signed(
    from: &AccountView,
    to: &AccountView,
    lamports: u64,
    space: u64,
    owner: &Address,
    signers: &[Signer],
) -> ProgramResult {
    let instruction_accounts = [
        InstructionAccount::writable_signer(from.address()),
        InstructionAccount::writable_signer(to.address()),
    ];

    // [discriminator:4=0][lamports:8][space:8][owner:32] = 52 bytes
    let mut instruction_data = [0u8; 52];
    instruction_data[4..12].copy_from_slice(&lamports.to_le_bytes());
    instruction_data[12..20].copy_from_slice(&space.to_le_bytes());
    instruction_data[20..52].copy_from_slice(owner.as_array());

    let instruction = InstructionView {
        program_id: &pinocchio_system::ID,
        accounts: &instruction_accounts,
        data: &instruction_data,
    };

    let cpi_accounts = [CpiAccount::from(from), CpiAccount::from(to)];

    unsafe { invoke_signed_unchecked(&instruction, &cpi_accounts, signers) }

    Ok(())
}

#[inline(always)]
pub fn raw_transfer_signed(
    from: &AccountView,
    to: &AccountView,
    authority: &AccountView,
    amount: u64,
    signers: &[Signer],
) -> ProgramResult {
    let instruction_accounts = [
        InstructionAccount::writable(from.address()),
        InstructionAccount::writable(to.address()),
        InstructionAccount::readonly_signer(authority.address()),
    ];

    let mut data = [UNINIT_BYTE; 9];
    unsafe {
        (data.as_mut_ptr() as *mut u8).write(3u8);
        ((data.as_mut_ptr() as *mut u8).add(1) as *mut u64).write_unaligned(amount);
    }

    let instruction = InstructionView {
        program_id: &pinocchio_token::ID,
        accounts: &instruction_accounts,
        data: unsafe { from_raw_parts(data.as_ptr() as *const u8, 9) },
    };

    let cpi_accounts = [
        CpiAccount::from(from),
        CpiAccount::from(to),
        CpiAccount::from(authority),
    ];

    unsafe { invoke_signed_unchecked(&instruction, &cpi_accounts, signers) }

    Ok(())
}
