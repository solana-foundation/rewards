use pinocchio::{account::AccountView, Address, ProgramResult};

use crate::{
    errors::RewardsProgramError,
    events::BalanceSyncedEvent,
    state::{RewardPool, UserRewardAccount},
    traits::{AccountSerialize, EventSerialize},
    utils::{emit_event, get_token_account_balance, sync_user_balance, update_user_rewards, BalanceSource},
    ID,
};

use super::SyncContinuousBalance;

pub fn process_sync_continuous_balance(
    _program_id: &Address,
    accounts: &[AccountView],
    instruction_data: &[u8],
) -> ProgramResult {
    let ix = SyncContinuousBalance::try_from((instruction_data, accounts))?;

    let pool_data = ix.accounts.reward_pool.try_borrow()?;
    let mut pool = RewardPool::from_account(&pool_data, ix.accounts.reward_pool, &ID)?;
    drop(pool_data);

    if pool.balance_source != BalanceSource::OnChain {
        return Err(RewardsProgramError::BalanceSourceMismatch.into());
    }

    pool.validate_tracked_mint(ix.accounts.tracked_mint.address())?;

    let user_data = ix.accounts.user_reward_account.try_borrow()?;
    let mut user = UserRewardAccount::from_account(
        &user_data,
        ix.accounts.user_reward_account,
        &ID,
        ix.accounts.reward_pool.address(),
        ix.accounts.user.address(),
    )?;
    drop(user_data);

    update_user_rewards(&pool, &mut user)?;

    let old_balance = user.last_known_balance;
    let current_balance = get_token_account_balance(ix.accounts.user_tracked_token_account)?;
    sync_user_balance(&mut pool, &mut user, current_balance)?;

    let mut user_data = ix.accounts.user_reward_account.try_borrow_mut()?;
    user.write_to_slice(&mut user_data)?;
    drop(user_data);

    let mut pool_data = ix.accounts.reward_pool.try_borrow_mut()?;
    pool.write_to_slice(&mut pool_data)?;
    drop(pool_data);

    let event = BalanceSyncedEvent::new(
        *ix.accounts.reward_pool.address(),
        *ix.accounts.user.address(),
        old_balance,
        current_balance,
    );
    emit_event(&ID, ix.accounts.event_authority, ix.accounts.program, &event.to_bytes())?;

    Ok(())
}
