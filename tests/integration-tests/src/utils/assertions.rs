use rewards_program_client::accounts::{DirectDistribution, DirectRecipient, MerkleClaim, MerkleDistribution};
use solana_sdk::{instruction::InstructionError, pubkey::Pubkey, transaction::TransactionError};

use crate::utils::{get_reward_pool, TestContext, PROGRAM_ID};

pub use rewards_program_client::errors::RewardsProgramError as RewardsError;

/// Mirrors the program's `calculate_linear_unlock` formula exactly:
/// `total_amount * elapsed / duration` using u128 intermediate math.
pub fn expected_linear_unlock(total_amount: u64, start_ts: i64, end_ts: i64, current_ts: i64) -> u64 {
    if current_ts <= start_ts {
        return 0;
    }
    if current_ts >= end_ts {
        return total_amount;
    }
    let elapsed = (current_ts - start_ts) as u128;
    let duration = (end_ts - start_ts) as u128;
    ((total_amount as u128) * elapsed / duration) as u64
}

/// Assert that a transaction error is the expected rewards program error
pub fn assert_rewards_error(tx_error: TransactionError, expected: RewardsError) {
    assert_instruction_error(tx_error, InstructionError::Custom(expected as u32));
}

/// Assert that a transaction error contains the expected instruction error
pub fn assert_instruction_error(tx_error: TransactionError, expected: InstructionError) {
    match tx_error {
        TransactionError::InstructionError(_, err) => {
            assert_eq!(err, expected, "Expected {expected:?}, got {err:?}");
        }
        other => panic!("Expected InstructionError, got {other:?}"),
    }
}

/// Assert that a direct distribution account exists with expected values
pub fn assert_direct_distribution(
    ctx: &TestContext,
    distribution_pda: &Pubkey,
    expected_authority: &Pubkey,
    expected_mint: &Pubkey,
    expected_bump: u8,
) {
    let account = ctx.get_account(distribution_pda).expect("Distribution account should exist");
    assert_eq!(account.owner, PROGRAM_ID, "Distribution should be owned by program");

    let data = DirectDistribution::from_bytes(&account.data).expect("Failed to deserialize distribution");

    assert_eq!(data.bump, expected_bump);
    assert_eq!(data.authority, *expected_authority);
    assert_eq!(data.mint, *expected_mint);
}

/// Assert that a direct recipient account exists with expected values
pub fn assert_direct_recipient(
    ctx: &TestContext,
    recipient_pda: &Pubkey,
    expected_recipient: &Pubkey,
    expected_total_amount: u64,
    expected_claimed_amount: u64,
    expected_bump: u8,
) {
    let account = ctx.get_account(recipient_pda).expect("Recipient account should exist");
    assert_eq!(account.owner, PROGRAM_ID, "Recipient should be owned by program");

    let data = DirectRecipient::from_bytes(&account.data).expect("Failed to deserialize recipient");

    assert_eq!(data.bump, expected_bump);
    assert_eq!(data.recipient, *expected_recipient);
    assert_eq!(data.total_amount, expected_total_amount);
    assert_eq!(data.claimed_amount, expected_claimed_amount);
}

/// Assert that an account does not exist (was closed)
pub fn assert_account_closed(ctx: &TestContext, pubkey: &Pubkey) {
    assert!(ctx.get_account(pubkey).is_none(), "Account {} should be closed", pubkey);
}

/// Assert that a merkle distribution account exists with expected values
pub fn assert_merkle_distribution(
    ctx: &TestContext,
    distribution_pda: &Pubkey,
    expected_authority: &Pubkey,
    expected_mint: &Pubkey,
    expected_merkle_root: &[u8; 32],
    expected_total_amount: u64,
    expected_bump: u8,
) {
    let account = ctx.get_account(distribution_pda).expect("Distribution account should exist");
    assert_eq!(account.owner, PROGRAM_ID, "Distribution should be owned by program");

    let data = MerkleDistribution::from_bytes(&account.data).expect("Failed to deserialize merkle distribution");

    assert_eq!(data.bump, expected_bump);
    assert_eq!(data.authority, *expected_authority);
    assert_eq!(data.mint, *expected_mint);
    assert_eq!(data.merkle_root, *expected_merkle_root);
    assert_eq!(data.total_amount, expected_total_amount);
}

/// Assert that a merkle claim account exists with expected values
pub fn assert_merkle_claim(ctx: &TestContext, claim_pda: &Pubkey, expected_claimed_amount: u64, expected_bump: u8) {
    let account = ctx.get_account(claim_pda).expect("Claim account should exist");
    assert_eq!(account.owner, PROGRAM_ID, "Claim should be owned by program");

    let data = MerkleClaim::from_bytes(&account.data).expect("Failed to deserialize merkle claim");

    assert_eq!(data.bump, expected_bump);
    assert_eq!(data.claimed_amount, expected_claimed_amount);
}

pub fn assert_reward_pool(
    ctx: &TestContext,
    pool_pda: &Pubkey,
    expected_authority: &Pubkey,
    expected_tracked_mint: &Pubkey,
    expected_reward_mint: &Pubkey,
    expected_bump: u8,
) {
    let data = get_reward_pool(ctx, pool_pda);
    assert_eq!(data.bump, expected_bump);
    assert_eq!(data.authority, *expected_authority);
    assert_eq!(data.tracked_mint, *expected_tracked_mint);
    assert_eq!(data.reward_mint, *expected_reward_mint);
}
