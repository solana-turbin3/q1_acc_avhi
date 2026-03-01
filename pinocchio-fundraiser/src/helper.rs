use pinocchio::AccountView;
use pinocchio::error::ProgramError;

pub fn validate_eq<T: PartialEq>(a: T, b: T, err: ProgramError) -> Result<(), ProgramError> {
    if a != b {
        return Err(err);
    }
    Ok(())
}

pub fn check_signer(a: &AccountView, err: ProgramError) -> Result<(), ProgramError> {
    if !a.is_signer() {
        return Err(err);
    }
    Ok(())
}
