use crate::fixtures::{build_opt_out_instruction, ClaimContinuousMerkleSetup, ContinuousOptOutFixture};
use crate::utils::{
    assert_rewards_error, test_missing_signer, test_not_writable, test_wrong_current_program, RewardsError, TestContext,
};

// ─── Generic validation tests ───

#[test]
fn test_opt_out_missing_user_signer() {
    let mut ctx = TestContext::new();
    test_missing_signer::<ContinuousOptOutFixture>(&mut ctx, 0, 0);
}

#[test]
fn test_opt_out_pool_not_writable() {
    let mut ctx = TestContext::new();
    test_not_writable::<ContinuousOptOutFixture>(&mut ctx, 1);
}

#[test]
fn test_opt_out_user_reward_account_not_writable() {
    let mut ctx = TestContext::new();
    test_not_writable::<ContinuousOptOutFixture>(&mut ctx, 2);
}

#[test]
fn test_opt_out_vault_not_writable() {
    let mut ctx = TestContext::new();
    test_not_writable::<ContinuousOptOutFixture>(&mut ctx, 4);
}

#[test]
fn test_opt_out_user_reward_token_account_not_writable() {
    let mut ctx = TestContext::new();
    test_not_writable::<ContinuousOptOutFixture>(&mut ctx, 5);
}

#[test]
fn test_opt_out_wrong_current_program() {
    let mut ctx = TestContext::new();
    test_wrong_current_program::<ContinuousOptOutFixture>(&mut ctx);
}

#[test]
fn test_opt_out_disabled_when_merkle_mode_enabled() {
    let mut ctx = TestContext::new();
    let setup = ClaimContinuousMerkleSetup::new(&mut ctx);

    let pool_setup = &setup.distribute_setup.opt_in_setup.pool_setup;
    let user = &setup.distribute_setup.opt_in_setup.user;
    let user_reward_pda = &setup.distribute_setup.opt_in_setup.user_reward_pda;
    let user_tracked_ta = &setup.distribute_setup.opt_in_setup.user_tracked_token_account;

    let ix = build_opt_out_instruction(
        &ctx,
        pool_setup,
        user,
        user_reward_pda,
        user_tracked_ta,
        &setup.user_reward_token_account,
    );
    let error = ix.send_expect_error(&mut ctx);
    assert_rewards_error(error, RewardsError::ContinuousMerkleModeEnabled);
}
