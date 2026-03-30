use rewards_program_client::accounts::{PointsConfig, RewardPool, UserRewardAccount};
use solana_sdk::pubkey::Pubkey;
use spl_associated_token_account::get_associated_token_address_with_program_id;
use spl_token_2022::ID as TOKEN_2022_PROGRAM_ID;

use crate::utils::{TestContext, PROGRAM_ID};

pub fn get_points_config(ctx: &TestContext, config_pda: &Pubkey) -> PointsConfig {
    let account = ctx.get_account(config_pda).expect("PointsConfig account should exist");
    assert_eq!(account.owner, PROGRAM_ID, "PointsConfig should be owned by program");
    PointsConfig::from_bytes(&account.data).expect("Failed to deserialize points config")
}

/// Get the Token-2022 ATA balance for a user's points token account.
pub fn get_token_2022_ata_balance(ctx: &TestContext, user: &Pubkey, mint: &Pubkey) -> u64 {
    let ata = get_associated_token_address_with_program_id(user, mint, &TOKEN_2022_PROGRAM_ID);
    let account = ctx.get_account(&ata).expect("Token-2022 ATA should exist");
    // Balance is at offset 64..72 in SPL token account layout
    u64::from_le_bytes(account.data[64..72].try_into().unwrap())
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
