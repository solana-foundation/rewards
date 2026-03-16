use pinocchio::{account::AccountView, Address, ProgramResult};
use pinocchio_token_2022::instructions::{CloseAccount, TransferChecked};

use crate::{
    errors::RewardsProgramError,
    events::DistributionClosedEvent,
    state::RewardPool,
    traits::EventSerialize,
    utils::{
        close_pda_account, emit_event, get_current_timestamp, get_mint_decimals, get_token_account_balance,
        ConfidentialEmptyAccount, ConfidentialWithdraw,
    },
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

    let decimals = get_mint_decimals(ix.accounts.reward_mint)?;

    // For confidential pools with unclaimed rewards: convert the vault's CT available
    // balance back to plaintext so the existing TransferChecked sweep can handle it.
    if pool.confidential_rewards != 0 && pool.total_distributed != pool.total_claimed {
        let withdrawal_amount =
            pool.total_distributed.checked_sub(pool.total_claimed).ok_or(RewardsProgramError::MathOverflow)?;

        let new_decryptable =
            ix.data.new_decryptable_available_balance.as_ref().ok_or(RewardsProgramError::InvalidAccountData)?;
        let eq_ctx = ix.accounts.equality_proof_context.ok_or(RewardsProgramError::InvalidAccountData)?;
        let range_ctx = ix.accounts.range_proof_context.ok_or(RewardsProgramError::InvalidAccountData)?;
        let zero_ctx = ix.accounts.zero_ciphertext_proof_context.ok_or(RewardsProgramError::InvalidAccountData)?;

        pool.with_signer(|signers| {
            ConfidentialWithdraw {
                token_account: ix.accounts.reward_vault,
                mint: ix.accounts.reward_mint,
                equality_proof_context: eq_ctx,
                range_proof_context: range_ctx,
                authority: ix.accounts.reward_pool,
                amount: withdrawal_amount,
                decimals,
                new_decryptable_available_balance: new_decryptable,
                signers,
            }
            .invoke()?;

            ConfidentialEmptyAccount {
                token_account: ix.accounts.reward_vault,
                zero_ciphertext_proof_context: zero_ctx,
                authority: ix.accounts.reward_pool,
                signers,
            }
            .invoke()
        })?;
    }

    // After ConfidentialWithdraw (if any), the vault's plaintext balance equals the
    // withdrawn amount. For non-CT pools this reads the vault balance directly.
    let remaining_amount = get_token_account_balance(ix.accounts.reward_vault)?;

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
