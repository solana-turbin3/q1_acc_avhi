#![allow(unexpected_cfgs)]
use pinocchio::{
    Address, ProgramResult, SUCCESS, default_panic_handler, error::ProgramError, no_allocator,
    entrypoint::{InstructionContext, NON_DUP_MARKER},
};

pinocchio_pubkey::declare_id!("FNDR111111111111111111111111111111111111111");

pub const PROGRAM_ADDRESS: Address = Address::new_from_array(ID);

use crate::instructions::{
    process_checker, process_contribute, process_create_contributor, process_initialize,
    process_refund,
};

no_allocator!();
default_panic_handler!();

// Mirrors the RuntimeAccount layout in the BPF input buffer.
// borrow_state(1) + is_signer(1) + is_writable(1) + executable(1) + resize_delta(4)
// + address(32) + owner(32) + lamports(8) + data_len(8) = 88 bytes
#[repr(C)]
struct AccountLayout {
    borrow_state: u8,
    _flags: [u8; 3],
    _resize_delta: i32,
    _address: [u8; 32],
    _owner: [u8; 32],
    _lamports: u64,
    data_len: u64,
}

// sizeof(AccountLayout) = 88; MAX_PERMITTED_DATA_INCREASE = 10_240
const STATIC_ACCOUNT_DATA: usize = 88 + 10_240;

#[unsafe(no_mangle)]
pub unsafe extern "C" fn entrypoint(input: *mut u8) -> u64 {
    let discrim = unsafe { peek_discriminator(input) };
    let mut ctx = unsafe { InstructionContext::new_unchecked(input) };

    let result: ProgramResult = match discrim {
        0 => process_initialize(&mut ctx),
        1 => process_create_contributor(&mut ctx),
        2 => process_contribute(&mut ctx),
        3 => process_checker(&mut ctx),
        4 => process_refund(&mut ctx),
        _ => Err(ProgramError::InvalidInstructionData),
    };

    match result {
        Ok(()) => SUCCESS,
        Err(e) => e.into(),
    }
}

/// Traverse the BPF input buffer to read the discriminator byte from instruction data.
///
/// The layout is: [num_accounts: u64][accounts...][data_len: u64][data: bytes][program_id]
/// Each non-duplicate account is: [AccountLayout: 88 bytes][data: data_len bytes]
///   [MAX_PERMITTED_DATA_INCREASE: 10240 bytes][alignment padding]
/// prefixed by an 8-byte "rent epoch" region before the AccountLayout.
#[inline(always)]
unsafe fn peek_discriminator(input: *const u8) -> u8 {
    let num_accounts = unsafe { *(input as *const u64) };
    let mut ptr = unsafe { input.add(8) }; // skip num_accounts

    let mut i = 0u64;
    while i < num_accounts {
        let account = ptr as *const AccountLayout;
        let borrow_state = unsafe { (*account).borrow_state };
        if borrow_state == NON_DUP_MARKER {
            let data_len = unsafe { (*account).data_len };
            // advance: 8 (fixed prefix) + STATIC_ACCOUNT_DATA + actual data
            ptr = unsafe { ptr.add(8 + STATIC_ACCOUNT_DATA + data_len as usize) };
            // align to 8 bytes (BPF_ALIGN_OF_U128 = 8)
            let aligned = (ptr as usize + 7) & !7;
            ptr = aligned as *const u8;
        } else {
            // duplicate account: just 8 bytes (dup_idx + 7 padding)
            ptr = unsafe { ptr.add(8) };
        }
        i += 1;
    }

    // ptr now at instruction data: [len: u64][discriminator: u8][rest...]
    let data_len = unsafe { *(ptr as *const u64) };
    if data_len == 0 {
        return 0xFF;
    }
    unsafe { *ptr.add(8) }
}
