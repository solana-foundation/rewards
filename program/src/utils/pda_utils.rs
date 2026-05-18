use pinocchio::{account::AccountView, address::Address, error::ProgramError};
use pinocchio::{
    cpi::{Seed, Signer},
    sysvars::{rent::Rent, Sysvar},
    ProgramResult,
};
use pinocchio_system::instructions::{Allocate, Assign, CreateAccount, Transfer};

use crate::errors::RewardsProgramError;
use crate::state::RevocationSeeds;
use crate::traits::PdaSeeds;

/// Check if a PDA account is uninitialized (owned by system program).
/// Safe alternative to checking lamports == 0, which can be manipulated.
#[inline(always)]
pub fn is_pda_uninitialized(account: &AccountView) -> bool {
    account.owned_by(&pinocchio_system::ID)
}

/// Verify that the revocation marker PDA for this parent+user pair is uninitialized.
/// Returns the provided error if the user has been revoked.
pub fn verify_not_revoked(
    parent: &Address,
    user: &Address,
    revocation_marker: &AccountView,
    program_id: &Address,
    err: RewardsProgramError,
) -> ProgramResult {
    let revocation_seeds = RevocationSeeds { parent: *parent, user: *user };
    revocation_seeds.validate_pda_address(revocation_marker, program_id)?;

    if !is_pda_uninitialized(revocation_marker) {
        return Err(err.into());
    }

    Ok(())
}

/// Close a PDA account and return the lamports to the recipient.
pub fn close_pda_account(pda_account: &AccountView, recipient: &AccountView) -> ProgramResult {
    let payer_lamports = recipient.lamports();
    recipient
        .set_lamports(payer_lamports.checked_add(pda_account.lamports()).ok_or(RewardsProgramError::MathOverflow)?);
    pda_account.set_lamports(0);
    pda_account.close()?;

    Ok(())
}

/// Refund any lamports above the rent-exempt minimum for `new_size` back to `recipient`.
///
/// Call after shrinking a program-owned PDA via `account.resize(new_size)` to return
/// the freed rent. The account must be owned by the invoking program so that direct
/// lamport manipulation is valid.
pub fn refund_excess_rent(account: &AccountView, recipient: &AccountView, new_size: usize) -> ProgramResult {
    let rent = Rent::get()?;
    let required = rent.try_minimum_balance(new_size).map_err(|_| RewardsProgramError::RentCalculationFailed)?;
    let current = account.lamports();
    let excess = current.saturating_sub(required);

    if excess > 0 {
        account.set_lamports(current.saturating_sub(excess));
        recipient.set_lamports(recipient.lamports().checked_add(excess).ok_or(RewardsProgramError::MathOverflow)?);
    }

    Ok(())
}

/// Create a PDA account for the given seeds.
///
/// Supports pre-funded, system-owned PDA addresses with zero data by
/// completing initialization via transfer + allocate + assign.
///
/// Returns `AccountAlreadyInitialized` for non-system or data-bearing accounts.
pub fn create_pda_account<const N: usize>(
    payer: &AccountView,
    space: usize,
    owner: &Address,
    pda_account: &AccountView,
    pda_signer_seeds: [Seed; N],
) -> ProgramResult {
    let rent = Rent::get()?;

    let required_lamports =
        rent.try_minimum_balance(space).map_err(|_| RewardsProgramError::RentCalculationFailed)?.max(1);

    let signers = [Signer::from(&pda_signer_seeds)];

    if pda_account.lamports() == 0 {
        return CreateAccount { from: payer, to: pda_account, lamports: required_lamports, space: space as u64, owner }
            .invoke_signed(&signers);
    }

    // Only permit the pre-funded edge case on a still-uninitialized system account.
    if !is_pda_uninitialized(pda_account) || pda_account.data_len() != 0 {
        return Err(ProgramError::AccountAlreadyInitialized);
    }

    let additional_lamports = required_lamports.saturating_sub(pda_account.lamports());
    if additional_lamports > 0 {
        Transfer { from: payer, to: pda_account, lamports: additional_lamports }.invoke()?;
    }

    Allocate { account: pda_account, space: space as u64 }.invoke_signed(&signers)?;
    Assign { account: pda_account, owner }.invoke_signed(&signers)?;
    Ok(())
}

/// Create a PDA account idempotently for the given seeds.
///
/// **Security Warning**: This function allows re-initialization of existing accounts.
/// Use `create_pda_account` for strict "create once" semantics where re-init should error.
///
/// Use this for idempotent operations where re-initialization is acceptable.
/// If the account already exists and has data, it will be resized to the new space.
pub fn create_pda_account_idempotent<const N: usize>(
    payer: &AccountView,
    space: usize,
    owner: &Address,
    pda_account: &AccountView,
    pda_signer_seeds: [Seed; N],
) -> ProgramResult {
    let rent = Rent::get()?;

    let required_lamports =
        rent.try_minimum_balance(space).map_err(|_| RewardsProgramError::RentCalculationFailed)?.max(1);

    let signers = [Signer::from(&pda_signer_seeds)];

    if pda_account.lamports() > 0 {
        // Account exists - check if it needs resizing
        let current_len = pda_account.data_len();

        if current_len > 0 {
            // Account has data - use resize instead of Allocate
            if space > current_len {
                // Need to grow - first add lamports if needed
                let additional_lamports = required_lamports.saturating_sub(pda_account.lamports());
                if additional_lamports > 0 {
                    Transfer { from: payer, to: pda_account, lamports: additional_lamports }.invoke()?;
                }
                // Resize the account
                pda_account.resize(space)?;
            }
            // If space <= current_len, no action needed (data already fits)
        } else {
            // Account has lamports but no data (e.g., someone transferred lamports before init)
            let additional_lamports = required_lamports.saturating_sub(pda_account.lamports());
            if additional_lamports > 0 {
                Transfer { from: payer, to: pda_account, lamports: additional_lamports }.invoke()?;
            }
            Allocate { account: pda_account, space: space as u64 }.invoke_signed(&signers)?;
            Assign { account: pda_account, owner }.invoke_signed(&signers)?;
        }
        Ok(())
    } else {
        CreateAccount { from: payer, to: pda_account, lamports: required_lamports, space: space as u64, owner }
            .invoke_signed(&signers)
    }
}
