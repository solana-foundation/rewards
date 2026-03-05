use pinocchio::{account::AccountView, Address, ProgramResult};
use pinocchio_token_2022::instructions::TransferChecked;

use crate::{
    errors::RewardsProgramError,
    events::ClaimedEvent,
    state::{RewardPool, UserRewardAccount},
    traits::{AccountSerialize, EventSerialize},
    utils::{
        emit_event, get_mint_decimals, get_token_account_balance, resolve_claim_amount, sync_user_balance,
        update_user_rewards, BalanceSource,
    },
    ID,
};

use super::ClaimContinuous;

pub fn process_claim_continuous(
    _program_id: &Address,
    accounts: &[AccountView],
    instruction_data: &[u8],
) -> ProgramResult {
    let ix = ClaimContinuous::try_from((instruction_data, accounts))?;

    let pool_data = ix.accounts.reward_pool.try_borrow()?;
    let mut pool = RewardPool::from_account(&pool_data, ix.accounts.reward_pool, &ID)?;
    drop(pool_data);

    pool.validate_tracked_mint(ix.accounts.tracked_mint.address())?;
    pool.validate_reward_mint(ix.accounts.reward_mint.address())?;

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

    if pool.balance_source == BalanceSource::OnChain {
        let current_balance = get_token_account_balance(ix.accounts.user_tracked_token_account)?;
        sync_user_balance(&mut pool, &mut user, current_balance)?;
    }

    let claim_amount = resolve_claim_amount(ix.data.amount, user.accrued_rewards)?;

    user.accrued_rewards = user.accrued_rewards.checked_sub(claim_amount).ok_or(RewardsProgramError::MathOverflow)?;

    pool.total_claimed = pool.total_claimed.checked_add(claim_amount).ok_or(RewardsProgramError::MathOverflow)?;

    let mut user_data = ix.accounts.user_reward_account.try_borrow_mut()?;
    user.write_to_slice(&mut user_data)?;
    drop(user_data);

    let mut pool_data = ix.accounts.reward_pool.try_borrow_mut()?;
    pool.write_to_slice(&mut pool_data)?;
    drop(pool_data);

    let decimals = get_mint_decimals(ix.accounts.reward_mint)?;

    pool.with_signer(|signers| {
        TransferChecked {
            from: ix.accounts.reward_vault,
            mint: ix.accounts.reward_mint,
            to: ix.accounts.user_reward_token_account,
            authority: ix.accounts.reward_pool,
            amount: claim_amount,
            decimals,
            token_program: ix.accounts.reward_token_program.address(),
        }
        .invoke_signed(signers)
    })?;

    let event = ClaimedEvent::new(*ix.accounts.reward_pool.address(), *ix.accounts.user.address(), claim_amount);
    emit_event(&ID, ix.accounts.event_authority, ix.accounts.program, &event.to_bytes())?;

    Ok(())
}
