use solana_sdk::signature::Signer;

use crate::fixtures::{build_sync_balance_instruction, ContinuousOptInSetup, SyncContinuousBalanceFixture};
use crate::utils::{assert_rewards_error, test_not_writable, RewardsError, TestContext};

// ─── Generic validation tests ───

#[test]
fn test_sync_balance_pool_not_writable() {
    let mut ctx = TestContext::new();
    test_not_writable::<SyncContinuousBalanceFixture>(&mut ctx, 0);
}

#[test]
fn test_sync_balance_user_reward_account_not_writable() {
    let mut ctx = TestContext::new();
    test_not_writable::<SyncContinuousBalanceFixture>(&mut ctx, 1);
}

// ─── Custom error tests ───

#[test]
fn test_sync_balance_wrong_balance_source() {
    let mut ctx = TestContext::new();

    let setup = ContinuousOptInSetup::new_authority_set(&mut ctx);
    setup.build_instruction(&ctx).send_expect_success(&mut ctx);

    let sync_ix = build_sync_balance_instruction(
        &setup.pool_setup,
        &setup.user.pubkey(),
        &setup.user_reward_pda,
        &setup.user_tracked_token_account,
    );

    let error = sync_ix.send_expect_error(&mut ctx);
    assert_rewards_error(error, RewardsError::BalanceSourceMismatch);
}
