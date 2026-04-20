use pinocchio::{
    account::AccountView,
    error::ProgramError,
    sysvars::{rent::Rent, Sysvar},
    Address, ProgramResult,
};
use pinocchio_token_2022::instructions::{CloseAccount, TransferChecked};

use crate::{
    errors::RewardsProgramError,
    events::DistributionClosedEvent,
    state::{DirectDistribution, DirectDistributionClosed},
    traits::{AccountSize, Discriminator, Distribution, DistributionSigner, EventSerialize, Versioned},
    utils::{emit_event, get_current_timestamp, get_mint_decimals, get_token_account_balance},
    ID,
};

use super::CloseDirectDistribution;

pub fn process_close_direct_distribution(
    _program_id: &Address,
    accounts: &[AccountView],
    instruction_data: &[u8],
) -> ProgramResult {
    let ix = CloseDirectDistribution::try_from((instruction_data, accounts))?;

    let distribution_data = ix.accounts.distribution.try_borrow()?;
    let distribution = DirectDistribution::from_account(&distribution_data, ix.accounts.distribution, &ID)?;
    distribution.validate_authority(ix.accounts.authority.address())?;
    distribution.validate_mint(ix.accounts.mint.address())?;
    drop(distribution_data);

    if distribution.clawback_ts != 0 {
        let current_ts = get_current_timestamp()?;
        if current_ts < distribution.clawback_ts {
            return Err(RewardsProgramError::ClawbackNotReached.into());
        }
    }

    let remaining_amount = get_token_account_balance(ix.accounts.distribution_vault)?;
    let decimals = get_mint_decimals(ix.accounts.mint)?;

    if remaining_amount > 0 {
        distribution.with_signer(|signers| {
            TransferChecked {
                from: ix.accounts.distribution_vault,
                mint: ix.accounts.mint,
                to: ix.accounts.authority_token_account,
                authority: ix.accounts.distribution,
                amount: remaining_amount,
                decimals,
                token_program: ix.accounts.token_program.address(),
            }
            .invoke_signed(signers)
        })?;
    }

    distribution.with_signer(|signers| {
        CloseAccount {
            account: ix.accounts.distribution_vault,
            destination: ix.accounts.authority,
            authority: ix.accounts.distribution,
            token_program: ix.accounts.token_program.address(),
        }
        .invoke_signed(signers)
    })?;

    // Flip the distribution into its closed state: overwrite header bytes with
    // the `DirectDistributionClosed` discriminator, keep the bump at byte 2,
    // then shrink the account to its minimum size and refund the freed rent.
    {
        let mut data = ix.accounts.distribution.try_borrow_mut()?;
        data[0] = DirectDistributionClosed::DISCRIMINATOR;
        data[1] = DirectDistributionClosed::VERSION;
        // data[2] already holds the bump (unchanged from active layout)
    }

    ix.accounts.distribution.resize(DirectDistributionClosed::LEN)?;
    refund_excess_rent(ix.accounts.distribution, ix.accounts.authority, DirectDistributionClosed::LEN)?;

    let event = DistributionClosedEvent::new(*ix.accounts.distribution.address(), remaining_amount);
    emit_event(&ID, ix.accounts.event_authority, ix.accounts.program, &event.to_bytes())?;

    Ok(())
}

/// Refund any lamports above the rent-exempt minimum for `new_size` back to `recipient`.
/// The account must be owned by this program so direct lamport manipulation is valid.
fn refund_excess_rent(account: &AccountView, recipient: &AccountView, new_size: usize) -> Result<(), ProgramError> {
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
