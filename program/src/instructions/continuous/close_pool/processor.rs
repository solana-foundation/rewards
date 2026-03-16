use pinocchio::{account::AccountView, Address, ProgramResult};
use pinocchio_token_2022::instructions::{CloseAccount, TransferChecked};

use crate::{
    errors::RewardsProgramError,
    events::DistributionClosedEvent,
    state::RewardPool,
    traits::EventSerialize,
    utils::{close_pda_account, emit_event, get_current_timestamp, get_mint_decimals, get_token_account_balance},
    ID,
};

use super::CloseContinuousPool;

pub fn process_close_continuous_pool(
    _program_id: &Address,
    accounts: &[AccountView],
    instruction_data: &[u8],
) -> ProgramResult {
    let ix = CloseContinuousPool::try_from((instruction_data, accounts))?;

    let pool_data = ix.accounts.reward_pool.try_borrow()?;
    let pool = RewardPool::from_account(&pool_data, ix.accounts.reward_pool, &ID)?;
    drop(pool_data);

    pool.validate_authority(ix.accounts.authority.address())?;
    pool.validate_reward_mint(ix.accounts.reward_mint.address())?;

    if pool.clawback_ts != 0 {
        let current_ts = get_current_timestamp()?;
        if current_ts < pool.clawback_ts {
            return Err(RewardsProgramError::ClawbackNotReached.into());
        }
    }

    // For confidential pools the vault's plaintext balance is always 0, so TransferChecked
    // would be a no-op but CloseAccount fails if CT available balance > 0. Require all
    // rewards to be claimed before closing so the vault is truly empty.
    if pool.confidential_rewards != 0 && pool.total_distributed != pool.total_claimed {
        return Err(RewardsProgramError::ConfidentialPoolNotDrained.into());
    }

    // Intentionally does not check unclaimed == 0 for non-CT pools. Authority can sweep
    // remaining vault funds after clawback_ts.
    let remaining_amount = get_token_account_balance(ix.accounts.reward_vault)?;
    let decimals = get_mint_decimals(ix.accounts.reward_mint)?;

    if remaining_amount > 0 {
        pool.with_signer(|signers| {
            TransferChecked {
                from: ix.accounts.reward_vault,
                mint: ix.accounts.reward_mint,
                to: ix.accounts.authority_token_account,
                authority: ix.accounts.reward_pool,
                amount: remaining_amount,
                decimals,
                token_program: ix.accounts.reward_token_program.address(),
            }
            .invoke_signed(signers)
        })?;
    }

    pool.with_signer(|signers| {
        CloseAccount {
            account: ix.accounts.reward_vault,
            destination: ix.accounts.authority,
            authority: ix.accounts.reward_pool,
            token_program: ix.accounts.reward_token_program.address(),
        }
        .invoke_signed(signers)
    })?;

    close_pda_account(ix.accounts.reward_pool, ix.accounts.authority)?;

    let event = DistributionClosedEvent::new(*ix.accounts.reward_pool.address(), remaining_amount);
    emit_event(&ID, ix.accounts.event_authority, ix.accounts.program, &event.to_bytes())?;

    Ok(())
}
