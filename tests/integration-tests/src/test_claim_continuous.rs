use crate::fixtures::{
    build_claim_continuous_instruction, ClaimContinuousFixture, ClaimContinuousSetup, DEFAULT_REWARD_AMOUNT,
};
use crate::utils::{
    assert_rewards_error, get_user_reward_account, test_empty_data, test_missing_signer, test_not_writable,
    test_truncated_data, test_wrong_current_program, RewardsError, TestContext,
};

// ─── Generic validation tests ───

#[test]
fn test_claim_continuous_missing_user_signer() {
    let mut ctx = TestContext::new();
    test_missing_signer::<ClaimContinuousFixture>(&mut ctx, 0, 0);
}

#[test]
fn test_claim_continuous_pool_not_writable() {
    let mut ctx = TestContext::new();
    test_not_writable::<ClaimContinuousFixture>(&mut ctx, 1);
}

#[test]
fn test_claim_continuous_user_reward_account_not_writable() {
    let mut ctx = TestContext::new();
    test_not_writable::<ClaimContinuousFixture>(&mut ctx, 2);
}

#[test]
fn test_claim_continuous_vault_not_writable() {
    let mut ctx = TestContext::new();
    test_not_writable::<ClaimContinuousFixture>(&mut ctx, 4);
}

#[test]
fn test_claim_continuous_user_reward_token_account_not_writable() {
    let mut ctx = TestContext::new();
    test_not_writable::<ClaimContinuousFixture>(&mut ctx, 5);
}

#[test]
fn test_claim_continuous_wrong_current_program() {
    let mut ctx = TestContext::new();
    test_wrong_current_program::<ClaimContinuousFixture>(&mut ctx);
}

#[test]
fn test_claim_continuous_empty_data() {
    let mut ctx = TestContext::new();
    test_empty_data::<ClaimContinuousFixture>(&mut ctx);
}

#[test]
fn test_claim_continuous_truncated_data() {
    let mut ctx = TestContext::new();
    test_truncated_data::<ClaimContinuousFixture>(&mut ctx);
}

// ─── Custom error tests ───

#[test]
fn test_claim_continuous_nothing_to_claim() {
    let mut ctx = TestContext::new();
    let setup = ClaimContinuousSetup::new(&mut ctx);

    let pool_setup = &setup.distribute_setup.opt_in_setup.pool_setup;
    let user = &setup.distribute_setup.opt_in_setup.user;
    let user_reward_pda = &setup.distribute_setup.opt_in_setup.user_reward_pda;
    let user_tracked_ta = &setup.distribute_setup.opt_in_setup.user_tracked_token_account;

    let claim_ix = build_claim_continuous_instruction(
        &ctx,
        pool_setup,
        user,
        user_reward_pda,
        user_tracked_ta,
        &setup.user_reward_token_account,
        0,
    );
    claim_ix.send_expect_success(&mut ctx);

    let user_account = get_user_reward_account(&ctx, user_reward_pda);
    assert_eq!(user_account.accrued_rewards, 0);

    ctx.advance_slot();

    let claim_ix2 = build_claim_continuous_instruction(
        &ctx,
        pool_setup,
        user,
        user_reward_pda,
        user_tracked_ta,
        &setup.user_reward_token_account,
        0,
    );
    let error = claim_ix2.send_expect_error(&mut ctx);
    assert_rewards_error(error, RewardsError::NothingToClaim);
}

#[test]
fn test_claim_continuous_exceeds_claimable() {
    let mut ctx = TestContext::new();
    let setup = ClaimContinuousSetup::new(&mut ctx);

    let pool_setup = &setup.distribute_setup.opt_in_setup.pool_setup;
    let user = &setup.distribute_setup.opt_in_setup.user;
    let user_reward_pda = &setup.distribute_setup.opt_in_setup.user_reward_pda;
    let user_tracked_ta = &setup.distribute_setup.opt_in_setup.user_tracked_token_account;

    let claim_ix = build_claim_continuous_instruction(
        &ctx,
        pool_setup,
        user,
        user_reward_pda,
        user_tracked_ta,
        &setup.user_reward_token_account,
        DEFAULT_REWARD_AMOUNT + 1,
    );
    let error = claim_ix.send_expect_error(&mut ctx);
    assert_rewards_error(error, RewardsError::ExceedsClaimableAmount);
}
