macro_rules! impl_len {
    ($t: ty) => {
        impl $t {
            #[allow(dead_code)]
            pub const LEN: usize = core::mem::size_of::<Self>();
        }
    };
}

macro_rules! impl_load {
    ($t: ty) => {
        impl $t {
            #[allow(dead_code)]
            pub fn load(data: &[u8]) -> Result<&Self, pinocchio::error::ProgramError> {
                if (data.len() != core::mem::size_of::<$t>()) {
                    Err(pinocchio::error::ProgramError::InvalidAccountData)
                } else if (data.as_ptr() as usize % core::mem::align_of::<$t>() != 0) {
                    Err(pinocchio::error::ProgramError::InvalidAccountData)
                } else {
                    Ok(unsafe { &*(data.as_ptr() as *const Self) })
                }
            }
            #[allow(dead_code)]
            pub fn load_mut(data: &mut [u8]) -> Result<&mut Self, pinocchio::error::ProgramError> {
                if (data.len() != core::mem::size_of::<$t>()) {
                    Err(pinocchio::error::ProgramError::InvalidAccountData)
                } else if (data.as_ptr() as usize % core::mem::align_of::<$t>() != 0) {
                    Err(pinocchio::error::ProgramError::InvalidAccountData)
                } else {
                    Ok(unsafe { &mut *(data.as_mut_ptr() as *mut Self) })
                }
            }
        }
    };
}

macro_rules! check_zero {
    (== $a:expr, $err:expr) => {
        if $a == 0 {
            return Err($err);
        }
    };
    (!= $a:expr, $err:expr) => {
        if $a != 0 {
            return Err($err);
        }
    };
}

pub(crate) use check_zero;
pub(crate) use impl_len;
pub(crate) use impl_load;
