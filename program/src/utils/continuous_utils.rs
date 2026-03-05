use pinocchio::error::ProgramError;

use crate::errors::RewardsProgramError;
use crate::state::{RewardPool, UserRewardAccount};

pub const REWARD_PRECISION: u128 = 1_000_000_000_000; // 1e12

/// Accrue pending rewards for a user based on the delta between the global
/// reward_per_token and the user's last snapshot.
///
/// Must be called before any change to the user's balance or before claiming.
#[inline(always)]
pub fn update_user_rewards(pool: &RewardPool, user: &mut UserRewardAccount) -> Result<(), ProgramError> {
    let delta =
        pool.reward_per_token.checked_sub(user.reward_per_token_paid).ok_or(RewardsProgramError::MathOverflow)?;

    if delta > 0 {
        let earned = (user.last_known_balance as u128)
            .checked_mul(delta)
            .ok_or(RewardsProgramError::MathOverflow)?
            .checked_div(REWARD_PRECISION)
            .ok_or(RewardsProgramError::MathOverflow)?;

        let earned_u64 = u64::try_from(earned).map_err(|_| RewardsProgramError::MathOverflow)?;

        user.accrued_rewards = user.accrued_rewards.checked_add(earned_u64).ok_or(RewardsProgramError::MathOverflow)?;
    }

    user.reward_per_token_paid = pool.reward_per_token;
    Ok(())
}

/// Sync a user's tracked balance to a new value and adjust the pool's
/// opted_in_supply accordingly.
#[inline(always)]
pub fn sync_user_balance(
    pool: &mut RewardPool,
    user: &mut UserRewardAccount,
    new_balance: u64,
) -> Result<(), ProgramError> {
    let old_balance = user.last_known_balance;

    if new_balance >= old_balance {
        let increase = new_balance - old_balance;
        pool.opted_in_supply = pool.opted_in_supply.checked_add(increase).ok_or(RewardsProgramError::MathOverflow)?;
    } else {
        let decrease = old_balance - new_balance;
        pool.opted_in_supply = pool.opted_in_supply.checked_sub(decrease).ok_or(RewardsProgramError::MathOverflow)?;
    }

    user.last_known_balance = new_balance;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::BalanceSource;
    use pinocchio::Address;

    fn create_test_pool(reward_per_token: u128, opted_in_supply: u64) -> RewardPool {
        let mut pool = RewardPool::new(
            255,
            BalanceSource::OnChain,
            0,
            0,
            Address::new_from_array([1u8; 32]),
            Address::new_from_array([2u8; 32]),
            Address::new_from_array([3u8; 32]),
            Address::new_from_array([4u8; 32]),
        );
        pool.reward_per_token = reward_per_token;
        pool.opted_in_supply = opted_in_supply;
        pool
    }

    fn create_test_user(reward_per_token_paid: u128, balance: u64, accrued: u64) -> UserRewardAccount {
        let mut user = UserRewardAccount::new(255, reward_per_token_paid, balance);
        user.accrued_rewards = accrued;
        user
    }

    #[test]
    fn test_update_rewards_no_delta() {
        let pool = create_test_pool(1000, 100);
        let mut user = create_test_user(1000, 50, 0);

        update_user_rewards(&pool, &mut user).unwrap();

        assert_eq!(user.accrued_rewards, 0);
        assert_eq!(user.reward_per_token_paid, 1000);
    }

    #[test]
    fn test_update_rewards_with_delta() {
        // Pool RPT = 2e12, user RPT = 1e12, user balance = 1000
        // delta = 1e12, earned = 1000 * 1e12 / 1e12 = 1000
        let pool = create_test_pool(2 * REWARD_PRECISION, 1000);
        let mut user = create_test_user(REWARD_PRECISION, 1000, 0);

        update_user_rewards(&pool, &mut user).unwrap();

        assert_eq!(user.accrued_rewards, 1000);
        assert_eq!(user.reward_per_token_paid, 2 * REWARD_PRECISION);
    }

    #[test]
    fn test_update_rewards_accumulates() {
        let pool = create_test_pool(2 * REWARD_PRECISION, 1000);
        let mut user = create_test_user(REWARD_PRECISION, 500, 200);

        update_user_rewards(&pool, &mut user).unwrap();

        // earned = 500 * 1e12 / 1e12 = 500, plus existing 200 = 700
        assert_eq!(user.accrued_rewards, 700);
    }

    #[test]
    fn test_update_rewards_zero_balance() {
        let pool = create_test_pool(2 * REWARD_PRECISION, 1000);
        let mut user = create_test_user(REWARD_PRECISION, 0, 0);

        update_user_rewards(&pool, &mut user).unwrap();

        assert_eq!(user.accrued_rewards, 0);
    }

    #[test]
    fn test_update_rewards_fractional() {
        // Pool RPT = 1.5e12, user RPT = 0, user balance = 3
        // delta = 1.5e12, earned = 3 * 1.5e12 / 1e12 = 4 (truncated from 4.5)
        let pool = create_test_pool(REWARD_PRECISION * 3 / 2, 1000);
        let mut user = create_test_user(0, 3, 0);

        update_user_rewards(&pool, &mut user).unwrap();

        assert_eq!(user.accrued_rewards, 4);
    }

    #[test]
    fn test_sync_balance_increase() {
        let mut pool = create_test_pool(0, 1000);
        let mut user = create_test_user(0, 500, 0);

        sync_user_balance(&mut pool, &mut user, 800).unwrap();

        assert_eq!(user.last_known_balance, 800);
        assert_eq!(pool.opted_in_supply, 1300); // 1000 + 300
    }

    #[test]
    fn test_sync_balance_decrease() {
        let mut pool = create_test_pool(0, 1000);
        let mut user = create_test_user(0, 500, 0);

        sync_user_balance(&mut pool, &mut user, 200).unwrap();

        assert_eq!(user.last_known_balance, 200);
        assert_eq!(pool.opted_in_supply, 700); // 1000 - 300
    }

    #[test]
    fn test_sync_balance_no_change() {
        let mut pool = create_test_pool(0, 1000);
        let mut user = create_test_user(0, 500, 0);

        sync_user_balance(&mut pool, &mut user, 500).unwrap();

        assert_eq!(user.last_known_balance, 500);
        assert_eq!(pool.opted_in_supply, 1000);
    }

    #[test]
    fn test_sync_balance_to_zero() {
        let mut pool = create_test_pool(0, 500);
        let mut user = create_test_user(0, 500, 0);

        sync_user_balance(&mut pool, &mut user, 0).unwrap();

        assert_eq!(user.last_known_balance, 0);
        assert_eq!(pool.opted_in_supply, 0);
    }

    #[test]
    fn test_sync_balance_overflow() {
        let mut pool = create_test_pool(0, u64::MAX);
        let mut user = create_test_user(0, 0, 0);

        let result = sync_user_balance(&mut pool, &mut user, 1);
        assert!(result.is_err());
    }

    #[test]
    fn test_sync_balance_underflow() {
        let mut pool = create_test_pool(0, 0);
        let mut user = create_test_user(0, 100, 0);

        let result = sync_user_balance(&mut pool, &mut user, 0);
        assert!(result.is_err());
    }

    #[test]
    fn test_full_flow_distribute_and_claim() {
        // Simulate: 2 users opt in, authority distributes, users claim

        // User A: 1000 tokens, User B: 500 tokens
        let mut pool = create_test_pool(0, 1500);
        let mut user_a = create_test_user(0, 1000, 0);
        let mut user_b = create_test_user(0, 500, 0);

        // Authority distributes 150 tokens
        // delta_rpt = 150 * 1e12 / 1500 = 1e11
        let amount: u128 = 150;
        let delta_rpt = amount * REWARD_PRECISION / 1500;
        pool.reward_per_token += delta_rpt;
        pool.total_distributed += 150;

        // User A claims
        update_user_rewards(&pool, &mut user_a).unwrap();
        // earned = 1000 * 1e11 / 1e12 = 100
        assert_eq!(user_a.accrued_rewards, 100);

        // User B claims
        update_user_rewards(&pool, &mut user_b).unwrap();
        // earned = 500 * 1e11 / 1e12 = 50
        assert_eq!(user_b.accrued_rewards, 50);
    }
}
