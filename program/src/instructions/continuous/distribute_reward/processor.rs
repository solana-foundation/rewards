use pinocchio::{account::AccountView, Address, ProgramResult};
use pinocchio_token_2022::instructions::TransferChecked;

use crate::{
    errors::RewardsProgramError,
    events::RewardDistributedEvent,
    state::RewardPool,
    traits::{AccountSerialize, EventSerialize, InstructionData},
    utils::{continuous_utils::REWARD_PRECISION, emit_event, get_mint_decimals, maybe_confidential_deposit},
    ID,
};

use super::DistributeContinuousReward;

pub fn process_distribute_continuous_reward(
    _program_id: &Address,
    accounts: &[AccountView],
    instruction_data: &[u8],
) -> ProgramResult {
    let ix = DistributeContinuousReward::try_from((instruction_data, accounts))?;
    ix.data.validate()?;

    let pool_data = ix.accounts.reward_pool.try_borrow()?;
    let mut pool = RewardPool::from_account(&pool_data, ix.accounts.reward_pool, &ID)?;
    drop(pool_data);

    pool.validate_authority(ix.accounts.authority.address())?;
    pool.validate_reward_mint(ix.accounts.reward_mint.address())?;

    if pool.opted_in_supply == 0 {
        return Err(RewardsProgramError::NoOptedInUsers.into());
    }

    let delta_rpt = (ix.data.amount as u128)
        .checked_mul(REWARD_PRECISION)
        .ok_or(RewardsProgramError::MathOverflow)?
        .checked_div(pool.opted_in_supply as u128)
        .ok_or(RewardsProgramError::MathOverflow)?;

    if delta_rpt == 0 {
        return Err(RewardsProgramError::DistributionAmountTooSmall.into());
    }

    // Back-compute the effective amount that users can actually claim from delta_rpt.
    // This avoids stranded dust in the vault from integer division truncation.
    let effective_amount = delta_rpt
        .checked_mul(pool.opted_in_supply as u128)
        .ok_or(RewardsProgramError::MathOverflow)?
        .checked_div(REWARD_PRECISION)
        .ok_or(RewardsProgramError::MathOverflow)? as u64;

    if effective_amount == 0 {
        return Err(RewardsProgramError::DistributionAmountTooSmall.into());
    }

    pool.reward_per_token = pool.reward_per_token.checked_add(delta_rpt).ok_or(RewardsProgramError::MathOverflow)?;

    pool.total_distributed =
        pool.total_distributed.checked_add(effective_amount).ok_or(RewardsProgramError::MathOverflow)?;

    let decimals = get_mint_decimals(ix.accounts.reward_mint)?;

    TransferChecked {
        from: ix.accounts.authority_token_account,
        mint: ix.accounts.reward_mint,
        to: ix.accounts.reward_vault,
        authority: ix.accounts.authority,
        amount: effective_amount,
        decimals,
        token_program: ix.accounts.reward_token_program.address(),
    }
    .invoke()?;

    maybe_confidential_deposit(
        &pool,
        ix.accounts.reward_vault,
        ix.accounts.reward_mint,
        ix.accounts.reward_pool,
        effective_amount,
        decimals,
        ix.data.expected_pending_balance_credit_counter,
        ix.data.new_decryptable_available_balance.as_ref(),
    )?;

    let mut pool_data = ix.accounts.reward_pool.try_borrow_mut()?;
    pool.write_to_slice(&mut pool_data)?;
    drop(pool_data);

    let event =
        RewardDistributedEvent::new(*ix.accounts.reward_pool.address(), effective_amount, pool.reward_per_token);
    emit_event(&ID, ix.accounts.event_authority, ix.accounts.program, &event.to_bytes())?;

    Ok(())
}
