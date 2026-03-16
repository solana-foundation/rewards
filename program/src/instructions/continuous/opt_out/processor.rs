use pinocchio::{account::AccountView, Address, ProgramResult};

use crate::{
    errors::RewardsProgramError,
    events::OptOutEvent,
    state::{RewardPool, UserRewardAccount},
    traits::{AccountSerialize, EventSerialize},
    utils::{
        close_pda_account, emit_event, get_mint_decimals, get_token_account_balance, sync_user_balance,
        transfer_reward_tokens, update_user_rewards, BalanceSource,
    },
    ID,
};

use super::ContinuousOptOut;

pub fn process_continuous_opt_out(
    _program_id: &Address,
    accounts: &[AccountView],
    instruction_data: &[u8],
) -> ProgramResult {
    let ix = ContinuousOptOut::try_from((instruction_data, accounts))?;

    let pool_data = ix.accounts.reward_pool.try_borrow()?;
    let mut pool = RewardPool::from_account(&pool_data, ix.accounts.reward_pool, &ID)?;
    drop(pool_data);

    pool.validate_tracked_mint(ix.accounts.tracked_mint.address())?;
    pool.validate_reward_mint(ix.accounts.reward_mint.address())?;
    pool.ensure_merkle_mode_disabled()?;

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

    // For AuthoritySet pools, the authority is responsible for calling set_balance
    // before user opt-out to ensure accurate reward settlement.
    if pool.balance_source == BalanceSource::OnChain {
        let current_balance = get_token_account_balance(ix.accounts.user_tracked_token_account)?;
        sync_user_balance(&mut pool, &mut user, current_balance)?;
    }

    let rewards_to_claim = user.accrued_rewards;

    if rewards_to_claim > 0 {
        pool.total_claimed =
            pool.total_claimed.checked_add(rewards_to_claim).ok_or(RewardsProgramError::MathOverflow)?;

        let decimals = get_mint_decimals(ix.accounts.reward_mint)?;

        let proof_contexts = match (
            ix.accounts.equality_proof_context,
            ix.accounts.ciphertext_validity_proof_context,
            ix.accounts.range_proof_context,
        ) {
            (Some(eq), Some(cv), Some(rp)) => Some([eq, cv, rp]),
            _ => None,
        };

        transfer_reward_tokens(
            &pool,
            ix.accounts.reward_vault,
            ix.accounts.user_reward_token_account,
            ix.accounts.reward_pool,
            ix.accounts.reward_mint,
            ix.accounts.reward_token_program.address(),
            rewards_to_claim,
            decimals,
            ix.data.confidential_transfer_bytes.as_ref().map(|b| b as &[u8]),
            proof_contexts,
        )?;
    }

    pool.opted_in_supply =
        pool.opted_in_supply.checked_sub(user.last_known_balance).ok_or(RewardsProgramError::MathOverflow)?;

    let mut pool_data = ix.accounts.reward_pool.try_borrow_mut()?;
    pool.write_to_slice(&mut pool_data)?;
    drop(pool_data);

    close_pda_account(ix.accounts.user_reward_account, ix.accounts.user)?;

    let event = OptOutEvent::new(*ix.accounts.reward_pool.address(), *ix.accounts.user.address(), rewards_to_claim);
    emit_event(&ID, ix.accounts.event_authority, ix.accounts.program, &event.to_bytes())?;

    Ok(())
}
