use rewards_program_client::accounts::{PointsConfig, RewardPool, UserPointsAccount, UserRewardAccount};
use solana_sdk::pubkey::Pubkey;

use crate::utils::{TestContext, PROGRAM_ID};

pub fn get_points_config(ctx: &TestContext, config_pda: &Pubkey) -> PointsConfig {
    let account = ctx.get_account(config_pda).expect("PointsConfig account should exist");
    assert_eq!(account.owner, PROGRAM_ID, "PointsConfig should be owned by program");
    PointsConfig::from_bytes(&account.data).expect("Failed to deserialize points config")
}

pub fn get_user_points_account(ctx: &TestContext, user_points_pda: &Pubkey) -> UserPointsAccount {
    let account = ctx.get_account(user_points_pda).expect("UserPointsAccount should exist");
    assert_eq!(account.owner, PROGRAM_ID, "UserPointsAccount should be owned by program");
    UserPointsAccount::from_bytes(&account.data).expect("Failed to deserialize user points account")
}

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
