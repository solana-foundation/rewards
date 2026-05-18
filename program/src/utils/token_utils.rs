use pinocchio::ProgramResult;
use pinocchio::{account::AccountView, address::Address, error::ProgramError};
use pinocchio_associated_token_account::ID as ASSOCIATED_TOKEN_ACCOUNT_PROGRAM_ID;
use pinocchio_token_2022::state::{Mint as TokenMint, TokenAccount};
use pinocchio_token_2022::ID as TOKEN_2022_PROGRAM_ID;
use spl_token_2022::{
    extension::{BaseStateWithExtensions, ExtensionType, StateWithExtensions},
    state::Mint,
};

use crate::errors::RewardsProgramError;
use crate::utils::verify_token_program_account;

pub const UNSUPPORTED_MINT_EXTENSIONS: &[ExtensionType] = &[ExtensionType::TransferHook];

/// Validates an Associated Token Account address.
///
/// # Arguments
/// * `ata_info` - The ATA account to validate/create
/// * `wallet_key` - The wallet that should own the ATA
/// * `mint_info` - The token mint for the ATA
/// * `token_program_info` - The token program account
///
/// # Returns
/// * `ProgramResult` - Success if validation passes
#[inline(always)]
pub fn validate_associated_token_account_address(
    ata_info: &AccountView,
    wallet_key: &Address,
    mint_info: &AccountView,
    token_program_info: &AccountView,
) -> ProgramResult {
    let expected_ata = Address::find_program_address(
        &[wallet_key.as_ref(), token_program_info.address().as_ref(), mint_info.address().as_ref()],
        &ASSOCIATED_TOKEN_ACCOUNT_PROGRAM_ID,
    )
    .0;

    if ata_info.address() != &expected_ata {
        return Err(ProgramError::InvalidAccountData);
    }

    Ok(())
}

/// Validates an Associated Token Account.
///
/// # Arguments
/// * `ata_info` - The ATA account to validate/create
/// * `wallet_key` - The wallet that should own the ATA
/// * `mint_info` - The token mint for the ATA
/// * `token_program_info` - The token program account
///
/// # Returns
/// * `ProgramResult` - Success if validation passes and ATA exists
#[inline(always)]
pub fn validate_associated_token_account(
    ata_info: &AccountView,
    wallet_key: &Address,
    mint_info: &AccountView,
    token_program_info: &AccountView,
) -> ProgramResult {
    // Verify the ATA account is a token program account
    verify_token_program_account(ata_info)?;

    validate_associated_token_account_address(ata_info, wallet_key, mint_info, token_program_info)?;

    if ata_info.is_data_empty() {
        return Err(ProgramError::InvalidAccountData);
    }

    Ok(())
}

/// Get decimals from a mint account.
///
/// Works with both SPL Token and Token-2022 mints since they share the same base layout.
///
/// # Arguments
/// * `mint` - The mint account (must be owned by Token or Token-2022 program)
///
/// # Returns
/// * `Result<u8, ProgramError>` - The mint decimals
#[inline(always)]
pub fn get_mint_decimals(mint: &AccountView) -> Result<u8, ProgramError> {
    verify_token_program_account(mint)?;

    let mint_data = mint.try_borrow()?;
    if mint.data_len() < TokenMint::BASE_LEN {
        return Err(ProgramError::InvalidAccountData);
    }

    let mint_state = unsafe { TokenMint::from_bytes_unchecked(&mint_data) };
    Ok(mint_state.decimals())
}

/// Get the balance from a token account.
///
/// Works with both SPL Token and Token-2022 accounts since they share the same base layout.
#[inline(always)]
pub fn get_token_account_balance(token_account: &AccountView) -> Result<u64, ProgramError> {
    verify_token_program_account(token_account)?;

    let data = token_account.try_borrow()?;
    if data.len() < TokenAccount::BASE_LEN {
        return Err(ProgramError::InvalidAccountData);
    }

    let account = unsafe { TokenAccount::from_bytes_unchecked(&data) };
    Ok(account.amount())
}

#[inline(always)]
pub fn reject_mint_extensions(mint: &AccountView, blocked_extensions: &[ExtensionType]) -> ProgramResult {
    if !mint.owned_by(&TOKEN_2022_PROGRAM_ID) {
        return Ok(());
    }

    let data = mint.try_borrow()?;
    let mint_state = StateWithExtensions::<Mint>::unpack(&data)?;
    for extension_type in mint_state.get_extension_types()? {
        if blocked_extensions.contains(&extension_type) {
            return match extension_type {
                ExtensionType::TransferHook => Err(RewardsProgramError::TransferHookMintUnsupported.into()),
                _ => Err(ProgramError::InvalidAccountData),
            };
        }
    }

    Ok(())
}

/// Verify that a token account's internal owner field matches the expected owner.
///
/// Token accounts store the wallet owner at bytes 32..64. This check prevents
/// substituting an arbitrary token account (e.g., the authority's) where a
/// specific recipient's account is expected.
#[inline(always)]
pub fn verify_token_account_owner(token_account: &AccountView, expected_owner: &Address) -> Result<(), ProgramError> {
    let data = token_account.try_borrow()?;
    if data.len() < TokenAccount::BASE_LEN {
        return Err(ProgramError::InvalidAccountData);
    }

    let account = unsafe { TokenAccount::from_bytes_unchecked(&data) };
    if account.owner() != expected_owner {
        return Err(ProgramError::InvalidAccountData);
    }

    Ok(())
}
