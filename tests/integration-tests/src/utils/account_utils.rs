use rewards_program_client::accounts::{RewardPool, UserRewardAccount};
use solana_sdk::pubkey::Pubkey;

use crate::utils::{TestContext, PROGRAM_ID};

pub fn get_reward_pool(ctx: &TestContext, pool_pda: &Pubkey) -> RewardPool {
    let account = ctx.get_account(pool_pda).expect("RewardPool account should exist");
    assert_eq!(account.owner, PROGRAM_ID, "RewardPool should be owned by program");
    RewardPool::from_bytes(&account.data).expect("Failed to deserialize reward pool")
}

pub fn get_user_reward_account(ctx: &TestContext, user_reward_pda: &Pubkey) -> UserRewardAccount {
    let account = ctx.get_account(user_reward_pda).expect("UserRewardAccount should exist");
    assert_eq!(account.owner, PROGRAM_ID, "UserRewardAccount should be owned by program");
    UserRewardAccount::from_bytes(&account.data).expect("Failed to deserialize user reward account")
}
