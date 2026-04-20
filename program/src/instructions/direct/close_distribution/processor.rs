use pinocchio::{account::AccountView, Address, ProgramResult};
use pinocchio_token_2022::instructions::{CloseAccount, TransferChecked};

use crate::{
    errors::RewardsProgramError,
    events::DistributionClosedEvent,
    state::DirectDistribution,
    traits::{Distribution, DistributionSigner, EventSerialize},
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

    // Flip the distribution PDA to its permanently-closed state and refund the freed rent.
    DirectDistribution::close_in_place(ix.accounts.distribution, ix.accounts.authority)?;

    let event = DistributionClosedEvent::new(*ix.accounts.distribution.address(), remaining_amount);
    emit_event(&ID, ix.accounts.event_authority, ix.accounts.program, &event.to_bytes())?;

    Ok(())
}
