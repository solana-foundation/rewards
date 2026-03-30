use crate::ID as REWARDS_PROGRAM_ID;
use pinocchio::{account::AccountView, error::ProgramError};
use pinocchio_associated_token_account::ID as ASSOCIATED_TOKEN_ACCOUNT_PROGRAM_ID;
use pinocchio_token::ID as TOKEN_PROGRAM_ID;
use pinocchio_token_2022::ID as TOKEN_2022_PROGRAM_ID;

/// Verify the account is the system program, returning an error if it is not.
///
/// # Arguments
/// * `account` - The account to verify.
///
/// # Returns
/// * `Result<(), ProgramError>` - The result of the operation
#[inline(always)]
pub fn verify_system_program(account: &AccountView) -> Result<(), ProgramError> {
    if account.address() != &pinocchio_system::ID {
        return Err(ProgramError::IncorrectProgramId);
    }

    Ok(())
}

/// Verify the account is a token program (SPL Token or Token-2022), returning an error if it is not.
///
/// # Arguments
/// * `account` - The account to verify.
///
/// # Returns
/// * `Result<(), ProgramError>` - The result of the operation
#[inline(always)]
pub fn verify_token_program(account: &AccountView) -> Result<(), ProgramError> {
    if account.address() != &TOKEN_PROGRAM_ID && account.address() != &TOKEN_2022_PROGRAM_ID {
        return Err(ProgramError::IncorrectProgramId);
    }

    Ok(())
}

/// Verify the account is the current program, returning an error if it is not.
///
/// # Arguments
/// * `account` - The account to verify.
///
/// # Returns
/// * `Result<(), ProgramError>` - The result of the operation
#[inline(always)]
pub fn verify_current_program(account: &AccountView) -> Result<(), ProgramError> {
    if account.address() != &REWARDS_PROGRAM_ID {
        return Err(ProgramError::IncorrectProgramId);
    }

    Ok(())
}

/// Verify the account is the Token-2022 program, returning an error if it is not.
///
/// Unlike `verify_token_program` which accepts both SPL Token and Token-2022,
/// this requires specifically the Token-2022 program.
#[inline(always)]
pub fn verify_token_2022_program(account: &AccountView) -> Result<(), ProgramError> {
    if account.address() != &TOKEN_2022_PROGRAM_ID {
        return Err(ProgramError::IncorrectProgramId);
    }

    Ok(())
}

/// Verify the account is the associated token program, returning an error if it is not.
///
/// # Arguments
/// * `account` - The account to verify.
///
/// # Returns
/// * `Result<(), ProgramError>` - The result of the operation
#[inline(always)]
pub fn verify_associated_token_program(account: &AccountView) -> Result<(), ProgramError> {
    if account.address() != &ASSOCIATED_TOKEN_ACCOUNT_PROGRAM_ID {
        return Err(ProgramError::IncorrectProgramId);
    }

    Ok(())
}
